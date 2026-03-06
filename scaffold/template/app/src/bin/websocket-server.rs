#[tokio::main]
async fn main() -> anyhow::Result<()> {
    bootstrap::realtime::start_server(
        app::internal::realtime::build_router,
        |ctx| async move {
            bootstrap::jobs::start_with_context(
                ctx,
                app::internal::jobs::register_jobs,
                Some(app::internal::jobs::register_schedules),
            )
            .await
        },
        bootstrap::realtime::RealtimeStartOptions::default(),
    )
    .await
}
