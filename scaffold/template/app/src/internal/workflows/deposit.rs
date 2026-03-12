use core_db::common::sql::{DbConn, Op};
use core_i18n::t;
use core_web::error::AppError;
use generated::models::{
    CreditTransactionType, CreditType, Deposit, DepositQuery, DepositReviewAction, DepositStatus,
    DepositWithRelations, OwnerType, User, UserCreditTransaction,
};
use time::OffsetDateTime;

use crate::{
    contracts::api::v1::admin::deposit::AdminDepositReviewInput,
    internal::api::state::AppApiState,
};

pub async fn detail(
    state: &AppApiState,
    deposit_id: i64,
) -> Result<DepositWithRelations, AppError> {
    DepositQuery::new(DbConn::pool(&state.db), None)
        .where_id(Op::Eq, deposit_id)
        .first()
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(t("Deposit not found")))
}

pub async fn review_deposit(
    state: &AppApiState,
    admin_id: i64,
    deposit_id: i64,
    req: AdminDepositReviewInput,
) -> Result<DepositWithRelations, AppError> {
    let deposit = DepositQuery::new(DbConn::pool(&state.db), None)
        .where_id(Op::Eq, deposit_id)
        .first()
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
            Deposit::new(conn.clone(), None)
                .update()
                .where_id(Op::Eq, deposit_id)
                .set_status(DepositStatus::Approved)
                .set_admin_id(Some(admin_id))
                .set_admin_remark(req.admin_remark)
                .set_reviewed_at(Some(now))
                .save()
                .await
                .map_err(AppError::from)?;

            // Credit the owner (for User owner_type)
            if deposit.owner_type == OwnerType::User {
                // Insert credit transaction
                UserCreditTransaction::new(conn.clone(), None)
                    .insert()
                    .set_user_id(deposit.owner_id)
                    .set_admin_id(Some(admin_id))
                    .set_credit_type(deposit.credit_type)
                    .set_amount(deposit.net_amount)
                    .set_transaction_type(CreditTransactionType::TopUp)
                    .set_related_key(Some(deposit_id.to_string()))
                    .set_remark(Some(format!("Deposit #{}", deposit_id)))
                    .set_custom_description(false)
                    .save()
                    .await
                    .map_err(AppError::from)?;

                // Increment user balance atomically
                let mut update = User::new(conn, None).update().where_id(Op::Eq, deposit.owner_id);
                update = match deposit.credit_type {
                    CreditType::Credit1 => update.increment_credit_1(deposit.net_amount),
                    CreditType::Credit2 => update.increment_credit_2(deposit.net_amount),
                };
                update.save().await.map_err(AppError::from)?;
            }

            scope.commit().await.map_err(AppError::from)?;
        }
        DepositReviewAction::Reject => {
            Deposit::new(DbConn::pool(&state.db), None)
                .update()
                .where_id(Op::Eq, deposit_id)
                .set_status(DepositStatus::Rejected)
                .set_admin_id(Some(admin_id))
                .set_admin_remark(req.admin_remark)
                .set_reviewed_at(Some(now))
                .save()
                .await
                .map_err(AppError::from)?;
        }
    }

    detail(state, deposit_id).await
}
