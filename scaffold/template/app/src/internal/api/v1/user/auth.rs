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
    extract::validation::transform_validation_errors,
    extract::CleanJson,
    openapi::{
        aide::axum::routing::{get_with, patch_with, post_with},
        ApiRouter,
    },
    response::ApiResponse,
    utils::cookie,
};
use generated::guards::UserGuard;
use std::ops::Deref;
use time::Duration;
use tower_cookies::{cookie::SameSite, Cookie, Cookies};
use validator::Validate;

use crate::{
    contracts::api::v1::user::auth::{
        ResolveReferralOutput, ResolveReferralQuery, UserAuthOutput, UserLocaleUpdateInput,
        UserLocaleUpdateOutput, UserLoginInput, UserLogoutInput, UserLogoutOutput, UserMeOutput,
        UserPasswordUpdateInput, UserPasswordUpdateOutput, UserProfileUpdateInput,
        UserProfileUpdateOutput, UserRefreshInput, UserRegisterInput,
    },
    internal::{api::state::AppApiState, workflows::user_auth as workflow},
};

const REFRESH_COOKIE_PATH: &str = "/api/v1/user/auth";

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
                op.summary("Get current user profile")
                    .tag("User Authentication")
            }),
        )
        .api_route(
            "/logout",
            post_with(logout, |op| {
                op.summary("Logout user").tag("User Authentication")
            }),
        )
        .api_route(
            "/profile_update",
            patch_with(profile_update, |op| {
                op.summary("Update own profile").tag("User Authentication")
            }),
        )
        .api_route(
            "/locale_update",
            patch_with(locale_update, |op| {
                op.summary("Update own locale").tag("User Authentication")
            }),
        )
        .api_route(
            "/password_update",
            patch_with(password_update, |op| {
                op.summary("Update own password")
                    .tag("User Authentication")
            }),
        )
        .layer(from_fn_with_state(
            state.clone(),
            crate::internal::middleware::auth::require_user,
        ));

    ApiRouter::new()
        .api_route(
            "/login",
            post_with(login, |op| {
                op.summary("Login user").tag("User Authentication")
            }),
        )
        .api_route(
            "/register",
            post_with(register, |op| {
                op.summary("Register user").tag("User Authentication")
            }),
        )
        .api_route(
            "/refresh",
            post_with(refresh, |op| {
                op.summary("Refresh user access token")
                    .tag("User Authentication")
            }),
        )
        .api_route(
            "/resolve_referral",
            get_with(resolve_referral, |op| {
                op.summary("Resolve referral code to username")
                    .tag("User Authentication")
            }),
        )
        .merge(protected)
        .with_state(state)
}

async fn resolve_referral(
    State(state): State<AppApiState>,
    axum::extract::Query(query): axum::extract::Query<ResolveReferralQuery>,
) -> Result<ApiResponse<ResolveReferralOutput>, AppError> {
    let code = query.code.unwrap_or_default();
    let code = code.trim();
    if code.is_empty() {
        return Err(AppError::NotFound(t("User not found")));
    }
    match workflow::resolve_referral(&state, code).await? {
        Some((username, name)) => Ok(ApiResponse::success(
            ResolveReferralOutput { username, name },
            &t("User found"),
        )),
        None => Err(AppError::NotFound(t("User not found"))),
    }
}

async fn login(
    State(state): State<AppApiState>,
    cookies: RequestCookies,
    ContractJson(req): ContractJson<UserLoginInput>,
) -> Result<ApiResponse<UserAuthOutput>, AppError> {
    let (_user, tokens) = workflow::login(&state, &req.username, &req.password).await?;
    let output = to_auth_output(&state, &cookies, req.client_type, tokens);
    Ok(ApiResponse::success(output, &t("Login successful")))
}

async fn register(
    State(state): State<AppApiState>,
    cookies: RequestCookies,
    CleanJson(req): CleanJson<UserRegisterInput>,
) -> Result<ApiResponse<UserAuthOutput>, AppError> {
    let req = validate_register_input(req)?;
    let (_user, tokens) = workflow::register(&state, req.clone()).await?;
    let output = to_auth_output(&state, &cookies, req.client_type, tokens);
    Ok(ApiResponse::success(output, &t("Registration successful")))
}

