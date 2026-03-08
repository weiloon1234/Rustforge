use core_db::common::{
    auth::hash::verify_password,
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
    models::{User, UserBanStatus, UserQuery, UserView},
};

use crate::contracts::api::v1::user::auth::{
    UserLocaleUpdateInput, UserPasswordUpdateInput, UserProfileUpdateInput, UserRegisterInput,
};
use crate::internal::api::state::AppApiState;

pub async fn login(
    state: &AppApiState,
    username: &str,
    password: &str,
) -> Result<(UserView, IssuedTokenPair), AppError> {
    let username = username.trim().to_ascii_lowercase();
    let user = UserQuery::new(DbConn::pool(&state.db), None)
        .where_username(Op::Eq, username)
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
) -> Result<(UserView, IssuedTokenPair), AppError> {
    let id = generate_snowflake_i64();
    let uuid = generate_unique_uuid(state).await?;

    let introducer_user_id = if let Some(ref referral_code) = req.referral_code {
        let introducer = UserQuery::new(DbConn::pool(&state.db), None)
            .where_uuid(Op::Eq, referral_code.clone())
            .first()
            .await
            .map_err(AppError::from)?;
        introducer.map(|u| u.id)
    } else {
        None
    };

    let mut insert = User::new(DbConn::pool(&state.db), None)
        .insert()
        .set_id(id)
        .set_uuid(uuid)
        .set_username(req.username.to_string())
        .set_password(&req.password)
        .map_err(AppError::from)?;

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
    if let Some(introducer_id) = introducer_user_id {
        insert = insert.set_introducer_user_id(Some(introducer_id));
    }

    insert.save().await.map_err(AppError::from)?;

    let user = User::new(DbConn::pool(&state.db), None)
        .find(id)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::BadRequest(t("Failed to create user")))?;

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
        let existing = UserQuery::new(DbConn::pool(&state.db), None)
            .where_uuid(Op::Eq, uuid.clone())
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
    let user = UserQuery::new(DbConn::pool(&state.db), None)
        .where_uuid(Op::Eq, code.to_string())
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
) -> Result<UserView, AppError> {
    let mut update = User::new(DbConn::pool(&state.db), None)
        .update()
        .where_id(Op::Eq, user_id);

    match req.name {
        Patch::Missing => {}
        Patch::Null => {
            update = update.set_name(None);
        }
        Patch::Value(name) => {
            update = update.set_name(Some(name));
        }
    }

    match req.email {
        Patch::Missing => {}
        Patch::Null => {
            update = update.set_email(None);
        }
        Patch::Value(email) => {
            update = update.set_email(Some(email));
        }
    }

    match req.country_iso2 {
        Patch::Missing => {}
        Patch::Null => {
            update = update.set_country_iso2(None);
        }
        Patch::Value(iso2) => {
            update = update.set_country_iso2(Some(iso2));
        }
    }

    match req.contact_number {
        Patch::Missing => {}
        Patch::Null => {
            update = update.set_contact_number(None);
        }
        Patch::Value(number) => {
            update = update.set_contact_number(Some(number));
        }
    }

    let affected = update.save().await.map_err(AppError::from)?;
    if affected == 0 {
        return Err(AppError::NotFound(t("User not found")));
    }

    User::new(DbConn::pool(&state.db), None)
        .find(user_id)
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

    let affected = User::new(DbConn::pool(&state.db), None)
        .update()
        .where_id(Op::Eq, user_id)
        .set_locale(Some(normalized.to_string()))
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
    let user = User::new(DbConn::pool(&state.db), None)
        .find(user_id)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(t("User not found")))?;

    let valid = verify_password(&req.current_password, &user.password).map_err(AppError::from)?;
    if !valid {
        return Err(AppError::Unauthorized(t("Current password is incorrect")));
    }

    let update = User::new(DbConn::pool(&state.db), None)
        .update()
        .where_id(Op::Eq, user_id)
        .set_password(&req.password)
        .map_err(AppError::from)?;

    let affected = update.save().await.map_err(AppError::from)?;
    if affected == 0 {
        return Err(AppError::NotFound(t("User not found")));
    }

    Ok(())
}

fn check_ban(user: &UserView) -> Result<(), AppError> {
    if matches!(user.ban, UserBanStatus::Yes) {
        return Err(AppError::Forbidden(t("Your account has been banned")));
    }
    Ok(())
}

pub async fn fetch_and_check_ban(state: &AppApiState, user_id: i64) -> Result<(), AppError> {
    let user = User::new(DbConn::pool(&state.db), None)
        .find(user_id)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(t("User not found")))?;
    check_ban(&user)
}
