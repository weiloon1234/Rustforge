#![allow(dead_code)]
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct AttachmentRules {
    pub allowed: Vec<String>,
    pub resize: Option<ResizeRule>,
}

#[derive(Debug, Clone)]
pub struct ResizeRule {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub quality: Option<u8>,
}

use schemars::JsonSchema;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Attachment {
    pub id: Uuid,
    pub path: String,
    #[serde(default)]
    pub url: String,
    pub content_type: String,
    pub size: i64,
    pub width: Option<i32>,
    pub height: Option<i32>,
    #[schemars(with = "String")]
    pub created_at: time::OffsetDateTime,
}

impl Attachment {
    /// Build a URL using an optional base (e.g. CDN). Falls back to the stored path.
    pub fn url_with_base(&self, base: Option<&str>) -> String {
        attachment_url(&self.path, base)
    }
}

pub fn attachment_url(path: &str, base: Option<&str>) -> String {
    let path = path.trim();
    if path.is_empty() {
        return String::new();
    }

    if path.starts_with("//")
        || path.starts_with("http://")
        || path.starts_with("https://")
        || path.starts_with("data:")
        || path.starts_with("blob:")
    {
        return path.to_string();
    }

    let Some(base) = base.map(str::trim).filter(|value| !value.is_empty()) else {
        return path.to_string();
    };

    let base = base.trim_end_matches('/');
    let path = path.trim_start_matches('/');
    format!("{base}/{path}")
}

#[derive(Debug, Clone, Default)]
pub struct AttachmentMap {
    // field -> owner_id -> attachments
    inner: HashMap<String, HashMap<i64, Vec<Attachment>>>,
}

impl AttachmentMap {
    pub fn new(inner: HashMap<String, HashMap<i64, Vec<Attachment>>>) -> Self {
        Self { inner }
    }

    pub fn get_single(&self, field: &str, owner_id: i64) -> Option<Attachment> {
        self.inner
            .get(field)
            .and_then(|by_owner| by_owner.get(&owner_id))
            .and_then(|list| list.first())
            .cloned()
    }

    pub fn get_many(&self, field: &str, owner_id: i64) -> Vec<Attachment> {
        self.inner
            .get(field)
            .and_then(|by_owner| by_owner.get(&owner_id))
            .cloned()
            .unwrap_or_default()
    }
}

/// Canonical attachment payload shape for app-level DTOs and generated model APIs.
///
/// It matches metadata-first upload flows and can also carry an optional `id`
/// for client-side references when needed.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AttachmentUploadDto {
    #[serde(default)]
    pub id: Option<Uuid>,
    #[serde(default)]
    pub name: Option<String>,
    pub path: String,
    #[serde(alias = "type")]
    pub content_type: String,
    pub size: i64,
    #[serde(default)]
    pub width: Option<i32>,
    #[serde(default)]
    pub height: Option<i32>,
}

impl AttachmentUploadDto {
    pub fn new(
        path: impl Into<String>,
        content_type: impl Into<String>,
        size: i64,
        width: Option<i32>,
        height: Option<i32>,
    ) -> Self {
        Self {
            id: None,
            name: None,
            path: path.into(),
            content_type: content_type.into(),
            size,
            width,
            height,
        }
    }

    pub fn with_id(mut self, id: Uuid) -> Self {
        self.id = Some(id);
        self
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
}

/// Backward-compatible alias used by generated model methods.
pub type AttachmentInput = AttachmentUploadDto;
