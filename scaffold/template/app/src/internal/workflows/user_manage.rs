use core_db::common::{
    auth::hash::hash_password,
    sql::{generate_snowflake_i64, DbConn, Op},
};
use core_i18n::t;
use core_web::{error::AppError, Patch};
use generated::{
    guards::user_guard,
    models::{UserBanStatus, UserCol, UserModel, UserRecord},
};

use crate::{
    contracts::api::v1::admin::user::{CreateUserInput, UpdateUserInput},
    internal::api::state::AppApiState,
};

pub async fn detail(state: &AppApiState, id: i64) -> Result<UserRecord, AppError> {
    UserModel::find(DbConn::pool(&state.db), id)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(t("User not found")))
}

pub async fn create(state: &AppApiState, req: CreateUserInput) -> Result<UserRecord, AppError> {
    let username = req.username.trim().to_ascii_lowercase();
    let uuid = generate_unique_uuid(state).await?;
    let password_hash = hash_password(&req.password).map_err(AppError::from)?;

    let mut insert = UserModel::create(DbConn::pool(&state.db))
        .set(UserCol::ID, generate_snowflake_i64())
        .map_err(AppError::from)?
        .set(UserCol::UUID, uuid)
        .map_err(AppError::from)?
        .set(UserCol::USERNAME, username)
        .map_err(AppError::from)?
        .set(UserCol::BAN, UserBanStatus::No)
        .map_err(AppError::from)?
        .set(UserCol::PASSWORD, password_hash)
        .map_err(AppError::from)?;

    if let Some(ref introducer_username) = req.introducer_username {
        let introducer = UserModel::query(DbConn::pool(&state.db))
            .where_col(UserCol::USERNAME, Op::Eq, introducer_username.clone())
            .first()
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::NotFound(t("Introducer not found")))?;
        insert = insert
            .set(UserCol::INTRODUCER_USER_ID, Some(introducer.id))
            .map_err(AppError::from)?;
    }

    if let Some(name) = &req.name {
        insert = insert
            .set(UserCol::NAME, Some(name.clone()))
            .map_err(AppError::from)?;
    }
    if let Some(email) = &req.email {
        insert = insert
            .set(UserCol::EMAIL, Some(email.clone()))
            .map_err(AppError::from)?;
    }
    if let Some(country_iso2) = &req.country_iso2 {
        insert = insert
            .set(UserCol::COUNTRY_ISO2, Some(country_iso2.clone()))
            .map_err(AppError::from)?;
    }
    if let Some(contact_number) = &req.contact_number {
        insert = insert
            .set(UserCol::CONTACT_NUMBER, Some(contact_number.clone()))
            .map_err(AppError::from)?;
    }

    let created = insert.save().await.map_err(AppError::from)?;
    Ok(created)
}

