use core_web::contracts::rustforge_contract;
use core_web::ids::SnowflakeId;
use core_web::{DateTime, Decimal};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Deserialize, JsonSchema, TS)]
#[ts(export, export_to = "user/types/")]
pub struct WalletLedgerQuery {
    #[serde(default)]
    pub limit: Option<i64>,
    #[serde(default)]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[ts(export, export_to = "user/types/")]
pub struct WalletLedgerResponse {
    pub items: Vec<WalletLedgerEntry>,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[ts(export, export_to = "user/types/")]
pub struct WalletLedgerEntry {
    pub id: SnowflakeId,
    pub transaction_type: String,
    pub amount: Decimal,
    pub related_key: Option<String>,
    pub created_at: DateTime,
}

// --- Crypto network dropdown option ---

#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[ts(export, export_to = "user/types/")]
pub struct CryptoNetworkOption {
    pub id: SnowflakeId,
    pub name: String,
    pub symbol: String,
}

// --- Withdrawal config response (wraps crypto-networks + fee config) ---

#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[ts(export, export_to = "user/types/")]
pub struct WithdrawalConfigResponse {
    pub networks: Vec<CryptoNetworkOption>,
    pub fee_percentage: Decimal,
    pub min_amount: Decimal,
}

// --- Withdrawal input ---

#[rustforge_contract]
#[derive(TS)]
#[ts(export, export_to = "user/types/")]
pub struct UserWithdrawalInput {
    pub crypto_network_id: i64,

    #[rf(length(min = 1, max = 256))]
    pub crypto_wallet_address: String,

    pub amount: Decimal,

    #[rf(length(min = 1, max = 128))]
    pub password: String,
}

// --- Withdrawal output (user-facing, no admin_remark) ---

#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[ts(export, export_to = "user/types/")]
pub struct UserWithdrawalOutput {
    pub id: SnowflakeId,
    pub status: String,
    pub crypto_network_name: String,
    pub crypto_wallet_address: String,
    pub amount: Decimal,
    pub fee: Decimal,
    pub net_amount: Decimal,
    pub created_at: DateTime,
}

// --- Withdrawal history pagination ---

#[derive(Debug, Clone, Deserialize, JsonSchema, TS)]
#[ts(export, export_to = "user/types/")]
pub struct UserWithdrawalHistoryQuery {
    #[serde(default)]
    pub limit: Option<i64>,
    #[serde(default)]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[ts(export, export_to = "user/types/")]
pub struct UserWithdrawalHistoryResponse {
    pub items: Vec<UserWithdrawalOutput>,
    pub next_cursor: Option<String>,
}
