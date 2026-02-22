use std::borrow::Cow;
pub mod meta;

use anyhow::Result;
use core_db::{common::sql::DbConn, platform::countries::repo::CountryRepo};
use sqlx::Row;
use validator::{ValidateEmail, ValidationError};

pub fn regex_pattern(value: &str, pattern: &str) -> Result<(), ValidationError> {
    let regex = regex::Regex::new(pattern)
        .map_err(|_| validation_error("regex", "Invalid regex pattern."))?;
    if regex.is_match(value) {
        Ok(())
    } else {
        Err(validation_error("regex", "Invalid format."))
    }
}

/// A trait for validation rules that require async database access.
#[async_trait::async_trait]
pub trait AsyncRule {
    async fn check(&self, db: &sqlx::PgPool) -> Result<bool>;
    fn message(&self) -> String;
}

pub struct Unique {
    table: &'static str,
    column: &'static str,
    value: String,
    ignore_id: Option<(String, String)>, // (col, val)
    where_eq: Vec<(String, String)>,
    where_not_eq: Vec<(String, String)>,
    where_null: Vec<String>,
    where_not_null: Vec<String>,
    message: Option<String>,
}

impl Unique {
    pub fn new(table: &'static str, column: &'static str, value: impl ToString) -> Self {
        Self {
            table,
            column,
            value: value.to_string(),
            ignore_id: None,
            where_eq: Vec::new(),
            where_not_eq: Vec::new(),
            where_null: Vec::new(),
            where_not_null: Vec::new(),
            message: None,
        }
    }

    pub fn ignore(mut self, col: &'static str, val: impl ToString) -> Self {
        self.ignore_id = Some((col.to_string(), val.to_string()));
        self
    }

    pub fn msg(mut self, msg: &str) -> Self {
        self.message = Some(msg.to_string());
        self
    }

    pub fn where_eq(mut self, col: &'static str, val: impl ToString) -> Self {
        self.where_eq.push((col.to_string(), val.to_string()));
        self
    }

    pub fn where_not_eq(mut self, col: &'static str, val: impl ToString) -> Self {
        self.where_not_eq.push((col.to_string(), val.to_string()));
        self
    }

    pub fn where_null(mut self, col: &'static str) -> Self {
        self.where_null.push(col.to_string());
        self
    }

    pub fn where_not_null(mut self, col: &'static str) -> Self {
        self.where_not_null.push(col.to_string());
        self
    }
}

#[async_trait::async_trait]
impl AsyncRule for Unique {
    async fn check(&self, db: &sqlx::PgPool) -> Result<bool> {
        let mut sql = format!(
            "SELECT count(*) FROM {} WHERE {} = $1",
            self.table, self.column
        );
        let mut binds = vec![self.value.clone()];

        if let Some((col, val)) = &self.ignore_id {
            let idx = binds.len() + 1;
            sql.push_str(&format!(" AND {} != ${}", col, idx));
            binds.push(val.clone());
        }

        for (col, val) in &self.where_eq {
            let idx = binds.len() + 1;
            sql.push_str(&format!(" AND {} = ${}", col, idx));
            binds.push(val.clone());
        }

        for (col, val) in &self.where_not_eq {
            let idx = binds.len() + 1;
            sql.push_str(&format!(" AND {} != ${}", col, idx));
            binds.push(val.clone());
        }

        for col in &self.where_null {
            sql.push_str(&format!(" AND {} IS NULL", col));
        }

        for col in &self.where_not_null {
            sql.push_str(&format!(" AND {} IS NOT NULL", col));
        }

        let mut query = sqlx::query(&sql);
        for val in binds {
            query = query.bind(val);
        }

        let count: i64 = query
            .map(|row: sqlx::postgres::PgRow| row.get(0))
            .fetch_one(db)
            .await?;

        Ok(count == 0)
    }

    fn message(&self) -> String {
        self.message
            .clone()
            .unwrap_or_else(|| format!("{} has already been taken.", self.column))
    }
}

