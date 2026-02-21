use async_trait::async_trait;
use axum::{
    extract::{FromRequestParts, Request, State},
    http::{header, request::Parts, HeaderMap},
    middleware::Next,
    response::Response,
};
use core_db::{
    common::{
        auth::token::{generate_token, hash_token},
        sql::DbConn,
    },
    platform::{
        auth_subject_permissions::repo::AuthSubjectPermissionRepo,
        personal_access_tokens::repo::PatRepo,
    },
};
use std::marker::PhantomData;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use crate::authz::permission_matches;
use crate::error::AppError;

/// Guard trait defines how a guard resolves a user.
#[async_trait]
pub trait Guard: Send + Sync + 'static {
    type User: Clone + Send + Sync + 'static;

    /// Logical guard name (e.g. "web", "partner", "staff_portal").
    fn name() -> &'static str;

    /// Optional token subject type check.
    /// If `Some`, middleware ensures PAT `tokenable_type` matches this value.
    fn tokenable_type() -> Option<&'static str> {
        None
    }

    /// Resolve authenticated user by ID.
    async fn fetch_user<'a>(db: DbConn<'a>, id: &str) -> anyhow::Result<Option<Self::User>>;
}

/// State contract required by framework auth middleware.
pub trait AuthState {
    fn auth_db(&self) -> &sqlx::PgPool;
}

/// Opaque token issuance result.
#[derive(Debug, Clone)]
pub struct IssuedToken {
    pub plain_text: String,
    pub token_id: Uuid,
    pub expires_at: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuthIdentity {
    pub subject_id: String,
    pub guard: String,
    pub abilities: Vec<String>,
    pub token_id: Option<Uuid>,
}

impl AuthIdentity {
    pub fn can(&self, ability: &str) -> bool {
        self.abilities
            .iter()
            .any(|permission| permission_matches(permission, ability))
    }
}

/// Authenticated user extractor/model.
#[derive(Debug)]
pub struct AuthUser<G: Guard> {
    pub user: G::User,
    pub subject_id: Option<String>,
    pub token_id: Option<Uuid>,
    pub token_exp: Option<usize>,
    pub abilities: Vec<String>,
    _guard: PhantomData<G>,
}

impl<G: Guard> Clone for AuthUser<G> {
    fn clone(&self) -> Self {
        Self {
            user: self.user.clone(),
            subject_id: self.subject_id.clone(),
            token_id: self.token_id,
            token_exp: self.token_exp,
            abilities: self.abilities.clone(),
            _guard: PhantomData,
        }
    }
}

impl<G: Guard> AuthUser<G> {
    pub fn new(
        user: G::User,
        subject_id: Option<String>,
        token_id: Option<Uuid>,
        token_exp: Option<usize>,
        abilities: Vec<String>,
    ) -> Self {
        Self {
            user,
            subject_id,
            token_id,
            token_exp,
            abilities,
            _guard: PhantomData,
        }
    }

    /// Ability check helper (`*` means wildcard).
    pub fn can(&self, ability: &str) -> bool {
        self.abilities
            .iter()
            .any(|permission| permission_matches(permission, ability))
    }

    pub fn has_permission(&self, permission: impl AsRef<str>) -> bool {
        self.can(permission.as_ref())
    }

    pub fn has_any_permission<T>(&self, permissions: &[T]) -> bool
    where
        T: AsRef<str>,
    {
        permissions
            .iter()
            .any(|permission| self.has_permission(permission.as_ref()))
    }

    pub fn has_all_permissions<T>(&self, permissions: &[T]) -> bool
    where
        T: AsRef<str>,
    {
        permissions
            .iter()
            .all(|permission| self.has_permission(permission.as_ref()))
    }

    pub fn as_identity(&self) -> Option<AuthIdentity> {
        Some(AuthIdentity {
            subject_id: self.subject_id.clone()?,
            guard: G::name().to_string(),
            abilities: self.abilities.clone(),
            token_id: self.token_id,
        })
    }
}

impl<S, G> FromRequestParts<S> for AuthUser<G>
where
    S: Send + Sync,
    G: Guard,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AuthUser<G>>()
            .cloned()
            .ok_or(AppError::Internal(anyhow::anyhow!(
                "Auth middleware missing or guard mismatch"
            )))
    }
}

pub fn guard_config<'a>(
    auth: &'a core_config::AuthSettings,
    guard_name: &str,
) -> Result<&'a core_config::GuardConfig, AppError> {
    auth.guards
        .get(guard_name)
        .ok_or_else(|| AppError::Unauthorized(format!("Guard '{guard_name}' is not configured")))
}

