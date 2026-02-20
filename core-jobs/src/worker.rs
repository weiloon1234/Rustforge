use crate::{Job, JobContext};
use async_trait::async_trait;
use redis::AsyncCommands;
use serde_json::Value;
use std::collections::HashMap;

use crate::JobPayload;

/// Result of execution
pub enum JobResult {
    Success,
    Failure {
        backoff: u64,
        max_retries: u32,
        err: String,
    },
}

#[derive(serde::Serialize, serde::Deserialize)]
struct DelayedGroupSignal {
    queue: String,
    group_id: String,
}

/// Type-erased handler for jobs
#[async_trait]
trait JobHandler: Send + Sync {
    async fn execute(
        &self,
        json: Value,
        ctx: &JobContext,
        attempts: u32,
    ) -> anyhow::Result<JobResult>;
}

struct JobShim<J>(std::marker::PhantomData<J>);

#[async_trait]
impl<J: Job> JobHandler for JobShim<J> {
    async fn execute(
        &self,
        json: Value,
        ctx: &JobContext,
        attempts: u32,
    ) -> anyhow::Result<JobResult> {
        let job: J = serde_json::from_value(json)?;
        match job.handle(ctx).await {
            Ok(_) => Ok(JobResult::Success),
            Err(e) => Ok(JobResult::Failure {
                backoff: job.backoff(attempts),
                max_retries: job.max_retries(),
                err: e.to_string(),
            }),
        }
    }
}

use std::sync::Arc;

#[derive(Clone)]
pub struct Worker {
    registry: Arc<HashMap<&'static str, Box<dyn JobHandler>>>,
    queues: Vec<&'static str>,
    redis: redis::Client, // Raw redis for BLPOP
    context: JobContext,
    prefix: String,
    sweeper_config: Option<(sqlx::PgPool, std::time::Duration)>,
    config: Option<WorkerInternalConfig>,
}

#[derive(Clone, Debug)]
pub struct WorkerInternalConfig {
    pub concurrency: usize,
    pub sweep_interval: std::time::Duration,
    pub redis_url: String,
}

impl Worker {
    pub fn new(url: &str, context: JobContext) -> anyhow::Result<Self> {
        let redis = redis::Client::open(url)?;
        Ok(Self {
            registry: Arc::new(HashMap::new()),
            queues: vec!["default"],
            redis,
            context,
            prefix: "queue".to_string(),
            sweeper_config: None,
            config: None,
        })
    }

    pub async fn from_settings(context: JobContext) -> anyhow::Result<Self> {
        let config = &context.settings.worker;

        let redis_url = context.settings.redis.url.clone();
        // RedisSettings might not have just URL? Settings constructs it.
        // Wait, context.redis is Cache (Client+Multiplexed).
        // Worker currently creates own Client from URL.
        // We can reuse URL from settings.

        // Actually Worker uses `redis::Client` which is distinct from `deadpool_redis::Pool` if that's what Cache is,
        // but core_db Cache is likely `redis::Client`.
        // Let's check `core_db::infra::cache`.
        // core-db `Cache` is type alias for `redis::Client`.
        // So we can clone the client from context.redis?
        // `Worker::new` takes `redis::Client`? No, it takes `redis::Client` locally.
        // Let's use `context.settings.redis.url`.

        let redis = redis::Client::open(redis_url.as_str())?;
        let queue_prefix = context
            .settings
            .redis
            .prefix
            .as_ref()
            .map(|prefix| format!("{prefix}:queue"))
            .unwrap_or_else(|| "queue".to_string());

        let mut worker = Self {
            registry: Arc::new(HashMap::new()),
            queues: vec!["default"],
            redis,
            context: context.clone(),
            prefix: queue_prefix,
            sweeper_config: None,
            config: Some(WorkerInternalConfig {
                concurrency: config.concurrency,
                sweep_interval: std::time::Duration::from_secs(config.sweep_interval),
                redis_url: redis_url,
            }),
        };

        // Enable sweeper by default with config settings
        worker.enable_outbox_sweeper(
            context.db,
            std::time::Duration::from_secs(config.sweep_interval),
        );

        Ok(worker)
    }

