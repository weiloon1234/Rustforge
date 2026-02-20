#![allow(dead_code)]

use anyhow::{Context, Result};

use crate::common::sql::DbConn;
use crate::platform::countries::model::CountryRow;
use crate::platform::countries::types::{Country, CountryCurrency, CountrySeed};

const BUILTIN_COUNTRIES_JSON: &str = include_str!("seed/countries.seed.json");
const COUNTRY_STATUS_ENABLED: &str = "enabled";
const COUNTRY_STATUS_DISABLED: &str = "disabled";

pub struct CountryRepo<'a> {
    db: DbConn<'a>,
}

impl<'a> CountryRepo<'a> {
    pub fn new(db: DbConn<'a>) -> Self {
        Self { db }
    }

    pub fn load_builtin_seed() -> Result<Vec<CountrySeed>> {
        serde_json::from_str(BUILTIN_COUNTRIES_JSON)
            .context("failed to parse built-in countries seed")
    }

    pub async fn seed_builtin(&self) -> Result<usize> {
        let countries = Self::load_builtin_seed()?;
        self.upsert_many(&countries).await
    }

    pub async fn upsert_many(&self, countries: &[CountrySeed]) -> Result<usize> {
        if countries.is_empty() {
            return Ok(0);
        }

        for seed in countries {
            let seed = normalize_seed(seed.clone());
            let status = default_status_for_iso2(&seed.iso2);
            let currencies = serde_json::to_value(&seed.currencies)?;

            let q = sqlx::query(
                r#"
                INSERT INTO countries (
                    iso2,
                    iso3,
                    iso_numeric,
                    name,
                    official_name,
                    capital,
                    capitals,
                    region,
                    subregion,
                    currencies,
                    primary_currency_code,
                    calling_code,
                    calling_root,
                    calling_suffixes,
                    tlds,
                    timezones,
                    latitude,
                    longitude,
                    independent,
                    status,
                    assignment_status,
                    un_member,
                    flag_emoji
                )
                VALUES (
                    $1, $2, $3, $4, $5, $6, $7, $8, $9,
                    $10, $11, $12, $13, $14, $15, $16,
                    $17, $18, $19, $20, $21, $22, $23
                )
                ON CONFLICT (iso2) DO UPDATE
                SET
                    iso3 = EXCLUDED.iso3,
                    iso_numeric = EXCLUDED.iso_numeric,
                    name = EXCLUDED.name,
                    official_name = EXCLUDED.official_name,
                    capital = EXCLUDED.capital,
                    capitals = EXCLUDED.capitals,
                    region = EXCLUDED.region,
                    subregion = EXCLUDED.subregion,
                    currencies = EXCLUDED.currencies,
                    primary_currency_code = EXCLUDED.primary_currency_code,
                    calling_code = EXCLUDED.calling_code,
                    calling_root = EXCLUDED.calling_root,
                    calling_suffixes = EXCLUDED.calling_suffixes,
                    tlds = EXCLUDED.tlds,
                    timezones = EXCLUDED.timezones,
                    latitude = EXCLUDED.latitude,
                    longitude = EXCLUDED.longitude,
                    independent = EXCLUDED.independent,
                    status = EXCLUDED.status,
                    assignment_status = EXCLUDED.assignment_status,
                    un_member = EXCLUDED.un_member,
                    flag_emoji = EXCLUDED.flag_emoji,
                    updated_at = NOW()
                "#,
            )
            .bind(seed.iso2)
            .bind(seed.iso3)
            .bind(seed.iso_numeric)
            .bind(seed.name)
            .bind(seed.official_name)
            .bind(seed.capital)
            .bind(seed.capitals)
            .bind(seed.region)
            .bind(seed.subregion)
            .bind(currencies)
            .bind(seed.primary_currency_code)
            .bind(seed.calling_code)
            .bind(seed.calling_root)
            .bind(seed.calling_suffixes)
            .bind(seed.tlds)
            .bind(seed.timezones)
            .bind(seed.latitude)
            .bind(seed.longitude)
            .bind(seed.independent)
            .bind(status)
            .bind(seed.assignment_status)
            .bind(seed.un_member)
            .bind(seed.flag_emoji);

            self.db.execute(q).await?;
        }

        Ok(countries.len())
    }

    pub async fn list_all(&self) -> Result<Vec<Country>> {
        let q = sqlx::query_as::<_, CountryRow>(
            r#"
            SELECT
                iso2,
                iso3,
                iso_numeric,
                name,
                official_name,
                capital,
                capitals,
                region,
                subregion,
                currencies,
                primary_currency_code,
                calling_code,
                calling_root,
                calling_suffixes,
                tlds,
                timezones,
                latitude,
                longitude,
                independent,
                status,
                assignment_status,
                un_member,
                flag_emoji,
                created_at,
                updated_at
            FROM countries
            ORDER BY name ASC
            "#,
        );
        let rows = self.db.fetch_all(q).await?;
        rows.into_iter().map(row_to_country).collect()
    }

    pub async fn find_by_iso2(&self, iso2: &str) -> Result<Option<Country>> {
        let iso2 = iso2.trim().to_ascii_uppercase();
        let q = sqlx::query_as::<_, CountryRow>(
            r#"
            SELECT
                iso2,
                iso3,
                iso_numeric,
                name,
                official_name,
                capital,
                capitals,
                region,
                subregion,
                currencies,
                primary_currency_code,
                calling_code,
                calling_root,
                calling_suffixes,
                tlds,
                timezones,
                latitude,
                longitude,
                independent,
                status,
                assignment_status,
                un_member,
                flag_emoji,
                created_at,
                updated_at
            FROM countries
            WHERE iso2 = $1
            LIMIT 1
            "#,
        )
        .bind(iso2);
        let row = self.db.fetch_optional(q).await?;
        row.map(row_to_country).transpose()
    }

