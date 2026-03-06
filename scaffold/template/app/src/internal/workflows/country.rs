use core_db::{
    common::sql::{DbConn, Op, OrderDir},
    generated::models::{
        Country as CountryModel, CountryCol, CountryStatus as GeneratedCountryStatus,
    },
    platform::countries::{
        normalize_country_iso2, normalize_country_status, Country, CountryCurrency,
        COUNTRY_STATUS_ENABLED,
    },
};
use core_i18n::t;
use core_web::error::AppError;

use crate::internal::api::state::AppApiState;

pub const BOOTSTRAP_COUNTRIES_CACHE_KEY: &str = "bootstrap:countries:enabled:v1";
pub const BOOTSTRAP_COUNTRIES_CACHE_TTL_SECS: u64 = 300;

pub async fn update_status(
    state: &AppApiState,
    iso2: &str,
    status: &str,
) -> Result<Country, AppError> {
    let iso2 =
        normalize_country_iso2(iso2).ok_or_else(|| AppError::NotFound(t("Country not found")))?;
    let status = normalize_country_status(status)
        .ok_or_else(|| AppError::BadRequest(t("Invalid country status")))?;
    let status_enum = GeneratedCountryStatus::from_storage(status)
        .ok_or_else(|| AppError::BadRequest(t("Invalid country status")))?;

    let affected = CountryModel::new(DbConn::pool(&state.db), None)
        .update()
        .where_iso2(Op::Eq, iso2.clone())
        .set_status(status_enum)
        .set_updated_at(time::OffsetDateTime::now_utc())
        .save()
        .await
        .map_err(AppError::from)?;

    if affected == 0 {
        return Err(AppError::NotFound(t("Country not found")));
    }

    let updated = CountryModel::new(DbConn::pool(&state.db), None)
        .query()
        .where_iso2(Op::Eq, iso2)
        .first()
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(t("Country not found")))?;

    invalidate_bootstrap_country_cache(state).await?;
    Ok(country_view_to_runtime(updated))
}

pub async fn list_enabled_for_bootstrap(state: &AppApiState) -> Result<Vec<Country>, AppError> {
    let cache = state.redis.clone();
    let db = state.db.clone();

    let countries = cache
        .remember(
            BOOTSTRAP_COUNTRIES_CACHE_KEY,
            BOOTSTRAP_COUNTRIES_CACHE_TTL_SECS,
            move || async move {
                let rows = CountryModel::new(DbConn::pool(&db), None)
                    .query()
                    .where_status(Op::Eq, GeneratedCountryStatus::Enabled)
                    .order_by(CountryCol::Name, OrderDir::Asc)
                    .order_by(CountryCol::Iso2, OrderDir::Asc)
                    .get()
                    .await?;
                Ok(rows
                    .into_iter()
                    .map(country_view_to_runtime)
                    .filter(|country| country.status == COUNTRY_STATUS_ENABLED)
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

fn country_view_to_runtime(view: core_db::generated::models::CountryView) -> Country {
    let currencies =
        serde_json::from_value::<Vec<CountryCurrency>>(view.currencies).unwrap_or_default();

    Country {
        iso2: view.iso2,
        iso3: view.iso3,
        iso_numeric: view.iso_numeric,
        name: view.name,
        official_name: view.official_name,
        capital: view.capital,
        capitals: view.capitals,
        region: view.region,
        subregion: view.subregion,
        currencies,
        primary_currency_code: view.primary_currency_code,
        calling_code: view.calling_code,
        calling_root: view.calling_root,
        calling_suffixes: view.calling_suffixes,
        tlds: view.tlds,
        timezones: view.timezones,
        latitude: view.latitude,
        longitude: view.longitude,
        independent: view.independent,
        status: view.status.as_str().to_string(),
        assignment_status: view.assignment_status,
        un_member: view.un_member,
        flag_emoji: view.flag_emoji,
        created_at: view.created_at,
        updated_at: view.updated_at,
    }
}
