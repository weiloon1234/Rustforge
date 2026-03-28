use core_db::common::{
    auth::hash::verify_password,
    sql::{generate_snowflake_i64, DbConn, Op, OrderDir},
};
use core_i18n::t;
use core_web::error::AppError;
use generated::models::{
    CreditType, CryptoNetworkCol, CryptoNetworkModel, CryptoNetworkStatus, OwnerType, UserModel,
    WithdrawalCol, WithdrawalMethod, WithdrawalModel, WithdrawalStatus,
};
use rust_decimal::Decimal;

use crate::{
    contracts::api::v1::user::wallet::{
        CryptoNetworkOption, UserWithdrawalHistoryResponse, UserWithdrawalInput,
        UserWithdrawalOutput, WalletLedgerEntry, WalletLedgerResponse,
    },
    internal::api::state::AppApiState,
};

/// Get authenticated user's credit ledger with cursor-based pagination
pub async fn get_ledger(
    state: &AppApiState,
    user_id: i64,
    limit: Option<i64>,
    cursor: Option<&str>,
) -> Result<WalletLedgerResponse, AppError> {
    let limit = limit.unwrap_or(20).min(50);
    let fetch_limit = limit + 1;
    let cursor_id: Option<i64> = cursor
        .map(|c| c.parse::<i64>())
        .transpose()
        .map_err(|_| AppError::BadRequest(t("Invalid cursor")))?;

    let mut rows = sqlx::query_as::<_, (i64, i16, Decimal, Option<String>, time::OffsetDateTime)>(
        "SELECT id, transaction_type, amount, related_key, created_at
         FROM user_credit_transactions
         WHERE user_id = $1
           AND credit_type = $2
           AND ($3::BIGINT IS NULL OR id < $3)
         ORDER BY id DESC
         LIMIT $4",
    )
    .bind(user_id)
    .bind(CreditType::Credit1 as i16)
    .bind(cursor_id)
    .bind(fetch_limit)
    .fetch_all(&state.db)
    .await
    .map_err(AppError::from)?;

    let has_more = rows.len() as i64 > limit;
    if has_more {
        rows.truncate(limit as usize);
    }
    let next_cursor = if has_more {
        rows.last().map(|r| r.0.to_string())
    } else {
        None
    };

    let items = rows
        .into_iter()
        .map(|(id, tx_type, amount, related_key, created_at)| {
            let transaction_type = match tx_type {
                101 => "admin_add",
                102 => "admin_deduct",
                201 => "transfer_in",
                202 => "transfer_out",
                301 => "withdraw",
                302 => "withdraw_refund",
                401 => "top_up",
                501 => "crash_bet",
                502 => "crash_win",
                _ => "unknown",
            }
            .to_string();
            WalletLedgerEntry {
                id: id.into(),
                transaction_type,
                amount: amount.into(),
                related_key,
                created_at: created_at.into(),
            }
        })
        .collect();

    Ok(WalletLedgerResponse { items, next_cursor })
}

/// Get enabled crypto networks for withdrawal dropdown
pub async fn get_crypto_networks(
    state: &AppApiState,
) -> Result<Vec<CryptoNetworkOption>, AppError> {
    let rows = CryptoNetworkModel::query()
        .where_col(CryptoNetworkCol::STATUS, Op::Eq, CryptoNetworkStatus::Enabled)
        .order_by(CryptoNetworkCol::SORT_ORDER, OrderDir::Asc)
        .all(DbConn::pool(&state.db))
        .await
        .map_err(AppError::from)?;

    Ok(rows
        .into_iter()
        .map(|r| CryptoNetworkOption {
            id: r.id.into(),
            name: r.name,
            symbol: r.symbol,
        })
        .collect())
}

