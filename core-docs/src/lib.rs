use std::path::PathBuf;

use axum::{
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use tower_http::services::{ServeDir, ServeFile};

pub fn router() -> Router {
    if let Some(dist_path) = resolve_docs_dist_path() {
        let index_path = dist_path.join("index.html");
        tracing::info!("Docs: serving frontend assets from {}", dist_path.display());
        Router::new()
            .fallback_service(ServeDir::new(dist_path).fallback(ServeFile::new(index_path)))
    } else {
        let candidates = docs_dist_candidates()
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join(", ");
        tracing::warn!(
            "Docs: frontend assets missing. Checked: {}. Build docs into starter public path (`make framework-docs-build`) or set FRAMEWORK_DOCS_DIST_DIR.",
            candidates
        );
        Router::new()
            .route("/", get(missing_assets))
            .fallback(get(missing_assets))
    }
}

fn resolve_docs_dist_path() -> Option<PathBuf> {
    docs_dist_candidates().into_iter().find(|dir| {
        let index = dir.join("index.html");
        dir.is_dir() && index.is_file()
    })
}

fn docs_dist_candidates() -> Vec<PathBuf> {
    let mut out = Vec::new();

    if let Some(override_dir) = read_env("FRAMEWORK_DOCS_DIST_DIR") {
        out.push(PathBuf::from(override_dir));
    }

    let public_path = read_env("PUBLIC_PATH").unwrap_or_else(|| "public".to_string());
    let docs_route = read_env("FRAMEWORK_DOCS_PATH")
        .unwrap_or_else(|| "/framework-documentation".to_string());
    let docs_route = docs_route.trim().trim_matches('/');
    if !docs_route.is_empty() {
        out.push(PathBuf::from(&public_path).join(docs_route));
    }

    out.push(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("frontend")
            .join("dist"),
    );

    out
}

fn read_env(key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

async fn missing_assets() -> impl IntoResponse {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Html(
            "<h1>Framework docs assets are missing</h1><p>Build and publish from starter with <code>make framework-docs-build</code>, or set <code>FRAMEWORK_DOCS_DIST_DIR</code>.</p>",
        ),
    )
}
