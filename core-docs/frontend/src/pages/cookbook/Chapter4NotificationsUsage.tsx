import { useEffect } from 'react'
import Prism from 'prismjs'

export function Chapter4NotificationsUsage() {
    useEffect(() => {
        Prism.highlightAll()
    }, [])

    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">
                    Chapter 4: Notifications Recipe (Now + Queued)
                </h1>
                <p className="text-xl text-gray-500">
                    Build a minimal notification flow using <code>core_notify</code> +
                    <code>core_mailer</code>, then choose immediate send or queued delivery.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>Step 0: Scope and Defaults</h2>
                <ul>
                    <li>
                        This chapter uses built-in <code>MailChannel</code> from{' '}
                        <code>core_notify::channel</code>.
                    </li>
                    <li>
                        Queued mail uses <code>core_mailer::SendMailJob</code>; worker must
                        register that job.
                    </li>
                    <li>
                        API examples stay under <code>/api/v1</code>.
                    </li>
                </ul>

                <h2>Step 1: Define Notification Payload and Notifiable</h2>
                <h3>
                    File: <code>app/src/internal/notifications/channels.rs</code>
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-rust">{`use core_notify::{Mailable, Notifiable};
use core_mailer::MailPayload;

pub struct WelcomeMail {
    pub app_name: String,
}

impl Mailable for WelcomeMail {
    fn to_mail(&self, notifiable: &dyn Notifiable) -> Option<MailPayload> {
        Some(MailPayload {
            to: vec![notifiable.email()?],
            subject: format!("Welcome to {}", self.app_name),
            body: "<p>Thanks for joining.</p>".to_string(),
        })
    }
}

pub struct MailUser {
    pub id: String,
    pub email: String,
}

impl Notifiable for MailUser {
    fn route_notification_for(&self, driver: &str) -> Option<String> {
        if driver == "mail" {
            Some(self.email.clone())
        } else {
            None
        }
    }

    fn id(&self) -> String {
        self.id.clone()
    }
}`}</code>
                </pre>

                <h2>Step 2: Register Mail Queue Job for Worker</h2>
                <h3>
                    File: <code>app/src/internal/notifications/mod.rs</code>
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-rust">{`use core_jobs::worker::Worker;

pub mod channels;
pub mod jobs;

pub fn register_jobs(worker: &mut Worker) {
    worker.register::<core_mailer::SendMailJob>();
}`}</code>
                </pre>

                <h3>
                    File: <code>app/Cargo.toml</code> (dependency)
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-toml">{`[dependencies]
core-notify = { workspace = true }
core-mailer = { workspace = true }`}</code>
                </pre>

                <h3>
                    File: <code>app/src/internal/jobs/mod.rs</code>
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-rust">{`use core_jobs::worker::Worker;

pub mod definitions;

pub fn register_jobs(worker: &mut Worker) {
    worker.register::<definitions::rebuild_article_index::RebuildArticleIndexJob>();
    crate::internal::notifications::register_jobs(worker);
}`}</code>
                </pre>

                <h2>Step 3: Add Notification Workflow</h2>
                <h3>
                    File: <code>app/src/internal/workflows/notification.rs</code>
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`use anyhow::Result;
use core_mailer::Mailer;
use core_notify::channel::MailChannel;
use std::sync::Arc;

use crate::internal::notifications::channels::{MailUser, WelcomeMail};

pub async fn send_welcome_now(mailer: &Arc<Mailer>, user_id: String, email: String) -> Result<()> {
    let user = MailUser { id: user_id, email };
    let mail = WelcomeMail {
        app_name: "Rustforge".to_string(),
    };

    MailChannel::dispatch_now(mailer.as_ref(), &user, &mail).await
}

pub async fn send_welcome_queued(
    mailer: &Arc<Mailer>,
    user_id: String,
    email: String,
) -> Result<()> {
    let user = MailUser { id: user_id, email };
    let mail = WelcomeMail {
        app_name: "Rustforge".to_string(),
    };

    MailChannel::dispatch(mailer.as_ref(), &user, &mail).await
}`}</code>
                </pre>

                <h2>Step 4: Add API Endpoint</h2>
                <h3>
                    File: <code>app/src/internal/api/state.rs</code> (include mailer)
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-rust">{`#[derive(Clone)]
pub struct AppApiState {
    pub db: sqlx::PgPool,
    pub queue: core_jobs::queue::RedisQueue,
    pub mailer: std::sync::Arc<core_mailer::Mailer>,
    pub cdn_base: Option<String>,
}

impl AppApiState {
    pub fn new(ctx: &bootstrap::boot::BootContext) -> Self {
        Self {
            db: ctx.db.clone(),
            queue: ctx.queue.clone(),
            mailer: ctx.mailer.clone(),
            cdn_base: ctx.settings.cdn.base_url.clone(),
        }
    }
}`}</code>
                </pre>

                <h3>
                    File: <code>app/src/internal/api/v1/notification.rs</code>
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`use axum::{extract::State, routing::post, Json, Router};
use core_i18n::t;
use core_web::{extract::validated_json::ValidatedJson, response::ApiResponse};
use schemars::JsonSchema;
use serde::Deserialize;
use validator::Validate;

use crate::internal::api::state::{ApiResult, AppApiState};
use crate::internal::workflows;

pub fn router() -> Router<AppApiState> {
    Router::new().route("/notifications/welcome", post(send_welcome))
}

#[derive(Debug, Deserialize, Validate, JsonSchema)]
pub struct SendWelcomeRequest {
    pub user_id: String,
    #[validate(email)]
    pub email: String,
    #[serde(default)]
    pub queued: bool,
}

async fn send_welcome(
    State(state): State<AppApiState>,
    ValidatedJson(req): ValidatedJson<SendWelcomeRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    if req.queued {
        workflows::notification::send_welcome_queued(&state.mailer, req.user_id, req.email).await?;
    } else {
        workflows::notification::send_welcome_now(&state.mailer, req.user_id, req.email).await?;
    }

    Ok(ApiResponse::success(
        Json(serde_json::json!({ "sent": true, "queued": req.queued })),
        &t("Welcome notification dispatched"),
    ))
}`}</code>
                </pre>

                <h2>Step 5: Wire Router</h2>
                <h3>
                    File: <code>app/src/internal/api/v1/mod.rs</code>
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`pub mod notification;

pub fn router(state: AppApiState) -> Router {
    Router::new()
        .merge(notification::router())
        // merge other v1 routers
        .with_state(state)
}`}</code>
                </pre>

                <h2>Step 6: Verify</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-bash">{`# immediate send
curl -X POST http://127.0.0.1:3000/api/v1/notifications/welcome \
  -H 'Content-Type: application/json' \
  -d '{"user_id":"u-1","email":"demo@example.com","queued":false}'

# queued send (requires RUN_WORKER=true)
curl -X POST http://127.0.0.1:3000/api/v1/notifications/welcome \
  -H 'Content-Type: application/json' \
  -d '{"user_id":"u-2","email":"demo@example.com","queued":true}'`}</code>
                </pre>
            </div>
        </div>
    )
}
