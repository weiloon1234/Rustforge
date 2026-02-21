use std::collections::BTreeSet;
use std::convert::Infallible;

use crate::{
    auth::Guard,
    authz::{PermissionGuard, PermissionMode},
};

// Re-export common types for easier usage
pub use aide;
pub use aide::axum::ApiRouter;
pub use schemars;

pub type AppRouter<S> = ApiRouter<S>;
pub const BEARER_AUTH_SCHEME: &str = "bearerAuth";

/// Helper to create a new ApiRouter with default configuration if needed
pub fn router<S>() -> ApiRouter<S>
where
    S: Clone + Send + Sync + 'static,
{
    ApiRouter::new()
}

/// Registers a standard Bearer auth scheme in OpenAPI components.
/// Use once when finalizing the top-level API router.
pub fn with_bearer_auth_scheme(
    api: aide::transform::TransformOpenApi<'_>,
) -> aide::transform::TransformOpenApi<'_> {
    api.security_scheme(
        BEARER_AUTH_SCHEME,
        aide::openapi::SecurityScheme::Http {
            scheme: "bearer".to_string(),
            bearer_format: Some("PAT".to_string()),
            description: Some("Use `Authorization: Bearer <access_token>`".to_string()),
            extensions: Default::default(),
        },
    )
}

/// Marks operation as requiring Bearer auth.
pub fn require_bearer_auth(
    op: aide::transform::TransformOperation<'_>,
) -> aide::transform::TransformOperation<'_> {
    op.security_requirement(BEARER_AUTH_SCHEME)
}

/// Adds bearer auth requirement and machine-readable permission metadata.
///
/// OpenAPI extensions emitted on the operation:
/// - `x-required-guard`
/// - `x-required-permission-mode`
/// - `x-required-permissions`
pub fn with_required_permissions<'t, P>(
    op: aide::transform::TransformOperation<'t>,
    guard: &str,
    mode: PermissionMode,
    permissions: impl IntoIterator<Item = P>,
) -> aide::transform::TransformOperation<'t>
where
    P: AsRef<str>,
{
    let mut unique = BTreeSet::new();
    for permission in permissions {
        let permission = permission.as_ref().trim();
        if !permission.is_empty() {
            unique.insert(permission.to_string());
        }
    }
    let permission_vec = unique.into_iter().collect::<Vec<_>>();

    let mut op = require_bearer_auth(op);

    {
        let inner = op.inner_mut();
        inner.extensions.insert(
            "x-required-guard".to_string(),
            serde_json::Value::String(guard.trim().to_string()),
        );
        inner.extensions.insert(
            "x-required-permission-mode".to_string(),
            serde_json::Value::String(mode.as_str().to_string()),
        );
        inner.extensions.insert(
            "x-required-permissions".to_string(),
            serde_json::Value::Array(
                permission_vec
                    .iter()
                    .map(|permission| serde_json::Value::String(permission.clone()))
                    .collect(),
            ),
        );
    }

    op
}

#[derive(Debug, Clone)]
pub struct RequiredPermissionSpec {
    guard: String,
    mode: PermissionMode,
    permissions: Vec<String>,
}

impl RequiredPermissionSpec {
    pub fn new<P>(
        guard: impl Into<String>,
        mode: PermissionMode,
        permissions: impl IntoIterator<Item = P>,
    ) -> Self
    where
        P: AsRef<str>,
    {
        let guard = guard.into().trim().to_string();
        let mut unique = BTreeSet::new();
        for permission in permissions {
            let permission = permission.as_ref().trim();
            if !permission.is_empty() {
                unique.insert(permission.to_string());
            }
        }
        Self {
            guard,
            mode,
            permissions: unique.into_iter().collect(),
        }
    }

    pub fn apply_to_operation<'t>(
        &self,
        op: aide::transform::TransformOperation<'t>,
    ) -> aide::transform::TransformOperation<'t> {
        with_required_permissions(
            op,
            self.guard.as_str(),
            self.mode,
            self.permissions.iter().map(String::as_str),
        )
    }

    pub fn permission_guard(&self) -> PermissionGuard {
        PermissionGuard::new(self.mode, self.permissions.iter().cloned())
    }
}

pub fn required_permissions<P>(
    guard: impl Into<String>,
    mode: PermissionMode,
    permissions: impl IntoIterator<Item = P>,
) -> RequiredPermissionSpec
where
    P: AsRef<str>,
{
    RequiredPermissionSpec::new(guard, mode, permissions)
}

