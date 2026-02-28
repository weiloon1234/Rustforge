use crate::boot::{init_app, BootContext};
use anyhow::Result;
use axum::{middleware::from_fn, response::Html, routing::get as axum_get, Router};
use std::sync::Arc;
use tracing::info;

/// Starts the web server with the given router factory and worker runner.
///
/// # Arguments
/// * `router_factory` - Returns the Axum router.
/// * `worker_runner` - Function to run the embedded worker (e.g. `jobs::run_worker`).
pub async fn start_server<R, W, FutR, FutW>(router_factory: R, worker_runner: W) -> Result<()>
where
    R: FnOnce(BootContext) -> FutR,
    FutR: std::future::Future<Output = Result<Router>> + Send + 'static,
    W: FnOnce(BootContext) -> FutW + Send + 'static,
    FutW: std::future::Future<Output = Result<()>> + Send + 'static,
{
    // 1. Common Boot
    let (ctx, _guard) = init_app().await?;

    // 2. Spawn Embedded Worker (if configured)
    let worker_ctx = ctx.clone();
    core_jobs::utils::spawn_embedded_worker(&ctx.settings, move |db, redis| {
        let mut ctx = worker_ctx;
        ctx.db = db;
        ctx.redis = redis;
        worker_runner(ctx)
    });

    // 3. Build Project Router
    let app_router = router_factory(ctx.clone()).await?;

    // 4. AUTOMATIC FRAMEWORK ROUTES
    let app_router = if ctx.settings.app.enable_docs {
        info!("Framework Docs Enabled at '{}'", ctx.settings.app.docs_path);
        app_router.nest(&ctx.settings.app.docs_path, core_docs::router())
    } else {
        info!("Framework Docs DISABLED");
        app_router
    };

    let app_router = if ctx.settings.app.enable_openapi_docs {
        let openapi_docs_path = ctx.settings.app.openapi_docs_path.clone();
        let openapi_json_path = ctx.settings.app.openapi_json_path.clone();
        info!(
            "OpenAPI Docs Enabled at '{}' (spec: '{}')",
            openapi_docs_path, openapi_json_path
        );
        let redoc_html = Arc::new(
            core_web::openapi::aide::redoc::Redoc::new(openapi_json_path)
                .with_title("OpenAPI")
                .html(),
        );
        app_router.route(
            openapi_docs_path.as_str(),
            axum_get({
                let redoc_html = redoc_html.clone();
                move || {
                    let redoc_html = redoc_html.clone();
                    async move { Html((*redoc_html).clone()) }
                }
            }),
        )
    } else {
        info!("OpenAPI Docs DISABLED");
        app_router
    }
    .layer(from_fn(core_i18n::middleware::locale_middleware));

    // 5. Apply Standard Middleware (CORS, cookies, compression, timeouts, etc.)
    let app_router =
        core_web::middleware::stack::apply_standard_middleware(app_router, &ctx.settings);

    let bind_addr = ctx.settings.server.bind_addr()?;
    info!("Starting server on {}", bind_addr);

    // 6. Serve
    core_web::server::serve(app_router, bind_addr).await?;

    Ok(())
}
