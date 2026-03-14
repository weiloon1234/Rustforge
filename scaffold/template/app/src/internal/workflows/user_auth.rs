use core_db::common::{
    auth::hash::{hash_password, verify_password},
    sql::{generate_snowflake_i64, DbConn, Op},
};
use core_i18n::t;
use core_web::{
    auth::{self, IssuedTokenPair, TokenScopeGrant},
    error::AppError,
    Patch,
};
use generated::{
    guards::UserGuard,
    models::{UserBanStatus, UserCol, UserModel, UserRecord},
};

use crate::contracts::api::v1::user::auth::{
    UserLocaleUpdateInput, UserPasswordUpdateInput, UserProfileUpdateInput, UserRegisterInput,
};
use crate::internal::api::state::AppApiState;

pub async fn login(
    state: &AppApiState,
    username: &str,
    password: &str,
) -> Result<(UserRecord, IssuedTokenPair), AppError> {
    let username = username.trim().to_ascii_lowercase();
    let user = UserModel::query(DbConn::pool(&state.db))
        .where_col(UserCol::USERNAME, Op::Eq, username)
        .first()
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::Unauthorized(t("Invalid credentials")))?;

    let valid = verify_password(password, &user.password).map_err(AppError::from)?;
    if !valid {
        return Err(AppError::Unauthorized(t("Invalid credentials")));
    }

    check_ban(&user)?;

    let tokens = auth::issue_guard_session::<UserGuard>(
        &state.db,
        &state.auth,
        user.id,
        "user-session",
        TokenScopeGrant::AuthOnly,
    )
    .await
    .map_err(AppError::from)?;

    Ok((user, tokens))
}

pub async fn register(
    state: &AppApiState,
    req: UserRegisterInput,
) -> Result<(UserRecord, IssuedTokenPair), AppError> {
    let id = generate_snowflake_i64();
    let uuid = generate_unique_uuid(state).await?;
    let password_hash = hash_password(&req.password).map_err(AppError::from)?;

    let introducer_user_id = if let Some(ref referral_code) = req.referral_code {
        let introducer = UserModel::query(DbConn::pool(&state.db))
            .where_col(UserCol::UUID, Op::Eq, referral_code.clone())
            .first()
            .await
            .map_err(AppError::from)?
            ;
        introducer.map(|u| u.id)
    } else {
        None
    };

    let mut insert = UserModel::create(DbConn::pool(&state.db))
        .set(UserCol::ID, id)
        .map_err(AppError::from)?
        .set(UserCol::UUID, uuid)
        .map_err(AppError::from)?
        .set(UserCol::USERNAME, req.username.to_string())
        .map_err(AppError::from)?
        .set(UserCol::PASSWORD, password_hash)
        .map_err(AppError::from)?;

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
    if let Some(introducer_id) = introducer_user_id {
        insert = insert
            .set(UserCol::INTRODUCER_USER_ID, Some(introducer_id))
            .map_err(AppError::from)?;
    }

    let user = insert.save().await.map_err(AppError::from)?;

    let tokens = auth::issue_guard_session::<UserGuard>(
        &state.db,
        &state.auth,
        user.id,
        "user-session",
        TokenScopeGrant::AuthOnly,
    )
    .await
    .map_err(AppError::from)?;

    Ok((user, tokens))
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

pub async fn resolve_referral(
    state: &AppApiState,
    code: &str,
) -> Result<Option<(String, Option<String>)>, AppError> {
    let user = UserModel::query(DbConn::pool(&state.db))
        .where_col(UserCol::UUID, Op::Eq, code.to_string())
        .first()
        .await
        .map_err(AppError::from)?;
    Ok(user.map(|u| (u.username, u.name)))
}

pub async fn refresh(
    state: &AppApiState,
    refresh_token: &str,
) -> Result<IssuedTokenPair, AppError> {
    auth::refresh_guard_session::<UserGuard>(
        &state.db,
        &state.auth,
        refresh_token,
        "user-session",
    )
    .await
}

pub async fn revoke_session(state: &AppApiState, refresh_token: &str) -> Result<(), AppError> {
    auth::revoke_session_by_refresh_token::<UserGuard>(&state.db, refresh_token).await
}

pub async fn profile_update(
    state: &AppApiState,
    user_id: i64,
    req: UserProfileUpdateInput,
) -> Result<UserRecord, AppError> {
    let mut update = Ok(
        UserModel::query(DbConn::pool(&state.db))
            .where_col(UserCol::ID, Op::Eq, user_id)
            .patch(),
    );

    match req.name {
        Patch::Missing => {}
        Patch::Null => {
            update = update.and_then(|patch| patch.assign(UserCol::NAME, None::<String>));
        }
        Patch::Value(name) => {
            update = update.and_then(|patch| patch.assign(UserCol::NAME, Some(name)));
        }
    }

    match req.email {
        Patch::Missing => {}
        Patch::Null => {
            update = update.and_then(|patch| patch.assign(UserCol::EMAIL, None::<String>));
        }
        Patch::Value(email) => {
            update = update.and_then(|patch| patch.assign(UserCol::EMAIL, Some(email)));
        }
    }

    match req.country_iso2 {
        Patch::Missing => {}
        Patch::Null => {
            update = update.and_then(|patch| patch.assign(UserCol::COUNTRY_ISO2, None::<String>));
        }
        Patch::Value(iso2) => {
            update = update.and_then(|patch| patch.assign(UserCol::COUNTRY_ISO2, Some(iso2)));
        }
    }

    match req.contact_number {
        Patch::Missing => {}
        Patch::Null => {
            update = update.and_then(|patch| patch.assign(UserCol::CONTACT_NUMBER, None::<String>));
        }
        Patch::Value(number) => {
            update = update.and_then(|patch| patch.assign(UserCol::CONTACT_NUMBER, Some(number)));
        }
    }

    let affected = update
        .map_err(AppError::from)?
        .save()
        .await
        .map_err(AppError::from)?;
    if affected == 0 {
        return Err(AppError::NotFound(t("User not found")));
    }

    UserModel::find(DbConn::pool(&state.db), user_id)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(t("User not found")))
}

