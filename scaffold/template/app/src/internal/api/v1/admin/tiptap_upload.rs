use axum::{
    body::Bytes,
    extract::{Multipart, State},
};
use core_i18n::t;
use core_web::{
    auth::AuthUser,
    error::AppError,
    openapi::{aide::axum::routing::post_with, ApiRouter},
    response::ApiResponse,
};
use generated::{guards::AdminGuard, models::AdminType, permissions::Permission};
use uuid::Uuid;

use crate::{
    contracts::api::v1::admin::tiptap_upload::AdminTiptapImageUploadOutput,
    internal::api::state::AppApiState,
};

const MAX_TIPTAP_IMAGE_BYTES: usize = 5 * 1024 * 1024;
const TIPTAP_UPLOAD_FOLDER_PERMISSIONS: &[(&str, Permission)] =
    &[("uploads/content_page", Permission::ContentPageManage)];

struct UploadPayload {
    folder: String,
    file_name: Option<String>,
    content_type: String,
    bytes: Bytes,
}

pub fn router(state: AppApiState) -> ApiRouter {
    ApiRouter::new()
        .api_route(
            "/tiptap-image",
            post_with(upload_tiptap_image, |op| {
                op.summary("Upload TipTap image").tag("Admin Upload")
            }),
        )
        .with_state(state)
}

async fn upload_tiptap_image(
    State(state): State<AppApiState>,
    auth: AuthUser<AdminGuard>,
    multipart: Multipart,
) -> Result<ApiResponse<AdminTiptapImageUploadOutput>, AppError> {
    let payload = parse_upload_payload(multipart).await?;
    let permission = TIPTAP_UPLOAD_FOLDER_PERMISSIONS
        .iter()
        .find(|(folder, _)| *folder == payload.folder)
        .map(|(_, permission)| *permission)
        .ok_or_else(|| AppError::BadRequest(t("Upload folder is not allowed")))?;

    ensure_upload_permission(&auth, permission)?;

    let attachment = upload_tiptap_image_file(
        &state,
        &payload.folder,
        payload.file_name.as_deref(),
        &payload.content_type,
        payload.bytes,
    )
    .await?;

    let base_url = std::env::var("S3_URL").ok();
    let output = AdminTiptapImageUploadOutput {
        folder: payload.folder,
        path: attachment.path.clone(),
        url: build_attachment_url(&attachment.path, base_url.as_deref()),
        content_type: attachment.content_type,
        size: attachment.size,
        width: attachment.width,
        height: attachment.height,
    };

    Ok(ApiResponse::success(output, &t("TipTap image uploaded")))
}

fn ensure_upload_permission(
    auth: &AuthUser<AdminGuard>,
    required: Permission,
) -> Result<(), AppError> {
    if matches!(
        auth.user.admin_type,
        AdminType::Developer | AdminType::SuperAdmin
    ) {
        return Ok(());
    }

    if auth.has_permission(required.as_str()) {
        return Ok(());
    }

    Err(AppError::Forbidden(t(
        "Missing permission for upload folder",
    )))
}

async fn parse_upload_payload(mut multipart: Multipart) -> Result<UploadPayload, AppError> {
    let mut folder: Option<String> = None;
    let mut file_name: Option<String> = None;
    let mut content_type: Option<String> = None;
    let mut bytes: Option<Bytes> = None;

    while let Some(field) = multipart.next_field().await.map_err(AppError::from)? {
        let field_name = field
            .name()
            .map(str::to_string)
            .ok_or_else(|| AppError::BadRequest(t("Invalid multipart field")))?;

        match field_name.as_str() {
            "folder" => {
                if folder.is_some() {
                    return Err(AppError::BadRequest(t("Invalid multipart field")));
                }
                let value = field.text().await.map_err(AppError::from)?;
                folder = Some(normalize_folder(&value)?);
            }
            "fileUpload" => {
                if bytes.is_some() {
                    return Err(AppError::BadRequest(t("Invalid multipart field")));
                }

                if field.file_name().is_none() {
                    return Err(AppError::BadRequest(t("Invalid upload file field")));
                }

                let next_file_name = field.file_name().map(ToString::to_string);
                let next_content_type = field
                    .content_type()
                    .map(ToString::to_string)
                    .unwrap_or_else(|| "application/octet-stream".to_string());
                let next_bytes = field.bytes().await.map_err(AppError::from)?;

                if next_bytes.is_empty() {
                    return Err(AppError::BadRequest(t("Uploaded file is empty")));
                }
                if next_bytes.len() > MAX_TIPTAP_IMAGE_BYTES {
                    return Err(AppError::BadRequest(t("File exceeds maximum allowed size")));
                }

                file_name = next_file_name;
                content_type = Some(next_content_type);
                bytes = Some(next_bytes);
            }
            _ => return Err(AppError::BadRequest(t("Unknown multipart field"))),
        }
    }

    let folder = folder.ok_or_else(|| AppError::BadRequest(t("Missing required field: folder")))?;
    let bytes =
        bytes.ok_or_else(|| AppError::BadRequest(t("Missing required field: fileUpload")))?;
    let content_type = content_type.unwrap_or_else(|| "application/octet-stream".to_string());

    Ok(UploadPayload {
        folder,
        file_name,
        content_type,
        bytes,
    })
}

