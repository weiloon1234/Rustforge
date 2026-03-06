// Manual extensions and strongly typed custom model shapes.
// Safe to edit.

pub mod admin {
    pub mod types {
        use crate::{
            models::{AdminType, AdminView},
            permissions::Permission,
        };
        use core_db::common::auth::permissions::has_permission as granted_has_permission;

        pub trait AdminViewComputedExt {
            /// Preferred display identity for admin-facing UI.
            /// Fallback order: username -> name -> email -> id.
            fn identity(&self) -> String;
        }

        pub trait AdminViewPermissionExt {
            fn has_permission(&self, permission: Permission) -> bool;
            fn has_any_permissions(&self, permissions: &[Permission]) -> bool;
            fn has_all_permissions(&self, permissions: &[Permission]) -> bool;
        }

        impl AdminViewComputedExt for AdminView {
            fn identity(&self) -> String {
                admin_identity(
                    Some(self.username.as_str()),
                    Some(self.name.as_str()),
                    self.email.as_deref(),
                    Some(self.id),
                )
            }
        }

        impl AdminViewPermissionExt for AdminView {
            fn has_permission(&self, permission: Permission) -> bool {
                if matches!(
                    self.admin_type,
                    AdminType::Developer | AdminType::SuperAdmin
                ) {
                    return true;
                }

                let granted = admin_permissions(self);
                granted_has_permission(&granted, permission.as_str())
            }

            fn has_any_permissions(&self, permissions: &[Permission]) -> bool {
                permissions
                    .iter()
                    .copied()
                    .any(|permission| self.has_permission(permission))
            }

            fn has_all_permissions(&self, permissions: &[Permission]) -> bool {
                permissions
                    .iter()
                    .copied()
                    .all(|permission| self.has_permission(permission))
            }
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

        #[cfg(test)]
        mod tests {
            use super::AdminViewPermissionExt;
            use crate::{
                models::{AdminType, AdminView},
                permissions::Permission,
            };
            use serde_json::json;
            use time::OffsetDateTime;

            fn sample_admin(admin_type: AdminType, abilities: serde_json::Value) -> AdminView {
                AdminView {
                    id: 1,
                    username: "abcd".to_string(),
                    email: None,
                    locale: None,
                    password: "secret".to_string(),
                    name: "ABCD".to_string(),
                    admin_type,
                    abilities,
                    created_at: OffsetDateTime::UNIX_EPOCH,
                    updated_at: OffsetDateTime::UNIX_EPOCH,
                    deleted_at: None,
                    admin_type_explained: String::new(),
                }
            }

            #[test]
            fn privileged_admin_types_have_all_permissions() {
                let developer = sample_admin(AdminType::Developer, json!([]));
                let superadmin = sample_admin(AdminType::SuperAdmin, json!([]));

                assert!(developer.has_permission(Permission::CountryManage));
                assert!(superadmin.has_all_permissions(&[
                    Permission::AdminManage,
                    Permission::ContentPageManage,
                ]));
            }

            #[test]
            fn normal_admin_uses_explicit_abilities_with_shared_matching_rules() {
                let admin = sample_admin(
                    AdminType::Admin,
                    json!(["content_page.manage", "country.read"]),
                );

                assert!(admin.has_permission(Permission::ContentPageManage));
                assert!(admin.has_permission(Permission::ContentPageRead));
                assert!(admin.has_any_permissions(&[
                    Permission::CountryManage,
                    Permission::CountryRead,
                ]));
                assert!(!admin.has_permission(Permission::CountryManage));
            }

            #[test]
            fn invalid_ability_values_are_ignored() {
                let admin = sample_admin(
                    AdminType::Admin,
                    json!([123, "", "nope", "country.manage"]),
                );

                assert!(admin.has_permission(Permission::CountryManage));
                assert!(admin.has_permission(Permission::CountryRead));
                assert!(!admin.has_permission(Permission::AdminRead));
            }
        }
    }
}

pub mod content_page {
    pub mod types {}
}
