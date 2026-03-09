use core_db::common::sql::{generate_snowflake_i64, DbConn, Op};
use core_i18n::t;
use core_web::{error::AppError, Patch};
use generated::{
    guards::user_guard,
    models::{User, UserBanStatus, UserQuery, UserView},
};

use crate::{
    contracts::api::v1::admin::user::{CreateUserInput, UpdateUserInput},
    internal::api::state::AppApiState,
};

pub async fn detail(state: &AppApiState, id: i64) -> Result<UserView, AppError> {
    User::new(DbConn::pool(&state.db), None)
        .find(id)
        .await
        .map_err(AppError::from)?
        .map(|r| r.into_row())
        .ok_or_else(|| AppError::NotFound(t("User not found")))
}

pub async fn create(state: &AppApiState, req: CreateUserInput) -> Result<UserView, AppError> {
    let username = req.username.trim().to_ascii_lowercase();
    let uuid = generate_unique_uuid(state).await?;

    let mut insert = User::new(DbConn::pool(&state.db), None)
        .insert()
        .set_id(generate_snowflake_i64())
        .set_uuid(uuid)
        .set_username(username)
        .set_ban(UserBanStatus::No);

    if let Some(ref introducer_username) = req.introducer_username {
        let introducer = UserQuery::new(DbConn::pool(&state.db), None)
            .where_username(Op::Eq, introducer_username.clone())
            .first()
            .await
            .map_err(AppError::from)?
            .map(|r| r.into_row())
            .ok_or_else(|| AppError::NotFound(t("Introducer not found")))?;
        insert = insert.set_introducer_user_id(Some(introducer.id));
    }

    if let Some(name) = &req.name {
        insert = insert.set_name(Some(name.clone()));
    }
    if let Some(email) = &req.email {
        insert = insert.set_email(Some(email.clone()));
    }
    if let Some(country_iso2) = &req.country_iso2 {
        insert = insert.set_country_iso2(Some(country_iso2.clone()));
    }
    if let Some(contact_number) = &req.contact_number {
        insert = insert.set_contact_number(Some(contact_number.clone()));
    }

    let insert = insert.set_password(&req.password).map_err(AppError::from)?;
    insert.save().await.map_err(AppError::from)
}

pub async fn update(
    state: &AppApiState,
    id: i64,
    req: UpdateUserInput,
) -> Result<UserView, AppError> {
    let existing = detail(state, id).await?;
    let mut update = User::new(DbConn::pool(&state.db), None)
        .update()
        .where_id(Op::Eq, id);
    let mut touched = false;

    if let Some(username) = req.username {
        let username = username.trim().to_ascii_lowercase();
        if username != existing.username {
            update = update.set_username(username);
            touched = true;
        }
    }

    match req.name {
        Patch::Missing => {}
        Patch::Null => {
            if existing.name.is_some() {
                update = update.set_name(None);
                touched = true;
            }
        }
        Patch::Value(name) => {
            if existing.name.as_deref() != Some(&name) {
                update = update.set_name(Some(name));
                touched = true;
            }
        }
    }

    match req.email {
        Patch::Missing => {}
        Patch::Null => {
            if existing.email.is_some() {
                update = update.set_email(None);
                touched = true;
            }
        }
        Patch::Value(email) => {
            if existing.email.as_deref() != Some(&email) {
                update = update.set_email(Some(email));
                touched = true;
            }
        }
    }

    match req.country_iso2 {
        Patch::Missing => {}
        Patch::Null => {
            if existing.country_iso2.is_some() {
                update = update.set_country_iso2(None);
                touched = true;
            }
        }
        Patch::Value(value) => {
            if existing.country_iso2.as_deref() != Some(&value) {
                update = update.set_country_iso2(Some(value));
                touched = true;
            }
        }
    }

    match req.contact_number {
        Patch::Missing => {}
        Patch::Null => {
            if existing.contact_number.is_some() {
                update = update.set_contact_number(None);
                touched = true;
            }
        }
        Patch::Value(value) => {
            if existing.contact_number.as_deref() != Some(&value) {
                update = update.set_contact_number(Some(value));
                touched = true;
            }
        }
    }

    if let Some(password) = req.password {
        update = update.set_password(&password).map_err(AppError::from)?;
        touched = true;
    }

    if !touched {
        return Ok(existing);
    }

    let affected = update.save().await.map_err(AppError::from)?;
    if affected == 0 {
        return Err(AppError::NotFound(t("User not found")));
    }

    // Revoke all tokens so the user must re-login with updated credentials.
    let _ = user_guard::revoke_tokens(DbConn::pool(&state.db), &id.to_string()).await;

    detail(state, id).await
}

pub async fn set_ban(
    state: &AppApiState,
    id: i64,
    ban: UserBanStatus,
) -> Result<UserView, AppError> {
    let _existing = detail(state, id).await?;

    let affected = User::new(DbConn::pool(&state.db), None)
        .update()
        .where_id(Op::Eq, id)
        .set_ban(ban)
        .save()
        .await
        .map_err(AppError::from)?;

    if affected == 0 {
        return Err(AppError::NotFound(t("User not found")));
    }

    // Revoke all tokens on ban/unban so the user must re-login.
    let _ = user_guard::revoke_tokens(DbConn::pool(&state.db), &id.to_string()).await;

    detail(state, id).await
}

pub async fn batch_resolve_usernames(
    state: &AppApiState,
    ids: &[i64],
) -> Result<Vec<(i64, String, Option<String>)>, AppError> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }

    let mut results = Vec::new();
    for &id in ids {
        if let Ok(Some(user)) = User::new(DbConn::pool(&state.db), None).find(id).await {
            let user = user.into_row();
            results.push((user.id, user.username, user.name));
        }
    }
    Ok(results)
}

async fn generate_unique_uuid(state: &AppApiState) -> Result<String, AppError> {
    for _ in 0..10 {
        let uuid = nanoid::nanoid!(8);
        let existing = UserQuery::new(DbConn::pool(&state.db), None)
            .where_uuid(Op::Eq, uuid.clone())
            .first()
            .await
            .map_err(AppError::from)?
            .map(|r| r.into_row());
        if existing.is_none() {
            return Ok(uuid);
        }
    }
    Err(AppError::BadRequest(t("Failed to generate unique ID")))
}
