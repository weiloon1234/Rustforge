use core_realtime::{RealtimeEvent, RealtimeTarget};

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
    let deposit: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM deposits WHERE status = 1")
            .fetch_one(db)
            .await?;

    let withdrawal: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM withdrawals WHERE status IN (1, 2)")
            .fetch_one(db)
            .await?;

    Ok(NotificationCounts {
        deposit: deposit.0,
        withdrawal: withdrawal.0,
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
