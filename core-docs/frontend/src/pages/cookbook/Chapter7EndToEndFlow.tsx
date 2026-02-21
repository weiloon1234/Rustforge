import { useEffect } from 'react'
import Prism from 'prismjs'

export function Chapter7EndToEndFlow() {
    useEffect(() => {
        Prism.highlightAll()
    }, [])

    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">
                    Chapter 8: End-to-End Recipe (API -&gt; Job -&gt; Notification -&gt; WebSocket)
                </h1>
                <p className="text-xl text-gray-500">
                    One full process recipe: publish article from API, enqueue fanout job, send
                    notification, and push realtime event.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>Step 0: Preconditions</h2>
                <ul>
                    <li>Chapter 3 (jobs), Chapter 4 (notifications), Chapter 5/6 (websocket) are ready.</li>
                    <li>
                        API process runs with <code>RUN_WORKER=true</code> so queued jobs execute.
                    </li>
                    <li>
                        WebSocket server is running and channel policy for <code>admin_notifications</code>{' '}
                        is configured.
                    </li>
                </ul>

                <h2>Step 1: Add Fanout Job</h2>
                <h3>
                    File: <code>app/src/internal/jobs/definitions/mod.rs</code>
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`pub mod article_published_fanout;
pub mod rebuild_article_index;`}</code>
                </pre>

                <h3>
                    File: <code>app/src/internal/jobs/definitions/article_published_fanout.rs</code>
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`use core_jobs::{Job, JobContext};
use core_notify::channel::MailChannel;
use core_realtime::RealtimePublisher;
use notifications::channels::{MailUser, WelcomeMail};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticlePublishedFanoutJob {
    pub article_id: i64,
    pub tenant_id: String,
    pub recipient_user_id: String,
    pub recipient_email: String,
}

#[async_trait::async_trait]
impl Job for ArticlePublishedFanoutJob {
    const NAME: &'static str = "article.published_fanout";
    const QUEUE: &'static str = "default";

    async fn handle(&self, ctx: &JobContext) -> anyhow::Result<()> {
        // 1) Notify mail
        let mailer = core_mailer::Mailer::from_settings(&ctx.settings.mail)?;
        let user = MailUser {
            id: self.recipient_user_id.clone(),
            email: self.recipient_email.clone(),
        };
        let mail = WelcomeMail {
            app_name: "Rustforge".to_string(),
        };
        MailChannel::dispatch_now(&mailer, &user, &mail).await?;

        // 2) Push realtime event
        let publisher = RealtimePublisher::from_realtime_settings(
            &ctx.settings.redis.url,
            &ctx.settings.realtime,
        )?;
        publisher
            .publish_raw(
                "admin_notifications",
                "article_published",
                Some(&format!("tenant:{}", self.tenant_id)),
                serde_json::json!({
                    "article_id": self.article_id,
                    "recipient_user_id": self.recipient_user_id,
                }),
            )
            .await?;

        Ok(())
    }
}`}</code>
                </pre>

                <h2>Step 2: Register Job in Worker</h2>
                <h3>
                    File: <code>app/Cargo.toml</code> (dependencies)
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-toml">{`[dependencies]
core-realtime = { workspace = true }
core-notify = { workspace = true }
core-mailer = { workspace = true }
serde = { workspace = true, features = ["derive"] }
serde_json = { workspace = true }
async-trait = { workspace = true }`}</code>
                </pre>

                <h3>
                    File: <code>app/src/internal/jobs/mod.rs</code>
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-rust">{`use core_jobs::worker::Worker;

pub mod definitions;

pub fn register_jobs(worker: &mut Worker) {
    worker.register::<definitions::rebuild_article_index::RebuildArticleIndexJob>();
    worker.register::<definitions::article_published_fanout::ArticlePublishedFanoutJob>();
    crate::internal::notifications::register_jobs(worker);
}`}</code>
                </pre>

                <h2>Step 3: Dispatch Fanout Job from API Publish Action</h2>
                <h3>
                    File: <code>app/src/api/v1/article_publish.rs</code>
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
use jobs::definitions::article_published_fanout::ArticlePublishedFanoutJob;

use crate::api::state::{ApiResult, AppApiState};

pub fn router() -> Router<AppApiState> {
    Router::new().route("/articles/{id}/publish", post(publish_article))
}

async fn publish_article(
    State(state): State<AppApiState>,
    Path(id): Path<i64>,
) -> ApiResult<serde_json::Value> {
    // 1) your domain publish workflow here (set status=published, etc.)

    // 2) queue fanout work
    ArticlePublishedFanoutJob {
        article_id: id,
        tenant_id: "1".to_string(),
        recipient_user_id: "admin-1".to_string(),
        recipient_email: "admin@example.com".to_string(),
    }
    .dispatch(&state.queue)
    .await?;

    Ok(ApiResponse::success(
        serde_json::json!({ "published": true, "article_id": id, "fanout_queued": true }),
        &t("Article published"),
    ))
}`}</code>
                </pre>

                <h3>
                    File: <code>app/src/api/v1/mod.rs</code>
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`pub mod article_publish;

pub fn router(state: AppApiState) -> Router {
    Router::new()
        .merge(article_publish::router())
        // merge other modules
        .with_state(state)
}`}</code>
                </pre>

                <h2>Step 4: Run Processes</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-bash">{`# terminal 1: websocket server
./bin/websocket-server

# terminal 2: api + embedded worker
RUN_WORKER=true ./bin/api-server`}</code>
                </pre>

                <h2>Step 5: Verify End-to-End</h2>
                <p>
                    Open websocket client and authenticate (Chapter 6 flow) to channel
                    <code>admin_notifications</code> room <code>tenant:1</code>.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-javascript">{`const ws = new WebSocket('ws://127.0.0.1:3010/ws');
ws.onopen = () => {
  ws.send(JSON.stringify({ op: 'auth', token: accessToken }));
  ws.send(JSON.stringify({
    op: 'subscribe',
    channel: 'admin_notifications',
    room: 'tenant:1'
  }));
};
ws.onmessage = (evt) => console.log(JSON.parse(evt.data));`}</code>
                </pre>

                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-bash">{`curl -X POST http://127.0.0.1:3000/api/v1/articles/1/publish`}</code>
                </pre>

                <p>Expected result:</p>
                <ul>
                    <li>HTTP returns <code>fanout_queued: true</code>.</li>
                    <li>Worker runs <code>ArticlePublishedFanoutJob</code>.</li>
                    <li>Email send/log is produced by mailer.</li>
                    <li>WebSocket client receives <code>article_published</code> event.</li>
                </ul>
            </div>
        </div>
    )
}
