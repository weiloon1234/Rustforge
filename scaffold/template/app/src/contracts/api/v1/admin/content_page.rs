use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::Serialize;
use ts_rs::TS;

#[derive(Debug, Clone, Default, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct AdminContentPageUpdateInput {
    pub tag: String,
    #[ts(type = "MultiLang")]
    pub title: BTreeMap<String, String>,
    #[ts(type = "MultiLang")]
    pub content: BTreeMap<String, String>,
    #[ts(type = "MultiLang")]
    pub cover: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct AdminContentPageOutput {
    pub id: i64,
    pub tag: String,
    #[ts(type = "ContentPageSystemFlag")]
    pub is_system: String,
    #[ts(type = "MultiLang")]
    pub title: BTreeMap<String, String>,
    #[ts(type = "MultiLang")]
    pub content: BTreeMap<String, String>,
    #[ts(type = "MultiLang")]
    pub cover: BTreeMap<String, String>,
    #[ts(type = "MultiLang")]
    pub cover_url: BTreeMap<String, String>,
    #[schemars(with = "String")]
    #[ts(type = "string")]
    pub created_at: time::OffsetDateTime,
    #[schemars(with = "String")]
    #[ts(type = "string")]
    pub updated_at: time::OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct AdminContentPageUpdateOutput {
    pub id: i64,
    pub tag: String,
    #[ts(type = "ContentPageSystemFlag")]
    pub is_system: String,
    #[ts(type = "MultiLang")]
    pub title: BTreeMap<String, String>,
    #[ts(type = "MultiLang")]
    pub content: BTreeMap<String, String>,
    #[ts(type = "MultiLang")]
    pub cover: BTreeMap<String, String>,
    #[ts(type = "MultiLang")]
    pub cover_url: BTreeMap<String, String>,
    #[schemars(with = "String")]
    #[ts(type = "string")]
    pub created_at: time::OffsetDateTime,
    #[schemars(with = "String")]
    #[ts(type = "string")]
    pub updated_at: time::OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct AdminContentPageDeleteOutput {
    pub deleted: bool,
}

impl From<generated::models::ContentPageView> for AdminContentPageOutput {
    fn from(value: generated::models::ContentPageView) -> Self {
        let cover = multilang_to_map(value.cover_translations.as_ref());
        Self {
            id: value.id,
            tag: value.tag,
            is_system: value.is_system.as_str().to_string(),
            title: multilang_to_map(value.title_translations.as_ref()),
            content: multilang_to_map(value.content_translations.as_ref()),
            cover_url: attachment_urls_from_map(&cover),
            cover,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

impl From<generated::models::ContentPageView> for AdminContentPageUpdateOutput {
    fn from(value: generated::models::ContentPageView) -> Self {
        let cover = multilang_to_map(value.cover_translations.as_ref());
        Self {
            id: value.id,
            tag: value.tag,
            is_system: value.is_system.as_str().to_string(),
            title: multilang_to_map(value.title_translations.as_ref()),
            content: multilang_to_map(value.content_translations.as_ref()),
            cover_url: attachment_urls_from_map(&cover),
            cover,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

fn multilang_to_map(multilang: Option<&generated::MultiLang>) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for &locale in generated::SUPPORTED_LOCALES {
        let value = multilang
            .map(|current| current.get(locale).to_string())
            .unwrap_or_default();
        out.insert(locale.to_string(), value);
    }
    out
}

fn attachment_urls_from_map(values: &BTreeMap<String, String>) -> BTreeMap<String, String> {
    let base = std::env::var("S3_URL").ok();
    values
        .iter()
        .map(|(locale, path)| {
            (locale.to_string(), build_attachment_url(path, base.as_deref()))
        })
        .collect()
}

fn build_attachment_url(path: &str, base: Option<&str>) -> String {
    let raw = path.trim();
    if raw.is_empty() {
        return String::new();
    }
    if raw.starts_with("//")
        || raw.starts_with("http://")
        || raw.starts_with("https://")
        || raw.starts_with("data:")
        || raw.starts_with("blob:")
    {
        return raw.to_string();
    }

    let Some(base) = base.map(str::trim).filter(|value| !value.is_empty()) else {
        return raw.to_string();
    };
    format!(
        "{}/{}",
        base.trim_end_matches('/'),
        raw.trim_start_matches('/')
    )
}
