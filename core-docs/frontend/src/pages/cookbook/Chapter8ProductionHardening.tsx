import { useEffect } from 'react'
import Prism from 'prismjs'

export function Chapter8ProductionHardening() {
    useEffect(() => {
        Prism.highlightAll()
    }, [])

    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">
                    Chapter 9: Production Hardening Recipe
                </h1>
                <p className="text-xl text-gray-500">
                    Harden the end-to-end flow with idempotency, retries, failed-job recovery,
                    replay-gap handling, and observability.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>Step 0: Scope</h2>
                <ul>
                    <li>Use this after Chapter 8 end-to-end flow is working.</li>
                    <li>
                        Focus here is operational safety, not new domain features.
                    </li>
                </ul>

                <h2>Step 1: Add Realtime Idempotency at Consumer Side</h2>
                <h3>
                    File: <code>app/src/workflows/realtime_consumer.rs</code>
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`use core_realtime::RealtimeIdempotency;

pub async fn consume_article_event(
    redis_url: &str,
    delivery_id: &str,
    article_id: i64,
) -> anyhow::Result<()> {
    let idem = RealtimeIdempotency::new(redis_url)?;

    // Skip duplicate deliveries within TTL window.
    let first = idem
        .claim_once("article_published_consumer", delivery_id)
        .await?;
    if !first {
        return Ok(());
    }

    // Side effects here (write DB, trigger webhooks, etc.)
    println!("process article event once: {article_id}");
    Ok(())
}`}</code>
                </pre>

                <h2>Step 2: Make Job Retry Policy Explicit</h2>
                <h3>
                    File: <code>app/src/internal/jobs/definitions/article_published_fanout.rs</code>
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`#[async_trait::async_trait]
impl Job for ArticlePublishedFanoutJob {
    const NAME: &'static str = "article.published_fanout";
    const QUEUE: &'static str = "default";

    fn max_retries(&self) -> u32 {
        5
    }

    fn backoff(&self, attempt: u32) -> u64 {
        match attempt {
            0 => 5,
            1 => 30,
            2 => 60,
            3 => 300,
            _ => 600,
        }
    }

    // Optional: serialize by tenant to avoid race in same tenant fanout stream.
    fn group_id(&self) -> Option<String> {
        Some(format!("tenant:{}", self.tenant_id))
    }
}`}</code>
                </pre>

                <h2>Step 3: Expose Failed-Job Recovery Endpoints (Optional)</h2>
                <p>
                    Framework already provides queue/failed-jobs admin routes in{' '}
                    <code>core_jobs::api::routes</code>.
                </p>
                <h3>
                    File: <code>app/src/app.rs</code>
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-rust">{`pub async fn build_router(ctx: bootstrap::boot::BootContext) -> anyhow::Result<Router> {
    let app_api_state = AppApiState::new(&ctx);

    Ok(Router::new()
        .nest("/api/v1", v1::router(app_api_state.clone()))
        .nest(
            "/admin/jobs",
            core_jobs::api::routes(ctx.queue.clone(), Some(ctx.db.clone())),
        )
        // other routers ...
    )
}`}</code>
                </pre>

                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-bash">{`# failed job list
curl http://127.0.0.1:3000/admin/jobs/failed

# retry a failed job
curl -X POST http://127.0.0.1:3000/admin/jobs/failed/<JOB_ID>/retry`}</code>
                </pre>

                <h2>Step 4: Tune Realtime Durability and Replay-Gap SLO</h2>
                <h3>
                    File: <code>.env</code>
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-bash">{`REALTIME_DELIVERY_MODE=durable
REALTIME_STREAM_MAX_LEN=100000
REALTIME_STREAM_RETENTION_SECS=86400
REALTIME_REPLAY_LIMIT_DEFAULT=200
REALTIME_REPLAY_LIMIT_MAX=1000
REALTIME_REPLAY_GAP_ALERT_THRESHOLD=100
REALTIME_REPLAY_GAP_ALERT_WINDOW_SECS=60`}</code>
                </pre>
                <p>
                    If replay-gap grows, increase retention and investigate consumer reconnect lag.
                </p>

                <h2>Step 5: Handle Replay-Gap on Client</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-javascript">{`ws.onmessage = (evt) => {
  const msg = JSON.parse(evt.data);

  if (msg.op === 'event' && msg.delivery_id) {
    localStorage.setItem('rt:last_delivery_id', msg.delivery_id);
    ws.send(JSON.stringify({
      op: 'ack',
      channel: msg.channel,
      room: msg.room ?? null,
      delivery_id: msg.delivery_id
    }));
  }

  if (msg.op === 'replay_gap') {
    // Cursor is too old for retained stream window.
    // Trigger full-state refetch from HTTP API.
    window.location.reload();
  }
};`}</code>
                </pre>

                <h2>Step 6: Add Operational Checks</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-bash">{`# websocket metrics json
curl http://127.0.0.1:3010/realtime/metrics

# websocket metrics prometheus
curl http://127.0.0.1:3010/realtime/metrics.prom

# queue keys and sizes
curl http://127.0.0.1:3000/admin/jobs/

# run load smoke benchmark
./console realtime bench --clients 50 --messages 200 --json`}</code>
                </pre>

                <h2>Hardening Rule</h2>
                <ul>
                    <li>At-least-once delivery means consumers must be idempotent.</li>
                    <li>
                        Retries without failure visibility are dangerous; expose failed-job recovery.
                    </li>
                    <li>
                        Durable realtime without replay-gap monitoring is incomplete; always track
                        metrics and alert thresholds.
                    </li>
                </ul>
            </div>
        </div>
    )
}
