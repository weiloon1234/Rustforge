use std::path::PathBuf;

use axum::{
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use tower_http::services::{ServeDir, ServeFile};

pub fn router() -> Router {
    let dist_path = docs_dist_path();
    let index_path = dist_path.join("index.html");

    if dist_path.exists() && index_path.exists() {
        tracing::info!("Docs: serving frontend assets from {}", dist_path.display());
        Router::new()
            .fallback_service(ServeDir::new(dist_path).fallback(ServeFile::new(index_path)))
    } else {
        tracing::warn!(
            "Docs: frontend assets missing at {}. Run `npm --prefix core-docs/frontend run build` in Rustforge.",
            dist_path.display()
        );
        Router::new()
            .route("/", get(missing_assets))
            .fallback(get(missing_assets))
    }
}

fn docs_dist_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("frontend")
        .join("dist")
}

async fn missing_assets() -> impl IntoResponse {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Html(
            "<h1>Framework docs assets are missing</h1><p>Run <code>make docs-build</code> in Rustforge, then reload.</p>",
        ),
    )
}
