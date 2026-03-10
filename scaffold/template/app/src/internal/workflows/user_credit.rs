use core_db::common::sql::{DbConn, Op};
use core_i18n::t;
use core_web::error::AppError;
use generated::models::{
    CreditTransactionType, CreditType, User, UserCreditTransaction, UserCreditTransactionView,
    UserQuery,
};
use rust_decimal::Decimal;

use crate::{
    contracts::api::v1::admin::user_credit::AdminCreditAdjustInput,
    internal::api::state::AppApiState,
};

pub async fn adjust_credit(
    state: &AppApiState,
    admin_id: i64,
    req: AdminCreditAdjustInput,
) -> Result<UserCreditTransactionView, AppError> {
    let username = req.username.trim().to_ascii_lowercase();
    let amount = req.amount;

    if amount.is_zero() {
        return Err(AppError::BadRequest(t("Amount must not be zero")));
    }

    // Convert AdjustableCreditType → CreditType (same i16 values)
    let credit_type = CreditType::from_storage(req.credit_type.as_str())
        .ok_or_else(|| AppError::BadRequest(t("Invalid credit type")))?;

    // Resolve user by username
    let user = UserQuery::new(DbConn::pool(&state.db), None)
        .where_username(Op::Eq, username)
        .first()
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(t("User not found")))?;

    let transaction_type = if amount > Decimal::ZERO {
        CreditTransactionType::AdminAdd
    } else {
        CreditTransactionType::AdminDeduct
    };

    let has_custom_desc = req.custom_description.is_some();
    let custom_desc_default = req
        .custom_description
        .as_ref()
        .and_then(|m| m.get("en").or_else(|| m.values().next()))
        .cloned();

    // Begin transaction scope — both operations share the same DB transaction
    let scope = DbConn::pool(&state.db).begin_scope().await.map_err(AppError::from)?;
    let conn = scope.conn();

    // Insert transaction record
    let txn = UserCreditTransaction::new(conn.clone(), None)
        .insert()
        .set_user_id(user.id)
        .set_admin_id(Some(admin_id))
        .set_credit_type(credit_type)
        .set_amount(amount)
        .set_transaction_type(transaction_type)
        .set_related_key(None)
        .set_remark(req.remark)
        .set_custom_description(has_custom_desc)
        .set_custom_description_text(custom_desc_default)
        .save()
        .await
        .map_err(AppError::from)?;

    // Save localized custom description text (multi-locale)
    if let Some(ref descriptions) = req.custom_description {
        generated::localized::upsert_localized_many(
            conn.clone(),
            "user_credit_transaction",
            txn.id,
            "custom_description_text",
            descriptions,
        )
        .await
        .map_err(AppError::from)?;
    }

    // Atomic relative balance update
    let mut update = User::new(conn, None).update().where_id(Op::Eq, user.id);
    update = match credit_type {
        CreditType::Credit1 => update.increment_credit_1(amount),
        CreditType::Credit2 => update.increment_credit_2(amount),
    };
    update.save().await.map_err(AppError::from)?;

    scope.commit().await.map_err(AppError::from)?;

    Ok(txn)
}