    pub async fn find_by_iso3(&self, iso3: &str) -> Result<Option<Country>> {
        let iso3 = iso3.trim().to_ascii_uppercase();
        let q = sqlx::query_as::<_, CountryRow>(
            r#"
            SELECT
                iso2,
                iso3,
                iso_numeric,
                name,
                official_name,
                capital,
                capitals,
                region,
                subregion,
                currencies,
                primary_currency_code,
                calling_code,
                calling_root,
                calling_suffixes,
                tlds,
                timezones,
                latitude,
                longitude,
                independent,
                status,
                assignment_status,
                un_member,
                flag_emoji,
                created_at,
                updated_at
            FROM countries
            WHERE iso3 = $1
            LIMIT 1
            "#,
        )
        .bind(iso3);
        let row = self.db.fetch_optional(q).await?;
        row.map(row_to_country).transpose()
    }

    pub async fn search_by_name(&self, keyword: &str, limit: i64) -> Result<Vec<Country>> {
        let keyword = keyword.trim();
        if keyword.is_empty() {
            return Ok(Vec::new());
        }

        let cap = limit.clamp(1, 500);
        let pattern = format!("%{keyword}%");

        let q = sqlx::query_as::<_, CountryRow>(
            r#"
            SELECT
                iso2,
                iso3,
                iso_numeric,
                name,
                official_name,
                capital,
                capitals,
                region,
                subregion,
                currencies,
                primary_currency_code,
                calling_code,
                calling_root,
                calling_suffixes,
                tlds,
                timezones,
                latitude,
                longitude,
                independent,
                status,
                assignment_status,
                un_member,
                flag_emoji,
                created_at,
                updated_at
            FROM countries
            WHERE name ILIKE $1 OR official_name ILIKE $1
            ORDER BY name ASC
            LIMIT $2
            "#,
        )
        .bind(pattern)
        .bind(cap);

        let rows = self.db.fetch_all(q).await?;
        rows.into_iter().map(row_to_country).collect()
    }
}

fn row_to_country(row: CountryRow) -> Result<Country> {
    let currencies: Vec<CountryCurrency> = serde_json::from_value(row.currencies)
        .context("failed to decode countries.currencies json")?;

    Ok(Country {
        iso2: row.iso2,
        iso3: row.iso3,
        iso_numeric: row.iso_numeric,
        name: row.name,
        official_name: row.official_name,
        capital: row.capital,
        capitals: row.capitals,
        region: row.region,
        subregion: row.subregion,
        currencies,
        primary_currency_code: row.primary_currency_code,
        calling_code: row.calling_code,
        calling_root: row.calling_root,
        calling_suffixes: row.calling_suffixes,
        tlds: row.tlds,
        timezones: row.timezones,
        latitude: row.latitude,
        longitude: row.longitude,
        independent: row.independent,
        status: row.status,
        assignment_status: row.assignment_status,
        un_member: row.un_member,
        flag_emoji: row.flag_emoji,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}

fn normalize_seed(mut seed: CountrySeed) -> CountrySeed {
    seed.iso2 = seed.iso2.trim().to_ascii_uppercase();
    seed.iso3 = seed.iso3.trim().to_ascii_uppercase();
    seed.iso_numeric = seed.iso_numeric.and_then(trim_opt);
    seed.name = seed.name.trim().to_string();
    seed.official_name = seed.official_name.and_then(trim_opt);
    seed.capital = seed.capital.and_then(trim_opt);
    seed.capitals = normalize_vec(seed.capitals);
    seed.region = seed.region.and_then(trim_opt);
    seed.subregion = seed.subregion.and_then(trim_opt);
    seed.primary_currency_code = seed
        .primary_currency_code
        .and_then(trim_opt)
        .map(|v| v.to_ascii_uppercase());
    seed.calling_code = seed.calling_code.and_then(trim_opt);
    seed.calling_root = seed.calling_root.and_then(trim_opt);
    seed.calling_suffixes = normalize_vec(seed.calling_suffixes);
    seed.tlds = normalize_vec(seed.tlds);
    seed.timezones = normalize_vec(seed.timezones);
    seed.assignment_status = seed.assignment_status.and_then(trim_opt);
    seed.flag_emoji = seed.flag_emoji.and_then(trim_opt);

    seed.currencies = seed
        .currencies
        .into_iter()
        .map(|mut c| {
            c.code = c.code.trim().to_ascii_uppercase();
            c.name = c.name.and_then(trim_opt);
            c.symbol = c.symbol.and_then(trim_opt);
            c
        })
        .filter(|c| !c.code.is_empty())
        .collect();

    seed
}

fn default_status_for_iso2(iso2: &str) -> &'static str {
    if iso2.eq_ignore_ascii_case("MY") {
        COUNTRY_STATUS_ENABLED
    } else {
        COUNTRY_STATUS_DISABLED
    }
}

fn trim_opt(input: String) -> Option<String> {
    let value = input.trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

fn normalize_vec(input: Vec<String>) -> Vec<String> {
    input.into_iter().filter_map(trim_opt).collect::<Vec<_>>()
}
