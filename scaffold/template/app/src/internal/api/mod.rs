pub mod datatable;
pub mod state;
pub mod v1;

use std::sync::Arc;

use axum::{extract::State, http::header, response::Html, routing::get as axum_get, Json, Router};
use bootstrap::boot::BootContext;
use core_db::platform::countries::Country;
use core_web::error::AppError;
use core_web::openapi::{
    aide::openapi::{Info, OpenApi},
    ApiRouter,
};
use tower_http::services::{ServeDir, ServeFile};

use state::AppApiState;

// ─── Portal configuration ────────────────────────────────────────
//
// Each SPA portal is defined by its base path, Vite dev port, HTML
// title, and the module entry point that Vite resolves.  Adding a
// new portal (e.g. merchant on :5175) only requires a new entry
// here plus the matching Vite config / source directory.

struct PortalDev {
    /// Mount path for the portal (e.g. "/admin").
    base: &'static str,
    /// Vite dev-server port.
    port: u16,
    /// HTML `<title>`.
    title: &'static str,
    /// Module entry relative to the Vite root (e.g. "src/admin/main.tsx").
    entry: &'static str,
    /// Vite base path (matches `base` in vite.config — e.g. "/admin/" or "/").
    vite_base: &'static str,
}

const ADMIN_PORTAL: PortalDev = PortalDev {
    base: "/admin",
    port: 5174,
    title: "Admin",
    entry: "src/admin/main.tsx",
    vite_base: "/admin/",
};

const USER_PORTAL: PortalDev = PortalDev {
    base: "/",
    port: 5173,
    title: "App",
    entry: "src/user/main.tsx",
    vite_base: "/",
};

// ─── Router builder ──────────────────────────────────────────────

pub async fn build_router(ctx: BootContext) -> anyhow::Result<Router> {
    let app_state = AppApiState::new(&ctx)?;
    let is_dev = matches!(
        ctx.settings.app.env.as_str(),
        "local" | "dev" | "development"
    );

    // 1. API + OpenAPI
    let mut router = build_api_router(&ctx, app_state.clone())?;

    // 2. Bootstrap script (shared frontend config)
    router = router.route(
        "/api/bootstrap.js",
        axum_get(bootstrap_script).with_state(app_state),
    );

    // 3. Frontend SPA serving
    router = mount_frontend(router, is_dev);

    Ok(router)
}

// ─── API routing ─────────────────────────────────────────────────

fn build_api_router(ctx: &BootContext, state: AppApiState) -> anyhow::Result<Router> {
    let api_router = ApiRouter::new().nest("/api/v1", v1::router(state));

    let mut api = OpenApi::default();
    api.info = Info {
        title: "{{PROJECT_NAME}}-api".to_string(),
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

    Ok(router)
}

// ─── Frontend SPA serving ────────────────────────────────────────

fn mount_frontend(router: Router, is_dev: bool) -> Router {
    if is_dev {
        mount_frontend_dev(router)
    } else {
        mount_frontend_prod(router)
    }
}

/// Dev mode: serve Vite HMR proxy HTML for each portal.
/// Also mount `frontend/public/` subdirectories so static assets
/// (audio, banners, etc.) are accessible without a production build.
fn mount_frontend_dev(mut router: Router) -> Router {
    // Nested portals first (admin, merchant, etc.)
    router = mount_portal_dev(router, &ADMIN_PORTAL);

    // Dev-mode static assets from frontend/public/ subdirectories.
    // In production these live inside public/ and are served by static_assets_router.
    if let Ok(entries) = std::fs::read_dir("frontend/public") {
        for entry in entries.flatten() {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                let mount = format!("/{}", entry.file_name().to_string_lossy());
                router = router.nest_service(&mount, ServeDir::new(entry.path()));
            }
        }
    }

    // User portal as catch-all fallback (must be last)
    let user_html = dev_html(&USER_PORTAL);
    router.fallback(move || async move { Html(user_html) })
}

/// Production mode: serve compiled static files from public/.
/// Nested portals are served from their subdirectories with SPA
/// fallback; user portal is the root catch-all.
fn mount_frontend_prod(mut router: Router) -> Router {
    let public_path = core_web::static_assets::public_path_from_env();

    // Nested portals (admin, etc.) — serve from public/{base}/
    let admin_public = public_path.join("admin");
    let admin_index = admin_public.join("index.html");
    if admin_public.is_dir() && admin_index.is_file() {
        router = router.nest_service(
            "/admin",
            ServeDir::new(&admin_public).fallback(ServeFile::new(&admin_index)),
        );
    }

    // User portal — root catch-all from public/
    if let Some(static_router) = core_web::static_assets::static_assets_router(&public_path) {
        router = router.merge(static_router);
    }

    router
}

/// Mount a single portal in dev mode: exact path + wildcard → Vite HMR HTML.
fn mount_portal_dev(router: Router, portal: &PortalDev) -> Router {
    let html = dev_html(portal);
    let html2 = html.clone();
    router
        .route(
            portal.base,
            axum_get(move || {
                let h = html.clone();
                async move { Html(h) }
            }),
        )
        .route(
            &format!("{}{{*path}}", portal.base.trim_end_matches('/')),
            axum_get(move || {
                let h = html2.clone();
                async move { Html(h) }
            }),
        )
}

/// Build the Vite HMR proxy HTML for a portal. The HTML loads the
/// Vite client, React Refresh runtime, and the portal entry module
/// directly from the Vite dev server.
fn dev_html(portal: &PortalDev) -> String {
    format!(
        r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>{title}</title>
    <script src="/api/bootstrap.js"></script>
    <script type="module" src="http://localhost:{port}{vite_base}@vite/client"></script>
    <script type="module">
      import RefreshRuntime from "http://localhost:{port}{vite_base}@react-refresh"
      RefreshRuntime.injectIntoGlobalHook(window)
      window.$RefreshReg$ = () => {{}}
      window.$RefreshSig$ = () => (type) => type
      window.__vite_plugin_react_preamble_installed__ = true
    </script>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="http://localhost:{port}{vite_base}{entry}"></script>
  </body>
</html>"#,
        title = portal.title,
        port = portal.port,
        vite_base = portal.vite_base,
        entry = portal.entry,
    )
}

// ─── Bootstrap script ────────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize)]
struct FrontendBootstrapPayload {
    i18n: FrontendI18nPayload,
    countries: Vec<Country>,
}

#[derive(Debug, Clone, serde::Serialize)]
struct FrontendI18nPayload {
    default_locale: String,
    supported_locales: Vec<String>,
    default_timezone: String,
}

async fn bootstrap_script(
    State(state): State<AppApiState>,
) -> Result<([(header::HeaderName, &'static str); 2], String), AppError> {
    let countries = crate::internal::workflows::country::list_enabled_for_bootstrap(&state).await?;

    let payload = FrontendBootstrapPayload {
        i18n: FrontendI18nPayload {
            default_locale: state.i18n_default_locale.clone(),
            supported_locales: state.i18n_supported_locales.clone(),
            default_timezone: state.app_timezone.clone(),
        },
        countries,
    };

    let mut json = serde_json::to_string(&payload).map_err(AppError::from)?;
    json = json.replace("</", "<\\/");

    Ok((
        [
            (
                header::CONTENT_TYPE,
                "application/javascript; charset=utf-8",
            ),
            (header::CACHE_CONTROL, "no-store, max-age=0"),
        ],
        format!("window.__RUSTFORGE_BOOTSTRAP__ = Object.freeze({json});\n"),
    ))
}
