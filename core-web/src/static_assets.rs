use std::path::{Path, PathBuf};

use axum::Router;
use tower_http::services::{ServeDir, ServeFile};

pub fn public_path_from_env() -> PathBuf {
    std::env::var("PUBLIC_PATH")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("public"))
}

pub fn static_assets_router(public_dir: impl AsRef<Path>) -> Option<Router> {
    let public_dir = public_dir.as_ref();

    if !public_dir.is_dir() {
        return None;
    }

    let index_path = public_dir.join("index.html");
    if !index_path.is_file() {
        return None;
    }

    let router = Router::new()
        .fallback_service(ServeDir::new(public_dir).fallback(ServeFile::new(index_path)));

    Some(router)
}
