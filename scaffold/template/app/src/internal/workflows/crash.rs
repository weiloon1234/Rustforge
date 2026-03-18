use core_db::common::sql::{DbConn, Op};
use core_i18n::t;
use core_web::error::AppError;
use generated::models::{
    CrashBetCol, CrashBetModel, CrashBetStatus, CrashPoolCol, CrashPoolModel,
    CrashRoundCol, CrashRoundModel, CrashRoundStatus,
    CreditTransactionType, CreditType, UserCol, UserCreditTransactionCol,
    UserCreditTransactionModel, UserModel,
};
use redis::AsyncCommands;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

use crate::{
    contracts::api::v1::user::crash::{
        CrashCashoutOutput, CrashHistoryEntry, CrashJoinOutput, CrashMyBetEntry,
        CrashMyHistoryResponse, CrashRoomOutput,
    },
    internal::api::state::AppApiState,
};

/// Get all rooms with current state from Redis
pub async fn get_rooms(state: &AppApiState) -> Result<Vec<CrashRoomOutput>, AppError> {
    let mut pools = CrashPoolModel::query(DbConn::pool(&state.db))
        .all()
        .await
        .map_err(AppError::from)?;
    pools.sort_by_key(|p| p.sort_order);

    let mut rooms = Vec::new();
    for pool in pools {
        let redis_state = read_redis_room_state(&state.redis_url, &pool.room_key).await;
        let now_ms = time::OffsetDateTime::now_utc().unix_timestamp_nanos() / 1_000_000;

        // Fetch last settled round's crash point
        let last_crash_point: Option<String> = sqlx::query_scalar(
            "SELECT crash_point::TEXT FROM crash_rounds
             WHERE pool_id = $1 AND status = $2 AND crash_point IS NOT NULL
             ORDER BY created_at DESC LIMIT 1",
        )
        .bind(pool.id)
        .bind(CrashRoundStatus::Settled as i16)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten();

        rooms.push(CrashRoomOutput {
            room_key: pool.room_key,
            slug: pool.slug,
            bet_amount: pool.bet_amount,
            fee_rate: pool.fee_rate,
            sort_order: pool.sort_order,
            phase: redis_state.phase,
            round_id: redis_state.round_id.map(|id| id.into()),
            phase_end_at: redis_state.phase_end_at.map(|v| v.to_string()),
            started_at: redis_state.started_at.map(|v| v.to_string()),
            server_time: Some(now_ms.to_string()),
            last_crash_point,
        });
    }

    Ok(rooms)
}

/// Join the current preparing round
pub async fn join_round(
    state: &AppApiState,
    user_id: i64,
    room_key: &str,
) -> Result<CrashJoinOutput, AppError> {
    // Find pool
    let pool = CrashPoolModel::query(DbConn::pool(&state.db))
        .where_col(CrashPoolCol::ROOM_KEY, Op::Eq, room_key)
        .first()
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(t("Room not found")))?;

    // Check Redis phase
    let redis_state = read_redis_room_state(&state.redis_url, room_key).await;
    if redis_state.phase != "preparing" {
        return Err(AppError::BadRequest(t("Room is not accepting bets")));
    }
    let round_id = redis_state
        .round_id
        .ok_or_else(|| AppError::BadRequest(t("No active round")))?;

    // Check user balance
    let user = UserModel::find(DbConn::pool(&state.db), user_id)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(t("User not found")))?;

    if user.credit_1 < pool.bet_amount {
        return Err(AppError::BadRequest(t("Insufficient balance")));
    }

    let fee_amount = (pool.bet_amount * pool.fee_rate)
        .round_dp(8);
    let effective_bet = pool.bet_amount - fee_amount;

    // Begin transaction
    let scope = DbConn::pool(&state.db)
        .begin_scope()
        .await
        .map_err(AppError::from)?;
    let conn = scope.conn();

    // Debit user (fee is house revenue, does NOT go to pool)
    UserModel::query(conn.clone())
        .where_col(UserCol::ID, Op::Eq, user_id)
        .patch()
        .increment(UserCol::CREDIT_1, -pool.bet_amount)
        .map_err(AppError::from)?
        .save()
        .await
        .map_err(AppError::from)?;

    // Create bet record
    let bet = CrashBetModel::create(conn.clone())
        .set(CrashBetCol::ROUND_ID, round_id)?
        .set(CrashBetCol::USER_ID, user_id)?
        .set(CrashBetCol::BET_AMOUNT, pool.bet_amount)?
        .set(CrashBetCol::FEE_AMOUNT, fee_amount)?
        .set(CrashBetCol::EFFECTIVE_BET, effective_bet)?
        .set(CrashBetCol::STATUS, CrashBetStatus::Active)?
        .save()
        .await
        .map_err(AppError::from)?;

    // Insert credit transaction
    UserCreditTransactionModel::create(conn.clone())
        .set(UserCreditTransactionCol::USER_ID, user_id)?
        .set(UserCreditTransactionCol::CREDIT_TYPE, CreditType::Credit1)?
        .set(UserCreditTransactionCol::AMOUNT, -pool.bet_amount)?
        .set(
            UserCreditTransactionCol::TRANSACTION_TYPE,
            CreditTransactionType::CrashBet,
        )?
        .set(
            UserCreditTransactionCol::RELATED_KEY,
            Some(round_id.to_string()),
        )?
        .save()
        .await
        .map_err(AppError::from)?;

    // Update round counters
    CrashRoundModel::query(conn)
        .where_col(CrashRoundCol::ID, Op::Eq, round_id)
        .patch()
        .increment(CrashRoundCol::TOTAL_BETS, pool.bet_amount)
        .map_err(AppError::from)?
        .increment(CrashRoundCol::TOTAL_EFFECTIVE_BETS, effective_bet)
        .map_err(AppError::from)?
        .increment(CrashRoundCol::TOTAL_FEES, fee_amount)
        .map_err(AppError::from)?
        .increment(CrashRoundCol::PLAYER_COUNT, 1i32)
        .map_err(AppError::from)?
        .save()
        .await
        .map_err(AppError::from)?;

    scope.commit().await.map_err(AppError::from)?;

    // Get updated user balance
    let user = UserModel::find(DbConn::pool(&state.db), user_id)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(t("User not found")))?;

    // Publish player join event (username only, no player count)
    let username = user.username.clone();
    let _ = state
        .realtime
        .publish_raw(
            "user",
            "crash:player_joined",
            Some(&format!("crash:{room_key}")),
            serde_json::json!({
                "room_key": room_key,
                "username": username,
            }),
        )
        .await;

    Ok(CrashJoinOutput {
        bet_id: bet.id.into(),
        round_id: round_id.into(),
        bet_amount: pool.bet_amount,
        fee_amount,
        effective_bet,
        credit_1: user.credit_1,
    })
}

