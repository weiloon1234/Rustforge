import { useEffect } from 'react'
import Prism from 'prismjs'

export function Chapter3JobsUsage() {
    useEffect(() => {
        Prism.highlightAll()
    }, [])

    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">
                    Chapter 3: Job Creation and Usage Recipe
                </h1>
                <p className="text-xl text-gray-500">
                    Create a job, register it, dispatch from API, and verify worker execution with
                    the default embedded worker flow.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>Step 0: Scope and Defaults</h2>
                <ul>
                    <li>
                        This chapter assumes Chapter 1 API base already exists under{' '}
                        <code>/api/v1</code>.
                    </li>
                    <li>
                        Framework baseline migrations already include <code>failed_jobs</code> and{' '}
                        <code>outbox_jobs</code>.
                    </li>
                    <li>
                        API server can run embedded worker when <code>RUN_WORKER=true</code>.
                    </li>
                </ul>

                <h2>Step 1: Create Job Definition File</h2>
                <h3>
                    File: <code>app/src/internal/jobs/definitions/mod.rs</code>
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`pub mod rebuild_article_index;`}</code>
                </pre>

                <h3>
                    File: <code>app/src/internal/jobs/definitions/rebuild_article_index.rs</code>
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-rust">{`use core_jobs::{Job, JobContext};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RebuildArticleIndexJob {
    pub article_id: i64,
}

#[async_trait::async_trait]
impl Job for RebuildArticleIndexJob {
    const NAME: &'static str = "article.rebuild_index";
    const QUEUE: &'static str = "default";

    async fn handle(&self, ctx: &JobContext) -> anyhow::Result<()> {
        let _db = &ctx.db;
        println!("[job] rebuild article index: article_id={}", self.article_id);
        Ok(())
    }

    fn max_retries(&self) -> u32 {
        3
    }
}`}</code>
                </pre>

                <h3>
                    File: <code>app/Cargo.toml</code> (dependencies)
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-toml">{`[dependencies]
anyhow = { workspace = true }
core-jobs = { workspace = true }
bootstrap = { workspace = true }
serde = { workspace = true, features = ["derive"] }
async-trait = { workspace = true }`}</code>
                </pre>

                <h2>Step 2: Register Job in Worker</h2>
                <h3>
                    File: <code>app/src/internal/jobs/mod.rs</code>
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-rust">{`use core_jobs::worker::Worker;

pub mod definitions;

pub fn register_jobs(worker: &mut Worker) {
    worker.register::<definitions::rebuild_article_index::RebuildArticleIndexJob>();
}

#[allow(unused_variables)]
pub fn register_schedules(scheduler: &mut core_jobs::cron::Scheduler) {}

pub async fn run_worker(ctx: bootstrap::boot::BootContext) -> anyhow::Result<()> {
    bootstrap::jobs::start_with_context(ctx, register_jobs, Some(register_schedules)).await
}`}</code>
                </pre>

                <h2>Step 3: Ensure API State Has Queue</h2>
                <h3>
                    File: <code>app/src/internal/api/state.rs</code>
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-rust">{`use core_jobs::queue::RedisQueue;
use core_web::{error::AppError, response::ApiResponse};
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppApiState {
    pub db: PgPool,
    pub queue: RedisQueue,
    pub cdn_base: Option<String>,
}

impl AppApiState {
    pub fn new(ctx: &bootstrap::boot::BootContext) -> Self {
        Self {
            db: ctx.db.clone(),
            queue: ctx.queue.clone(),
            cdn_base: ctx.settings.cdn.base_url.clone(),
        }
    }
}

pub type ApiResult<T> = Result<ApiResponse<T>, AppError>;`}</code>
                </pre>

                <h2>Step 4: Create Dispatch Endpoint</h2>
                <h3>
                    File: <code>app/src/internal/api/v1/article_job.rs</code>
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`use axum::{
    extract::{Path, State},
    routing::post,
    Router,
};
use core_i18n::t;
use core_jobs::Job;
use core_web::response::ApiResponse;
use jobs::definitions::rebuild_article_index::RebuildArticleIndexJob;

use crate::internal::api::state::{ApiResult, AppApiState};

pub fn router() -> Router<AppApiState> {
    Router::new().route("/articles/{id}/rebuild-index", post(dispatch_rebuild_index))
}

async fn dispatch_rebuild_index(
    State(state): State<AppApiState>,
    Path(id): Path<i64>,
) -> ApiResult<serde_json::Value> {
    RebuildArticleIndexJob { article_id: id }
        .dispatch(&state.queue)
        .await?;

    Ok(ApiResponse::success(
        serde_json::json!({ "queued": true, "article_id": id }),
        &t("Rebuild job queued"),
    ))
}`}</code>
                </pre>

                <h3>
                    File: <code>app/src/internal/api/v1/mod.rs</code>
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`use core_web::openapi::ApiRouter;

use crate::internal::api::state::AppApiState;

pub mod article;
pub mod article_category;
pub mod article_job;

pub fn router(state: AppApiState) -> ApiRouter {
    ApiRouter::new()
        .merge(article_category::router())
        .merge(article::router())
        .merge(article_job::router())
        .with_state(state)
}`}</code>
                </pre>

                <h2>Step 5: Optional Transactional Dispatch (Outbox)</h2>
                <p>
                    If dispatch must commit together with DB changes, use <code>JobBuffer</code>{' '}
                    in the same transaction.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-rust">{`use core_jobs::buffer::JobBuffer;
use jobs::definitions::rebuild_article_index::RebuildArticleIndexJob;

let mut tx = state.db.begin().await?;

// 1) domain writes in tx
// sqlx::query(...).execute(&mut *tx).await?;

// 2) queue intent in outbox_jobs (same tx)
let mut buffer = JobBuffer::new(&mut tx);
buffer
    .push(RebuildArticleIndexJob { article_id: id })
    .await?;

// 3) single commit
tx.commit().await?;`}</code>
                </pre>

                <h2>Step 6: Run and Verify</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-bash">{`# start API + embedded worker
RUN_WORKER=true ./bin/api-server

# queue job via API
curl -X POST http://127.0.0.1:3000/api/v1/user/articles/1/rebuild-index

# expected:
# - HTTP response data.queued = true
# - server log: [job] rebuild article index: article_id=1`}</code>
                </pre>

                <h2>Chapter Decision Rule</h2>
                <ul>
                    <li>
                        Use direct <code>job.dispatch(&queue)</code> for non-transactional, fast
                        enqueue.
                    </li>
                    <li>
                        Use <code>JobBuffer</code> when enqueue must succeed or fail with the same DB
                        transaction.
                    </li>
                </ul>
            </div>
        </div>
    )
}
