use core_db::common::sql::{DbConn, Op};
use core_i18n::t;
use core_web::error::AppError;
use generated::models::{
    CreditTransactionType, CreditType, DepositCol, DepositModel, DepositRecord,
    DepositReviewAction, DepositStatus, OwnerType, UserCol, UserCreditTransactionCol,
    UserCreditTransactionModel, UserModel,
};
use time::OffsetDateTime;

use crate::{
    contracts::api::v1::admin::deposit::AdminDepositReviewInput,
    internal::api::state::AppApiState,
};

pub async fn detail(
    state: &AppApiState,
    deposit_id: i64,
) -> Result<DepositRecord, AppError> {
    DepositModel::find(DbConn::pool(&state.db), deposit_id)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(t("Deposit not found")))
}

pub async fn review_deposit(
    state: &AppApiState,
    admin_id: i64,
    deposit_id: i64,
    req: AdminDepositReviewInput,
) -> Result<DepositRecord, AppError> {
    let deposit = DepositModel::find(DbConn::pool(&state.db), deposit_id)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(t("Deposit not found")))?;

    if deposit.status != DepositStatus::Pending {
        return Err(AppError::BadRequest(t("Deposit is not pending")));
    }

    let now = OffsetDateTime::now_utc();

    match req.action {
        DepositReviewAction::Approve => {
            let scope = DbConn::pool(&state.db).begin_scope().await.map_err(AppError::from)?;
            let conn = scope.conn();

            // Update deposit status
            DepositModel::query(conn.clone())
                .where_col(DepositCol::ID, Op::Eq, deposit_id)
                .patch()
                .assign(DepositCol::STATUS, DepositStatus::Approved)
                .map_err(AppError::from)?
                .assign(DepositCol::ADMIN_ID, Some(admin_id))
                .map_err(AppError::from)?
                .assign(DepositCol::ADMIN_REMARK, req.admin_remark)
                .map_err(AppError::from)?
                .assign(DepositCol::REVIEWED_AT, Some(now))
                .map_err(AppError::from)?
                .save()
                .await
                .map_err(AppError::from)?;

            // Credit the owner (for User owner_type)
            if deposit.owner_type == OwnerType::User {
                // Insert credit transaction
                UserCreditTransactionModel::create(conn.clone())
                    .set(UserCreditTransactionCol::USER_ID, deposit.owner_id)?
                    .set(UserCreditTransactionCol::ADMIN_ID, Some(admin_id))?
                    .set(UserCreditTransactionCol::CREDIT_TYPE, deposit.credit_type)?
                    .set(UserCreditTransactionCol::AMOUNT, deposit.net_amount)?
                    .set(
                        UserCreditTransactionCol::TRANSACTION_TYPE,
                        CreditTransactionType::TopUp,
                    )?
                    .set(
                        UserCreditTransactionCol::RELATED_KEY,
                        Some(deposit_id.to_string()),
                    )?
                    .set(
                        UserCreditTransactionCol::REMARK,
                        Some(format!("Deposit #{}", deposit_id)),
                    )?
                    .set(UserCreditTransactionCol::CUSTOM_DESCRIPTION, false)?
                    .save()
                    .await
                    .map_err(AppError::from)?;

                // Increment user balance atomically
                let update = match deposit.credit_type {
                    CreditType::Credit1 => UserModel::query(conn.clone())
                        .where_col(UserCol::ID, Op::Eq, deposit.owner_id)
                        .patch()
                        .increment(UserCol::CREDIT_1, deposit.net_amount),
                    CreditType::Credit2 => UserModel::query(conn.clone())
                        .where_col(UserCol::ID, Op::Eq, deposit.owner_id)
                        .patch()
                        .increment(UserCol::CREDIT_2, deposit.net_amount),
                };
                update
                    .map_err(AppError::from)?
                    .save()
                    .await
                    .map_err(AppError::from)?;
            }

            scope.commit().await.map_err(AppError::from)?;
        }
        DepositReviewAction::Reject => {
            DepositModel::query(DbConn::pool(&state.db))
                .where_col(DepositCol::ID, Op::Eq, deposit_id)
                .patch()
                .assign(DepositCol::STATUS, DepositStatus::Rejected)
                .map_err(AppError::from)?
                .assign(DepositCol::ADMIN_ID, Some(admin_id))
                .map_err(AppError::from)?
                .assign(DepositCol::ADMIN_REMARK, req.admin_remark)
                .map_err(AppError::from)?
                .assign(DepositCol::REVIEWED_AT, Some(now))
                .map_err(AppError::from)?
                .save()
                .await
                .map_err(AppError::from)?;
        }
    }

    crate::internal::workflows::notification::dispatch_admin_notification_counts(state).await;

    detail(state, deposit_id).await
}