pub async fn update(
    state: &AppApiState,
    id: i64,
    req: UpdateUserInput,
) -> Result<UserRecord, AppError> {
    let existing = detail(state, id).await?;
    let scope = DbConn::pool(&state.db)
        .begin_scope()
        .await
        .map_err(AppError::from)?;
    let conn = scope.conn();
    let mut update = Ok(
        UserModel::query(conn.clone())
            .where_col(UserCol::ID, Op::Eq, id)
            .patch(),
    );
    let mut touched = false;

    if let Some(username) = req.username {
        let username = username.trim().to_ascii_lowercase();
        if username != existing.username {
            update = update.and_then(|patch| patch.assign(UserCol::USERNAME, username));
            touched = true;
        }
    }

    match req.name {
        Patch::Missing => {}
        Patch::Null => {
            if existing.name.is_some() {
                update = update.and_then(|patch| patch.assign(UserCol::NAME, None::<String>));
                touched = true;
            }
        }
        Patch::Value(name) => {
            if existing.name.as_deref() != Some(&name) {
                update = update.and_then(|patch| patch.assign(UserCol::NAME, Some(name)));
                touched = true;
            }
        }
    }

    match req.email {
        Patch::Missing => {}
        Patch::Null => {
            if existing.email.is_some() {
                update = update.and_then(|patch| patch.assign(UserCol::EMAIL, None::<String>));
                touched = true;
            }
        }
        Patch::Value(email) => {
            if existing.email.as_deref() != Some(&email) {
                update = update.and_then(|patch| patch.assign(UserCol::EMAIL, Some(email)));
                touched = true;
            }
        }
    }

    match req.country_iso2 {
        Patch::Missing => {}
        Patch::Null => {
            if existing.country_iso2.is_some() {
                update = update.and_then(|patch| patch.assign(UserCol::COUNTRY_ISO2, None::<String>));
                touched = true;
            }
        }
        Patch::Value(value) => {
            if existing.country_iso2.as_deref() != Some(&value) {
                update = update.and_then(|patch| patch.assign(UserCol::COUNTRY_ISO2, Some(value)));
                touched = true;
            }
        }
    }

    match req.contact_number {
        Patch::Missing => {}
        Patch::Null => {
            if existing.contact_number.is_some() {
                update = update.and_then(|patch| patch.assign(UserCol::CONTACT_NUMBER, None::<String>));
                touched = true;
            }
        }
        Patch::Value(value) => {
            if existing.contact_number.as_deref() != Some(&value) {
                update = update.and_then(|patch| patch.assign(UserCol::CONTACT_NUMBER, Some(value)));
                touched = true;
            }
        }
    }

    if let Some(password) = req.password {
        let password_hash = hash_password(&password).map_err(AppError::from)?;
        update = update.and_then(|patch| patch.assign(UserCol::PASSWORD, password_hash));
        touched = true;
    }

    if !touched {
        return Ok(existing);
    }

    let affected = update
        .map_err(AppError::from)?
        .save()
        .await
        .map_err(AppError::from)?;
    if affected == 0 {
        return Err(AppError::NotFound(t("User not found")));
    }

    // Revoke all tokens so the user must re-login with updated credentials.
    let _ = user_guard::revoke_tokens(conn, &id.to_string()).await;

    scope.commit().await.map_err(AppError::from)?;

    detail(state, id).await
}

pub async fn set_ban(
    state: &AppApiState,
    id: i64,
    ban: UserBanStatus,
) -> Result<UserRecord, AppError> {
    let _existing = detail(state, id).await?;

    let scope = DbConn::pool(&state.db)
        .begin_scope()
        .await
        .map_err(AppError::from)?;
    let conn = scope.conn();

    let affected = UserModel::query(conn.clone())
        .where_col(UserCol::ID, Op::Eq, id)
        .patch()
        .assign(UserCol::BAN, ban)
        .map_err(AppError::from)?
        .save()
        .await
        .map_err(AppError::from)?;

    if affected == 0 {
        return Err(AppError::NotFound(t("User not found")));
    }

    // Revoke all tokens on ban/unban so the user must re-login.
    let _ = user_guard::revoke_tokens(conn, &id.to_string()).await;

    scope.commit().await.map_err(AppError::from)?;

    detail(state, id).await
}

pub async fn batch_resolve_usernames(
    state: &AppApiState,
    ids: &[i64],
) -> Result<Vec<(i64, String, Option<String>)>, AppError> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }

    let users = UserModel::query(DbConn::pool(&state.db))
        .where_in(UserCol::ID, ids.iter().copied())
        .all()
        .await
        .map_err(AppError::from)?;

    Ok(users
        .into_iter()
        .map(|u| (u.id, u.username, u.name))
        .collect())
}

async fn generate_unique_uuid(state: &AppApiState) -> Result<String, AppError> {
    for _ in 0..10 {
        let uuid = nanoid::nanoid!(8);
        let existing = UserModel::query(DbConn::pool(&state.db))
            .where_col(UserCol::UUID, Op::Eq, uuid.clone())
            .first()
            .await
            .map_err(AppError::from)?;
        if existing.is_none() {
            return Ok(uuid);
        }
    }
    Err(AppError::BadRequest(t("Failed to generate unique ID")))
}
