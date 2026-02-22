import { useEffect } from 'react'
import Prism from 'prismjs'

export function Chapter12EventFanOutRecipe() {
    useEffect(() => {
        Prism.highlightAll()
    }, [])

    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">
                    Chapter 12: Event Fan-Out Recipe
                </h1>
                <p className="text-xl text-gray-500">
                    Use Jobs + Notify as your event system: one fan-out job dispatches to multiple
                    channels (email, realtime, webhook) with idempotency and failure alerting.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>Step 0: Scope</h2>
                <ul>
                    <li>
                        In Rustforge, Jobs + Notify <strong>IS</strong> the event system.
                    </li>
                    <li>
                        "Event" = dispatching a fan-out job.
                    </li>
                    <li>
                        "Listeners" = the job handler logic that fans out to channels.
                    </li>
                    <li>
                        No separate Event/Listener abstraction needed.
                    </li>
                    <li>
                        This chapter builds on Chapter 3 (Jobs) and Chapter 4 (Notifications).
                    </li>
                </ul>

                <h2>Step 1: Define Fan-Out Job</h2>
                <h3>
                    File: <code>app/src/internal/jobs/definitions/order_placed_fanout.rs</code>
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-rust">{`use core_jobs::{Job, JobContext};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderPlacedFanoutJob {
    pub order_id: i64,
    pub customer_email: String,
    pub order_total: f64,
}

#[async_trait::async_trait]
impl Job for OrderPlacedFanoutJob {
    const NAME: &'static str = "order.placed.fanout";
    const QUEUE: &'static str = "default";

    async fn handle(&self, ctx: &JobContext) -> anyhow::Result<()> {
        // Fan out to each channel independently
        // Partial failure in one channel should not block others

        // 1. Send confirmation email
        if let Err(e) = self.send_confirmation_email(ctx).await {
            tracing::error!("Email channel failed for order {}: {}", self.order_id, e);
        }

        // 2. Push realtime event
        if let Err(e) = self.push_realtime_event(ctx).await {
            tracing::error!("Realtime channel failed for order {}: {}", self.order_id, e);
        }

        // 3. Call external webhook
        if let Err(e) = self.call_webhook(ctx).await {
            tracing::error!("Webhook channel failed for order {}: {}", self.order_id, e);
        }

        Ok(())
    }

    fn max_retries(&self) -> u32 {
        3
    }