pub async fn locale_update(
    state: &AppApiState,
    user_id: i64,
    req: UserLocaleUpdateInput,
) -> Result<String, AppError> {
    let normalized = core_i18n::match_supported_locale(req.locale.trim())
        .ok_or_else(|| AppError::BadRequest(t("Unsupported locale")))?;

    let affected = UserModel::query(DbConn::pool(&state.db))
        .where_col(UserCol::ID, Op::Eq, user_id)
        .patch()
        .assign(UserCol::LOCALE, Some(normalized.to_string()))
        .map_err(AppError::from)?
        .save()
        .await
        .map_err(AppError::from)?;

    if affected == 0 {
        return Err(AppError::NotFound(t("User not found")));
    }

    Ok(normalized.to_string())
}

pub async fn password_update(
    state: &AppApiState,
    user_id: i64,
    req: UserPasswordUpdateInput,
) -> Result<(), AppError> {
    let user = UserModel::find(DbConn::pool(&state.db), user_id)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(t("User not found")))?;

    let valid = verify_password(&req.current_password, &user.password).map_err(AppError::from)?;
    if !valid {
        return Err(AppError::Unauthorized(t("Current password is incorrect")));
    }

    let password_hash = hash_password(&req.password).map_err(AppError::from)?;
    let affected = UserModel::query(DbConn::pool(&state.db))
        .where_col(UserCol::ID, Op::Eq, user_id)
        .patch()
        .assign(UserCol::PASSWORD, password_hash)
        .map_err(AppError::from)?
        .save()
        .await
        .map_err(AppError::from)?;
    if affected == 0 {
        return Err(AppError::NotFound(t("User not found")));
    }

    Ok(())
}

fn check_ban(user: &UserRecord) -> Result<(), AppError> {
    if matches!(user.ban, UserBanStatus::Yes) {
        return Err(AppError::Forbidden(t("Your account has been banned")));
    }
    Ok(())
}

pub async fn fetch_and_check_ban(state: &AppApiState, user_id: i64) -> Result<(), AppError> {
    let user = UserModel::find(DbConn::pool(&state.db), user_id)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(t("User not found")))?;
    check_ban(&user)
}