pub struct Exists {
    table: &'static str,
    column: &'static str,
    value: String,
    where_eq: Vec<(String, String)>,
    where_not_eq: Vec<(String, String)>,
    where_null: Vec<String>,
    where_not_null: Vec<String>,
    message: Option<String>,
}

impl Exists {
    pub fn new(table: &'static str, column: &'static str, value: impl ToString) -> Self {
        Self {
            table,
            column,
            value: value.to_string(),
            where_eq: Vec::new(),
            where_not_eq: Vec::new(),
            where_null: Vec::new(),
            where_not_null: Vec::new(),
            message: None,
        }
    }

    pub fn msg(mut self, msg: &str) -> Self {
        self.message = Some(msg.to_string());
        self
    }

    pub fn where_eq(mut self, col: &'static str, val: impl ToString) -> Self {
        self.where_eq.push((col.to_string(), val.to_string()));
        self
    }

    pub fn where_not_eq(mut self, col: &'static str, val: impl ToString) -> Self {
        self.where_not_eq.push((col.to_string(), val.to_string()));
        self
    }

    pub fn where_null(mut self, col: &'static str) -> Self {
        self.where_null.push(col.to_string());
        self
    }

    pub fn where_not_null(mut self, col: &'static str) -> Self {
        self.where_not_null.push(col.to_string());
        self
    }
}

#[async_trait::async_trait]
impl AsyncRule for Exists {
    async fn check(&self, db: &sqlx::PgPool) -> Result<bool> {
        let mut sql = format!(
            "SELECT count(*) FROM {} WHERE {} = $1",
            self.table, self.column
        );
        let mut binds = vec![self.value.clone()];

        for (col, val) in &self.where_eq {
            let idx = binds.len() + 1;
            sql.push_str(&format!(" AND {} = ${}", col, idx));
            binds.push(val.clone());
        }

        for (col, val) in &self.where_not_eq {
            let idx = binds.len() + 1;
            sql.push_str(&format!(" AND {} != ${}", col, idx));
            binds.push(val.clone());
        }

        for col in &self.where_null {
            sql.push_str(&format!(" AND {} IS NULL", col));
        }

        for col in &self.where_not_null {
            sql.push_str(&format!(" AND {} IS NOT NULL", col));
        }

        let mut query = sqlx::query(&sql);
        for val in binds {
            query = query.bind(val);
        }

        let count: i64 = query
            .map(|row: sqlx::postgres::PgRow| row.get(0))
            .fetch_one(db)
            .await?;

        Ok(count > 0)
    }

    fn message(&self) -> String {
        self.message
            .clone()
            .unwrap_or_else(|| format!("Selected {} is invalid.", self.column))
    }
}

pub struct NotExists {
    table: &'static str,
    column: &'static str,
    value: String,
    where_eq: Vec<(String, String)>,
    where_not_eq: Vec<(String, String)>,
    where_null: Vec<String>,
    where_not_null: Vec<String>,
    message: Option<String>,
}

impl NotExists {
    pub fn new(table: &'static str, column: &'static str, value: impl ToString) -> Self {
        Self {
            table,
            column,
            value: value.to_string(),
            where_eq: Vec::new(),
            where_not_eq: Vec::new(),
            where_null: Vec::new(),
            where_not_null: Vec::new(),
            message: None,
        }
    }

    pub fn msg(mut self, msg: &str) -> Self {
        self.message = Some(msg.to_string());
        self
    }

    pub fn where_eq(mut self, col: &'static str, val: impl ToString) -> Self {
        self.where_eq.push((col.to_string(), val.to_string()));
        self
    }

    pub fn where_not_eq(mut self, col: &'static str, val: impl ToString) -> Self {
        self.where_not_eq.push((col.to_string(), val.to_string()));
        self
    }

