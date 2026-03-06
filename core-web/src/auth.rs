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
        sql::{DbConn, Op},
    },
    generated::models::{PersonalAccessToken, PersonalAccessTokenKind, PersonalAccessTokenView},
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

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum AuthClientType {
    Web,
    Mobile,
}

impl ts_rs::TS for AuthClientType {
    type WithoutGenerics = Self;

    fn name() -> String {
        "AuthClientType".to_string()
    }

    fn inline() -> String {
        Self::name()
    }

    fn inline_flattened() -> String {
        panic!("AuthClientType cannot be flattened")
    }

    fn decl() -> String {
        "type AuthClientType = \"web\" | \"mobile\";".to_string()
    }

    fn decl_concrete() -> String {
        Self::decl()
    }
}

#[derive(Debug, Clone)]
pub enum TokenScopeGrant {
    /// Authenticated identity only, no permission scopes.
    AuthOnly,
    /// Explicit scopes snapshot.
    Explicit(Vec<String>),
    /// Full wildcard scope (`*`).
    Wildcard,
}

impl TokenScopeGrant {
    pub fn to_abilities(self) -> Vec<String> {
        match self {
            Self::AuthOnly => Vec::new(),
            Self::Explicit(values) => normalize_abilities(values),
            Self::Wildcard => vec!["*".to_string()],
        }
    }
}

/// Opaque token issuance result.
#[derive(Debug, Clone)]
pub struct IssuedTokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub access_token_id: Uuid,
    pub refresh_token_id: Uuid,
    pub family_id: Uuid,
    pub access_expires_at: Option<OffsetDateTime>,
    pub refresh_expires_at: Option<OffsetDateTime>,
    pub abilities: Vec<String>,
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

impl<G> aide::OperationInput for AuthUser<G> where G: Guard {}

pub fn guard_config<'a>(
    auth: &'a core_config::AuthSettings,
    guard_name: &str,
) -> Result<&'a core_config::GuardConfig, AppError> {
    auth.guards
        .get(guard_name)
        .ok_or_else(|| AppError::Unauthorized(format!("Guard '{guard_name}' is not configured")))
}

fn normalize_abilities(values: Vec<String>) -> Vec<String> {
    let mut normalized = values
        .into_iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    normalized.sort();
    normalized.dedup();
    normalized
}

fn abilities_to_json(abilities: Option<Vec<String>>) -> anyhow::Result<Option<serde_json::Value>> {
    match abilities {
        Some(values) => Ok(Some(serde_json::to_value(values)?)),
        None => Ok(None),
    }
}

fn abilities_from_json(value: &Option<serde_json::Value>) -> Vec<String> {
    value
        .as_ref()
        .and_then(|raw| serde_json::from_value::<Vec<String>>(raw.clone()).ok())
        .unwrap_or_default()
}

fn token_row_is_revoked(row: &PersonalAccessTokenView) -> bool {
    row.revoked_at.is_some()
}

fn token_row_is_expired(row: &PersonalAccessTokenView, now: OffsetDateTime) -> bool {
    row.expires_at.is_some_and(|exp| exp <= now)
}

async fn issue_token_row(
    db: &sqlx::PgPool,
    tokenable_type: &str,
    tokenable_id: &str,
    name: &str,
    token_kind: PersonalAccessTokenKind,
    family_id: Uuid,
    parent_token_id: Option<Uuid>,
    abilities: Option<Vec<String>>,
    expires_at: Option<OffsetDateTime>,
) -> anyhow::Result<(String, PersonalAccessTokenView)> {
    let plain = generate_token();
    let token_hash = hash_token(&plain);
    let abilities = abilities_to_json(abilities)?;

    let row = PersonalAccessToken::new(DbConn::pool(db), None)
        .insert()
        .set_tokenable_type(tokenable_type.to_string())
        .set_tokenable_id(tokenable_id.to_string())
        .set_name(name.to_string())
        .set_token(token_hash)
        .set_token_kind(token_kind)
        .set_family_id(family_id)
        .set_parent_token_id(parent_token_id)
        .set_abilities(abilities)
        .set_expires_at(expires_at)
        .save()
        .await?;

    Ok((plain, row))
}

