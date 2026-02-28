#[tokio::main]
async fn main() -> anyhow::Result<()> {
    bootstrap::jobs::start_worker(
        app::internal::jobs::register_jobs,
        Some(app::internal::jobs::register_schedules),
    )
    .await
}
