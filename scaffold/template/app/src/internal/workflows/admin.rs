use core_db::common::{
    auth::hash::hash_password,
    sql::{generate_snowflake_i64, DbConn, Op},
};
use core_i18n::t;
use core_web::{auth::AuthUser, error::AppError, Patch};
use generated::{
    guards::AdminGuard,
    models::{AdminCol, AdminModel, AdminRecord, AdminType},
    permissions::Permission,
};

use crate::{
    contracts::api::v1::admin::account::{CreateAdminInput, UpdateAdminInput},
    internal::api::state::AppApiState,
};

pub async fn detail(state: &AppApiState, id: i64) -> Result<AdminRecord, AppError> {
    AdminModel::find(DbConn::pool(&state.db), id)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(t("Admin not found")))
}

pub async fn create(
    state: &AppApiState,
    auth: &AuthUser<AdminGuard>,
    req: CreateAdminInput,
) -> Result<AdminRecord, AppError> {
    let username = req.username.trim().to_ascii_lowercase();
    let password_hash = hash_password(&req.password).map_err(AppError::from)?;

    let abilities = ensure_assignable_permissions(auth, &req.abilities)?;

    let mut insert = AdminModel::create(DbConn::pool(&state.db))
        .set(AdminCol::ID, generate_snowflake_i64())
        .map_err(AppError::from)?
        .set(AdminCol::USERNAME, username)
        .map_err(AppError::from)?
        .set(AdminCol::NAME, req.name.trim().to_string())
        .map_err(AppError::from)?
        .set(AdminCol::ADMIN_TYPE, AdminType::Admin)
        .map_err(AppError::from)?
        .set(AdminCol::ABILITIES, permissions_to_json(&abilities))
        .map_err(AppError::from)?
        .set(AdminCol::PASSWORD, password_hash)
        .map_err(AppError::from)?;

    if let Some(email) = req.email.as_deref().and_then(normalize_email_value) {
        insert = insert
            .set(AdminCol::EMAIL, Some(email))
            .map_err(AppError::from)?;
    }

    let created = insert.save().await.map_err(AppError::from)?;
    Ok(created)
}

pub async fn update(
    state: &AppApiState,
    auth: &AuthUser<AdminGuard>,
    id: i64,
    req: UpdateAdminInput,
) -> Result<AdminRecord, AppError> {
    if auth.user.id == id {
        return Err(AppError::Forbidden(t(
            "You cannot update your own admin account here",
        )));
    }

    let existing = detail(state, id).await?;
    let mut update = Ok(
        AdminModel::query(DbConn::pool(&state.db))
            .where_col(AdminCol::ID, Op::Eq, id)
            .patch(),
    );
    let mut touched = false;

    if let Some(username) = req.username {
        let username = username.trim().to_ascii_lowercase();
        if username != existing.username {
            update = update.and_then(|patch| patch.assign(AdminCol::USERNAME, username));
            touched = true;
        }
    }

    if let Some(name) = req.name {
        update = update.and_then(|patch| patch.assign(AdminCol::NAME, name.trim().to_string()));
        touched = true;
    }

    if let Some(password) = req.password {
        let password_hash = hash_password(&password).map_err(AppError::from)?;
        update = update.and_then(|patch| patch.assign(AdminCol::PASSWORD, password_hash));
        touched = true;
    }

    match req.email {
        Patch::Missing => {}
        Patch::Null => {
            if existing.email.is_some() {
                update = update.and_then(|patch| patch.assign(AdminCol::EMAIL, None::<String>));
                touched = true;
            }
        }
        Patch::Value(email) => {
            let normalized = normalize_email_value(&email);
            if existing.email != normalized {
                update = update.and_then(|patch| patch.assign(AdminCol::EMAIL, normalized));
                touched = true;
            }
        }
    }

    if let Some(abilities) = req.abilities {
        let abilities = ensure_assignable_permissions(auth, &abilities)?;
        update = update.and_then(|patch| patch.assign(AdminCol::ABILITIES, permissions_to_json(&abilities)));
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
        return Err(AppError::NotFound(t("Admin not found")));
    }

    let updated = detail(state, id).await?;
    Ok(updated)
}

pub async fn remove(
    state: &AppApiState,
    auth: &AuthUser<AdminGuard>,
    id: i64,
) -> Result<(), AppError> {
    if auth.user.id == id {
        return Err(AppError::Forbidden(t(
            "You cannot delete your own admin account here",
        )));
    }

    let target = detail(state, id).await?;
    if matches!(
        target.admin_type,
        AdminType::Developer | AdminType::SuperAdmin
    ) {
        return Err(AppError::Forbidden(t(
            "Cannot delete developer or superadmin accounts",
        )));
    }

    let affected = AdminModel::query(DbConn::pool(&state.db))
        .where_col(generated::models::AdminCol::ID, Op::Eq, id)
        .delete()
        .await
        .map_err(AppError::from)?;
    if affected == 0 {
        return Err(AppError::NotFound(t("Admin not found")));
    }
    Ok(())
}

fn normalize_email_value(email: &str) -> Option<String> {
    Some(email)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_ascii_lowercase)
}

fn ensure_assignable_permissions(
    auth: &AuthUser<AdminGuard>,
    requested: &[Permission],
) -> Result<Vec<String>, AppError> {
    if matches!(auth.user.admin_type, AdminType::Admin)
        && requested
            .iter()
            .any(|permission| matches!(permission, Permission::AdminRead | Permission::AdminManage))
    {
        return Err(AppError::Forbidden(t(
            "Normal admin cannot assign admin.read or admin.manage",
        )));
    }

    if matches!(
        auth.user.admin_type,
        AdminType::Developer | AdminType::SuperAdmin
    ) {
        return Ok(permission_strings(requested));
    }

    if requested
        .iter()
        .copied()
        .all(|permission| auth.user.has_permission(permission))
    {
        return Ok(permission_strings(requested));
    }

    Err(AppError::Forbidden(t(
        "Cannot assign permissions you do not have",
    )))
}

fn permission_strings(values: &[Permission]) -> Vec<String> {
    values
        .iter()
        .map(|permission| permission.as_str().to_string())
        .collect()
}

fn permissions_to_json(values: &[String]) -> serde_json::Value {
    serde_json::Value::Array(
        values
            .iter()
            .map(|value| serde_json::Value::String(value.clone()))
            .collect(),
    )
}
