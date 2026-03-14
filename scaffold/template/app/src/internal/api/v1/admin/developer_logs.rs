use axum::extract::{Path, State};
use core_web::{
    auth::AuthUser,
    error::AppError,
    openapi::{
        aide::axum::routing::{delete_with, get_with},
        ApiRouter,
    },
    response::ApiResponse,
};
use generated::guards::AdminGuard;
use generated::models::AdminType;

use crate::{
    contracts::api::v1::admin::developer_logs::{
        LogFileDeleteOutput, LogFileEntry, LogFileListOutput,
    },
    internal::api::state::AppApiState,
};

pub fn router(state: AppApiState) -> ApiRouter {
    ApiRouter::new()
        .api_route(
            "/",
            get_with(list_logs, |op| {
                op.summary("List log files").tag("Developer Logs")
            }),
        )
        .api_route(
            "/{filename}",
            get_with(read_log, |op| {
                op.summary("Read log file content").tag("Developer Logs")
            })
            .merge(delete_with(delete_log, |op| {
                op.summary("Delete log file").tag("Developer Logs")
            })),
        )
        .with_state(state)
}

fn require_developer(auth: &AuthUser<AdminGuard>) -> Result<(), AppError> {
    if !matches!(auth.user.admin_type, AdminType::Developer) {
        return Err(AppError::Forbidden(
            "Developer access required".to_string(),
        ));
    }
    Ok(())
}

fn logs_dir() -> std::path::PathBuf {
    std::path::PathBuf::from("./logs")
}

/// Validate filename to prevent path traversal.
fn validate_filename(filename: &str) -> Result<std::path::PathBuf, AppError> {
    if filename.contains('/') || filename.contains('\\') || filename.contains("..") {
        return Err(AppError::BadRequest("Invalid filename".to_string()));
    }
    let path = logs_dir().join(filename);
    if !path.starts_with(logs_dir()) {
        return Err(AppError::BadRequest("Invalid filename".to_string()));
    }
    Ok(path)
}

async fn list_logs(
    _state: State<AppApiState>,
    auth: AuthUser<AdminGuard>,
) -> Result<ApiResponse<LogFileListOutput>, AppError> {
    require_developer(&auth)?;

    let dir = logs_dir();
    let mut files = Vec::new();

    if dir.exists() {
        let mut entries = tokio::fs::read_dir(&dir)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to read logs directory: {e}")))?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to read directory entry: {e}")))?
        {
            let metadata = entry.metadata().await.ok();
            let Some(meta) = metadata else { continue };
            if !meta.is_file() {
                continue;
            }
            let filename = entry.file_name().to_string_lossy().to_string();
            let modified_at = meta
                .modified()
                .ok()
                .and_then(|t| {
                    let dt: time::OffsetDateTime = t.into();
                    Some(
                        dt.format(&time::format_description::well_known::Rfc3339)
                            .ok()?,
                    )
                })
                .unwrap_or_default();
            files.push(LogFileEntry {
                filename,
                size_bytes: meta.len(),
                modified_at,
            });
        }
    }

    files.sort_by(|a, b| b.modified_at.cmp(&a.modified_at));

    Ok(ApiResponse::success(LogFileListOutput { files }, "ok"))
}

async fn read_log(
    _state: State<AppApiState>,
    auth: AuthUser<AdminGuard>,
    Path(filename): Path<String>,
) -> Result<ApiResponse<String>, AppError> {
    require_developer(&auth)?;

    let path = validate_filename(&filename)?;
    if !path.exists() {
        return Err(AppError::NotFound("Log file not found".to_string()));
    }

    let content = tokio::fs::read_to_string(&path)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to read log file: {e}")))?;

    Ok(ApiResponse::success(content, "ok"))
}

async fn delete_log(
    _state: State<AppApiState>,
    auth: AuthUser<AdminGuard>,
    Path(filename): Path<String>,
) -> Result<ApiResponse<LogFileDeleteOutput>, AppError> {
    require_developer(&auth)?;

    let path = validate_filename(&filename)?;
    if !path.exists() {
        return Err(AppError::NotFound("Log file not found".to_string()));
    }

    tokio::fs::remove_file(&path)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to delete log file: {e}")))?;

    Ok(ApiResponse::success(
        LogFileDeleteOutput { deleted: true },
        "Log file deleted",
    ))
}