    pub fn where_null(mut self, col: &'static str) -> Self {
        self.where_null.push(col.to_string());
        self
    }

    pub fn where_not_null(mut self, col: &'static str) -> Self {
        self.where_not_null.push(col.to_string());
        self
    }
}

#[async_trait::async_trait]
impl AsyncRule for NotExists {
    async fn check(&self, db: &sqlx::PgPool) -> Result<bool> {
        let mut sql = format!(
            "SELECT count(*) FROM {} WHERE {} = $1",
            self.table, self.column
        );
        let mut binds = vec![self.value.clone()];

        for (col, val) in &self.where_eq {
            let idx = binds.len() + 1;
            sql.push_str(&format!(" AND {} = ${}", col, idx));
            binds.push(val.clone());
        }

        for (col, val) in &self.where_not_eq {
            let idx = binds.len() + 1;
            sql.push_str(&format!(" AND {} != ${}", col, idx));
            binds.push(val.clone());
        }

        for col in &self.where_null {
            sql.push_str(&format!(" AND {} IS NULL", col));
        }

        for col in &self.where_not_null {
            sql.push_str(&format!(" AND {} IS NOT NULL", col));
        }

        let mut query = sqlx::query(&sql);
        for val in binds {
            query = query.bind(val);
        }

        let count: i64 = query
            .map(|row: sqlx::postgres::PgRow| row.get(0))
            .fetch_one(db)
            .await?;

        Ok(count == 0)
    }

    fn message(&self) -> String {
        self.message
            .clone()
            .unwrap_or_else(|| format!("{} has already been taken.", self.column))
    }
}

pub struct PhoneByCountryIso2 {
    country_iso2: String,
    phone_number: String,
    enabled_only: bool,
    message: Option<String>,
}

impl PhoneByCountryIso2 {
    pub fn new(country_iso2: impl ToString, phone_number: impl ToString) -> Self {
        Self {
            country_iso2: country_iso2.to_string(),
            phone_number: phone_number.to_string(),
            enabled_only: true,
            message: None,
        }
    }

    pub fn enabled_only(mut self, enabled_only: bool) -> Self {
        self.enabled_only = enabled_only;
        self
    }

    pub fn msg(mut self, msg: &str) -> Self {
        self.message = Some(msg.to_string());
        self
    }
}

#[async_trait::async_trait]
impl AsyncRule for PhoneByCountryIso2 {
    async fn check(&self, db: &sqlx::PgPool) -> Result<bool> {
        let value = normalize_phone_by_country_iso2(
            db,
            &self.country_iso2,
            &self.phone_number,
            self.enabled_only,
        )
        .await?;
        Ok(value.is_some())
    }

    fn message(&self) -> String {
        self.message
            .clone()
            .unwrap_or_else(|| "The phone number is invalid for the selected country.".to_string())
    }
}

pub async fn normalize_phone_by_country_iso2(
    db: &sqlx::PgPool,
    country_iso2: &str,
    phone_number: &str,
    enabled_only: bool,
) -> Result<Option<String>> {
    let repo = CountryRepo::new(DbConn::pool(db));
    let Some(country) = repo.find_by_iso2(country_iso2).await? else {
        return Ok(None);
    };

    if enabled_only && country.status != "enabled" {
        return Ok(None);
    }

    country
        .format_phone_number(phone_number, false)
        .map_err(anyhow::Error::from)
}

pub fn required_trimmed(value: &str) -> Result<(), ValidationError> {
    if value.trim().is_empty() {
        return Err(validation_error(
            "required_trimmed",
            "This field is required.",
        ));
    }
    Ok(())
}

pub fn email(value: &str) -> Result<(), ValidationError> {
    if !value.validate_email() {
        return Err(validation_error("email", "Invalid email address."));
    }
    Ok(())
}

pub fn eq<T>(value: &T, other: &T) -> Result<(), ValidationError>
where
    T: PartialEq,
{
    if value == other {
        Ok(())
    } else {
        Err(validation_error(
            "eq",
            "Value must be equal to comparison value.",
        ))
    }
}

