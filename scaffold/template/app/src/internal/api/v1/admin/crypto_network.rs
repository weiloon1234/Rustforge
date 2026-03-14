use axum::extract::{Multipart, Path, State};
use core_db::platform::attachments::types::AttachmentInput;
use core_i18n::t;
use core_web::{
    auth::AuthUser,
    authz::PermissionMode,
    error::AppError,
    extract::validation::transform_validation_errors,
    openapi::{
        with_permission_check_delete_with, with_permission_check_get_with,
        with_permission_check_post_with, with_permission_check_put_with, ApiRouter,
    },
    response::ApiResponse,
};
use generated::{guards::AdminGuard, models::CryptoNetworkStatus, permissions::Permission};
use uuid::Uuid;
use validator::Validate;

use crate::{
    contracts::api::v1::admin::crypto_network::{AdminCryptoNetworkInput, CryptoNetworkOutput},
    internal::{api::state::AppApiState, workflows::crypto_network as workflow},
};

use super::receipt_upload::validate_attachment_allowed;

pub fn router(state: AppApiState) -> ApiRouter {
    ApiRouter::new()
        .api_route(
            "/",
            with_permission_check_post_with(
                create,
                AdminGuard,
                PermissionMode::Any,
                [Permission::CryptoNetworkManage.as_str()],
                |op| op.summary("Create crypto network").tag("Admin Crypto Networks"),
            ),
        )
        .api_route(
            "/{id}",
            with_permission_check_get_with(
                detail,
                AdminGuard,
                PermissionMode::Any,
                [
                    Permission::CryptoNetworkRead.as_str(),
                    Permission::CryptoNetworkManage.as_str(),
                ],
                |op| op.summary("Get crypto network detail").tag("Admin Crypto Networks"),
            ),
        )
        .api_route(
            "/{id}",
            with_permission_check_put_with(
                update,
                AdminGuard,
                PermissionMode::Any,
                [Permission::CryptoNetworkManage.as_str()],
                |op| op.summary("Update crypto network").tag("Admin Crypto Networks"),
            ),
        )
        .api_route(
            "/{id}",
            with_permission_check_delete_with(
                delete,
                AdminGuard,
                PermissionMode::Any,
                [Permission::CryptoNetworkManage.as_str()],
                |op| op.summary("Delete crypto network").tag("Admin Crypto Networks"),
            ),
        )
        .with_state(state)
}

struct ParsedCryptoNetworkMultipart {
    input: AdminCryptoNetworkInput,
    logo: Option<AttachmentInput>,
}