/// Cash out from current running round
pub async fn cashout(
    state: &AppApiState,
    user_id: i64,
    round_id: i64,
) -> Result<CrashCashoutOutput, AppError> {
    // Find active bet
    let bet = CrashBetModel::query(DbConn::pool(&state.db))
        .where_col(CrashBetCol::ROUND_ID, Op::Eq, round_id)
        .where_col(CrashBetCol::USER_ID, Op::Eq, user_id)
        .where_col(CrashBetCol::STATUS, Op::Eq, CrashBetStatus::Active)
        .first()
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(t("No active bet found")))?;

    // Get round and pool
    let round = CrashRoundModel::find(DbConn::pool(&state.db), round_id)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(t("Round not found")))?;

    let pool = CrashPoolModel::find(DbConn::pool(&state.db), round.pool_id)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(t("Pool not found")))?;

    // Check Redis state
    let redis_state = read_redis_room_state(&state.redis_url, &pool.room_key).await;
    if redis_state.phase != "running" {
        return Err(AppError::BadRequest(t("Game is not running")));
    }

    // Compute current multiplier from elapsed time
    let started_at = redis_state
        .started_at
        .ok_or_else(|| AppError::BadRequest(t("Game start time not found")))?;
    let now_ms = time::OffsetDateTime::now_utc().unix_timestamp_nanos() / 1_000_000;
    let elapsed_secs = (now_ms - started_at) as f64 / 1000.0;
    let current_multiplier = state.crash_config.multiplier_at(elapsed_secs);

    // Check if already crashed
    let crash_point = redis_state
        .crash_point
        .ok_or_else(|| AppError::BadRequest(t("Crash point not set")))?;
    let crash_point_f64 = crash_point.to_f64().unwrap_or(1.0);

    if current_multiplier >= crash_point_f64 {
        return Err(AppError::BadRequest(t("Game already crashed")));
    }

    // Round multiplier to 2dp
    let start_floor = Decimal::try_from(state.crash_config.start_multiplier).unwrap_or(Decimal::ONE);
    let multiplier = Decimal::try_from(current_multiplier)
        .unwrap_or(start_floor)
        .round_dp(2);
    let multiplier = multiplier.max(start_floor);

    let payout_amount = (bet.effective_bet * multiplier).round_dp(8);

    // First-cashout-wins: SETNX to claim winner atomically
    let winner_key = format!("crash:room:{}:winner", pool.room_key);
    let winner_claimed = set_winner_if_first(
        &state.redis_url,
        &winner_key,
        user_id,
        &multiplier.to_string(),
        &payout_amount.to_string(),
    )
    .await;
    if !winner_claimed {
        return Err(AppError::BadRequest(t("Someone already won this round")));
    }

    let now = time::OffsetDateTime::now_utc();

    // Begin transaction
    let scope = DbConn::pool(&state.db)
        .begin_scope()
        .await
        .map_err(AppError::from)?;
    let conn = scope.conn();

    // Update bet
    CrashBetModel::query(conn.clone())
        .where_col(CrashBetCol::ID, Op::Eq, bet.id)
        .patch()
        .assign(CrashBetCol::STATUS, CrashBetStatus::CashedOut)?
        .assign(CrashBetCol::CASHOUT_MULTIPLIER, Some(multiplier))?
        .assign(CrashBetCol::PAYOUT_AMOUNT, Some(payout_amount))?
        .assign(CrashBetCol::CASHED_OUT_AT, Some(now))?
        .save()
        .await
        .map_err(AppError::from)?;

    // Credit user
    UserModel::query(conn.clone())
        .where_col(UserCol::ID, Op::Eq, user_id)
        .patch()
        .increment(UserCol::CREDIT_1, payout_amount)
        .map_err(AppError::from)?
        .save()
        .await
        .map_err(AppError::from)?;

    // Insert credit transaction
    UserCreditTransactionModel::create(conn.clone())
        .set(UserCreditTransactionCol::USER_ID, user_id)?
        .set(UserCreditTransactionCol::CREDIT_TYPE, CreditType::Credit1)?
        .set(UserCreditTransactionCol::AMOUNT, payout_amount)?
        .set(
            UserCreditTransactionCol::TRANSACTION_TYPE,
            CreditTransactionType::CrashWin,
        )?
        .set(
            UserCreditTransactionCol::RELATED_KEY,
            Some(round_id.to_string()),
        )?
        .save()
        .await
        .map_err(AppError::from)?;

    // Update round total payouts
    CrashRoundModel::query(conn)
        .where_col(CrashRoundCol::ID, Op::Eq, round_id)
        .patch()
        .increment(CrashRoundCol::TOTAL_PAYOUTS, payout_amount)
        .map_err(AppError::from)?
        .save()
        .await
        .map_err(AppError::from)?;

    match scope.commit().await {
        Ok(_) => {}
        Err(e) => {
            // Rollback Redis winner claim on DB failure
            delete_winner_key(&state.redis_url, &winner_key).await;
            return Err(AppError::from(e));
        }
    }

    // Bust all other active bets (first-cashout-wins, safe after SETNX + commit)
    let _ = sqlx::query(
        "UPDATE crash_bets SET status = $1, updated_at = NOW()
         WHERE round_id = $2 AND user_id != $3 AND status = $4",
    )
    .bind(CrashBetStatus::Busted as i16)
    .bind(round_id)
    .bind(user_id)
    .bind(CrashBetStatus::Active as i16)
    .execute(&state.db)
    .await;

    // Get updated balance
    let user = UserModel::find(DbConn::pool(&state.db), user_id)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(t("User not found")))?;

    // Publish cashout/winner event
    let _ = state
        .realtime
        .publish_raw(
            "user",
            "crash:player_cashout",
            Some(&format!("crash:{}", pool.room_key)),
            serde_json::json!({
                "room_key": pool.room_key,
                "username": user.username,
                "multiplier": multiplier.to_string(),
                "payout": payout_amount.to_string(),
                "is_winner": true,
            }),
        )
        .await;

    Ok(CrashCashoutOutput {
        multiplier,
        payout: payout_amount,
        credit_1: user.credit_1,
    })
}