pub fn gt<T>(value: &T, other: &T) -> Result<(), ValidationError>
where
    T: PartialOrd,
{
    if value > other {
        Ok(())
    } else {
        Err(validation_error(
            "gt",
            "Value must be greater than comparison value.",
        ))
    }
}

pub fn gte<T>(value: &T, other: &T) -> Result<(), ValidationError>
where
    T: PartialOrd,
{
    if value >= other {
        Ok(())
    } else {
        Err(validation_error(
            "gte",
            "Value must be greater than or equal to comparison value.",
        ))
    }
}

pub fn lt<T>(value: &T, other: &T) -> Result<(), ValidationError>
where
    T: PartialOrd,
{
    if value < other {
        Ok(())
    } else {
        Err(validation_error(
            "lt",
            "Value must be less than comparison value.",
        ))
    }
}

pub fn lte<T>(value: &T, other: &T) -> Result<(), ValidationError>
where
    T: PartialOrd,
{
    if value <= other {
        Ok(())
    } else {
        Err(validation_error(
            "lte",
            "Value must be less than or equal to comparison value.",
        ))
    }
}

pub fn date(value: &str, format: &str) -> Result<(), ValidationError> {
    parse_date(value, format).map(|_| ())
}

pub fn datetime(value: &str, format: &str) -> Result<(), ValidationError> {
    parse_datetime(value, format).map(|_| ())
}

pub fn date_eq(value: &str, other: &str, format: &str) -> Result<(), ValidationError> {
    let lhs = parse_date(value, format)?;
    let rhs = parse_date(other, format)?;
    eq(&lhs, &rhs).map_err(|_| validation_error("date_eq", "Date must equal comparison date."))
}

pub fn date_gt(value: &str, other: &str, format: &str) -> Result<(), ValidationError> {
    let lhs = parse_date(value, format)?;
    let rhs = parse_date(other, format)?;
    gt(&lhs, &rhs).map_err(|_| validation_error("date_gt", "Date must be after comparison date."))
}

pub fn date_gte(value: &str, other: &str, format: &str) -> Result<(), ValidationError> {
    let lhs = parse_date(value, format)?;
    let rhs = parse_date(other, format)?;
    gte(&lhs, &rhs).map_err(|_| {
        validation_error(
            "date_gte",
            "Date must be after or equal to comparison date.",
        )
    })
}

pub fn date_lt(value: &str, other: &str, format: &str) -> Result<(), ValidationError> {
    let lhs = parse_date(value, format)?;
    let rhs = parse_date(other, format)?;
    lt(&lhs, &rhs).map_err(|_| validation_error("date_lt", "Date must be before comparison date."))
}

pub fn date_lte(value: &str, other: &str, format: &str) -> Result<(), ValidationError> {
    let lhs = parse_date(value, format)?;
    let rhs = parse_date(other, format)?;
    lte(&lhs, &rhs).map_err(|_| {
        validation_error(
            "date_lte",
            "Date must be before or equal to comparison date.",
        )
    })
}

pub fn datetime_eq(value: &str, other: &str, format: &str) -> Result<(), ValidationError> {
    let lhs = parse_datetime(value, format)?;
    let rhs = parse_datetime(other, format)?;
    eq(&lhs, &rhs)
        .map_err(|_| validation_error("datetime_eq", "Datetime must equal comparison datetime."))
}

pub fn datetime_gt(value: &str, other: &str, format: &str) -> Result<(), ValidationError> {
    let lhs = parse_datetime(value, format)?;
    let rhs = parse_datetime(other, format)?;
    gt(&lhs, &rhs)
        .map_err(|_| validation_error("datetime_gt", "Datetime must be after comparison datetime."))
}

