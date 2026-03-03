use std::collections::BTreeMap;

use generated::models::PageSystemFlag;
use schemars::JsonSchema;
use serde::Serialize;
use ts_rs::TS;

#[derive(Debug, Clone, Default, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct AdminPageUpdateInput {
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
pub struct AdminPageOutput {
    pub id: i64,
    pub tag: String,
    #[ts(type = "PageSystemFlag")]
    pub is_system: PageSystemFlag,
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
pub struct AdminPageUpdateOutput {
    pub id: i64,
    pub tag: String,
    #[ts(type = "PageSystemFlag")]
    pub is_system: PageSystemFlag,
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
pub struct AdminPageDeleteOutput {
    pub deleted: bool,
}

impl From<generated::models::PageView> for AdminPageOutput {
    fn from(value: generated::models::PageView) -> Self {
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

impl From<generated::models::PageView> for AdminPageUpdateOutput {
    fn from(value: generated::models::PageView) -> Self {
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
