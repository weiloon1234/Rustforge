use axum::extract::State;
use core_i18n::t;
use core_web::{
    auth::AuthUser,
    authz::PermissionMode,
    error::AppError,
    contracts::ContractJson,
    openapi::{with_permission_check_post_with, ApiRouter},
    response::ApiResponse,
};
use generated::{guards::AdminGuard, permissions::Permission};

use crate::{
    contracts::api::v1::admin::user_credit::{AdminCreditAdjustInput, UserCreditTransactionOutput},
    internal::{api::state::AppApiState, workflows::user_credit as workflow},
};

pub fn router(state: AppApiState) -> ApiRouter {
    ApiRouter::new()
        .api_route(
            "/adjust",
            with_permission_check_post_with(
                adjust_credit,
                AdminGuard,
                PermissionMode::Any,
                [Permission::UserCreditManage.as_str()],
                |op| op.summary("Adjust user credit").tag("Admin User Credit"),
            ),
        )
        .with_state(state)
}

async fn adjust_credit(
    State(state): State<AppApiState>,
    auth: AuthUser<AdminGuard>,
    ContractJson(req): ContractJson<AdminCreditAdjustInput>,
) -> Result<ApiResponse<UserCreditTransactionOutput>, AppError> {
    let txn = workflow::adjust_credit(&state, auth.user.id, req).await?;

    Ok(ApiResponse::success(
        UserCreditTransactionOutput {
            id: txn.id.into(),
            user_id: txn.user_id.into(),
            credit_type: txn.credit_type,
            amount: txn.amount,
            transaction_type: txn.transaction_type,
            related_key: txn.related_key,
            remark: txn.remark,
            created_at: txn.created_at,
        },
        &t("Credit adjusted"),
    ))
}