    pub fn register<J: Job>(&mut self) {
        if let Some(map) = Arc::get_mut(&mut self.registry) {
            map.insert(J::NAME, Box::new(JobShim::<J>(std::marker::PhantomData)));
        } else {
            tracing::error!("Cannot register job after worker has been cloned/started");
        }

        if !self.queues.contains(&J::QUEUE) {
            self.queues.push(J::QUEUE);
        }
    }

    pub fn context(&self) -> &JobContext {
        &self.context
    }

    pub fn enable_outbox_sweeper(&mut self, db: sqlx::PgPool, interval: std::time::Duration) {
        self.sweeper_config = Some((db, interval));
    }

    pub async fn run(self) -> anyhow::Result<()> {
        let concurrency = self.config.as_ref().map(|c| c.concurrency).unwrap_or(10);
        self.run_concurrent(concurrency).await
    }

    // Internal run loop for a single thread
    async fn run_internal(self) -> anyhow::Result<()> {
        let client = self.redis.clone();
        let mut conn = client.get_multiplexed_async_connection().await?;

        loop {
            let mut keys = Vec::new();
            for q in &self.queues {
                // Standard queue: queue:default
                keys.push(format!("{}:{}", self.prefix, q));
                // Meta queue: queue:default:meta
                keys.push(format!("{}:{}:meta", self.prefix, q));
            }

            // BLPOP (blocking pop with timeout)
            let result: Option<(String, String)> = conn.blpop(&keys, 5.0).await.ok();

            if let Some((source_queue, payload_str)) = result {
                // Check if it's a Meta Queue (Ordered Group)
                if source_queue.ends_with(":meta") {
                    let group_id = payload_str; // Payload is just the group string

                    // Derive base queue name from source_queue (remove :meta)
                    // source_queue: queue:default:meta -> base: queue:default
                    let base_queue = source_queue.trim_end_matches(":meta");
                    let lock_key = format!("{}:lock:{}", base_queue, group_id);
                    let queue_name = base_queue
                        .strip_prefix(&format!("{}:", self.prefix))
                        .unwrap_or("default")
                        .to_string();

                    let group_list = format!("{}:{}", base_queue, group_id);

                    // Attempt Lock (SETNX with TTL 60s)
                    // If locked, another worker is draining it. We can ignore this signal.
                    let is_locked: bool = redis::cmd("SET")
                        .arg(&lock_key)
                        .arg("1")
                        .arg("NX")
                        .arg("EX")
                        .arg(60)
                        .query_async(&mut conn)
                        .await
                        .unwrap_or(false);

                    if is_locked {
                        tracing::info!("Locked group: {}, draining...", group_id);

                        // Drain Loop
                        loop {
                            let item: Option<String> = conn.lpop(&group_list, None).await.ok();
                            if let Some(payload_s) = item {
                                let mut wrapper: JobPayload = match serde_json::from_str(&payload_s)
                                {
                                    Ok(w) => w,
                                    Err(_) => continue, // Bad payload, drop
                                };

                                match self.process_wrapper(&wrapper).await {
                                    Ok(JobResult::Success) => {
                                        let _: () = redis::cmd("EXPIRE")
                                            .arg(&lock_key)
                                            .arg(60)
                                            .query_async(&mut conn)
                                            .await
                                            .unwrap_or(());
                                    }
                                    Ok(JobResult::Failure {
                                        backoff,
                                        max_retries,
                                        err,
                                    }) => {
                                        tracing::error!("Job failed (Group {}): {}", group_id, err);
                                        if wrapper.attempts < max_retries {
                                            tracing::info!("Retrying in {}s...", backoff);
                                            wrapper.attempts += 1;
                                            let new_payload = serde_json::to_string(&wrapper)?;

                                            // 1. Push back to HEAD (Maintain Order)
                                            let _: () = conn
                                                .lpush(&group_list, new_payload)
                                                .await
                                                .unwrap_or(());

                                            // 2. Schedule Delay (Pause Group)
                                            let now = std::time::SystemTime::now()
                                                .duration_since(std::time::UNIX_EPOCH)?
                                                .as_secs();
                                            let score = now + backoff;
                                            let signal = DelayedGroupSignal {
                                                queue: queue_name.clone(),
                                                group_id: group_id.clone(),
                                            };
                                            let signal_str = serde_json::to_string(&signal)?;
                                            let _: () = conn
                                                .zadd(
                                                    format!("{}:scheduler:groups", self.prefix),
                                                    signal_str,
                                                    score,
                                                )
                                                .await
                                                .unwrap_or(());

                                            // 3. Break (Release Lock)
                                            break;
                                        } else {
                                            tracing::error!(
                                                "Max retries reached for job in group {}",
                                                group_id
                                            );
                                            // Permanently Failed
                                            self.persist_failure(&wrapper, Some(&group_id), &err)
                                                .await;
                                        }
                                    }
                                    Err(e) => {
                                        tracing::error!("System error processing job: {}", e);
                                        // Push back?
                                        let _: () =
                                            conn.lpush(&group_list, payload_s).await.unwrap_or(());
                                        break;
                                    }
                                }
                            } else {
                                // Empty list
                                break;
                            }
                        }
                        // Unlock
                        let _: () = conn.del(&lock_key).await.unwrap_or(());
                        tracing::info!("Unlocked group: {}", group_id);
                    }
                } else {
                    // --- STANDARD QUEUE ---
                    let mut wrapper: JobPayload = match serde_json::from_str(&payload_str) {
                        Ok(w) => w,
                        Err(e) => {
                            tracing::error!("Bad payload: {}", e);
                            continue;
                        }
                    };

                    match self.process_wrapper(&wrapper).await {
                        Ok(JobResult::Success) => {}
                        Ok(JobResult::Failure {
                            backoff,
                            max_retries,
                            err,
                        }) => {
                            tracing::error!("Standard Job failed: {}", err);
                            if wrapper.attempts < max_retries {
                                wrapper.attempts += 1;
                                let new_payload = serde_json::to_string(&wrapper)?;
                                let now = std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)?
                                    .as_secs();
                                let score = now + backoff;
                                // ZADD to scheduler
                                // Key: queue:scheduler
                                let _: () = conn
                                    .zadd(format!("{}:scheduler", self.prefix), new_payload, score)
                                    .await
                                    .unwrap_or(());
                            } else {
                                // Permanently Failed
                                self.persist_failure(&wrapper, None, &err).await;
                            }
                        }
                        Err(e) => {
                            tracing::error!("System error: {}", e);
                        }
                    }
                }
            }
        }
    }

    async fn persist_failure(&self, wrapper: &JobPayload, group_id: Option<&str>, err: &str) {
        // Log to DB
        // Query: INSERT INTO failed_jobs ...

        let query = "INSERT INTO failed_jobs (job_name, queue, payload, error, attempts, group_id) VALUES ($1, $2, $3, $4, $5, $6)";

        // Payload as JSONB
        let payload_json = serde_json::to_value(wrapper).unwrap_or(serde_json::json!({}));

        if let Err(e) = sqlx::query(query)
            .bind(&wrapper.job)
            .bind(&wrapper.queue)
            .bind(payload_json)
            .bind(err)
            .bind(wrapper.attempts as i32)
            .bind(group_id)
            .execute(&self.context.db)
            .await
        {
            tracing::error!("Failed to persist job failure log: {}", e);
        }
    }

    async fn process_wrapper(&self, wrapper: &JobPayload) -> anyhow::Result<JobResult> {
        if let Some(handler) = self.registry.get(wrapper.job.as_str()) {
            tracing::info!(
                "Processing job: {} (Attempt {})",
                wrapper.job,
                wrapper.attempts
            );
            handler
                .execute(wrapper.data.clone(), &self.context, wrapper.attempts)
                .await
        } else {
            Err(anyhow::anyhow!("Unknown job type: {}", wrapper.job))
        }
    }

    pub async fn run_scheduler(self) -> anyhow::Result<()> {
        tracing::info!("Scheduler started");
        let client = self.redis.clone();
        let mut conn = client.get_multiplexed_async_connection().await?;
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));

        loop {
            interval.tick().await;
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs();

            // 1. Standard Jobs
            // ZRANGEBYSCORE queue:scheduler -inf now
            let key = format!("{}:scheduler", self.prefix);
            let jobs: Vec<String> = conn
                .zrangebyscore(&key, "-inf", now)
                .await
                .unwrap_or_default();

            if !jobs.is_empty() {
                // Remove from ZSET
                let _: () = conn.zrem(&key, &jobs).await.unwrap_or(());

                for j in jobs {
                    let queue_name = serde_json::from_str::<JobPayload>(&j)
                        .map(|wrapper| wrapper.queue)
                        .unwrap_or_else(|_| "default".to_string());
                    let dest = format!("{}:{}", self.prefix, queue_name);
                    let _: () = conn.rpush(&dest, j).await.unwrap_or(());
                }
            }

            // 2. Group Meta
            let group_key = format!("{}:scheduler:groups", self.prefix);
            let groups: Vec<String> = conn
                .zrangebyscore(&group_key, "-inf", now)
                .await
                .unwrap_or_default();

            if !groups.is_empty() {
                let _: () = conn.zrem(&group_key, &groups).await.unwrap_or(());

                for g in groups {
                    let signal = serde_json::from_str::<DelayedGroupSignal>(&g).unwrap_or(
                        DelayedGroupSignal {
                            queue: "default".to_string(),
                            group_id: g,
                        },
                    );
                    let dest = format!("{}:{}:meta", self.prefix, signal.queue);
                    let _: () = conn.rpush(&dest, signal.group_id).await.unwrap_or(());
                }
            }
        }
    }

    pub async fn run_concurrent(self, concurrency: usize) -> anyhow::Result<()> {
        tracing::info!(
            "Worker started. Queues: {:?}, Jobs: {:?}",
            self.queues,
            self.registry.keys()
        );
        let mut set = tokio::task::JoinSet::new();

        // Spawn Scheduler (Single)
        let s = self.clone();
        set.spawn(async move {
            if let Err(e) = s.run_scheduler().await {
                tracing::error!("Scheduler crashed: {}", e);
            }
        });

        // Spawn Outbox Sweeper (Single)
        if let Some((db, interval)) = self.sweeper_config.clone() {
            let client = self.redis.clone();
            let queue_prefix = self.prefix.clone();
            set.spawn(async move {
                tracing::info!("Outbox sweeper started");
                let queue = crate::queue::RedisQueue::from_client_with_prefix(client, queue_prefix);
                let mut interval_timer = tokio::time::interval(interval);
                loop {
                    interval_timer.tick().await;
                    match crate::buffer::OutboxFlusher::flush(&db, &queue).await {
                        Ok(n) if n > 0 => tracing::info!("Sweeper recovered {} jobs", n),
                        Ok(_) => {} // Empty
                        Err(e) => tracing::error!("Sweeper error: {}", e),
                    }
                }
            });
        }

        // Spawn Workers
        for i in 0..concurrency {
            let w = self.clone();
            set.spawn(async move {
                tracing::info!("Starting worker thread {}", i);
                if let Err(e) = w.run_internal().await {
                    tracing::error!("Worker thread {} crashed: {}", i, e);
                }
            });
        }

        while let Some(res) = set.join_next().await {
            // A worker finished (likely crashed or stopped).
            // Ideally restart it? For now just log.
            if let Err(e) = res {
                tracing::error!("Join error: {}", e);
            }
        }
        Ok(())
    }
}