    async fn failed(&self, _ctx: &JobContext, error: &str) -> anyhow::Result<()> {
        tracing::error!(
            "ALERT: Order {} fan-out permanently failed: {}",
            self.order_id, error
        );
        // Send alert to ops Slack channel, PagerDuty, etc.
        Ok(())
    }
}`}</code>
                </pre>

                <h2>Step 2: Implement Channel Methods</h2>
                <h3>
                    File: <code>app/src/internal/jobs/definitions/order_placed_fanout.rs</code> (same file, impl block)
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-rust">{`impl OrderPlacedFanoutJob {
    async fn send_confirmation_email(&self, ctx: &JobContext) -> anyhow::Result<()> {
        use core_notify::mail::MailChannel;

        let mail = OrderConfirmationMail {
            to: self.customer_email.clone(),
            order_id: self.order_id,
            total: self.order_total,
        };

        MailChannel::dispatch_now(&mail, ctx).await
    }

    async fn push_realtime_event(&self, ctx: &JobContext) -> anyhow::Result<()> {
        use core_realtime::publisher::RealtimePublisher;

        let publisher = ctx.extensions.get::<RealtimePublisher>()
            .ok_or_else(|| anyhow::anyhow!("RealtimePublisher not in extensions"))?;

        publisher.publish(
            "orders",
            serde_json::json!({
                "event": "order.placed",
                "order_id": self.order_id,
                "total": self.order_total,
            }),
        ).await
    }

    async fn call_webhook(&self, ctx: &JobContext) -> anyhow::Result<()> {
        let url = ctx.settings.webhooks.order_placed_url.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Webhook URL not configured"))?;

        let client = reqwest::Client::new();
        client.post(url)
            .header("X-Idempotency-Key", format!("order-{}", self.order_id))
            .json(&serde_json::json!({
                "order_id": self.order_id,
                "total": self.order_total,
            }))
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}`}</code>
                </pre>

                <h2>Step 3: Register + Transactional Dispatch</h2>
                <h3>
                    File: <code>app/src/internal/jobs/mod.rs</code> — register the job
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-rust">{`pub fn register_jobs(worker: &mut Worker) {
    worker.register::<definitions::order_placed_fanout::OrderPlacedFanoutJob>();
    // ... other jobs
}`}</code>
                </pre>
                <h3>
                    File: <code>app/src/internal/workflows/order/create_order.rs</code> — dispatch with outbox
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-rust">{`use core_jobs::buffer::JobBuffer;
use crate::internal::jobs::definitions::order_placed_fanout::OrderPlacedFanoutJob;

let mut tx = state.db.begin().await?;

// 1. Insert order
let order = Order::new(&mut *tx)
    .insert()
    .set_customer_email(&input.email)
    .set_total(input.total)
    .save()
    .await?;

// 2. Queue fan-out (same transaction)
let mut buffer = JobBuffer::new(&mut tx);
buffer.push(OrderPlacedFanoutJob {
    order_id: order.id,
    customer_email: input.email.clone(),
    order_total: input.total,
}).await?;

// 3. Single commit — if this fails, neither order nor job persists
tx.commit().await?;`}</code>
                </pre>

                <h2>Step 4: Idempotency Per Channel</h2>
                <p>
                    If the fan-out job retries (because <code>handle()</code> returned{' '}
                    <code>Err</code>), channels that already succeeded will be called again.
                    Each channel <strong>MUST</strong> handle duplicates gracefully.
                </p>
                <ul>
                    <li>
                        <strong>Email:</strong> Use <code>order_id</code> as dedup key — check
                        the <code>sent_notifications</code> table before sending, or use the
                        provider's idempotency key.
                    </li>
                    <li>
                        <strong>Realtime:</strong> Client-side dedup via event ID
                        (<code>order_id</code> + event type) — clients ignore duplicate events.
                    </li>
                    <li>
                        <strong>Webhook:</strong> Include <code>X-Idempotency-Key</code> header
                        — the receiver must handle duplicate POSTs.
                    </li>
                </ul>
                <p>
                    <strong>Key rule:</strong> At-least-once delivery means duplicates are normal.
                    Every channel must be idempotent by design.
                </p>

                <h2>Step 5: Verify</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-bash">{`# Create order (triggers fan-out)
curl -X POST http://127.0.0.1:3000/api/v1/user/orders \\
  -H 'Content-Type: application/json' \\
  -d '{"email":"customer@example.com","total":99.99}'

# Check: email sent (check mailer logs or mailbox)
# Check: realtime event (connect WebSocket client to orders channel)
# Check: webhook called (check external service or request bin)
# Check: no failed jobs
psql -c "SELECT * FROM failed_jobs WHERE job_name = 'order.placed.fanout'"`}</code>
                </pre>

                <h2>Chapter Decision Rule</h2>
                <ul>
                    <li>
                        <strong>Direct dispatch:</strong> Single side effect (just send email)
                        — dispatch the specific job directly.
                    </li>
                    <li>
                        <strong>Fan-out job:</strong> 2+ independent side effects from one event
                        — create one fan-out job.
                    </li>
                    <li>
                        <strong>Separate jobs:</strong> Side effects have different retry/backoff
                        needs — dispatch separate jobs from the workflow.
                    </li>
                    <li>
                        The <code>failed()</code> callback is for alerting only — don't retry
                        logic there.
                    </li>
                    <li>
                        Always make channels idempotent — at-least-once delivery means duplicates
                        are normal.
                    </li>
                </ul>
            </div>
        </div>
    )
}
