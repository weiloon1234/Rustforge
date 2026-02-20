#![allow(dead_code)]

use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Deserialize)]
pub struct Pagination {
    #[serde(default = "default_page")]
    pub page: u32,

    #[serde(default = "default_per_page")]
    pub per_page: u32,
}

#[derive(Debug, Serialize)]
pub struct Paginated<T> {
    pub data: Vec<T>,
    pub page: u32,
    pub per_page: u32,
    pub total: i64,
}

fn default_page() -> u32 {
    1
}

fn default_per_page() -> u32 {
    30
}

pub fn resolve_per_page(per_page: i64) -> i64 {
    if per_page > 0 {
        return per_page;
    }
    core_config::AppSettings::from_env()
        .ok()
        .and_then(|settings| i64::try_from(settings.default_per_page).ok())
        .filter(|n| *n > 0)
        .unwrap_or(30)
}

impl Pagination {
    pub fn limit(&self) -> i64 {
        self.per_page.min(100) as i64
    }

    pub fn offset(&self) -> i64 {
        ((self.page.max(1) - 1) * self.per_page) as i64
    }
}
