use axum::extract::{Multipart, Path, State};
use core_i18n::t;
use core_web::{
    auth::AuthUser,
    authz::PermissionMode,
    contracts::ContractJson,
    error::AppError,
    openapi::{with_permission_check_post_with, ApiRouter},
    response::ApiResponse,
};
use generated::{guards::AdminGuard, models::DepositStatus, permissions::Permission};
use uuid::Uuid;

use crate::{
    contracts::api::v1::admin::deposit::{AdminDepositReviewInput, DepositOutput},
    internal::{api::state::AppApiState, workflows::deposit as workflow},
};

use super::receipt_upload::{build_attachment_url, parse_receipt_multipart, validate_attachment_allowed};

pub fn router(state: AppApiState) -> ApiRouter {
    ApiRouter::new()
        .api_route(
            "/{id}/review",
            with_permission_check_post_with(
                review_deposit,
                AdminGuard,
                PermissionMode::Any,
                [Permission::DepositManage.as_str()],
                |op| op.summary("Review deposit").tag("Admin Deposits"),
            ),
        )
        .api_route(
            "/{id}/upload-receipt",
            with_permission_check_post_with(
                upload_receipt,
                AdminGuard,
                PermissionMode::Any,
                [Permission::DepositManage.as_str()],
                |op| op.summary("Upload deposit receipt").tag("Admin Deposits"),
            ),
        )
        .with_state(state)
}

async fn review_deposit(
    State(state): State<AppApiState>,
    auth: AuthUser<AdminGuard>,
    Path(id): Path<i64>,
    ContractJson(req): ContractJson<AdminDepositReviewInput>,
) -> Result<ApiResponse<DepositOutput>, AppError> {
    let deposit = workflow::review_deposit(&state, auth.user.id, id, req).await?;

    Ok(ApiResponse::success(
        DepositOutput {
            id: deposit.id.into(),
            owner_type: deposit.owner_type,
            owner_id: deposit.owner_id.into(),
            credit_type: deposit.credit_type,
            deposit_method: deposit.deposit_method,
            company_bank_account_id: deposit.company_bank_account_id.map(Into::into),
            company_crypto_account_id: deposit.company_crypto_account_id.map(Into::into),
            conversion_rate: deposit.conversion_rate,
            status: deposit.status,
            amount: deposit.amount,
            fee: deposit.fee,
            net_amount: deposit.net_amount,
            related_key: deposit.related_key.clone(),
            remark: deposit.remark.clone(),
            admin_remark: deposit.admin_remark.clone(),
            reviewed_at: deposit.reviewed_at,
            created_at: deposit.created_at,
        },
        &t("Deposit reviewed"),
    ))
}

async fn upload_receipt(
    State(state): State<AppApiState>,
    _auth: AuthUser<AdminGuard>,
    Path(id): Path<i64>,
    multipart: Multipart,
) -> Result<ApiResponse<serde_json::Value>, AppError> {
    let deposit = workflow::detail(&state, id).await?;
    if deposit.status != DepositStatus::Pending {
        return Err(AppError::BadRequest(t("Deposit is not pending")));
    }

    let (file_name, content_type, bytes) = parse_receipt_multipart(multipart).await?;

    let rules = generated::get_attachment_rules("image")
        .ok_or_else(|| AppError::BadRequest(t("Unknown attachment type: image")))?;
    validate_attachment_allowed(&rules.allowed, file_name.as_deref(), &content_type)?;

    let ext = file_name
        .as_deref()
        .and_then(|name| {
            std::path::Path::new(name)
                .extension()
                .and_then(|e| e.to_str())
        })
        .unwrap_or_else(|| content_type.split('/').nth(1).unwrap_or("bin"));

    let now = time::OffsetDateTime::now_utc();
    let object_key = format!(
        "deposits/{:04}/{:02}/{:02}/receipt/{}.{}",
        now.year(),
        now.month() as u8,
        now.day(),
        Uuid::new_v4(),
        ext
    );

    state
        .storage
        .put(&object_key, bytes.clone(), &content_type)
        .await
        .map_err(AppError::from)?;

    let base_url = std::env::var("S3_URL").ok();
    let receipt_url = build_attachment_url(&object_key, base_url.as_deref());

    let params = serde_json::json!({ "receipt_url": receipt_url, "receipt_path": object_key });
    generated::models::DepositModel::query(core_db::common::sql::DbConn::pool(&state.db))
        .where_col(generated::models::DepositCol::ID, core_db::common::sql::Op::Eq, id)
        .patch()
        .assign(generated::models::DepositCol::PARAMS, Some(params.clone()))
        .map_err(AppError::from)?
        .save()
        .await
        .map_err(AppError::from)?;

    Ok(ApiResponse::success(params, &t("Receipt uploaded")))
}