pub fn datetime_gte(value: &str, other: &str, format: &str) -> Result<(), ValidationError> {
    let lhs = parse_datetime(value, format)?;
    let rhs = parse_datetime(other, format)?;
    gte(&lhs, &rhs).map_err(|_| {
        validation_error(
            "datetime_gte",
            "Datetime must be after or equal to comparison datetime.",
        )
    })
}

pub fn datetime_lt(value: &str, other: &str, format: &str) -> Result<(), ValidationError> {
    let lhs = parse_datetime(value, format)?;
    let rhs = parse_datetime(other, format)?;
    lt(&lhs, &rhs).map_err(|_| {
        validation_error(
            "datetime_lt",
            "Datetime must be before comparison datetime.",
        )
    })
}

pub fn datetime_lte(value: &str, other: &str, format: &str) -> Result<(), ValidationError> {
    let lhs = parse_datetime(value, format)?;
    let rhs = parse_datetime(other, format)?;
    lte(&lhs, &rhs).map_err(|_| {
        validation_error(
            "datetime_lte",
            "Datetime must be before or equal to comparison datetime.",
        )
    })
}

fn parse_date(value: &str, format: &str) -> Result<time::Date, ValidationError> {
    let items = time::format_description::parse(format)
        .map_err(|_| validation_error("date_format", "Invalid date format."))?;
    time::Date::parse(value, &items)
        .map_err(|_| validation_error("date", "Invalid date value for the configured format."))
}

fn parse_datetime(value: &str, format: &str) -> Result<time::PrimitiveDateTime, ValidationError> {
    let items = time::format_description::parse(format)
        .map_err(|_| validation_error("datetime_format", "Invalid datetime format."))?;
    time::PrimitiveDateTime::parse(value, &items).map_err(|_| {
        validation_error(
            "datetime",
            "Invalid datetime value for the configured format.",
        )
    })
}

pub fn alpha_dash(value: &str) -> Result<(), ValidationError> {
    if value.is_empty() {
        return Err(validation_error(
            "alpha_dash",
            "Only letters, numbers, dashes, and underscores are allowed.",
        ));
    }
    if value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
    {
        return Ok(());
    }
    Err(validation_error(
        "alpha_dash",
        "Only letters, numbers, dashes, and underscores are allowed.",
    ))
}

pub fn lowercase_slug(value: &str) -> Result<(), ValidationError> {
    if value.is_empty() || value.starts_with('-') || value.ends_with('-') || value.contains("--") {
        return Err(validation_error(
            "slug",
            "Slug must be lowercase and may contain letters, numbers, and single dashes.",
        ));
    }
    if value
        .chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
    {
        return Ok(());
    }
    Err(validation_error(
        "slug",
        "Slug must be lowercase and may contain letters, numbers, and single dashes.",
    ))
}

pub fn strong_password(value: &str) -> Result<(), ValidationError> {
    if value.len() < 8 {
        return Err(validation_error(
            "strong_password",
            "Password must be at least 8 characters.",
        ));
    }
    let has_lower = value.chars().any(|c| c.is_ascii_lowercase());
    let has_upper = value.chars().any(|c| c.is_ascii_uppercase());
    let has_digit = value.chars().any(|c| c.is_ascii_digit());
    if has_lower && has_upper && has_digit {
        return Ok(());
    }
    Err(validation_error(
        "strong_password",
        "Password must include uppercase, lowercase, and a number.",
    ))
}

pub fn one_of(value: &str, allowed: &[&str]) -> Result<(), ValidationError> {
    if allowed.iter().any(|candidate| candidate == &value) {
        return Ok(());
    }
    Err(validation_error("one_of", "The selected value is invalid."))
}

pub fn none_of(value: &str, blocked: &[&str]) -> Result<(), ValidationError> {
    if blocked.iter().any(|candidate| candidate == &value) {
        return Err(validation_error(
            "none_of",
            "The selected value is not allowed.",
        ));
    }
    Ok(())
}

