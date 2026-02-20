#![allow(dead_code)] // For unused code during development
use crate::{queue::RedisQueue, Job};
use chrono::Timelike; // Needed for with_nanosecond
use cron::Schedule as CronSchedule;
use redis::AsyncCommands;
use std::str::FromStr;
use std::sync::Arc;
use tokio::time::Duration;

#[derive(Clone)]
pub struct Scheduler {
    redis: RedisQueue,
    tasks: Vec<Task>,
}

#[derive(Clone)]
struct Task {
    name: String,
    expression: String, // Cron expression
    job_factory: Arc<dyn Fn() -> Box<dyn JobWrapper> + Send + Sync>,
    condition: Option<Arc<dyn Fn() -> bool + Send + Sync>>,
    without_overlapping_ttl: Option<usize>, // TTL in seconds
}

trait JobWrapper: Send + Sync {
    fn name(&self) -> &'static str;
    fn queue_name(&self) -> &'static str;
    fn to_json(&self) -> String;
}

impl<J: Job> JobWrapper for J {
    fn name(&self) -> &'static str {
        J::NAME
    }
    fn queue_name(&self) -> &'static str {
        J::QUEUE
    }
    fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

impl Scheduler {
    pub fn new(queue: RedisQueue) -> Self {
        Self {
            redis: queue,
            tasks: Vec::new(),
        }
    }

    /// Schedule a job with a cron expression.
    /// Example: "0 * * * * *" (Every minute)
    pub fn cron<J: Job + Default + Clone>(&mut self, expression: &str) -> &mut Self {
        let expression = expression.to_string();
        let job_factory = Arc::new(|| Box::new(J::default()) as Box<dyn JobWrapper>);

        self.tasks.push(Task {
            name: J::NAME.to_string(),
            expression,
            job_factory,
            condition: None,
            without_overlapping_ttl: None,
        });
        self
    }

    /// Add a condition to the last scheduled job.
    pub fn when<F>(&mut self, f: F) -> &mut Self
    where
        F: Fn() -> bool + Send + Sync + 'static,
    {
        if let Some(task) = self.tasks.last_mut() {
            task.condition = Some(Arc::new(f));
        }
        self
    }

    /// Prevent the job from overlapping with previous runs (using a lock with TTL).
    /// Note: This relies on TTL expiration if not cleared manually.
    /// Real "running" check requires Worker support.
    pub fn without_overlapping(&mut self, ttl_seconds: usize) -> &mut Self {
        if let Some(task) = self.tasks.last_mut() {
            task.without_overlapping_ttl = Some(ttl_seconds);
        }
        self
    }

    /// Schedule a job to run every N seconds (Approximation via cron?).
    /// Cron doesn't support "Every 45 seconds" effectively if not aligned.
    /// For "Every N minutes", cron works.
    /// For simple interval, we might need custom logic.
    /// For now, let's stick to CRON syntax as primary.

