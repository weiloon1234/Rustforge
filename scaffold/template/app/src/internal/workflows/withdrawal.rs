use core_db::common::sql::{DbConn, Op};
use core_i18n::t;
use core_web::error::AppError;
use generated::models::{
    CreditTransactionType, CreditType, OwnerType, UserCol, UserCreditTransactionCol,
    UserCreditTransactionModel, UserModel, WithdrawalCol, WithdrawalModel, WithdrawalRecord,
    WithdrawalReviewAction, WithdrawalStatus,
};
use time::OffsetDateTime;

use crate::{
    contracts::api::v1::admin::withdrawal::AdminWithdrawalReviewInput,
    internal::api::state::AppApiState,
};

pub async fn detail(state: &AppApiState, withdrawal_id: i64) -> Result<WithdrawalRecord, AppError> {
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
    let AdminWithdrawalReviewInput {
        action,
        admin_remark,
    } = req;

    let updated_withdrawal = match action {
        WithdrawalReviewAction::Process => {
            if withdrawal.status != WithdrawalStatus::Pending {
                return Err(AppError::BadRequest(t("Withdrawal is not pending")));
            }

            let mut rows = WithdrawalModel::query()
                .where_col(WithdrawalCol::ID, Op::Eq, withdrawal_id)
                .where_col(WithdrawalCol::STATUS, Op::Eq, WithdrawalStatus::Pending)
                .patch()
                .assign(WithdrawalCol::STATUS, WithdrawalStatus::Processing)
                .map_err(AppError::from)?
                .assign(WithdrawalCol::ADMIN_ID, Some(admin_id))
                .map_err(AppError::from)?
                .assign(WithdrawalCol::ADMIN_REMARK, admin_remark)
                .map_err(AppError::from)?
                .assign(WithdrawalCol::REVIEWED_AT, Some(now))
                .map_err(AppError::from)?
                .returning_all()
                .fetch(DbConn::pool(&state.db))
                .await
                .map_err(AppError::from)?;
            rows.pop()
                .ok_or_else(|| AppError::BadRequest(t("Withdrawal is not pending")))?
        }
        WithdrawalReviewAction::Approve => {
            if withdrawal.status != WithdrawalStatus::Processing {
                return Err(AppError::BadRequest(t(
                    "Withdrawal must be in processing status to approve",
                )));
            }

            let scope = DbConn::pool(&state.db)
                .begin_scope()
                .await
                .map_err(AppError::from)?;
            let conn = scope.conn();

            // Update withdrawal status
            let mut rows = WithdrawalModel::query()
                .where_col(WithdrawalCol::ID, Op::Eq, withdrawal_id)
                .where_col(WithdrawalCol::STATUS, Op::Eq, WithdrawalStatus::Processing)
                .patch()
                .assign(WithdrawalCol::STATUS, WithdrawalStatus::Approved)
                .map_err(AppError::from)?
                .assign(WithdrawalCol::ADMIN_ID, Some(admin_id))
                .map_err(AppError::from)?
                .assign(WithdrawalCol::ADMIN_REMARK, admin_remark)
                .map_err(AppError::from)?
                .assign(WithdrawalCol::REVIEWED_AT, Some(now))
                .map_err(AppError::from)?
                .returning_all()
                .fetch(conn.clone())
                .await
                .map_err(AppError::from)?;
            let updated_withdrawal = rows.pop().ok_or_else(|| {
                AppError::BadRequest(t("Withdrawal must be in processing status to approve"))
            })?;

            // Deduct balance for User owner_type
            if updated_withdrawal.owner_type == OwnerType::User {
                // Insert credit transaction for withdrawal (net_amount = amount after fees)
                UserCreditTransactionModel::create()
                    .set(
                        UserCreditTransactionCol::USER_ID,
                        updated_withdrawal.owner_id,
                    )?
                    .set(UserCreditTransactionCol::ADMIN_ID, Some(admin_id))?
                    .set(
                        UserCreditTransactionCol::CREDIT_TYPE,
                        updated_withdrawal.credit_type,
                    )?
                    .set(
                        UserCreditTransactionCol::AMOUNT,
                        -updated_withdrawal.net_amount,
                    )?
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
                    .save(conn.clone())
                    .await
                    .map_err(AppError::from)?;

                // Decrement user balance atomically
                let update = match updated_withdrawal.credit_type {
                    CreditType::Credit1 => UserModel::query()
                        .where_col(UserCol::ID, Op::Eq, updated_withdrawal.owner_id)
                        .patch()
                        .increment(UserCol::CREDIT_1, -updated_withdrawal.net_amount),
                    CreditType::Credit2 => UserModel::query()
                        .where_col(UserCol::ID, Op::Eq, updated_withdrawal.owner_id)
                        .patch()
                        .increment(UserCol::CREDIT_2, -updated_withdrawal.net_amount),
                };
                update
                    .map_err(AppError::from)?
                    .save(conn.clone())
                    .await
                    .map_err(AppError::from)?;
            }

            scope.commit().await.map_err(AppError::from)?;
            updated_withdrawal
        }
        WithdrawalReviewAction::Reject => {
            if withdrawal.status != WithdrawalStatus::Pending
                && withdrawal.status != WithdrawalStatus::Processing
            {
                return Err(AppError::BadRequest(t(
                    "Withdrawal cannot be rejected in current status",
                )));
            }

            let mut rows = WithdrawalModel::query()
                .where_col(WithdrawalCol::ID, Op::Eq, withdrawal_id)
                .where_in(
                    WithdrawalCol::STATUS,
                    [WithdrawalStatus::Pending, WithdrawalStatus::Processing],
                )
                .patch()
                .assign(WithdrawalCol::STATUS, WithdrawalStatus::Rejected)
                .map_err(AppError::from)?
                .assign(WithdrawalCol::ADMIN_ID, Some(admin_id))
                .map_err(AppError::from)?
                .assign(WithdrawalCol::ADMIN_REMARK, admin_remark)
                .map_err(AppError::from)?
                .assign(WithdrawalCol::REVIEWED_AT, Some(now))
                .map_err(AppError::from)?
                .returning_all()
                .fetch(DbConn::pool(&state.db))
                .await
                .map_err(AppError::from)?;
            rows.pop().ok_or_else(|| {
                AppError::BadRequest(t("Withdrawal cannot be rejected in current status"))
            })?
        }
    };

    crate::internal::workflows::notification::dispatch_admin_notification_counts(state).await;

    Ok(updated_withdrawal)
}