fn normalize_folder(value: &str) -> Result<String, AppError> {
    let normalized = value.trim().trim_matches('/').to_ascii_lowercase();
    if normalized.is_empty()
        || normalized.contains("..")
        || normalized.contains('\\')
        || normalized
            .split('/')
            .any(|segment| segment.trim().is_empty())
    {
        return Err(AppError::BadRequest(t("Invalid upload folder")));
    }

    let has_invalid_segment = normalized.split('/').any(|segment| {
        !segment
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_' || ch == '-')
    });
    if has_invalid_segment {
        return Err(AppError::BadRequest(t("Invalid upload folder")));
    }

    Ok(normalized)
}

async fn upload_tiptap_image_file(
    state: &AppApiState,
    folder: &str,
    filename: Option<&str>,
    content_type: &str,
    bytes: Bytes,
) -> Result<core_db::platform::attachments::AttachmentUploadDto, AppError> {
    let rules = generated::get_attachment_rules("image")
        .ok_or_else(|| AppError::BadRequest(t("Unknown attachment type: image")))?;

    validate_attachment_allowed(&rules.allowed, filename, content_type)?;

    let ext = filename
        .and_then(|name| {
            std::path::Path::new(name)
                .extension()
                .and_then(|item| item.to_str())
        })
        .unwrap_or_else(|| content_type.split('/').nth(1).unwrap_or("bin"));

    let now = time::OffsetDateTime::now_utc();
    let object_key = format!(
        "{}/{:04}/{:02}/{:02}/tiptap/{}.{}",
        folder.trim_matches('/'),
        now.year(),
        now.month() as u8,
        now.day(),
        Uuid::new_v4(),
        ext
    );

    state
        .storage
        .put(&object_key, bytes.clone(), content_type)
        .await
        .map_err(AppError::from)?;

    let mut dto = core_db::platform::attachments::AttachmentUploadDto::new(
        object_key,
        content_type.to_string(),
        bytes.len() as i64,
        None,
        None,
    );
    if let Some(name) = filename {
        dto = dto.with_name(name.to_string());
    }
    Ok(dto)
}

fn validate_attachment_allowed(
    allowed: &[String],
    filename: Option<&str>,
    content_type: &str,
) -> Result<(), AppError> {
    if allowed.is_empty() {
        return Ok(());
    }

    let ct = content_type.to_ascii_lowercase();
    let filename_lower = filename.map(|item| item.to_ascii_lowercase());

    let mut ok = false;
    for rule in allowed {
        let rule = rule.to_ascii_lowercase();
        if rule == "*" || rule == "*/*" {
            ok = true;
            break;
        }
        if rule.starts_with("image/") && ct.starts_with("image/") {
            ok = true;
            break;
        }
        if rule.starts_with('.') {
            if let Some(name) = filename_lower.as_deref() {
                if name.ends_with(&rule) {
                    ok = true;
                    break;
                }
            }
            continue;
        }
        if ct == rule {
            ok = true;
            break;
        }
    }

    if ok {
        Ok(())
    } else {
        Err(AppError::BadRequest(t("Attachment type not allowed")))
    }
}

fn build_attachment_url(path: &str, base: Option<&str>) -> String {
    let raw = path.trim();
    if raw.is_empty() {
        return String::new();
    }
    if raw.starts_with("//")
        || raw.starts_with("http://")
        || raw.starts_with("https://")
        || raw.starts_with("data:")
        || raw.starts_with("blob:")
    {
        return raw.to_string();
    }

    let Some(base) = base.map(str::trim).filter(|value| !value.is_empty()) else {
        return raw.to_string();
    };
    format!(
        "{}/{}",
        base.trim_end_matches('/'),
        raw.trim_start_matches('/')
    )
}