async fn refresh(
    State(state): State<AppApiState>,
    headers: RequestHeaders,
    cookies: RequestCookies,
    ContractJson(req): ContractJson<UserRefreshInput>,
) -> Result<ApiResponse<UserAuthOutput>, AppError> {
    let refresh_token = auth::extract_refresh_token_for_client(
        &headers,
        UserGuard::name(),
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
    _auth: AuthUser<UserGuard>,
    ContractJson(req): ContractJson<UserLogoutInput>,
) -> Result<ApiResponse<UserLogoutOutput>, AppError> {
    let refresh_token = auth::extract_refresh_token_for_client(
        &headers,
        UserGuard::name(),
        req.client_type,
        req.refresh_token.as_deref(),
    )
    .ok_or_else(|| AppError::BadRequest(t("Missing refresh token")))?;

    workflow::revoke_session(&state, &refresh_token).await?;

    if matches!(req.client_type, AuthClientType::Web) {
        cookie::remove_guard_refresh(&cookies, UserGuard::name(), REFRESH_COOKIE_PATH);
    }

    Ok(ApiResponse::success(
        UserLogoutOutput { revoked: true },
        &t("Logout successful"),
    ))
}

async fn me(
    State(state): State<AppApiState>,
    auth: AuthUser<UserGuard>,
) -> Result<ApiResponse<UserMeOutput>, AppError> {
    workflow::fetch_and_check_ban(&state, auth.user.id).await?;
    let user = auth.user;
    Ok(ApiResponse::success(
        UserMeOutput {
            id: user.id.into(),
            uuid: user.uuid,
            username: user.username.clone(),
            name: user.name.unwrap_or_else(|| user.username),
            email: user.email,
            locale: user.locale,
            country_iso2: user.country_iso2,
            contact_number: user.contact_number,
        },
        &t("Profile loaded"),
    ))
}

async fn profile_update(
    State(state): State<AppApiState>,
    auth: AuthUser<UserGuard>,
    CleanJson(req): CleanJson<UserProfileUpdateInput>,
) -> Result<ApiResponse<UserProfileUpdateOutput>, AppError> {
    let req = validate_profile_update_input(req)?;
    let user = workflow::profile_update(&state, auth.user.id, req).await?;
    Ok(ApiResponse::success(
        UserProfileUpdateOutput {
            id: user.id.into(),
            username: user.username.clone(),
            name: user.name.unwrap_or_else(|| user.username),
            email: user.email,
            locale: user.locale,
            country_iso2: user.country_iso2,
            contact_number: user.contact_number,
        },
        &t("Profile updated successfully"),
    ))
}

async fn locale_update(
    State(state): State<AppApiState>,
    auth: AuthUser<UserGuard>,
    ContractJson(req): ContractJson<UserLocaleUpdateInput>,
) -> Result<ApiResponse<UserLocaleUpdateOutput>, AppError> {
    let locale = workflow::locale_update(&state, auth.user.id, req).await?;
    Ok(ApiResponse::success(
        UserLocaleUpdateOutput { locale },
        &t("Locale updated successfully"),
    ))
}

async fn password_update(
    State(state): State<AppApiState>,
    auth: AuthUser<UserGuard>,
    ContractJson(req): ContractJson<UserPasswordUpdateInput>,
) -> Result<ApiResponse<UserPasswordUpdateOutput>, AppError> {
    workflow::password_update(&state, auth.user.id, req).await?;
    Ok(ApiResponse::success(
        UserPasswordUpdateOutput { updated: true },
        &t("Password updated successfully"),
    ))
}

fn validate_register_input(
    req: UserRegisterInput,
) -> Result<UserRegisterInput, AppError> {
    let req = req.normalize();
    if let Err(e) = req.validate() {
        return Err(AppError::Validation {
            message: t("Validation failed"),
            errors: transform_validation_errors(e),
        });
    }
    Ok(req)
}

fn validate_profile_update_input(
    req: UserProfileUpdateInput,
) -> Result<UserProfileUpdateInput, AppError> {
    let req = req.normalize();
    if let Err(e) = req.validate() {
        return Err(AppError::Validation {
            message: t("Validation failed"),
            errors: transform_validation_errors(e),
        });
    }
    Ok(req)
}

fn to_auth_output(
    state: &AppApiState,
    cookies: &Cookies,
    client_type: AuthClientType,
    tokens: core_web::auth::IssuedTokenPair,
) -> UserAuthOutput {
    match client_type {
        AuthClientType::Web => {
            if let Some(ttl) = refresh_cookie_ttl(state) {
                set_guard_refresh_cookie(
                    cookies,
                    UserGuard::name(),
                    &tokens.refresh_token,
                    ttl,
                    REFRESH_COOKIE_PATH,
                );
            }

            UserAuthOutput {
                token_type: "Bearer".to_string(),
                access_token: tokens.access_token,
                access_expires_at: tokens.access_expires_at,
                refresh_token: None,
            }
        }
        AuthClientType::Mobile => UserAuthOutput {
            token_type: "Bearer".to_string(),
            access_token: tokens.access_token,
            access_expires_at: tokens.access_expires_at,
            refresh_token: Some(tokens.refresh_token),
        },
    }
}

fn refresh_cookie_ttl(state: &AppApiState) -> Option<Duration> {
    let days = state.auth.guard(UserGuard::name())?.refresh_ttl_days;
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