/// Issue an OAuth2-style access+refresh pair in one session family.
pub async fn issue_guard_session<G: Guard>(
    db: &sqlx::PgPool,
    auth: &core_config::AuthSettings,
    user_id: impl ToString,
    name: &str,
    scope_grant: TokenScopeGrant,
) -> anyhow::Result<IssuedTokenPair> {
    let cfg = guard_config(auth, G::name())
        .map_err(|e| anyhow::anyhow!("Invalid guard config: {:?}", e))?;

    let access_ttl_min =
        i64::try_from(cfg.ttl_min).map_err(|_| anyhow::anyhow!("ttl_min is too large"))?;
    let refresh_ttl_days = i64::try_from(cfg.refresh_ttl_days)
        .map_err(|_| anyhow::anyhow!("refresh_ttl_days is too large"))?;

    let access_expires_at = Some(OffsetDateTime::now_utc() + Duration::minutes(access_ttl_min));
    let refresh_expires_at = Some(OffsetDateTime::now_utc() + Duration::days(refresh_ttl_days));

    let tokenable_type = G::tokenable_type().unwrap_or_else(G::name);
    let tokenable_id = user_id.to_string();
    let abilities = scope_grant.to_abilities();
    let family_id = Uuid::new_v4();

    let (access_token, access_row) = issue_token_row(
        db,
        tokenable_type,
        &tokenable_id,
        name,
        PersonalAccessTokenKind::Access,
        family_id,
        None,
        Some(abilities.clone()),
        access_expires_at,
    )
    .await?;

    let (refresh_token, refresh_row) = issue_token_row(
        db,
        tokenable_type,
        &tokenable_id,
        name,
        PersonalAccessTokenKind::Refresh,
        family_id,
        None,
        Some(abilities.clone()),
        refresh_expires_at,
    )
    .await?;

    Ok(IssuedTokenPair {
        access_token,
        refresh_token,
        access_token_id: access_row.id,
        refresh_token_id: refresh_row.id,
        family_id,
        access_expires_at: access_row.expires_at,
        refresh_expires_at: refresh_row.expires_at,
        abilities,
    })
}

fn assert_token_row_valid<G: Guard>(
    pat: &PersonalAccessTokenView,
    expected_kind: PersonalAccessTokenKind,
) -> Result<(), AppError> {
    if pat.token_kind != expected_kind {
        return Err(AppError::Unauthorized("Invalid token kind".to_string()));
    }

    if token_row_is_revoked(pat) {
        return Err(AppError::Unauthorized("Token has been revoked".to_string()));
    }

    if token_row_is_expired(pat, OffsetDateTime::now_utc()) {
        return Err(AppError::Unauthorized("Token has expired".to_string()));
    }

    if let Some(expected) = G::tokenable_type() {
        if !pat.tokenable_type.eq_ignore_ascii_case(expected) {
            return Err(AppError::Unauthorized("Token guard mismatch".to_string()));
        }
    }

    Ok(())
}

/// Rotate refresh token (one-time-use) and issue new access+refresh pair.
pub async fn refresh_guard_session<G: Guard>(
    db: &sqlx::PgPool,
    auth: &core_config::AuthSettings,
    refresh_plain_token: &str,
    name: &str,
) -> Result<IssuedTokenPair, AppError> {
    let token_hash = hash_token(refresh_plain_token);
    let refresh_row = PersonalAccessToken::new(DbConn::pool(db), None)
        .query()
        .where_token(Op::Eq, token_hash)
        .where_token_kind(Op::Eq, PersonalAccessTokenKind::Refresh)
        .first()
        .await?
        .ok_or_else(|| AppError::Unauthorized("Invalid refresh token".to_string()))?;

    if token_row_is_revoked(&refresh_row) {
        let _ = PersonalAccessToken::new(DbConn::pool(db), None)
            .update()
            .where_family_id(Op::Eq, refresh_row.family_id)
            .set_revoked_at(Some(OffsetDateTime::now_utc()))
            .save()
            .await;
        return Err(AppError::Unauthorized(
            "Refresh token has already been used".to_string(),
        ));
    }

    assert_token_row_valid::<G>(&refresh_row, PersonalAccessTokenKind::Refresh)?;

    let _ = G::fetch_user(DbConn::pool(db), &refresh_row.tokenable_id)
        .await?
        .ok_or_else(|| AppError::Unauthorized("Token subject not found".to_string()))?;

    PersonalAccessToken::new(DbConn::pool(db), None)
        .update()
        .where_id(Op::Eq, refresh_row.id)
        .set_revoked_at(Some(OffsetDateTime::now_utc()))
        .save()
        .await
        .map_err(AppError::from)?;

    let cfg = guard_config(auth, G::name())?;
    let access_ttl_min = i64::try_from(cfg.ttl_min)
        .map_err(|_| AppError::Internal(anyhow::anyhow!("ttl_min is too large")))?;
    let refresh_ttl_days = i64::try_from(cfg.refresh_ttl_days)
        .map_err(|_| AppError::Internal(anyhow::anyhow!("refresh_ttl_days is too large")))?;

    let access_expires_at = Some(OffsetDateTime::now_utc() + Duration::minutes(access_ttl_min));
    let refresh_expires_at = Some(OffsetDateTime::now_utc() + Duration::days(refresh_ttl_days));

    let abilities = abilities_from_json(&refresh_row.abilities);
    let family_id = refresh_row.family_id;
    let tokenable_type = refresh_row.tokenable_type.clone();
    let tokenable_id = refresh_row.tokenable_id.clone();

    let (access_token, access_row) = issue_token_row(
        db,
        &tokenable_type,
        &tokenable_id,
        name,
        PersonalAccessTokenKind::Access,
        family_id,
        Some(refresh_row.id),
        Some(abilities.clone()),
        access_expires_at,
    )
    .await
    .map_err(AppError::from)?;

    let (refresh_token, new_refresh_row) = issue_token_row(
        db,
        &tokenable_type,
        &tokenable_id,
        name,
        PersonalAccessTokenKind::Refresh,
        family_id,
        Some(refresh_row.id),
        Some(abilities.clone()),
        refresh_expires_at,
    )
    .await
    .map_err(AppError::from)?;

    Ok(IssuedTokenPair {
        access_token,
        refresh_token,
        access_token_id: access_row.id,
        refresh_token_id: new_refresh_row.id,
        family_id,
        access_expires_at: access_row.expires_at,
        refresh_expires_at: new_refresh_row.expires_at,
        abilities,
    })
}