    pub async fn run(self) -> anyhow::Result<()> {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        let client = self.redis.client.clone();
        let mut conn = client.get_multiplexed_async_connection().await?;
        let prefix = self.redis.prefix.clone();

        loop {
            interval.tick().await;
            let now = chrono::Utc::now();
            tracing::info!("Scheduler Tick: {}", now);

            for task in &self.tasks {
                let schedule = match CronSchedule::from_str(&task.expression) {
                    Ok(s) => s,
                    Err(e) => {
                        tracing::error!("Invalid cron expression for {}: {}", task.name, e);
                        continue; // Skip
                    }
                };

                // Check if we should have run recently
                // Logic: Find the *previous* scheduled time. If it matches current minute (ignoring seconds), run it.
                // Or better: Is there an upcoming run in this minute?
                // `upcoming(Utc).next()` gives next run.
                // If `next_run` is close to `now`?

                // Typical Cron Loop Logic:
                // Wake up every minute (at :00).
                // Check if task matches this minute.
                // Cron crate `parse` gives explicit dates.
                // Does `schedule.includes(now)` exist? No.
                // `schedule.upcoming(Utc)`

                // Alternative:
                // Get the last run time?
                // Just check if `upcoming` is effectively `now` (we just ticked).
                // Or check if `schedule.after(now - 1 min).next() == now`?

                // Simpler check:
                // Is `now` inside the set of matching times?
                // `cron` crate doesn't easily return "is match".
                // But we can iterate `upcoming` from `now - 1 minute`.
                // If yield `dt` such that `dt.minute() == now.minute()`, then yes.

                // Let's rely on lock to deduplicate anyway.
                // Lock Key: `scheduler:run:{job_name}:{YYYY-MM-DD-HH-MM}`

                // Check if current minute is a match.
                // We construct a date iterator starting from slightly in past?
                // `schedule.after(&last_tick)`?

                // Let's assume we run exactly on minute boundary.
                // Find nearest upcoming event.
                // Check if current minute is a match.
                // We rely on cron iteration to see if the "current minute" (00 seconds) matches the schedule.
                let last_minute = now.with_nanosecond(0).unwrap().with_second(0).unwrap();
                // Check if `last_minute` is in the schedule.
                // Hack: `schedule.after(last_minute - 1ns).next()` == `last_minute`?

                if let Some(upcoming) = schedule
                    .after(&(last_minute - chrono::Duration::seconds(1)))
                    .next()
                {
                    if upcoming == last_minute {
                        // 1. Check Condition
                        if let Some(cond) = &task.condition {
                            if !cond() {
                                tracing::info!("Skipping {}: Condition false", task.name);
                                continue;
                            }
                        }

                        // 2. Check Overlapping Lock (if configured)
                        if let Some(ttl) = task.without_overlapping_ttl {
                            let overlap_key = format!("{}:cron:overlap:{}", prefix, task.name);
                            // Check if exists
                            let exists: bool = redis::cmd("EXISTS")
                                .arg(&overlap_key)
                                .query_async(&mut conn)
                                .await
                                .unwrap_or(false);
                            if exists {
                                tracing::info!("Skipping {}: Overlap lock exists", task.name);
                                continue;
                            }

                            // Set Overlap Lock
                            let _: () = redis::cmd("SET")
                                .arg(&overlap_key)
                                .arg("1")
                                .arg("EX")
                                .arg(ttl)
                                .query_async(&mut conn)
                                .await
                                .unwrap_or(());
                        }

                        // IT IS TIME.
                        let lock_key =
                            format!("{}:cron:{}:{}", prefix, task.name, last_minute.timestamp());

                        // Try Lock (Deduplication)
                        let is_locked: bool = redis::cmd("SET")
                            .arg(&lock_key)
                            .arg("1")
                            .arg("NX")
                            .arg("EX")
                            .arg(120) // 2 mins expire
                            .query_async(&mut conn)
                            .await
                            .unwrap_or(false);

                        if is_locked {
                            tracing::info!("Enqueueing Scheduled Task: {}", task.name);
                            // Enqueue Job
                            let job_instance = (task.job_factory)();
                            // Access queue methods directly?
                            // Need `push_raw`? Or reconstruct job wrapper?
                            // `RedisQueue::push` expects `&J`.
                            // I only have `Box<dyn JobWrapper>`.
                            // `JobWrapper` has `to_json()`.
                            // I can manually push the payload using `to_json`.
                            // `JobPayload` struct? I need to use `JobPayload` to be compatible with Worker.

                            // Let's expose `push_raw` in `RedisQueue` or `push_wrapper`.
                            // Or use `to_json` to get inner data, then wrap in `JobPayload`.

                            let payload = crate::JobPayload {
                                job: task.name.clone(),
                                data: serde_json::from_str(&job_instance.to_json())?, // Reparse? Inefficient but safe.
                                queue: job_instance.queue_name().to_string(),
                                attempts: 0,
                            };
                            let payload_str = serde_json::to_string(&payload)?;
                            let queue_key = format!("{}:{}", prefix, job_instance.queue_name());
                            let _: () = conn.rpush(queue_key, payload_str).await?;
                        }
                    }
                }
            }
        }
    }
}