pub fn required_permissions_for_guard<G, P>(
    mode: PermissionMode,
    permissions: impl IntoIterator<Item = P>,
) -> RequiredPermissionSpec
where
    G: Guard,
    P: AsRef<str>,
{
    RequiredPermissionSpec::new(G::name(), mode, permissions)
}

macro_rules! permission_checked_method_top_level {
    ($name:ident, $name_with:ident, $base_with:ident) => {
        pub fn $name<G, H, I, O, T, S, P>(
            handler: H,
            guard_type: G,
            mode: PermissionMode,
            permissions: impl IntoIterator<Item = P>,
        ) -> aide::axum::routing::ApiMethodRouter<S, Infallible>
        where
            G: Guard + 'static,
            H: axum::handler::Handler<T, S> + aide::operation::OperationHandler<I, O>,
            I: aide::operation::OperationInput,
            O: aide::operation::OperationOutput,
            S: Clone + Send + Sync + 'static,
            T: 'static,
            P: AsRef<str>,
        {
            $name_with::<G, H, I, O, T, S, P, _>(handler, guard_type, mode, permissions, |op| op)
        }

        pub fn $name_with<G, H, I, O, T, S, P, F>(
            handler: H,
            guard_type: G,
            mode: PermissionMode,
            permissions: impl IntoIterator<Item = P>,
            transform: F,
        ) -> aide::axum::routing::ApiMethodRouter<S, Infallible>
        where
            G: Guard + 'static,
            H: axum::handler::Handler<T, S> + aide::operation::OperationHandler<I, O>,
            I: aide::operation::OperationInput,
            O: aide::operation::OperationOutput,
            S: Clone + Send + Sync + 'static,
            T: 'static,
            P: AsRef<str>,
            F: FnOnce(
                aide::transform::TransformOperation<'_>,
            ) -> aide::transform::TransformOperation<'_>,
        {
            let _ = guard_type;
            let spec = RequiredPermissionSpec::new(G::name(), mode, permissions);
            let op_spec = spec.clone();
            let check_mode = spec.mode;
            let required = spec.permissions.clone();
            aide::axum::routing::$base_with(handler, move |op| {
                let op = op_spec.apply_to_operation(op);
                transform(op)
            })
            .route_layer(axum::middleware::from_fn(
                move |request: axum::extract::Request, next: axum::middleware::Next| {
                    let required = required.clone();
                    async move {
                        let auth = request
                            .extensions()
                            .get::<crate::auth::AuthUser<G>>()
                            .cloned()
                            .ok_or_else(|| {
                                crate::error::AppError::Unauthorized(
                                    "Missing authenticated guard context".to_string(),
                                )
                            })?;

                        crate::authz::ensure_permissions(&auth, check_mode, &required)?;
                        Ok::<axum::response::Response, crate::error::AppError>(
                            next.run(request).await,
                        )
                    }
                },
            ))
        }
    };
}

permission_checked_method_top_level!(
    with_permission_check_get,
    with_permission_check_get_with,
    get_with
);
permission_checked_method_top_level!(
    with_permission_check_post,
    with_permission_check_post_with,
    post_with
);
permission_checked_method_top_level!(
    with_permission_check_patch,
    with_permission_check_patch_with,
    patch_with
);
permission_checked_method_top_level!(
    with_permission_check_delete,
    with_permission_check_delete_with,
    delete_with
);

/// Appends a Markdown note block to the operation description.
pub fn with_route_notes<'t>(
    mut op: aide::transform::TransformOperation<'t>,
    notes: &[&str],
) -> aide::transform::TransformOperation<'t> {
    let mut unique = BTreeSet::new();
    for note in notes {
        let trimmed = note.trim();
        if !trimmed.is_empty() {
            unique.insert(trimmed.to_string());
        }
    }

    if unique.is_empty() {
        return op;
    }

    let block = {
        let mut out = String::from("Notes:\n");
        for note in unique {
            out.push_str("- ");
            out.push_str(&note);
            out.push('\n');
        }
        out
    };

    let current = op
        .inner_mut()
        .description
        .as_deref()
        .unwrap_or_default()
        .trim()
        .to_string();

    let next = if current.is_empty() {
        block
    } else {
        format!("{current}\n\n{block}")
    };

    op.inner_mut().description = Some(next);
    op
}