pub async fn revoke_token(db: &sqlx::PgPool, token_id: Uuid) -> anyhow::Result<()> {
    PersonalAccessToken::new(DbConn::pool(db), None)
        .update()
        .where_id(Op::Eq, token_id)
        .set_revoked_at(Some(OffsetDateTime::now_utc()))
        .save()
        .await?;
    Ok(())
}

pub async fn revoke_session_by_refresh_token<G: Guard>(
    db: &sqlx::PgPool,
    refresh_plain_token: &str,
) -> Result<(), AppError> {
    let token_hash = hash_token(refresh_plain_token);
    let refresh_row = PersonalAccessToken::new(DbConn::pool(db), None)
        .query()
        .where_token(Op::Eq, token_hash)
        .where_token_kind(Op::Eq, PersonalAccessTokenKind::Refresh)
        .first()
        .await?
        .ok_or_else(|| AppError::Unauthorized("Invalid refresh token".to_string()))?;

    if let Some(expected) = G::tokenable_type() {
        if !refresh_row.tokenable_type.eq_ignore_ascii_case(expected) {
            return Err(AppError::Unauthorized("Token guard mismatch".to_string()));
        }
    }

    PersonalAccessToken::new(DbConn::pool(db), None)
        .update()
        .where_family_id(Op::Eq, refresh_row.family_id)
        .set_revoked_at(Some(OffsetDateTime::now_utc()))
        .save()
        .await
        .map_err(AppError::from)?;
    Ok(())
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
    let pat = PersonalAccessToken::new(DbConn::pool(db), None)
        .query()
        .where_token(Op::Eq, token_hash)
        .where_token_kind(Op::Eq, PersonalAccessTokenKind::Access)
        .first()
        .await?
        .ok_or_else(|| AppError::Unauthorized("Invalid access token".to_string()))?;

    assert_token_row_valid::<G>(&pat, PersonalAccessTokenKind::Access)?;

    let user = G::fetch_user(DbConn::pool(db), &pat.tokenable_id)
        .await?
        .ok_or_else(|| AppError::Unauthorized("Token subject not found".to_string()))?;

    if let Err(e) = PersonalAccessToken::new(DbConn::pool(db), None)
        .update()
        .where_id(Op::Eq, pat.id)
        .set_last_used_at(Some(OffsetDateTime::now_utc()))
        .save()
        .await
    {
        tracing::warn!("Failed to update token last_used_at: {}", e);
    }

    let abilities = abilities_from_json(&pat.abilities);

    Ok(AuthUser::new(
        user,
        Some(pat.tokenable_id),
        Some(pat.id),
        None,
        abilities,
    ))
}

/// Framework auth middleware.
/// Reads Authorization Bearer access token only.
pub async fn require_auth<G, S>(
    State(state): State<S>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError>
where
    G: Guard,
    S: AuthState + Send + Sync,
{
    let token = extract_bearer_token(request.headers())
        .ok_or_else(|| AppError::Unauthorized("Missing access token".to_string()))?;

    let auth_user = authenticate_token::<G>(state.auth_db(), &token).await?;
    request.extensions_mut().insert(auth_user);

    Ok(next.run(request).await)
}

pub fn extract_bearer_token(headers: &HeaderMap) -> Option<String> {
    let raw = headers.get(header::AUTHORIZATION)?.to_str().ok()?;
    let (scheme, token) = raw.trim().split_once(' ')?;
    if scheme.eq_ignore_ascii_case("bearer") && !token.trim().is_empty() {
        Some(token.trim().to_string())
    } else {
        None
    }
}

pub fn extract_refresh_cookie(headers: &HeaderMap, guard: &str) -> Option<String> {
    let cookie_name = crate::utils::cookie::guard_refresh_cookie_name(guard);
    extract_cookie_token(headers, &cookie_name)
}

pub fn extract_refresh_token_for_client(
    headers: &HeaderMap,
    guard: &str,
    client_type: AuthClientType,
    mobile_refresh_token: Option<&str>,
) -> Option<String> {
    match client_type {
        AuthClientType::Web => extract_refresh_cookie(headers, guard),
        AuthClientType::Mobile => mobile_refresh_token
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string),
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