/// Issue PAT token for a guard using guard TTL settings.
pub async fn issue_guard_token<G: Guard>(
    db: &sqlx::PgPool,
    auth: &core_config::AuthSettings,
    user_id: impl ToString,
    name: &str,
    abilities: Option<Vec<String>>,
) -> anyhow::Result<IssuedToken> {
    let cfg = guard_config(auth, G::name())
        .map_err(|e| anyhow::anyhow!("Invalid guard config: {:?}", e))?;

    let ttl_min =
        i64::try_from(cfg.ttl_min).map_err(|_| anyhow::anyhow!("ttl_min is too large"))?;
    let expires_at = Some(OffsetDateTime::now_utc() + Duration::minutes(ttl_min));

    let plain = generate_token();
    let token_hash = hash_token(&plain);
    let tokenable_type = G::tokenable_type().unwrap_or_else(G::name);
    let tokenable_id = user_id.to_string();

    let repo = PatRepo::new(DbConn::pool(db));
    let row = repo
        .create(
            tokenable_type,
            &tokenable_id,
            name,
            &token_hash,
            abilities,
            expires_at,
        )
        .await?;

    Ok(IssuedToken {
        plain_text: plain,
        token_id: row.id,
        expires_at: row.expires_at,
    })
}

pub async fn revoke_token(db: &sqlx::PgPool, token_id: Uuid) -> anyhow::Result<()> {
    PatRepo::new(DbConn::pool(db)).delete_by_id(token_id).await
}

pub fn ensure_ability<G: Guard>(auth: &AuthUser<G>, ability: &str) -> Result<(), AppError> {
    if auth.can(ability) {
        Ok(())
    } else {
        Err(AppError::Forbidden(format!(
            "Missing required ability '{ability}'"
        )))
    }
}

pub async fn authenticate_token<G: Guard>(
    db: &sqlx::PgPool,
    plain_token: &str,
) -> Result<AuthUser<G>, AppError> {
    let token_hash = hash_token(plain_token);
    let repo = PatRepo::new(DbConn::pool(db));

    let pat = repo
        .find_by_token(&token_hash)
        .await?
        .ok_or_else(|| AppError::Unauthorized("Invalid access token".to_string()))?;

    if let Some(exp) = pat.expires_at {
        if exp <= OffsetDateTime::now_utc() {
            return Err(AppError::Unauthorized(
                "Access token has expired".to_string(),
            ));
        }
    }

    if let Some(expected) = G::tokenable_type() {
        if !pat.tokenable_type.eq_ignore_ascii_case(expected) {
            return Err(AppError::Unauthorized("Token guard mismatch".to_string()));
        }
    }

    let user = G::fetch_user(DbConn::pool(db), &pat.tokenable_id)
        .await?
        .ok_or_else(|| AppError::Unauthorized("Token subject not found".to_string()))?;

    if let Err(e) = repo.update_last_used(pat.id).await {
        tracing::warn!("Failed to update token last_used_at: {}", e);
    }

    let subject_repo = AuthSubjectPermissionRepo::new(DbConn::pool(db));
    let mut abilities = subject_repo
        .list_permission_strings(G::name(), &pat.tokenable_id)
        .await?;

    // Backward-compat bridge: PAT abilities are only used when subject permission rows are absent.
    if abilities.is_empty() {
        abilities = pat.abilities.map(|a| a.0).unwrap_or_default();
    }

    Ok(AuthUser::new(
        user,
        Some(pat.tokenable_id),
        Some(pat.id),
        None,
        abilities,
    ))
}

/// Framework auth middleware.
/// Reads Bearer token first, then `access_token` cookie fallback.
pub async fn require_auth<G, S>(
    State(state): State<S>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError>
where
    G: Guard,
    S: AuthState + Send + Sync,
{
    let token = extract_request_token(request.headers())
        .ok_or_else(|| AppError::Unauthorized("Missing access token".to_string()))?;

    let auth_user = authenticate_token::<G>(state.auth_db(), &token).await?;
    request.extensions_mut().insert(auth_user);

    Ok(next.run(request).await)
}

fn extract_request_token(headers: &HeaderMap) -> Option<String> {
    extract_bearer_token(headers).or_else(|| extract_cookie_token(headers, "access_token"))
}

fn extract_bearer_token(headers: &HeaderMap) -> Option<String> {
    let raw = headers.get(header::AUTHORIZATION)?.to_str().ok()?;
    let (scheme, token) = raw.trim().split_once(' ')?;
    if scheme.eq_ignore_ascii_case("bearer") && !token.trim().is_empty() {
        Some(token.trim().to_string())
    } else {
        None
    }
}

fn extract_cookie_token(headers: &HeaderMap, cookie_name: &str) -> Option<String> {
    let raw = headers.get(header::COOKIE)?.to_str().ok()?;
    for part in raw.split(';') {
        let trimmed = part.trim();
        if let Some((name, val)) = trimmed.split_once('=') {
            if name.trim() == cookie_name && !val.trim().is_empty() {
                return Some(val.trim().to_string());
            }
        }
    }
    None
}
