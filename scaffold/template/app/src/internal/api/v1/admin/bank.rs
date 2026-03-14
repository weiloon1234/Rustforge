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
use generated::{guards::AdminGuard, models::BankStatus, permissions::Permission};
use uuid::Uuid;
use validator::Validate;

use crate::{
    contracts::api::v1::admin::bank::{AdminBankInput, BankOutput},
    internal::{api::state::AppApiState, workflows::bank as workflow},
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
                [Permission::BankManage.as_str()],
                |op| op.summary("Create bank").tag("Admin Banks"),
            ),
        )
        .api_route(
            "/{id}",
            with_permission_check_get_with(
                detail,
                AdminGuard,
                PermissionMode::Any,
                [Permission::BankRead.as_str(), Permission::BankManage.as_str()],
                |op| op.summary("Get bank detail").tag("Admin Banks"),
            ),
        )
        .api_route(
            "/{id}",
            with_permission_check_put_with(
                update,
                AdminGuard,
                PermissionMode::Any,
                [Permission::BankManage.as_str()],
                |op| op.summary("Update bank").tag("Admin Banks"),
            ),
        )
        .api_route(
            "/{id}",
            with_permission_check_delete_with(
                delete,
                AdminGuard,
                PermissionMode::Any,
                [Permission::BankManage.as_str()],
                |op| op.summary("Delete bank").tag("Admin Banks"),
            ),
        )
        .with_state(state)
}

struct ParsedBankMultipart {
    input: AdminBankInput,
    logo: Option<AttachmentInput>,
}

async fn parse_bank_multipart(
    state: &AppApiState,
    mut multipart: Multipart,
) -> Result<ParsedBankMultipart, AppError> {
    let mut country_iso2: Option<String> = None;
    let mut name: Option<String> = None;
    let mut code: Option<String> = None;
    let mut status: Option<String> = None;
    let mut sort_order: Option<String> = None;
    let mut logo: Option<AttachmentInput> = None;

    while let Some(field) = multipart.next_field().await.map_err(AppError::from)? {
        let field_name = field
            .name()
            .map(str::to_string)
            .ok_or_else(|| AppError::BadRequest(t("Invalid multipart field")))?;

        match field_name.as_str() {
            "country_iso2" => country_iso2 = Some(field.text().await.map_err(AppError::from)?),
            "name" => name = Some(field.text().await.map_err(AppError::from)?),
            "code" => code = Some(field.text().await.map_err(AppError::from)?),
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
                            "banks/{:04}/{:02}/{:02}/logo/{}.{}",
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

    let country_iso2 = country_iso2
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .ok_or_else(|| AppError::BadRequest(t("Missing required field: country_iso2")))?;
    let name = name
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .ok_or_else(|| AppError::BadRequest(t("Missing required field: name")))?;
    let code = code.map(|v| v.trim().to_string()).filter(|v| !v.is_empty());
    let status_str = status.ok_or_else(|| AppError::BadRequest(t("Missing required field: status")))?;
    let status: BankStatus = serde_json::from_str(&format!("\"{status_str}\""))
        .or_else(|_| serde_json::from_str(&status_str))
        .map_err(|_| AppError::BadRequest(t("Invalid status value")))?;
    let sort_order = sort_order.and_then(|v| v.trim().parse::<i32>().ok());

    let input = AdminBankInput { country_iso2, name, code, status, sort_order };
    if let Err(e) = input.validate() {
        return Err(AppError::Validation {
            message: t("Validation failed"),
            errors: transform_validation_errors(e),
        });
    }

    Ok(ParsedBankMultipart { input, logo })
}

fn bank_output(bank: &generated::models::BankRecord) -> BankOutput {
    BankOutput {
        id: bank.id.into(),
        country_iso2: bank.country_iso2.clone(),
        name: bank.name.clone(),
        code: bank.code.clone(),
        logo_url: bank.logo_url.clone(),
        status: bank.status,
        sort_order: bank.sort_order,
        created_at: bank.created_at,
        updated_at: bank.updated_at,
    }
}

async fn create(
    State(state): State<AppApiState>,
    _auth: AuthUser<AdminGuard>,
    multipart: Multipart,
) -> Result<ApiResponse<BankOutput>, AppError> {
    let parsed = parse_bank_multipart(&state, multipart).await?;
    let bank = workflow::create(&state, parsed.input, parsed.logo).await?;

    Ok(ApiResponse::success(bank_output(&bank), &t("Bank created")))
}

async fn detail(
    State(state): State<AppApiState>,
    _auth: AuthUser<AdminGuard>,
    Path(id): Path<i64>,
) -> Result<ApiResponse<BankOutput>, AppError> {
    let bank = workflow::detail(&state, id).await?;

    Ok(ApiResponse::success(bank_output(&bank), &t("Bank detail")))
}

async fn update(
    State(state): State<AppApiState>,
    _auth: AuthUser<AdminGuard>,
    Path(id): Path<i64>,
    multipart: Multipart,
) -> Result<ApiResponse<BankOutput>, AppError> {
    let parsed = parse_bank_multipart(&state, multipart).await?;
    let bank = workflow::update(&state, id, parsed.input, parsed.logo).await?;

    Ok(ApiResponse::success(bank_output(&bank), &t("Bank updated")))
}

async fn delete(
    State(state): State<AppApiState>,
    _auth: AuthUser<AdminGuard>,
    Path(id): Path<i64>,
) -> Result<ApiResponse<serde_json::Value>, AppError> {
    workflow::delete(&state, id).await?;

    Ok(ApiResponse::success(serde_json::json!({ "deleted": true }), &t("Bank deleted")))
}
