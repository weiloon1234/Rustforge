use core_db::common::sql::{generate_snowflake_i64, DbConn, Op};
use core_i18n::t;
use core_web::{auth::AuthUser, error::AppError};
use generated::{
    guards::AdminGuard,
    models::{Admin, AdminType, AdminView},
    permissions::Permission,
};

use crate::{
    contracts::api::v1::admin::account::{CreateAdminInput, UpdateAdminInput},
    internal::api::state::AppApiState,
};

pub async fn detail(state: &AppApiState, id: i64) -> Result<AdminView, AppError> {
    Admin::new(DbConn::pool(&state.db), None)
        .find(id)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(t("Admin not found")))
}

pub async fn create(
    state: &AppApiState,
    auth: &AuthUser<AdminGuard>,
    req: CreateAdminInput,
) -> Result<AdminView, AppError> {
    let username = req.username.trim().to_ascii_lowercase();

    let abilities = ensure_assignable_permissions(auth, &req.abilities)?;

    let mut insert = Admin::new(DbConn::pool(&state.db), None)
        .insert()
        .set_id(generate_snowflake_i64())
        .set_username(username)
        .set_name(req.name.trim().to_string())
        .set_admin_type(AdminType::Admin)
        .set_abilities(permissions_to_json(&abilities));

    if let Some(email) = normalize_optional_email(req.email) {
        insert = insert.set_email(Some(email));
    }

    let insert = insert.set_password(&req.password).map_err(AppError::from)?;
    insert.save().await.map_err(AppError::from)
}

pub async fn update(
    state: &AppApiState,
    auth: &AuthUser<AdminGuard>,
    id: i64,
    req: UpdateAdminInput,
) -> Result<AdminView, AppError> {
    if auth.user.id == id {
        return Err(AppError::Forbidden(t(
            "You cannot update your own admin account here",
        )));
    }

    let existing = detail(state, id).await?;
    let mut update = Admin::new(DbConn::pool(&state.db), None)
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

    if let Some(name) = req.name {
        update = update.set_name(name.trim().to_string());
        touched = true;
    }

    if let Some(email) = normalize_optional_email(req.email) {
        update = update.set_email(Some(email));
        touched = true;
    }

    if let Some(abilities) = req.abilities {
        let abilities = ensure_assignable_permissions(auth, &abilities)?;
        update = update.set_abilities(permissions_to_json(&abilities));
        touched = true;
    }

    if !touched {
        return Ok(existing);
    }

    let affected = update.save().await.map_err(AppError::from)?;
    if affected == 0 {
        return Err(AppError::NotFound(t("Admin not found")));
    }

    detail(state, id).await
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

    let affected = Admin::new(DbConn::pool(&state.db), None)
        .delete(id)
        .await
        .map_err(AppError::from)?;
    if affected == 0 {
        return Err(AppError::NotFound(t("Admin not found")));
    }
    Ok(())
}

fn normalize_optional_email(email: Option<String>) -> Option<String> {
    email
        .as_deref()
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

    let requested = requested
        .iter()
        .map(|permission| permission.as_str().to_string())
        .collect::<Vec<_>>();

    if matches!(
        auth.user.admin_type,
        AdminType::Developer | AdminType::SuperAdmin
    ) {
        return Ok(requested);
    }

    if requested
        .iter()
        .all(|permission| auth.has_permission(permission.as_str()))
    {
        return Ok(requested);
    }

    Err(AppError::Forbidden(t(
        "Cannot assign permissions you do not have",
    )))
}

fn permissions_to_json(values: &[String]) -> serde_json::Value {
    serde_json::Value::Array(
        values
            .iter()
            .map(|value| serde_json::Value::String(value.clone()))
            .collect(),
    )
}
