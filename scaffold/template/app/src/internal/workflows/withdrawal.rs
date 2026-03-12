use core_db::common::sql::{DbConn, Op};
use core_i18n::t;
use core_web::error::AppError;
use generated::models::{
    CreditTransactionType, CreditType, OwnerType, User, UserCreditTransaction, Withdrawal,
    WithdrawalQuery, WithdrawalReviewAction, WithdrawalStatus, WithdrawalWithRelations,
};
use time::OffsetDateTime;

use crate::{
    contracts::api::v1::admin::withdrawal::AdminWithdrawalReviewInput,
    internal::api::state::AppApiState,
};

pub async fn detail(
    state: &AppApiState,
    withdrawal_id: i64,
) -> Result<WithdrawalWithRelations, AppError> {
    WithdrawalQuery::new(DbConn::pool(&state.db), None)
        .where_id(Op::Eq, withdrawal_id)
        .first()
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(t("Withdrawal not found")))
}

pub async fn review_withdrawal(
    state: &AppApiState,
    admin_id: i64,
    withdrawal_id: i64,
    req: AdminWithdrawalReviewInput,
) -> Result<WithdrawalWithRelations, AppError> {
    let withdrawal = WithdrawalQuery::new(DbConn::pool(&state.db), None)
        .where_id(Op::Eq, withdrawal_id)
        .first()
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(t("Withdrawal not found")))?;

    let now = OffsetDateTime::now_utc();

    match req.action {
        WithdrawalReviewAction::Process => {
            if withdrawal.status != WithdrawalStatus::Pending {
                return Err(AppError::BadRequest(t("Withdrawal is not pending")));
            }

            Withdrawal::new(DbConn::pool(&state.db), None)
                .update()
                .where_id(Op::Eq, withdrawal_id)
                .set_status(WithdrawalStatus::Processing)
                .set_admin_id(Some(admin_id))
                .set_admin_remark(req.admin_remark)
                .set_reviewed_at(Some(now))
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
            Withdrawal::new(conn.clone(), None)
                .update()
                .where_id(Op::Eq, withdrawal_id)
                .set_status(WithdrawalStatus::Approved)
                .set_admin_id(Some(admin_id))
                .set_admin_remark(req.admin_remark)
                .set_reviewed_at(Some(now))
                .save()
                .await
                .map_err(AppError::from)?;

            // Deduct balance for User owner_type
            if withdrawal.owner_type == OwnerType::User {
                // Insert credit transaction for withdrawal (net_amount = amount after fees)
                UserCreditTransaction::new(conn.clone(), None)
                    .insert()
                    .set_user_id(withdrawal.owner_id)
                    .set_admin_id(Some(admin_id))
                    .set_credit_type(withdrawal.credit_type)
                    .set_amount(-withdrawal.net_amount)
                    .set_transaction_type(CreditTransactionType::Withdraw)
                    .set_related_key(Some(withdrawal_id.to_string()))
                    .set_remark(Some(format!("Withdrawal #{}", withdrawal_id)))
                    .set_custom_description(false)
                    .save()
                    .await
                    .map_err(AppError::from)?;

                // Decrement user balance atomically
                let mut update = User::new(conn, None).update().where_id(Op::Eq, withdrawal.owner_id);
                update = match withdrawal.credit_type {
                    CreditType::Credit1 => update.increment_credit_1(-withdrawal.net_amount),
                    CreditType::Credit2 => update.increment_credit_2(-withdrawal.net_amount),
                };
                update.save().await.map_err(AppError::from)?;
            }

            scope.commit().await.map_err(AppError::from)?;
        }
        WithdrawalReviewAction::Reject => {
            if withdrawal.status != WithdrawalStatus::Pending
                && withdrawal.status != WithdrawalStatus::Processing
            {
                return Err(AppError::BadRequest(t("Withdrawal cannot be rejected in current status")));
            }

            Withdrawal::new(DbConn::pool(&state.db), None)
                .update()
                .where_id(Op::Eq, withdrawal_id)
                .set_status(WithdrawalStatus::Rejected)
                .set_admin_id(Some(admin_id))
                .set_admin_remark(req.admin_remark)
                .set_reviewed_at(Some(now))
                .save()
                .await
                .map_err(AppError::from)?;
        }
    }

    detail(state, withdrawal_id).await
}
