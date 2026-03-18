use core_web::contracts::rustforge_contract;
use core_web::ids::SnowflakeId;
use rust_decimal::Decimal;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use ts_rs::TS;

#[rustforge_contract]
#[derive(TS)]
#[ts(export, export_to = "user/types/")]
pub struct CrashJoinInput {
    #[rf(length(min = 1, max = 16))]
    pub room_key: String,
}

#[rustforge_contract]
#[derive(TS)]
#[ts(export, export_to = "user/types/")]
pub struct CrashCashoutInput {
    pub round_id: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema, TS)]
#[ts(export, export_to = "user/types/")]
pub struct CrashHistoryQuery {
    pub room_key: String,
    #[serde(default)]
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[ts(export, export_to = "user/types/")]
pub struct CrashGameConfig {
    pub preparing_duration_secs: u32,
    pub countdown_duration_secs: u32,
    pub post_crash_display_secs: u32,
    pub growth_rate: f64,
    pub start_multiplier: f64,
}

#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[ts(export, export_to = "user/types/")]
pub struct CrashRoomsResponse {
    pub config: CrashGameConfig,
    pub rooms: Vec<CrashRoomOutput>,
}

#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[ts(export, export_to = "user/types/")]
pub struct CrashRoomOutput {
    pub room_key: String,
    pub slug: String,
    #[schemars(with = "String")]
    #[ts(type = "string")]
    pub bet_amount: rust_decimal::Decimal,
    #[schemars(with = "String")]
    #[ts(type = "string")]
    pub fee_rate: rust_decimal::Decimal,
    pub sort_order: i32,
    pub phase: String,
    pub round_id: Option<SnowflakeId>,
    pub phase_end_at: Option<String>,
    pub started_at: Option<String>,
    pub server_time: Option<String>,
    pub last_crash_point: Option<String>,
}

#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[ts(export, export_to = "user/types/")]
pub struct CrashJoinOutput {
    pub bet_id: SnowflakeId,
    pub round_id: SnowflakeId,
    #[schemars(with = "String")]
    #[ts(type = "string")]
    pub bet_amount: rust_decimal::Decimal,
    #[schemars(with = "String")]
    #[ts(type = "string")]
    pub fee_amount: rust_decimal::Decimal,
    #[schemars(with = "String")]
    #[ts(type = "string")]
    pub effective_bet: rust_decimal::Decimal,
    #[schemars(with = "String")]
    #[ts(type = "string")]
    pub credit_1: rust_decimal::Decimal,
}

#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[ts(export, export_to = "user/types/")]
pub struct CrashCashoutOutput {
    #[schemars(with = "String")]
    #[ts(type = "string")]
    pub multiplier: rust_decimal::Decimal,
    #[schemars(with = "String")]
    #[ts(type = "string")]
    pub payout: rust_decimal::Decimal,
    #[schemars(with = "String")]
    #[ts(type = "string")]
    pub credit_1: rust_decimal::Decimal,
}

#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[ts(export, export_to = "user/types/")]
pub struct CrashHistoryEntry {
    pub round_id: SnowflakeId,
    #[schemars(with = "String")]
    #[ts(type = "string")]
    pub crash_point: rust_decimal::Decimal,
    pub player_count: i32,
    #[schemars(with = "String")]
    #[ts(type = "string")]
    pub created_at: time::OffsetDateTime,
}

#[derive(Debug, Clone, Deserialize, JsonSchema, TS)]
#[ts(export, export_to = "user/types/")]
pub struct CrashMyHistoryQuery {
    #[serde(default)]
    pub limit: Option<i64>,
    #[serde(default)]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[ts(export, export_to = "user/types/")]
pub struct CrashMyHistoryResponse {
    pub items: Vec<CrashMyBetEntry>,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[ts(export, export_to = "user/types/")]
pub struct CrashMyBetEntry {
    pub id: SnowflakeId,
    pub room_key: String,
    #[schemars(with = "String")]
    #[ts(type = "string")]
    pub bet_amount: Decimal,
    pub status: String,
    #[schemars(with = "Option<String>")]
    #[ts(type = "string | null")]
    pub cashout_multiplier: Option<Decimal>,
    #[schemars(with = "Option<String>")]
    #[ts(type = "string | null")]
    pub payout_amount: Option<Decimal>,
    #[schemars(with = "String")]
    #[ts(type = "string")]
    pub crash_point: Decimal,
    #[schemars(with = "String")]
    #[ts(type = "string")]
    pub created_at: OffsetDateTime,
}
