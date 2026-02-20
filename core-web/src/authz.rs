use std::sync::Arc;

use axum::{
    extract::Request,
    middleware::{from_fn, Next},
    response::Response,
};
use tower::Layer;

use crate::{
    auth::{AuthUser, Guard},
    error::AppError,
};
use core_db::common::auth::permissions::{
    has_permission as has_permission_in_granted,
    permission_matches as permission_matches_in_granted,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionMode {
    Any,
    All,
}

impl PermissionMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Any => "any",
            Self::All => "all",
        }
    }
}

#[derive(Debug, Clone)]
pub struct PermissionGuard {
    mode: PermissionMode,
    required: Arc<Vec<String>>,
}

impl PermissionGuard {
    pub fn new<T>(mode: PermissionMode, required: impl IntoIterator<Item = T>) -> Self
    where
        T: Into<String>,
    {
        let required = required
            .into_iter()
            .map(Into::into)
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .collect::<Vec<_>>();

        Self {
            mode,
            required: Arc::new(required),
        }
    }

    pub fn mode(&self) -> PermissionMode {
        self.mode
    }

    pub fn any<T>(required: impl IntoIterator<Item = T>) -> Self
    where
        T: Into<String>,
    {
        Self::new(PermissionMode::Any, required)
    }

    pub fn all<T>(required: impl IntoIterator<Item = T>) -> Self
    where
        T: Into<String>,
    {
        Self::new(PermissionMode::All, required)
    }

    pub fn required(&self) -> &[String] {
        self.required.as_ref()
    }

    pub fn ensure<G: Guard>(&self, auth: &AuthUser<G>) -> Result<(), AppError> {
        ensure_permissions(auth, self.mode, self.required())
    }

    pub fn layer<G>(&self) -> impl Layer<axum::routing::Route> + Clone
    where
        G: Guard + 'static,
    {
        let guard = self.clone();
        from_fn::<_, ()>(move |request: Request, next: Next| {
            let guard = guard.clone();
            async move {
                let auth = request
                    .extensions()
                    .get::<AuthUser<G>>()
                    .cloned()
                    .ok_or_else(|| {
                        AppError::Internal(anyhow::anyhow!(
                            "Auth middleware missing or guard mismatch"
                        ))
                    })?;

                guard.ensure(&auth)?;
                Ok::<Response, AppError>(next.run(request).await)
            }
        })
    }
}

pub fn ensure_permissions<G, P>(
    auth: &AuthUser<G>,
    mode: PermissionMode,
    required: &[P],
) -> Result<(), AppError>
where
    G: Guard,
    P: ToString,
{
    let required = required
        .iter()
        .map(ToString::to_string)
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();

    if required.is_empty() {
        return Ok(());
    }

    if has_required_permissions(&auth.abilities, &required, mode) {
        return Ok(());
    }

    let joined = required.join(", ");
    let mode_label = mode.as_str();
    Err(AppError::Forbidden(format!(
        "Missing required permission ({mode_label}): {joined}"
    )))
}

pub fn has_required_permissions<G, R>(granted: &[G], required: &[R], mode: PermissionMode) -> bool
where
    G: AsRef<str>,
    R: AsRef<str>,
{
    match mode {
        PermissionMode::Any => required
            .iter()
            .any(|needed| has_permission(granted, needed)),
        PermissionMode::All => required
            .iter()
            .all(|needed| has_permission(granted, needed)),
    }
}

pub fn has_permission<G, R>(granted: &[G], required: R) -> bool
where
    G: AsRef<str>,
    R: AsRef<str>,
{
    has_permission_in_granted(granted, required.as_ref())
}

pub fn permission_matches(granted: &str, required: &str) -> bool {
    permission_matches_in_granted(granted, required)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wildcard_matches_all() {
        assert!(permission_matches("*", "article.manage"));
    }

    #[test]
    fn scoped_wildcard_matches_nested() {
        assert!(permission_matches("article.*", "article.manage"));
        assert!(!permission_matches("article.*", "category.manage"));
    }

    #[test]
    fn reverse_pattern_match_supports_required_wildcard() {
        assert!(permission_matches("article.manage", "article.*"));
    }

    #[test]
    fn manage_implies_read() {
        assert!(permission_matches("article.manage", "article.read"));
    }
}
