use core_db::common::sql::{DbConn, Op};
use core_realtime::{RealtimeEvent, RealtimeTarget};
use generated::models::{
    DepositCol, DepositModel, DepositStatus, WithdrawalCol, WithdrawalModel, WithdrawalStatus,
};

use crate::internal::api::state::AppApiState;

/// Notification counts payload broadcast to the admin channel.
#[derive(Debug, Clone, serde::Serialize)]
pub struct NotificationCounts {
    pub deposit: i64,
    pub withdrawal: i64,
}

impl RealtimeEvent for NotificationCounts {
    const CHANNEL: &'static str = "admin";
    const EVENT: &'static str = "notification_counts";
}

/// Query current pending counts from the database.
pub async fn get_pending_counts(db: &sqlx::PgPool) -> Result<NotificationCounts, sqlx::Error> {
    let (deposit, withdrawal) = tokio::try_join!(
        DepositModel::query(DbConn::pool(db))
            .where_col(DepositCol::STATUS, Op::Eq, DepositStatus::Pending)
            .count(),
        WithdrawalModel::query(DbConn::pool(db))
            .where_in(
                WithdrawalCol::STATUS,
                [WithdrawalStatus::Pending, WithdrawalStatus::Processing],
            )
            .count(),
    )?;

    Ok(NotificationCounts {
        deposit,
        withdrawal,
    })
}

/// Query counts and broadcast to all admin channel subscribers.
/// Errors are logged but not propagated — notification dispatch must not fail the request.
pub async fn dispatch_admin_notification_counts(state: &AppApiState) {
    match get_pending_counts(&state.db).await {
        Ok(counts) => {
            let _ = state
                .realtime
                .publish(RealtimeTarget { room: None }, &counts)
                .await;
        }
        Err(_) => {
            // Silent fail — notification dispatch must not break the request
        }
    }
}
