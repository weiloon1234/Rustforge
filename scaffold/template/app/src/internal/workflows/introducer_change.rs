use core_db::common::sql::{generate_snowflake_i64, DbConn, Op};
use core_i18n::t;
use core_web::error::AppError;
use generated::{
    guards::user_guard,
    models::{IntroducerChange, IntroducerChangeView, User, UserQuery, UserView},
};

use crate::internal::api::state::AppApiState;

async fn would_create_cycle(
    state: &AppApiState,
    start_id: i64,
    target_id: i64,
) -> Result<bool, AppError> {
    let mut current_id = start_id;
    for _ in 0..10000 {
        let user = User::new(DbConn::pool(&state.db), None)
            .find(current_id)
            .await
            .map_err(AppError::from)?
            .map(|r| r.into_row());

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
) -> Result<UserView, AppError> {
    let username = username.trim().to_ascii_lowercase();
    UserQuery::new(DbConn::pool(&state.db), None)
        .where_username(Op::Eq, username)
        .first()
        .await
        .map_err(AppError::from)?
        .map(|r| r.into_row())
        .ok_or_else(|| AppError::NotFound(t("User not found")))
}

pub async fn change_introducer(
    state: &AppApiState,
    admin_id: i64,
    user_username: &str,
    new_introducer_username: &str,
    remark: Option<String>,
) -> Result<IntroducerChangeView, AppError> {
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

    User::new(DbConn::pool(&state.db), None)
        .update()
        .where_id(Op::Eq, target_user.id)
        .set_introducer_user_id(Some(new_introducer.id))
        .save()
        .await
        .map_err(AppError::from)?;

    let _ = user_guard::revoke_tokens(DbConn::pool(&state.db), &target_user.id.to_string()).await;

    let log = IntroducerChange::new(DbConn::pool(&state.db), None)
        .insert()
        .set_id(generate_snowflake_i64())
        .set_user_id(target_user.id)
        .set_from_user_id(from_user_id)
        .set_to_user_id(new_introducer.id)
        .set_admin_id(admin_id)
        .set_remark(remark)
        .save()
        .await
        .map_err(AppError::from)?;

    Ok(log)
}
