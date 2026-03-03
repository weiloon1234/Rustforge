use std::collections::BTreeMap;

use generated::models::ContentPageSystemFlag;
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
    pub is_system: ContentPageSystemFlag,
    #[ts(type = "MultiLang")]
    pub title: BTreeMap<String, String>,
    #[ts(type = "MultiLang")]
    pub content: BTreeMap<String, String>,
    #[ts(type = "MultiLang")]
    pub cover: BTreeMap<String, String>,
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
    pub is_system: ContentPageSystemFlag,
    #[ts(type = "MultiLang")]
    pub title: BTreeMap<String, String>,
    #[ts(type = "MultiLang")]
    pub content: BTreeMap<String, String>,
    #[ts(type = "MultiLang")]
    pub cover: BTreeMap<String, String>,
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
        Self {
            id: value.id,
            tag: value.tag,
            is_system: value.is_system,
            title: multilang_to_map(value.title_translations.as_ref()),
            content: multilang_to_map(value.content_translations.as_ref()),
            cover: multilang_to_map(value.cover_translations.as_ref()),
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

impl From<generated::models::ContentPageView> for AdminContentPageUpdateOutput {
    fn from(value: generated::models::ContentPageView) -> Self {
        Self {
            id: value.id,
            tag: value.tag,
            is_system: value.is_system,
            title: multilang_to_map(value.title_translations.as_ref()),
            content: multilang_to_map(value.content_translations.as_ref()),
            cover: multilang_to_map(value.cover_translations.as_ref()),
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
