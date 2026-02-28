pub mod datatable;
pub mod state;
pub mod v1;

use std::sync::Arc;

use axum::{routing::get as axum_get, Json, Router, response::Html};
use bootstrap::boot::BootContext;
use core_web::openapi::{
    aide::{
        openapi::{Info, OpenApi},
    },
    ApiRouter,
};
use tower_http::services::{ServeDir, ServeFile};

use state::AppApiState;

pub async fn build_router(ctx: BootContext) -> anyhow::Result<Router> {
    let app_state = AppApiState::new(&ctx)?;

    let api_router = ApiRouter::new().nest("/api/v1", v1::router(app_state));

    let mut api = OpenApi::default();
    api.info = Info {
        title: "starter-api".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        ..Default::default()
    };

    let mut router =
        api_router.finish_api_with(&mut api, core_web::openapi::with_bearer_auth_scheme);

    if ctx.settings.app.enable_openapi_docs {
        let openapi_json_path = ctx.settings.app.openapi_json_path.clone();
        let openapi = Arc::new(api);

        router = router.route(
            openapi_json_path.as_str(),
            axum_get({
                let openapi = openapi.clone();
                move || {
                    let openapi = openapi.clone();
                    async move { Json((*openapi).clone()) }
                }
            }),
        );
    }

    let public_path = core_web::static_assets::public_path_from_env();

    // Admin SPA: /admin/* → public/admin/index.html
    let admin_public = public_path.join("admin");
    let admin_index = admin_public.join("index.html");
    if admin_public.is_dir() && admin_index.is_file() {
        router = router.nest_service(
            "/admin",
            ServeDir::new(&admin_public).fallback(ServeFile::new(&admin_index)),
        );
    } else {
        router = router
            .route("/admin", axum_get(admin_dev))
            .route("/admin/{*path}", axum_get(admin_dev));
    }

    // User SPA: everything else → public/index.html (existing logic)
    if let Some(static_router) = core_web::static_assets::static_assets_router(&public_path) {
        router = router.merge(static_router);
    } else {
        router = router.fallback(axum_get(root));
    }

    Ok(router)
}

async fn root() -> Html<&'static str> {
    Html(r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>App</title>
    <script type="module" src="http://localhost:5173/@vite/client"></script>
    <script type="module">
      import RefreshRuntime from "http://localhost:5173/@react-refresh"
      RefreshRuntime.injectIntoGlobalHook(window)
      window.$RefreshReg$ = () => {}
      window.$RefreshSig$ = () => (type) => type
      window.__vite_plugin_react_preamble_installed__ = true
    </script>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="http://localhost:5173/src/user/main.tsx"></script>
  </body>
</html>"#)
}

async fn admin_dev() -> Html<&'static str> {
    Html(r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Admin</title>
    <script type="module" src="http://localhost:5174/admin/@vite/client"></script>
    <script type="module">
      import RefreshRuntime from "http://localhost:5174/admin/@react-refresh"
      RefreshRuntime.injectIntoGlobalHook(window)
      window.$RefreshReg$ = () => {}
      window.$RefreshSig$ = () => (type) => type
      window.__vite_plugin_react_preamble_installed__ = true
    </script>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="http://localhost:5174/admin/src/admin/main.tsx"></script>
  </body>
</html>"#)
}
