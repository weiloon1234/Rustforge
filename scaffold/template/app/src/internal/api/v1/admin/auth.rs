use axum::{
    extract::{FromRequestParts, State},
    http::request::Parts,
    middleware::from_fn_with_state,
};
use core_i18n::t;
use core_web::{
    auth::{self, AuthClientType, AuthUser, Guard},
    contracts::ContractJson,
    error::AppError,
    extract::request_headers::RequestHeaders,
    openapi::{
        aide::axum::routing::{get_with, patch_with, post_with},
        ApiRouter,
    },
    response::ApiResponse,
    utils::cookie,
};
use generated::guards::AdminGuard;
use std::ops::Deref;
use time::Duration;
use tower_cookies::{cookie::SameSite, Cookie, Cookies};

use crate::{
    contracts::api::v1::admin::auth::{
        AdminAuthOutput, AdminLocaleUpdateInput, AdminLocaleUpdateOutput, AdminLoginInput,
        AdminLogoutInput, AdminLogoutOutput, AdminMeOutput, AdminPasswordUpdateInput,
        AdminPasswordUpdateOutput, AdminProfileUpdateInput, AdminProfileUpdateOutput,
        AdminRefreshInput,
    },
    internal::{api::state::AppApiState, workflows::admin_auth as workflow},
};

const REFRESH_COOKIE_PATH: &str = "/api/v1/admin/auth";

#[derive(Debug, Clone)]
struct RequestCookies(Cookies);

impl Deref for RequestCookies {
    type Target = Cookies;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<S> FromRequestParts<S> for RequestCookies
where
    S: Send + Sync,
{
    type Rejection = <Cookies as FromRequestParts<S>>::Rejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let cookies = Cookies::from_request_parts(parts, state).await?;
        Ok(Self(cookies))
    }
}

impl core_web::openapi::aide::OperationInput for RequestCookies {}

pub fn router(state: AppApiState) -> ApiRouter {
    let protected = ApiRouter::new()
        .api_route(
            "/me",
            get_with(me, |op| {
                op.summary("Get current admin profile")
                    .tag("Admin Authentication")
            }),
        )
        .api_route(
            "/logout",
            post_with(logout, |op| {
                op.summary("Logout admin").tag("Admin Authentication")
            }),
        )
        .api_route(
            "/profile_update",
            patch_with(profile_update, |op| {
                op.summary("Update own profile").tag("Admin Authentication")
            }),
        )
        .api_route(
            "/locale_update",
            patch_with(locale_update, |op| {
                op.summary("Update own locale").tag("Admin Authentication")
            }),
        )
        .api_route(
            "/password_update",
            patch_with(password_update, |op| {
                op.summary("Update own password")
                    .tag("Admin Authentication")
            }),
        )
        .layer(from_fn_with_state(
            state.clone(),
            crate::internal::middleware::auth::require_admin,
        ));

    ApiRouter::new()
        .api_route(
            "/login",
            post_with(login, |op| {
                op.summary("Login admin").tag("Admin Authentication")
            }),
        )
        .api_route(
            "/refresh",
            post_with(refresh, |op| {
                op.summary("Refresh admin access token")
                    .tag("Admin Authentication")
            }),
        )
        .merge(protected)
        .with_state(state)
}

async fn login(
    State(state): State<AppApiState>,
    cookies: RequestCookies,
    ContractJson(req): ContractJson<AdminLoginInput>,
) -> Result<ApiResponse<AdminAuthOutput>, AppError> {
    let (_admin, tokens) = workflow::login(&state, &req.username, &req.password).await?;
    let output = to_auth_output(&state, &cookies, req.client_type, tokens);
    Ok(ApiResponse::success(output, &t("Login successful")))
}

async fn refresh(
    State(state): State<AppApiState>,
    headers: RequestHeaders,
    cookies: RequestCookies,
    ContractJson(req): ContractJson<AdminRefreshInput>,
) -> Result<ApiResponse<AdminAuthOutput>, AppError> {
    let refresh_token = auth::extract_refresh_token_for_client(
        &headers,
        AdminGuard::name(),
        req.client_type,
        req.refresh_token.as_deref(),
    )
    .ok_or_else(|| AppError::BadRequest(t("Missing refresh token")))?;

    let tokens = workflow::refresh(&state, &refresh_token).await?;
    let output = to_auth_output(&state, &cookies, req.client_type, tokens);
    Ok(ApiResponse::success(output, &t("Token refreshed")))
}

async fn logout(
    State(state): State<AppApiState>,
    headers: RequestHeaders,
    cookies: RequestCookies,
    _auth: AuthUser<AdminGuard>,
    ContractJson(req): ContractJson<AdminLogoutInput>,
) -> Result<ApiResponse<AdminLogoutOutput>, AppError> {
    let refresh_token = auth::extract_refresh_token_for_client(
        &headers,
        AdminGuard::name(),
        req.client_type,
        req.refresh_token.as_deref(),
    )
    .ok_or_else(|| AppError::BadRequest(t("Missing refresh token")))?;

    workflow::revoke_session(&state, &refresh_token).await?;

    if matches!(req.client_type, AuthClientType::Web) {
        cookie::remove_guard_refresh(&cookies, AdminGuard::name(), REFRESH_COOKIE_PATH);
    }

    Ok(ApiResponse::success(
        AdminLogoutOutput { revoked: true },
        &t("Logout successful"),
    ))
}

