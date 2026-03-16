use core_db::common::sql::{DbConn, Op};
use core_i18n::t;
use core_web::error::AppError;
use generated::models::{
    CreditTransactionType, CreditType, OwnerType, UserCol, UserCreditTransactionModel, UserModel,
    UserCreditTransactionCol, WithdrawalCol, WithdrawalModel, WithdrawalRecord,
    WithdrawalReviewAction, WithdrawalStatus,
};
use time::OffsetDateTime;

use crate::{
    contracts::api::v1::admin::withdrawal::AdminWithdrawalReviewInput,
    internal::api::state::AppApiState,
};

pub async fn detail(
    state: &AppApiState,
    withdrawal_id: i64,
) -> Result<WithdrawalRecord, AppError> {
    WithdrawalModel::find(DbConn::pool(&state.db), withdrawal_id)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(t("Withdrawal not found")))
}

pub async fn review_withdrawal(
    state: &AppApiState,
    admin_id: i64,
    withdrawal_id: i64,
    req: AdminWithdrawalReviewInput,
) -> Result<WithdrawalRecord, AppError> {
    let withdrawal = WithdrawalModel::find(DbConn::pool(&state.db), withdrawal_id)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(t("Withdrawal not found")))?;

    let now = OffsetDateTime::now_utc();

    match req.action {
        WithdrawalReviewAction::Process => {
            if withdrawal.status != WithdrawalStatus::Pending {
                return Err(AppError::BadRequest(t("Withdrawal is not pending")));
            }

            WithdrawalModel::query(DbConn::pool(&state.db))
                .where_col(WithdrawalCol::ID, Op::Eq, withdrawal_id)
                .patch()
                .assign(WithdrawalCol::STATUS, WithdrawalStatus::Processing)
                .map_err(AppError::from)?
                .assign(WithdrawalCol::ADMIN_ID, Some(admin_id))
                .map_err(AppError::from)?
                .assign(WithdrawalCol::ADMIN_REMARK, req.admin_remark)
                .map_err(AppError::from)?
                .assign(WithdrawalCol::REVIEWED_AT, Some(now))
                .map_err(AppError::from)?
                .save()
                .await
                .map_err(AppError::from)?;
        }
        WithdrawalReviewAction::Approve => {
            if withdrawal.status != WithdrawalStatus::Processing {
                return Err(AppError::BadRequest(t("Withdrawal must be in processing status to approve")));
            }

            let scope = DbConn::pool(&state.db).begin_scope().await.map_err(AppError::from)?;
            let conn = scope.conn();

            // Update withdrawal status
            WithdrawalModel::query(conn.clone())
                .where_col(WithdrawalCol::ID, Op::Eq, withdrawal_id)
                .patch()
                .assign(WithdrawalCol::STATUS, WithdrawalStatus::Approved)
                .map_err(AppError::from)?
                .assign(WithdrawalCol::ADMIN_ID, Some(admin_id))
                .map_err(AppError::from)?
                .assign(WithdrawalCol::ADMIN_REMARK, req.admin_remark)
                .map_err(AppError::from)?
                .assign(WithdrawalCol::REVIEWED_AT, Some(now))
                .map_err(AppError::from)?
                .save()
                .await
                .map_err(AppError::from)?;

            // Deduct balance for User owner_type
            if withdrawal.owner_type == OwnerType::User {
                // Insert credit transaction for withdrawal (net_amount = amount after fees)
                UserCreditTransactionModel::create(conn.clone())
                    .set(UserCreditTransactionCol::USER_ID, withdrawal.owner_id)?
                    .set(UserCreditTransactionCol::ADMIN_ID, Some(admin_id))?
                    .set(UserCreditTransactionCol::CREDIT_TYPE, withdrawal.credit_type)?
                    .set(UserCreditTransactionCol::AMOUNT, -withdrawal.net_amount)?
                    .set(
                        UserCreditTransactionCol::TRANSACTION_TYPE,
                        CreditTransactionType::Withdraw,
                    )?
                    .set(
                        UserCreditTransactionCol::RELATED_KEY,
                        Some(withdrawal_id.to_string()),
                    )?
                    .set(
                        UserCreditTransactionCol::REMARK,
                        Some(format!("Withdrawal #{}", withdrawal_id)),
                    )?
                    .set(UserCreditTransactionCol::CUSTOM_DESCRIPTION, false)?
                    .save()
                    .await
                    .map_err(AppError::from)?;

                // Decrement user balance atomically
                let update = match withdrawal.credit_type {
                    CreditType::Credit1 => UserModel::query(conn.clone())
                        .where_col(UserCol::ID, Op::Eq, withdrawal.owner_id)
                        .patch()
                        .increment(UserCol::CREDIT_1, -withdrawal.net_amount),
                    CreditType::Credit2 => UserModel::query(conn.clone())
                        .where_col(UserCol::ID, Op::Eq, withdrawal.owner_id)
                        .patch()
                        .increment(UserCol::CREDIT_2, -withdrawal.net_amount),
                };
                update
                    .map_err(AppError::from)?
                    .save()
                    .await
                    .map_err(AppError::from)?;
            }

            scope.commit().await.map_err(AppError::from)?;
        }
        WithdrawalReviewAction::Reject => {
            if withdrawal.status != WithdrawalStatus::Pending
                && withdrawal.status != WithdrawalStatus::Processing
            {
                return Err(AppError::BadRequest(t("Withdrawal cannot be rejected in current status")));
            }

            WithdrawalModel::query(DbConn::pool(&state.db))
                .where_col(WithdrawalCol::ID, Op::Eq, withdrawal_id)
                .patch()
                .assign(WithdrawalCol::STATUS, WithdrawalStatus::Rejected)
                .map_err(AppError::from)?
                .assign(WithdrawalCol::ADMIN_ID, Some(admin_id))
                .map_err(AppError::from)?
                .assign(WithdrawalCol::ADMIN_REMARK, req.admin_remark)
                .map_err(AppError::from)?
                .assign(WithdrawalCol::REVIEWED_AT, Some(now))
                .map_err(AppError::from)?
                .save()
                .await
                .map_err(AppError::from)?;
        }
    }

    crate::internal::workflows::notification::dispatch_admin_notification_counts(state).await;

    detail(state, withdrawal_id).await
}