/// Create a new withdrawal request for a user
pub async fn create_withdrawal(
    state: &AppApiState,
    user_id: i64,
    req: UserWithdrawalInput,
) -> Result<UserWithdrawalOutput, AppError> {
    // Load user
    let user = UserModel::find(DbConn::pool(&state.db), user_id)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(t("User not found")))?;

    // Verify password
    let valid = verify_password(&req.password, &user.password).map_err(AppError::from)?;
    if !valid {
        return Err(AppError::BadRequest(t("Invalid password")));
    }

    let amount: rust_decimal::Decimal = req.amount.into();

    // Validate amount > 0
    if amount <= Decimal::ZERO {
        return Err(AppError::BadRequest(t("Amount must be greater than zero")));
    }

    // Validate minimum withdrawal amount
    if amount < state.withdrawal_fee_config.min_amount {
        return Err(AppError::BadRequest(t("Amount is below minimum withdrawal amount")));
    }

    // Calculate fee
    let (fee, net_amount) = state.withdrawal_fee_config.calculate_fee(amount);

    // Validate sufficient balance
    if user.credit_1 < amount {
        return Err(AppError::BadRequest(t("Insufficient balance")));
    }

    // Validate crypto network exists and is enabled
    let network = CryptoNetworkModel::find(DbConn::pool(&state.db), req.crypto_network_id)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::BadRequest(t("Crypto network not found")))?;

    if network.status != CryptoNetworkStatus::Enabled {
        return Err(AppError::BadRequest(t("Crypto network is not enabled")));
    }

    // Insert withdrawal record
    let withdrawal_id = generate_snowflake_i64();
    WithdrawalModel::create()
        .set(WithdrawalCol::ID, withdrawal_id)?
        .set(WithdrawalCol::OWNER_TYPE, OwnerType::User)?
        .set(WithdrawalCol::OWNER_ID, user_id)?
        .set(WithdrawalCol::CREDIT_TYPE, CreditType::Credit1)?
        .set(WithdrawalCol::WITHDRAWAL_METHOD, WithdrawalMethod::Manual)?
        .set(WithdrawalCol::CRYPTO_NETWORK_ID, Some(req.crypto_network_id))?
        .set(WithdrawalCol::CRYPTO_WALLET_ADDRESS, Some(req.crypto_wallet_address.clone()))?
        .set(WithdrawalCol::STATUS, WithdrawalStatus::Pending)?
        .set(WithdrawalCol::AMOUNT, amount)?
        .set(WithdrawalCol::FEE, fee)?
        .set(WithdrawalCol::NET_AMOUNT, net_amount)?
        .save(DbConn::pool(&state.db))
        .await
        .map_err(AppError::from)?;

    Ok(UserWithdrawalOutput {
        id: withdrawal_id.into(),
        status: (WithdrawalStatus::Pending as i16).to_string(),
        crypto_network_name: network.name,
        crypto_wallet_address: req.crypto_wallet_address,
        amount: amount.into(),
        fee: fee.into(),
        net_amount: net_amount.into(),
        created_at: core_web::DateTime::now(),
    })
}

/// Get user's withdrawal history with cursor-based pagination
pub async fn get_withdrawal_history(
    state: &AppApiState,
    user_id: i64,
    limit: Option<i64>,
    cursor: Option<&str>,
) -> Result<UserWithdrawalHistoryResponse, AppError> {
    let limit = limit.unwrap_or(10).min(50);
    let fetch_limit = limit + 1;
    let cursor_id: Option<i64> = cursor
        .map(|c| c.parse::<i64>())
        .transpose()
        .map_err(|_| AppError::BadRequest(t("Invalid cursor")))?;

    let mut rows = sqlx::query_as::<_, (i64, i16, String, Option<String>, Decimal, Decimal, Decimal, time::OffsetDateTime)>(
        "SELECT w.id, w.status, COALESCE(cn.name, ''), w.crypto_wallet_address, w.amount, w.fee, w.net_amount, w.created_at
         FROM withdrawals w
         LEFT JOIN crypto_networks cn ON w.crypto_network_id = cn.id
         WHERE w.owner_type = $1 AND w.owner_id = $2
           AND ($3::BIGINT IS NULL OR w.id < $3)
         ORDER BY w.id DESC
         LIMIT $4",
    )
    .bind(OwnerType::User as i16)
    .bind(user_id)
    .bind(cursor_id)
    .bind(fetch_limit)
    .fetch_all(&state.db)
    .await
    .map_err(AppError::from)?;

    let has_more = rows.len() as i64 > limit;
    if has_more {
        rows.truncate(limit as usize);
    }
    let next_cursor = if has_more {
        rows.last().map(|r| r.0.to_string())
    } else {
        None
    };

    let items = rows
        .into_iter()
        .map(|(id, status, cn_name, wallet_addr, amount, fee, net_amount, created_at)| {
            UserWithdrawalOutput {
                id: id.into(),
                status: status.to_string(),
                crypto_network_name: cn_name,
                crypto_wallet_address: wallet_addr.unwrap_or_default(),
                amount: amount.into(),
                fee: fee.into(),
                net_amount: net_amount.into(),
                created_at: created_at.into(),
            }
        })
        .collect();

    Ok(UserWithdrawalHistoryResponse { items, next_cursor })
}
