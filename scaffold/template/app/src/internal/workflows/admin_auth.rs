use core_db::common::{
    auth::hash::verify_password,
    sql::{DbConn, Op},
};
use core_i18n::t;
use core_web::{
    auth::{self, IssuedTokenPair, TokenScopeGrant},
    error::AppError,
    Patch,
};
use generated::{
    guards::AdminGuard,
    models::{AdminCol, AdminModel, AdminRecord, AdminType},
};

use crate::contracts::api::v1::admin::auth::{
    AdminLocaleUpdateInput, AdminPasswordUpdateInput, AdminProfileUpdateInput,
};
use crate::internal::api::state::AppApiState;

pub fn resolve_scope_grant(admin: &AdminRecord) -> TokenScopeGrant {
    match admin.admin_type {
        AdminType::Developer | AdminType::SuperAdmin => TokenScopeGrant::Wildcard,
        AdminType::Admin => {
            let explicit = if admin
                .abilities
                .as_array()
                .is_some_and(|items| items.iter().any(|item| item.as_str() == Some("*")))
            {
                vec!["*".to_string()]
            } else {
                admin.parsed_abilities()
                    .into_iter()
                    .map(|permission| permission.as_str().to_string())
                    .collect::<Vec<_>>()
            };
            if explicit.is_empty() {
                TokenScopeGrant::AuthOnly
            } else {
                TokenScopeGrant::Explicit(explicit)
            }
        }
    }
}

pub async fn login(
    state: &AppApiState,
    username: &str,
    password: &str,
) -> Result<(AdminRecord, IssuedTokenPair), AppError> {
    let username = username.trim().to_ascii_lowercase();
    let admin = AdminModel::query(DbConn::pool(&state.db))
        .where_col(AdminCol::USERNAME, Op::Eq, username)
        .first()
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::Unauthorized(t("Invalid credentials")))?;

    let valid = verify_password(password, &admin.password).map_err(AppError::from)?;
    if !valid {
        return Err(AppError::Unauthorized(t("Invalid credentials")));
    }

    let scope_grant = resolve_scope_grant(&admin);
    let tokens = auth::issue_guard_session::<AdminGuard>(
        &state.db,
        &state.auth,
        admin.id,
        "admin-session",
        scope_grant,
    )
    .await
    .map_err(AppError::from)?;

    Ok((admin, tokens))
}

pub async fn refresh(
    state: &AppApiState,
    refresh_token: &str,
) -> Result<IssuedTokenPair, AppError> {
    auth::refresh_guard_session::<AdminGuard>(
        &state.db,
        &state.auth,
        refresh_token,
        "admin-session",
    )
    .await
}

pub async fn revoke_session(state: &AppApiState, refresh_token: &str) -> Result<(), AppError> {
    auth::revoke_session_by_refresh_token::<AdminGuard>(&state.db, refresh_token).await
}

pub async fn profile_update(
    state: &AppApiState,
    admin_id: i64,
    req: AdminProfileUpdateInput,
) -> Result<AdminRecord, AppError> {
    let mut update = AdminModel::query(DbConn::pool(&state.db))
        .where_col(AdminCol::ID, Op::Eq, admin_id)
        .patch()
        .assign(AdminCol::NAME, req.name.trim().to_string())
        .map_err(AppError::from)?;

    match req.email {
        Patch::Missing => {}
        Patch::Null => {
            update = update
                .assign(AdminCol::EMAIL, None::<String>)
                .map_err(AppError::from)?;
        }
        Patch::Value(email) => {
            let email = email.trim().to_ascii_lowercase();
            if email.is_empty() {
                update = update
                    .assign(AdminCol::EMAIL, None::<String>)
                    .map_err(AppError::from)?;
            } else {
                update = update
                    .assign(AdminCol::EMAIL, Some(email))
                    .map_err(AppError::from)?;
            }
        }
    }

    let affected = update.save().await.map_err(AppError::from)?;
    if affected == 0 {
        return Err(AppError::NotFound(t("Admin not found")));
    }

    AdminModel::find(DbConn::pool(&state.db), admin_id)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(t("Admin not found")))
}

pub async fn locale_update(
    state: &AppApiState,
    admin_id: i64,
    req: AdminLocaleUpdateInput,
) -> Result<String, AppError> {
    let normalized = core_i18n::match_supported_locale(req.locale.trim())
        .ok_or_else(|| AppError::BadRequest(t("Unsupported locale")))?;

    let affected = AdminModel::query(DbConn::pool(&state.db))
        .where_col(AdminCol::ID, Op::Eq, admin_id)
        .patch()
        .assign(AdminCol::LOCALE, Some(normalized.to_string()))
        .map_err(AppError::from)?
        .save()
        .await
        .map_err(AppError::from)?;

    if affected == 0 {
        return Err(AppError::NotFound(t("Admin not found")));
    }

    Ok(normalized.to_string())
}

pub async fn password_update(
    state: &AppApiState,
    admin_id: i64,
    req: AdminPasswordUpdateInput,
) -> Result<(), AppError> {
    let admin = AdminModel::find(DbConn::pool(&state.db), admin_id)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(t("Admin not found")))?;

    let valid = verify_password(&req.current_password, &admin.password).map_err(AppError::from)?;
    if !valid {
        return Err(AppError::Unauthorized(t("Current password is incorrect")));
    }

    let affected = AdminModel::query(DbConn::pool(&state.db))
        .where_col(AdminCol::ID, Op::Eq, admin_id)
        .patch()
        .assign(AdminCol::PASSWORD, req.password.to_string())
        .map_err(AppError::from)?
        .save()
        .await
        .map_err(AppError::from)?;
    if affected == 0 {
        return Err(AppError::NotFound(t("Admin not found")));
    }

    Ok(())
}