fn validation_error(code: &'static str, message: &'static str) -> ValidationError {
    ValidationError::new(code).with_message(Cow::Borrowed(message))
}

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    #[test]
    fn alpha_dash_accepts_common_values() {
        assert!(alpha_dash("admin_role-1").is_ok());
    }

    #[test]
    fn email_accepts_valid_address() {
        assert!(email("admin@example.com").is_ok());
    }

    #[test]
    fn email_rejects_invalid_address() {
        assert!(email("admin@").is_err());
    }

    #[test]
    fn comparison_rules_work_for_numbers() {
        assert!(eq(&5, &5).is_ok());
        assert!(gt(&6, &5).is_ok());
        assert!(gte(&5, &5).is_ok());
        assert!(lt(&4, &5).is_ok());
        assert!(lte(&5, &5).is_ok());

        assert!(gt(&5, &5).is_err());
        assert!(lt(&5, &5).is_err());
    }

    #[test]
    fn date_rules_support_custom_format_and_compare() {
        let fmt = "[year]-[month]-[day]";
        assert!(date("2026-02-20", fmt).is_ok());
        assert!(date_gt("2026-02-20", "2026-02-19", fmt).is_ok());
        assert!(date_gte("2026-02-20", "2026-02-20", fmt).is_ok());
        assert!(date_lt("2026-02-19", "2026-02-20", fmt).is_ok());
        assert!(date_lte("2026-02-20", "2026-02-20", fmt).is_ok());
        assert!(date_eq("2026-02-20", "2026-02-20", fmt).is_ok());
        assert!(date("20/02/2026", fmt).is_err());
    }

    #[test]
    fn datetime_rules_support_custom_format_and_compare() {
        let fmt = "[year]-[month]-[day] [hour]:[minute]:[second]";
        assert!(datetime("2026-02-20 10:20:30", fmt).is_ok());
        assert!(datetime_gt("2026-02-20 10:20:31", "2026-02-20 10:20:30", fmt).is_ok());
        assert!(datetime_gte("2026-02-20 10:20:30", "2026-02-20 10:20:30", fmt).is_ok());
        assert!(datetime_lt("2026-02-20 10:20:29", "2026-02-20 10:20:30", fmt).is_ok());
        assert!(datetime_lte("2026-02-20 10:20:30", "2026-02-20 10:20:30", fmt).is_ok());
        assert!(datetime_eq("2026-02-20 10:20:30", "2026-02-20 10:20:30", fmt).is_ok());
        assert!(datetime("2026-02-20T10:20:30", fmt).is_err());
    }

    #[test]
    fn alpha_dash_rejects_symbols() {
        assert!(alpha_dash("admin@role").is_err());
    }

    #[test]
    fn slug_accepts_lowercase_slug() {
        assert!(lowercase_slug("article-category-1").is_ok());
    }

    #[test]
    fn slug_rejects_uppercase_or_double_dash() {
        assert!(lowercase_slug("Article--category").is_err());
    }

    #[test]
    fn strong_password_requires_mixed_chars() {
        assert!(strong_password("Password1").is_ok());
        assert!(strong_password("password").is_err());
    }

    #[test]
    fn one_of_matches_allowed_values() {
        assert!(one_of("draft", &["draft", "published"]).is_ok());
        assert!(one_of("archived", &["draft", "published"]).is_err());
    }

    #[derive(Debug, Validate)]
    struct ContactInput {
        contact_country_iso2: String,
        #[validate(phonenumber(field = contact_country_iso2))]
        contact_phone: String,
    }

    #[test]
    fn derive_phonenumber_validates_against_country_field() {
        let ok = ContactInput {
            contact_country_iso2: "US".to_string(),
            contact_phone: "2025550123".to_string(),
        };
        assert!(ok.validate().is_ok());

        let invalid = ContactInput {
            contact_country_iso2: "US".to_string(),
            contact_phone: "123".to_string(),
        };
        assert!(invalid.validate().is_err());
    }
}