async fn me(auth: AuthUser<AdminGuard>) -> Result<ApiResponse<AdminMeOutput>, AppError> {
    let user = auth.user;
    let identity = user.identity();
    Ok(ApiResponse::success(
        AdminMeOutput {
            id: user.id.into(),
            identity,
            username: user.username,
            email: user.email,
            locale: user.locale,
            name: user.name,
            admin_type: user.admin_type,
            scopes: auth.abilities,
        },
        &t("Profile loaded"),
    ))
}

async fn profile_update(
    State(state): State<AppApiState>,
    auth: AuthUser<AdminGuard>,
    ContractJson(req): ContractJson<AdminProfileUpdateInput>,
) -> Result<ApiResponse<AdminProfileUpdateOutput>, AppError> {
    let admin = workflow::profile_update(&state, auth.user.id, req).await?;
    let identity = admin.identity();
    Ok(ApiResponse::success(
        AdminProfileUpdateOutput {
            id: admin.id.into(),
            identity,
            username: admin.username,
            email: admin.email,
            locale: admin.locale,
            name: admin.name,
            admin_type: admin.admin_type,
        },
        &t("Profile updated successfully"),
    ))
}

async fn locale_update(
    State(state): State<AppApiState>,
    auth: AuthUser<AdminGuard>,
    ContractJson(req): ContractJson<AdminLocaleUpdateInput>,
) -> Result<ApiResponse<AdminLocaleUpdateOutput>, AppError> {
    let locale = workflow::locale_update(&state, auth.user.id, req).await?;
    Ok(ApiResponse::success(
        AdminLocaleUpdateOutput { locale },
        &t("Locale updated successfully"),
    ))
}

async fn password_update(
    State(state): State<AppApiState>,
    auth: AuthUser<AdminGuard>,
    ContractJson(req): ContractJson<AdminPasswordUpdateInput>,
) -> Result<ApiResponse<AdminPasswordUpdateOutput>, AppError> {
    workflow::password_update(&state, auth.user.id, req).await?;
    Ok(ApiResponse::success(
        AdminPasswordUpdateOutput { updated: true },
        &t("Password updated successfully"),
    ))
}

fn to_auth_output(
    state: &AppApiState,
    cookies: &Cookies,
    client_type: AuthClientType,
    tokens: core_web::auth::IssuedTokenPair,
) -> AdminAuthOutput {
    match client_type {
        AuthClientType::Web => {
            if let Some(ttl) = refresh_cookie_ttl(state) {
                set_guard_refresh_cookie(
                    cookies,
                    AdminGuard::name(),
                    &tokens.refresh_token,
                    ttl,
                    REFRESH_COOKIE_PATH,
                );
            }

            AdminAuthOutput {
                token_type: "Bearer".to_string(),
                access_token: tokens.access_token,
                access_expires_at: tokens.access_expires_at,
                refresh_token: None,
                scopes: tokens.abilities,
            }
        }
        AuthClientType::Mobile => AdminAuthOutput {
            token_type: "Bearer".to_string(),
            access_token: tokens.access_token,
            access_expires_at: tokens.access_expires_at,
            refresh_token: Some(tokens.refresh_token),
            scopes: tokens.abilities,
        },
    }
}

fn refresh_cookie_ttl(state: &AppApiState) -> Option<Duration> {
    let days = state.auth.guard(AdminGuard::name())?.refresh_ttl_days;
    let days = i64::try_from(days).ok()?;
    Some(Duration::days(days))
}

fn bool_from_env(key: &str) -> Option<bool> {
    let raw = std::env::var(key).ok()?;
    match raw.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

fn should_use_secure_cookie() -> bool {
    if let Some(value) = bool_from_env("COOKIE_SECURE") {
        return value;
    }

    let app_env = std::env::var("APP_ENV").unwrap_or_else(|_| "production".to_string());
    !matches!(
        app_env.trim().to_ascii_lowercase().as_str(),
        "local" | "development" | "dev" | "test" | "testing"
    )
}

fn set_guard_refresh_cookie(
    cookies: &Cookies,
    guard: &str,
    refresh_token: &str,
    ttl: Duration,
    path: &str,
) {
    let mut refresh_cookie = Cookie::new(
        cookie::guard_refresh_cookie_name(guard),
        refresh_token.to_string(),
    );
    refresh_cookie.set_http_only(true);
    refresh_cookie.set_secure(should_use_secure_cookie());
    refresh_cookie.set_same_site(SameSite::Lax);
    refresh_cookie.set_path(path.to_string());
    refresh_cookie.set_max_age(tower_cookies::cookie::time::Duration::seconds(
        ttl.whole_seconds(),
    ));
    cookies.add(refresh_cookie);
}