/// Get recent crash history for a room
pub async fn get_history(
    state: &AppApiState,
    room_key: &str,
    limit: Option<i64>,
) -> Result<Vec<CrashHistoryEntry>, AppError> {
    let pool = CrashPoolModel::query(DbConn::pool(&state.db))
        .where_col(CrashPoolCol::ROOM_KEY, Op::Eq, room_key)
        .first()
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(t("Room not found")))?;

    let limit = limit.unwrap_or(20).min(50);

    let rows = sqlx::query_as::<_, (i64, rust_decimal::Decimal, i32, time::OffsetDateTime)>(
        "SELECT id, crash_point, player_count, created_at
         FROM crash_rounds
         WHERE pool_id = $1 AND status = $2 AND crash_point IS NOT NULL
         ORDER BY created_at DESC
         LIMIT $3",
    )
    .bind(pool.id)
    .bind(CrashRoundStatus::Settled as i16)
    .bind(limit)
    .fetch_all(&state.db)
    .await
    .map_err(AppError::from)?;

    let entries = rows
        .into_iter()
        .map(|(id, crash_point, player_count, created_at)| CrashHistoryEntry {
            round_id: id.into(),
            crash_point,
            player_count,
            created_at,
        })
        .collect();

    Ok(entries)
}

/// Get authenticated user's bet history with cursor-based pagination
pub async fn get_my_history(
    state: &AppApiState,
    user_id: i64,
    limit: Option<i64>,
    cursor: Option<&str>,
) -> Result<CrashMyHistoryResponse, AppError> {
    let limit = limit.unwrap_or(20).min(50);
    let fetch_limit = limit + 1; // fetch one extra to detect next page
    let cursor_id: Option<i64> = cursor
        .map(|c| c.parse::<i64>())
        .transpose()
        .map_err(|_| AppError::BadRequest(t("Invalid cursor")))?;

    let mut rows = sqlx::query_as::<_, (i64, Decimal, i16, Option<Decimal>, Option<Decimal>, time::OffsetDateTime, Decimal, String)>(
        "SELECT b.id, b.bet_amount, b.status, b.cashout_multiplier, b.payout_amount, b.created_at,
                r.crash_point, p.room_key
         FROM crash_bets b
         JOIN crash_rounds r ON r.id = b.round_id
         JOIN crash_pools p ON p.id = r.pool_id
         WHERE b.user_id = $1
           AND b.status != $2
           AND ($3::BIGINT IS NULL OR b.id < $3)
         ORDER BY b.id DESC
         LIMIT $4",
    )
    .bind(user_id)
    .bind(CrashBetStatus::Active as i16)
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
        .map(|(id, bet_amount, status, cashout_multiplier, payout_amount, created_at, crash_point, room_key)| {
            let status_str = if status == CrashBetStatus::CashedOut as i16 {
                "cashed_out".to_string()
            } else if status == CrashBetStatus::Refunded as i16 {
                "refunded".to_string()
            } else {
                "busted".to_string()
            };
            CrashMyBetEntry {
                id: id.into(),
                room_key,
                bet_amount,
                status: status_str,
                cashout_multiplier,
                payout_amount,
                crash_point,
                created_at,
            }
        })
        .collect();

    Ok(CrashMyHistoryResponse { items, next_cursor })
}

