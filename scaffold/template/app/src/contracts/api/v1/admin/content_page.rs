use core_web::ids::SnowflakeId;
use generated::models::ContentPageSystemFlag;
use schemars::JsonSchema;
use serde::Serialize;
use ts_rs::TS;

#[derive(Debug, Clone, Default, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct AdminContentPageUpdateInput {
    pub tag: String,
    pub title: generated::LocalizedText,
    pub content: generated::LocalizedText,
    pub cover: generated::LocalizedText,
}

#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct AdminContentPageOutput {
    pub id: SnowflakeId,
    pub tag: String,
    pub is_system: ContentPageSystemFlag,
    pub title: generated::LocalizedText,
    pub content: generated::LocalizedText,
    pub cover: generated::LocalizedText,
    pub cover_url: generated::LocalizedText,
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
    pub id: SnowflakeId,
    pub tag: String,
    pub is_system: ContentPageSystemFlag,
    pub title: generated::LocalizedText,
    pub content: generated::LocalizedText,
    pub cover: generated::LocalizedText,
    pub cover_url: generated::LocalizedText,
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
        let cover = value.cover_translations.unwrap_or_default();
        let cover_url = attachment_urls_from_localized_text(&cover);
        Self {
            id: value.id.into(),
            tag: value.tag,
            is_system: value.is_system,
            title: value.title_translations.unwrap_or_default(),
            content: value.content_translations.unwrap_or_default(),
            cover_url,
            cover,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

impl From<generated::models::ContentPageView> for AdminContentPageUpdateOutput {
    fn from(value: generated::models::ContentPageView) -> Self {
        let cover = value.cover_translations.unwrap_or_default();
        let cover_url = attachment_urls_from_localized_text(&cover);
        Self {
            id: value.id.into(),
            tag: value.tag,
            is_system: value.is_system,
            title: value.title_translations.unwrap_or_default(),
            content: value.content_translations.unwrap_or_default(),
            cover_url,
            cover,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

fn attachment_urls_from_localized_text(
    values: &generated::LocalizedText,
) -> generated::LocalizedText {
    let base = std::env::var("S3_URL").ok();
    let mut mapped = values.to_map();
    for path in mapped.values_mut() {
        *path = build_attachment_url(path, base.as_deref());
    }
    generated::LocalizedText::from_map(&mapped)
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
