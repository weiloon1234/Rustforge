use core_db::common::{
    auth::hash::verify_password,
    sql::{DbConn, Op},
};
use core_i18n::t;
use core_web::{
    auth::{self, IssuedTokenPair, TokenScopeGrant},
    error::AppError,
};
use generated::{
    guards::AdminGuard,
    models::{Admin, AdminQuery, AdminType, AdminView},
    permissions::Permission,
};

use crate::contracts::api::v1::admin_auth::{AdminPasswordUpdateInput, AdminProfileUpdateInput};
use crate::internal::api::state::AppApiState;

pub fn resolve_scope_grant(admin: &AdminView) -> TokenScopeGrant {
    match admin.admin_type {
        AdminType::Developer | AdminType::SuperAdmin => TokenScopeGrant::Wildcard,
        AdminType::Admin => {
            let explicit = admin_permissions(admin);
            if explicit.is_empty() {
                TokenScopeGrant::AuthOnly
            } else {
                TokenScopeGrant::Explicit(explicit)
            }
        }
    }
}

fn admin_permissions(admin: &AdminView) -> Vec<String> {
    let mut out = Vec::new();

    if let Some(items) = admin.abilities.as_array() {
        for item in items {
            let Some(raw) = item.as_str() else {
                continue;
            };
            let value = raw.trim();
            if value.is_empty() {
                continue;
            }
            if value == "*" {
                out.push("*".to_string());
                continue;
            }
            if let Some(permission) = Permission::from_str(value) {
                out.push(permission.as_str().to_string());
            }
        }
    }

    out.sort();
    out.dedup();
    out
}

pub async fn login(
    state: &AppApiState,
    username: &str,
    password: &str,
) -> Result<(AdminView, IssuedTokenPair), AppError> {
    let username = username.trim().to_ascii_lowercase();
    let admin = AdminQuery::new(DbConn::pool(&state.db), None)
        .where_username(Op::Eq, username)
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

pub async fn refresh(state: &AppApiState, refresh_token: &str) -> Result<IssuedTokenPair, AppError> {
    auth::refresh_guard_session::<AdminGuard>(&state.db, &state.auth, refresh_token, "admin-session")
        .await
}

pub async fn revoke_session(state: &AppApiState, refresh_token: &str) -> Result<(), AppError> {
    auth::revoke_session_by_refresh_token::<AdminGuard>(&state.db, refresh_token).await
}

pub async fn profile_update(
    state: &AppApiState,
    admin_id: i64,
    req: AdminProfileUpdateInput,
) -> Result<AdminView, AppError> {
    let mut update = Admin::new(DbConn::pool(&state.db), None)
        .update()
        .where_id(Op::Eq, admin_id)
        .set_name(req.name.trim().to_string());

    if let Some(email) = req.email {
        let email = email.trim().to_ascii_lowercase();
        if !email.is_empty() {
            update = update.set_email(Some(email));
        }
    }

    let affected = update.save().await.map_err(AppError::from)?;
    if affected == 0 {
        return Err(AppError::NotFound(t("Admin not found")));
    }

    Admin::new(DbConn::pool(&state.db), None)
        .find(admin_id)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(t("Admin not found")))
}

pub async fn password_update(
    state: &AppApiState,
    admin_id: i64,
    req: AdminPasswordUpdateInput,
) -> Result<(), AppError> {
    let admin = Admin::new(DbConn::pool(&state.db), None)
        .find(admin_id)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(t("Admin not found")))?;

    let valid = verify_password(&req.current_password, &admin.password).map_err(AppError::from)?;
    if !valid {
        return Err(AppError::Unauthorized(t("Current password is incorrect")));
    }

    let update = Admin::new(DbConn::pool(&state.db), None)
        .update()
        .where_id(Op::Eq, admin_id)
        .set_password(&req.password)
        .map_err(AppError::from)?;

    let affected = update.save().await.map_err(AppError::from)?;
    if affected == 0 {
        return Err(AppError::NotFound(t("Admin not found")));
    }

    Ok(())
}