// ── Redis helpers ──────────────────────────────────────────

struct RedisRoomState {
    phase: String,
    round_id: Option<i64>,
    crash_point: Option<Decimal>,
    started_at: Option<i128>,
    phase_end_at: Option<i128>,
    player_count: i32,
}

async fn read_redis_room_state(redis_url: &str, room_key: &str) -> RedisRoomState {
    let default = RedisRoomState {
        phase: "preparing".to_string(),
        round_id: None,
        crash_point: None,
        started_at: None,
        phase_end_at: None,
        player_count: 0,
    };

    let Ok(client) = redis::Client::open(redis_url) else {
        return default;
    };
    let Ok(mut conn) = client.get_multiplexed_async_connection().await else {
        return default;
    };

    let key = format!("crash:room:{room_key}");
    let result: Result<std::collections::HashMap<String, String>, _> =
        conn.hgetall(&key).await;

    match result {
        Ok(map) if !map.is_empty() => RedisRoomState {
            phase: map
                .get("phase")
                .cloned()
                .unwrap_or_else(|| "preparing".to_string()),
            round_id: map.get("round_id").and_then(|v| v.parse::<i64>().ok()),
            crash_point: map
                .get("crash_point")
                .and_then(|v| v.parse::<Decimal>().ok()),
            started_at: map
                .get("started_at")
                .and_then(|v| v.parse::<i128>().ok()),
            phase_end_at: map
                .get("phase_end_at")
                .and_then(|v| v.parse::<i128>().ok()),
            player_count: map
                .get("player_count")
                .and_then(|v| v.parse::<i32>().ok())
                .unwrap_or(0),
        },
        _ => default,
    }
}

/// Try to claim winner via Redis SETNX. Returns true if this caller is the first (winner).
async fn set_winner_if_first(
    redis_url: &str,
    key: &str,
    user_id: i64,
    multiplier: &str,
    payout: &str,
) -> bool {
    let Ok(client) = redis::Client::open(redis_url) else {
        return false;
    };
    let Ok(mut conn) = client.get_multiplexed_async_connection().await else {
        return false;
    };

    let value = format!("{user_id}:{multiplier}:{payout}");
    let result: Result<bool, _> = redis::cmd("SET")
        .arg(key)
        .arg(&value)
        .arg("NX")
        .arg("EX")
        .arg(120)
        .query_async(&mut conn)
        .await;

    result.unwrap_or(false)
}

/// Delete a Redis key (used to rollback winner claim on DB failure).
async fn delete_winner_key(redis_url: &str, key: &str) {
    let Ok(client) = redis::Client::open(redis_url) else {
        return;
    };
    let Ok(mut conn) = client.get_multiplexed_async_connection().await else {
        return;
    };
    let _: Result<(), _> = conn.del(key).await;
}
