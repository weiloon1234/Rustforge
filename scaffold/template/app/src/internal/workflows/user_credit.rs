use core_db::common::sql::{DbConn, Op};
use core_i18n::t;
use core_web::error::AppError;
use generated::models::{
    CreditTransactionType, CreditType, UserCol, UserCreditTransactionCol,
    UserCreditTransactionModel, UserCreditTransactionRecord, UserModel,
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
) -> Result<UserCreditTransactionRecord, AppError> {
    let username = req.username.to_ascii_lowercase();
    let amount = req.amount;

    if amount.is_zero() {
        return Err(AppError::BadRequest(t("Amount must not be zero")));
    }

    // Convert AdjustableCreditType → CreditType (same i16 values)
    let credit_type = CreditType::from_storage(req.credit_type.as_str())
        .ok_or_else(|| AppError::BadRequest(t("Invalid credit type")))?;

    // Resolve user by username
    let user = UserModel::query(DbConn::pool(&state.db))
        .where_col(UserCol::USERNAME, Op::Eq, username)
        .first()
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(t("User not found")))?;

    let transaction_type = if amount > Decimal::ZERO {
        CreditTransactionType::AdminAdd
    } else {
        CreditTransactionType::AdminDeduct
    };

    // Begin transaction scope — both operations share the same DB transaction
    let scope = DbConn::pool(&state.db).begin_scope().await.map_err(AppError::from)?;
    let conn = scope.conn();

    // Insert transaction record
    let txn = UserCreditTransactionModel::create(conn.clone())
        .set(UserCreditTransactionCol::USER_ID, user.id)?
        .set(UserCreditTransactionCol::ADMIN_ID, Some(admin_id))?
        .set(UserCreditTransactionCol::CREDIT_TYPE, credit_type)?
        .set(UserCreditTransactionCol::AMOUNT, amount)?
        .set(UserCreditTransactionCol::TRANSACTION_TYPE, transaction_type)?
        .set(UserCreditTransactionCol::RELATED_KEY, None::<String>)?
        .set(UserCreditTransactionCol::REMARK, req.remark)?
        .set(UserCreditTransactionCol::CUSTOM_DESCRIPTION, req.custom_description)?
        .save()
        .await
        .map_err(AppError::from)?;

    if let Some(custom_description_text) = req.custom_description_text {
        txn.upsert_custom_description_text(conn.clone(), Some(custom_description_text))
            .await
            .map_err(AppError::from)?;
    }

    // Atomic relative balance update
    let update = match credit_type {
        CreditType::Credit1 => UserModel::query(conn.clone())
            .where_col(UserCol::ID, Op::Eq, user.id)
            .patch()
            .increment(UserCol::CREDIT_1, amount),
        CreditType::Credit2 => UserModel::query(conn.clone())
            .where_col(UserCol::ID, Op::Eq, user.id)
            .patch()
            .increment(UserCol::CREDIT_2, amount),
    };
    update
        .map_err(AppError::from)?
        .save()
        .await
        .map_err(AppError::from)?;

    let txn = UserCreditTransactionModel::find(conn, txn.id)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::BadRequest(t("Failed to create credit transaction")))?;

    scope.commit().await.map_err(AppError::from)?;

    Ok(txn)
}