async fn parse_crypto_network_multipart(
    state: &AppApiState,
    mut multipart: Multipart,
) -> Result<ParsedCryptoNetworkMultipart, AppError> {
    let mut name: Option<String> = None;
    let mut symbol: Option<String> = None;
    let mut status: Option<String> = None;
    let mut sort_order: Option<String> = None;
    let mut logo: Option<AttachmentInput> = None;

    while let Some(field) = multipart.next_field().await.map_err(AppError::from)? {
        let field_name = field
            .name()
            .map(str::to_string)
            .ok_or_else(|| AppError::BadRequest(t("Invalid multipart field")))?;

        match field_name.as_str() {
            "name" => name = Some(field.text().await.map_err(AppError::from)?),
            "symbol" => symbol = Some(field.text().await.map_err(AppError::from)?),
            "status" => status = Some(field.text().await.map_err(AppError::from)?),
            "sort_order" => sort_order = Some(field.text().await.map_err(AppError::from)?),
            "logo" => {
                if field.file_name().is_some() {
                    let filename = field.file_name().map(ToString::to_string);
                    let content_type = field
                        .content_type()
                        .map(ToString::to_string)
                        .unwrap_or_else(|| "application/octet-stream".to_string());
                    let bytes = field.bytes().await.map_err(AppError::from)?;
                    if !bytes.is_empty() {
                        let rules = generated::get_attachment_rules("image")
                            .ok_or_else(|| AppError::BadRequest(t("Unknown attachment type")))?;
                        validate_attachment_allowed(&rules.allowed, filename.as_deref(), &content_type)?;

                        let ext = filename
                            .as_deref()
                            .and_then(|n| std::path::Path::new(n).extension().and_then(|e| e.to_str()))
                            .unwrap_or_else(|| content_type.split('/').nth(1).unwrap_or("bin"));

                        let now = time::OffsetDateTime::now_utc();
                        let object_key = format!(
                            "crypto_networks/{:04}/{:02}/{:02}/logo/{}.{}",
                            now.year(), now.month() as u8, now.day(), Uuid::new_v4(), ext,
                        );

                        state.storage.put(&object_key, bytes.clone(), &content_type).await.map_err(AppError::from)?;

                        logo = Some(AttachmentInput::new(object_key, content_type, bytes.len() as i64, None, None));
                    }
                }
            }
            _ => {}
        }
    }

    let name = name
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .ok_or_else(|| AppError::BadRequest(t("Missing required field: name")))?;
    let symbol = symbol
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .ok_or_else(|| AppError::BadRequest(t("Missing required field: symbol")))?;
    let status_str = status.ok_or_else(|| AppError::BadRequest(t("Missing required field: status")))?;
    let status: CryptoNetworkStatus = serde_json::from_str(&format!("\"{status_str}\""))
        .or_else(|_| serde_json::from_str(&status_str))
        .map_err(|_| AppError::BadRequest(t("Invalid status value")))?;
    let sort_order = sort_order.and_then(|v| v.trim().parse::<i32>().ok());

    let input = AdminCryptoNetworkInput { name, symbol, status, sort_order };
    if let Err(e) = input.validate() {
        return Err(AppError::Validation {
            message: t("Validation failed"),
            errors: transform_validation_errors(e),
        });
    }

    Ok(ParsedCryptoNetworkMultipart { input, logo })
}

fn network_output(network: &generated::models::CryptoNetworkRecord) -> CryptoNetworkOutput {
    CryptoNetworkOutput {
        id: network.id.into(),
        name: network.name.clone(),
        symbol: network.symbol.clone(),
        logo_url: network.logo_url.clone(),
        status: network.status,
        sort_order: network.sort_order,
        created_at: network.created_at,
        updated_at: network.updated_at,
    }
}

async fn create(
    State(state): State<AppApiState>,
    _auth: AuthUser<AdminGuard>,
    multipart: Multipart,
) -> Result<ApiResponse<CryptoNetworkOutput>, AppError> {
    let parsed = parse_crypto_network_multipart(&state, multipart).await?;
    let network = workflow::create(&state, parsed.input, parsed.logo).await?;

    Ok(ApiResponse::success(network_output(&network), &t("Crypto network created")))
}

async fn detail(
    State(state): State<AppApiState>,
    _auth: AuthUser<AdminGuard>,
    Path(id): Path<i64>,
) -> Result<ApiResponse<CryptoNetworkOutput>, AppError> {
    let network = workflow::detail(&state, id).await?;

    Ok(ApiResponse::success(network_output(&network), &t("Crypto network detail")))
}

async fn update(
    State(state): State<AppApiState>,
    _auth: AuthUser<AdminGuard>,
    Path(id): Path<i64>,
    multipart: Multipart,
) -> Result<ApiResponse<CryptoNetworkOutput>, AppError> {
    let parsed = parse_crypto_network_multipart(&state, multipart).await?;
    let network = workflow::update(&state, id, parsed.input, parsed.logo).await?;

    Ok(ApiResponse::success(network_output(&network), &t("Crypto network updated")))
}

async fn delete(
    State(state): State<AppApiState>,
    _auth: AuthUser<AdminGuard>,
    Path(id): Path<i64>,
) -> Result<ApiResponse<serde_json::Value>, AppError> {
    workflow::delete(&state, id).await?;

    Ok(ApiResponse::success(serde_json::json!({ "deleted": true }), &t("Crypto network deleted")))
}
