const TOKIO_THREAD_STACK_SIZE: usize = 8 * 1024 * 1024;

async fn run() -> anyhow::Result<()> {
    bootstrap::jobs::start_worker(
        app::internal::jobs::register_jobs,
        Some(app::internal::jobs::register_schedules),
    )
    .await
}

fn main() -> anyhow::Result<()> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_stack_size(TOKIO_THREAD_STACK_SIZE)
        .build()?
        .block_on(run())
}
