use crate::boot::{init_app, BootContext};
use anyhow::Result;
use axum::Router;
use tracing::info;

#[derive(Debug, Clone, Copy, Default)]
pub struct RealtimeStartOptions {
    pub embed_worker: bool,
}

pub async fn start_server<R, W, FutR, FutW>(
    router_factory: R,
    worker_runner: W,
    options: RealtimeStartOptions,
) -> Result<()>
where
    R: FnOnce(BootContext) -> FutR,
    FutR: std::future::Future<Output = Result<Router>> + Send + 'static,
    W: FnOnce(BootContext) -> FutW + Send + 'static,
    FutW: std::future::Future<Output = Result<()>> + Send + 'static,
{
    let (ctx, _guard) = init_app().await?;

    if options.embed_worker {
        let worker_ctx = ctx.clone();
        core_jobs::utils::spawn_embedded_worker(&ctx.settings, move |db, redis| {
            let mut worker_ctx = worker_ctx;
            worker_ctx.db = db;
            worker_ctx.redis = redis;
            worker_runner(worker_ctx)
        });
    }

    let app_router = router_factory(ctx.clone()).await?;
    let bind_addr = ctx.settings.realtime.bind_addr()?;
    info!(
        enabled = ctx.settings.realtime.enabled,
        require_auth = ctx.settings.realtime.require_auth,
        heartbeat_secs = ctx.settings.realtime.heartbeat_secs,
        presence_ttl_secs = ctx.settings.realtime.presence_ttl_secs,
        "Starting realtime server on {}",
        bind_addr
    );
    core_web::server::serve(app_router, bind_addr).await?;
    Ok(())
}
