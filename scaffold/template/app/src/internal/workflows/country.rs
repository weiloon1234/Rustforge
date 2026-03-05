use core_db::common::sql::DbConn;
use core_db::platform::countries::{repo::CountryRepo, Country};
use core_i18n::t;
use core_web::error::AppError;
use sqlx::Executor;

use crate::internal::api::state::AppApiState;

pub const BOOTSTRAP_COUNTRIES_CACHE_KEY: &str = "bootstrap:countries:enabled:v1";
pub const BOOTSTRAP_COUNTRIES_CACHE_TTL_SECS: u64 = 300;
const COUNTRY_STATUS_ENABLED: &str = "enabled";
const COUNTRY_STATUS_DISABLED: &str = "disabled";

pub async fn update_status(
    state: &AppApiState,
    iso2: &str,
    status: &str,
) -> Result<Country, AppError> {
    let iso2 = normalize_iso2(iso2).ok_or_else(|| AppError::NotFound(t("Country not found")))?;
    let status = normalize_country_status(status)
        .ok_or_else(|| AppError::BadRequest(t("Invalid country status")))?;

    let affected = state
        .db
        .execute(
            sqlx::query("UPDATE countries SET status = $1, updated_at = NOW() WHERE iso2 = $2")
                .bind(status)
                .bind(&iso2),
        )
        .await
        .map_err(AppError::from)?
        .rows_affected();
    if affected == 0 {
        return Err(AppError::NotFound(t("Country not found")));
    }

    let updated = CountryRepo::new(DbConn::pool(&state.db))
        .find_by_iso2(&iso2)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(t("Country not found")))?;

    invalidate_bootstrap_country_cache(state).await?;
    Ok(updated)
}

pub async fn list_enabled_for_bootstrap(state: &AppApiState) -> Result<Vec<Country>, AppError> {
    let cache = state.redis.clone();
    let db = state.db.clone();

    let countries = cache
        .remember(
            BOOTSTRAP_COUNTRIES_CACHE_KEY,
            BOOTSTRAP_COUNTRIES_CACHE_TTL_SECS,
            move || async move {
                let all = CountryRepo::new(DbConn::pool(&db)).list_all().await?;
                Ok(all
                    .into_iter()
                    .filter(|country| country.status.eq_ignore_ascii_case(COUNTRY_STATUS_ENABLED))
                    .collect::<Vec<_>>())
            },
        )
        .await
        .map_err(AppError::from)?;

    Ok(countries)
}

pub async fn invalidate_bootstrap_country_cache(state: &AppApiState) -> Result<(), AppError> {
    state
        .redis
        .forget(BOOTSTRAP_COUNTRIES_CACHE_KEY)
        .await
        .map_err(AppError::from)
}

fn normalize_country_status(value: &str) -> Option<&'static str> {
    match value.trim().to_ascii_lowercase().as_str() {
        COUNTRY_STATUS_ENABLED => Some(COUNTRY_STATUS_ENABLED),
        COUNTRY_STATUS_DISABLED => Some(COUNTRY_STATUS_DISABLED),
        _ => None,
    }
}

fn normalize_iso2(value: &str) -> Option<String> {
    let normalized = value.trim().to_ascii_uppercase();
    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}
