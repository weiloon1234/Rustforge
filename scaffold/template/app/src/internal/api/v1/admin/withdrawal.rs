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
use generated::{guards::AdminGuard, models::WithdrawalStatus, permissions::Permission};
use uuid::Uuid;

use crate::{
    contracts::api::v1::admin::withdrawal::{AdminWithdrawalReviewInput, WithdrawalOutput},
    internal::{api::state::AppApiState, workflows::withdrawal as workflow},
};

use super::receipt_upload::{build_attachment_url, parse_receipt_multipart, validate_attachment_allowed};

pub fn router(state: AppApiState) -> ApiRouter {
    ApiRouter::new()
        .api_route(
            "/{id}/review",
            with_permission_check_post_with(
                review_withdrawal,
                AdminGuard,
                PermissionMode::Any,
                [Permission::WithdrawalManage.as_str()],
                |op| op.summary("Review withdrawal").tag("Admin Withdrawals"),
            ),
        )
        .api_route(
            "/{id}/upload-receipt",
            with_permission_check_post_with(
                upload_receipt,
                AdminGuard,
                PermissionMode::Any,
                [Permission::WithdrawalManage.as_str()],
                |op| op.summary("Upload withdrawal receipt").tag("Admin Withdrawals"),
            ),
        )
        .with_state(state)
}

async fn review_withdrawal(
    State(state): State<AppApiState>,
    auth: AuthUser<AdminGuard>,
    Path(id): Path<i64>,
    ContractJson(req): ContractJson<AdminWithdrawalReviewInput>,
) -> Result<ApiResponse<WithdrawalOutput>, AppError> {
    let withdrawal = workflow::review_withdrawal(&state, auth.user.id, id, req).await?;

    Ok(ApiResponse::success(
        WithdrawalOutput {
            id: withdrawal.id.into(),
            owner_type: withdrawal.owner_type,
            owner_id: withdrawal.owner_id.into(),
            credit_type: withdrawal.credit_type,
            withdrawal_method: withdrawal.withdrawal_method,
            bank_id: withdrawal.bank_id.map(Into::into),
            bank_account_name: withdrawal.bank_account_name.clone(),
            bank_account_number: withdrawal.bank_account_number.clone(),
            crypto_network_id: withdrawal.crypto_network_id.map(Into::into),
            crypto_wallet_address: withdrawal.crypto_wallet_address.clone(),
            conversion_rate: withdrawal.conversion_rate,
            status: withdrawal.status,
            amount: withdrawal.amount,
            fee: withdrawal.fee,
            net_amount: withdrawal.net_amount,
            related_key: withdrawal.related_key.clone(),
            remark: withdrawal.remark.clone(),
            admin_remark: withdrawal.admin_remark.clone(),
            reviewed_at: withdrawal.reviewed_at,
            created_at: withdrawal.created_at,
        },
        &t("Withdrawal reviewed"),
    ))
}

async fn upload_receipt(
    State(state): State<AppApiState>,
    _auth: AuthUser<AdminGuard>,
    Path(id): Path<i64>,
    multipart: Multipart,
) -> Result<ApiResponse<serde_json::Value>, AppError> {
    let withdrawal = workflow::detail(&state, id).await?;
    if withdrawal.status != WithdrawalStatus::Pending
        && withdrawal.status != WithdrawalStatus::Processing
    {
        return Err(AppError::BadRequest(t(
            "Receipt can only be uploaded for pending or processing withdrawals",
        )));
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
        "withdrawals/{:04}/{:02}/{:02}/receipt/{}.{}",
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
    generated::models::WithdrawalModel::query(core_db::common::sql::DbConn::pool(&state.db))
        .where_col(
            generated::models::WithdrawalCol::ID,
            core_db::common::sql::Op::Eq,
            id,
        )
        .patch()
        .assign(generated::models::WithdrawalCol::PARAMS, Some(params.clone()))
        .map_err(AppError::from)?
        .save()
        .await
        .map_err(AppError::from)?;

    Ok(ApiResponse::success(params, &t("Receipt uploaded")))
}
