#[tokio::main]
async fn main() -> anyhow::Result<()> {
    bootstrap::realtime::start_server(
        |_ctx| async move { Ok(axum::Router::new()) },
        |_ctx| async move { Ok(()) },
        bootstrap::realtime::RealtimeStartOptions::default(),
    )
    .await
}
