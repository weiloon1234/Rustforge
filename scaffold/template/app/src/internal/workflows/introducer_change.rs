use core_db::common::sql::{generate_snowflake_i64, DbConn, Op};
use core_i18n::t;
use core_web::error::AppError;
use generated::{
    guards::user_guard,
    models::{
        IntroducerChangeCol, IntroducerChangeModel, IntroducerChangeRecord, UserCol, UserModel,
        UserRecord,
    },
};

use crate::internal::api::state::AppApiState;

async fn would_create_cycle(
    state: &AppApiState,
    start_id: i64,
    target_id: i64,
) -> Result<bool, AppError> {
    let mut current_id = start_id;
    for _ in 0..10000 {
        let user = UserModel::find(DbConn::pool(&state.db), current_id)
            .await
            .map_err(AppError::from)?;

        let Some(user) = user else {
            return Ok(false);
        };

        match user.introducer_user_id {
            None => return Ok(false),
            Some(parent_id) => {
                if parent_id == target_id {
                    return Ok(true);
                }
                current_id = parent_id;
            }
        }
    }
    Ok(true)
}

pub async fn resolve_user_by_username(
    state: &AppApiState,
    username: &str,
) -> Result<UserRecord, AppError> {
    let username = username.trim().to_ascii_lowercase();
    UserModel::query(DbConn::pool(&state.db))
        .where_col(UserCol::USERNAME, Op::Eq, username)
        .first()
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(t("User not found")))
}

pub async fn change_introducer(
    state: &AppApiState,
    admin_id: i64,
    user_username: &str,
    new_introducer_username: &str,
    remark: Option<String>,
) -> Result<IntroducerChangeRecord, AppError> {
    let target_user = resolve_user_by_username(state, user_username).await?;
    let new_introducer = resolve_user_by_username(state, new_introducer_username).await?;

    if target_user.id == new_introducer.id {
        return Err(AppError::BadRequest(t("Cannot set user as their own introducer")));
    }
    if target_user.introducer_user_id == Some(new_introducer.id) {
        return Err(AppError::BadRequest(t("User already has this introducer")));
    }
    if would_create_cycle(state, new_introducer.id, target_user.id).await? {
        return Err(AppError::BadRequest(t("Cannot change introducer: would create a circular hierarchy")));
    }

    let from_user_id = target_user.introducer_user_id;

    let scope = DbConn::pool(&state.db)
        .begin_scope()
        .await
        .map_err(AppError::from)?;
    let conn = scope.conn();

    UserModel::query(conn.clone())
        .where_col(UserCol::ID, Op::Eq, target_user.id)
        .patch()
        .assign(UserCol::INTRODUCER_USER_ID, Some(new_introducer.id))
        .map_err(AppError::from)?
        .save()
        .await
        .map_err(AppError::from)?;

    let _ = user_guard::revoke_tokens(conn.clone(), &target_user.id.to_string()).await;

    let log = IntroducerChangeModel::create(conn)
        .set(IntroducerChangeCol::ID, generate_snowflake_i64())
        .map_err(AppError::from)?
        .set(IntroducerChangeCol::USER_ID, target_user.id)
        .map_err(AppError::from)?
        .set(IntroducerChangeCol::FROM_USER_ID, from_user_id)
        .map_err(AppError::from)?
        .set(IntroducerChangeCol::TO_USER_ID, new_introducer.id)
        .map_err(AppError::from)?
        .set(IntroducerChangeCol::ADMIN_ID, admin_id)
        .map_err(AppError::from)?
        .set(IntroducerChangeCol::REMARK, remark)
        .map_err(AppError::from)?
        .save()
        .await
        .map_err(AppError::from)?;

    scope.commit().await.map_err(AppError::from)?;

    Ok(log)
}
