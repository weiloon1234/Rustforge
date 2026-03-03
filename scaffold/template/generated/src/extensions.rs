// Manual extensions and strongly typed custom model shapes.
// Safe to edit.

pub mod admin {
    pub mod types {
        use crate::models::AdminView;

        pub trait AdminViewComputedExt {
            /// Preferred display identity for admin-facing UI.
            /// Fallback order: username -> name -> email -> id.
            fn identity(&self) -> String;
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
    }
}

pub mod content_page {
    pub mod types {}
}
