use crate::permissions::Permission;
use core_db::common::auth::permissions::has_permission as granted_has_permission;

#[rf_db_enum(storage = "string")]
pub enum AdminType {
    Developer,
    SuperAdmin,
    Admin,
}

#[rf_model(table = "admin", soft_delete)]
pub struct Admin {
    #[rf(pk(strategy = snowflake))]
    pub id: i64,
    pub username: String,
    pub email: Option<String>,
    pub locale: Option<String>,
    #[rf(hashed)]
    pub password: String,
    pub name: String,
    pub admin_type: AdminType,
    pub abilities: serde_json::Value,
    pub created_at: time::OffsetDateTime,
    pub updated_at: time::OffsetDateTime,
}

pub fn admin_identity(
    username: Option<&str>,
    name: Option<&str>,
    email: Option<&str>,
    id: Option<i64>,
) -> String {
    if let Some(value) = first_non_empty(username) {
        return value.to_string();
    }
    if let Some(value) = first_non_empty(name) {
        return value.to_string();
    }
    if let Some(value) = first_non_empty(email) {
        return value.to_string();
    }
    id.map(|value| value.to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn first_non_empty(value: Option<&str>) -> Option<&str> {
    value.and_then(|v| {
        let trimmed = v.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}

pub fn admin_permissions(abilities: &serde_json::Value) -> Vec<String> {
    let mut out = Vec::new();

    if let Some(items) = abilities.as_array() {
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

#[rf_record_impl]
impl AdminRecord {
    #[rf_computed]
    pub fn identity(&self) -> String {
        admin_identity(
            Some(self.username.as_str()),
            Some(self.name.as_str()),
            self.email.as_deref(),
            Some(self.id),
        )
    }

    pub fn has_permission(&self, permission: Permission) -> bool {
        if matches!(
            self.admin_type,
            AdminType::Developer | AdminType::SuperAdmin
        ) {
            return true;
        }

        let granted = admin_permissions(&self.abilities);
        granted_has_permission(&granted, permission.as_str())
    }

    pub fn has_any_permissions(&self, permissions: &[Permission]) -> bool {
        permissions
            .iter()
            .copied()
            .any(|permission| self.has_permission(permission))
    }

    pub fn has_all_permissions(&self, permissions: &[Permission]) -> bool {
        permissions
            .iter()
            .copied()
            .all(|permission| self.has_permission(permission))
    }

    pub fn parsed_abilities(&self) -> Vec<Permission> {
        self.abilities
            .as_array()
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.as_str())
                    .filter_map(Permission::from_str)
                    .collect()
            })
            .unwrap_or_default()
    }
}
