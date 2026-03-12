use bytes::Bytes;
use core_db::platform::attachments::types::AttachmentRules;
use std::sync::OnceLock;

/// A file extracted from a multipart upload, before storage.
///
/// `Serialize` is derived (all fields skipped) solely to satisfy the `validator` crate's
/// `Validate` derive bound; `FileUpload` is never actually serialized.
#[derive(Debug, Clone, serde::Serialize)]
pub struct FileUpload {
    #[serde(skip)]
    pub filename: Option<String>,
    #[serde(skip)]
    pub content_type: String,
    #[serde(skip)]
    pub bytes: Bytes,
    #[serde(skip)]
    pub size: usize,
}

impl Default for FileUpload {
    fn default() -> Self {
        Self {
            filename: None,
            content_type: String::new(),
            bytes: Bytes::new(),
            size: 0,
        }
    }
}

impl FileUpload {
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    /// Derive file extension from filename, falling back to content_type subtype.
    pub fn extension(&self) -> &str {
        self.filename
            .as_deref()
            .and_then(|n| std::path::Path::new(n).extension().and_then(|e| e.to_str()))
            .unwrap_or_else(|| self.content_type.split('/').nth(1).unwrap_or("bin"))
    }
}

// ── Attachment validation ────────────────────────────────────────────────────

/// Validate a file upload against attachment rules (MIME type + max_size).
/// Returns `Err(ValidationError)` for use with the `validator` crate.
pub fn validate_attachment(
    file: &FileUpload,
    rules: &AttachmentRules,
) -> Result<(), validator::ValidationError> {
    if let Some(max_size) = rules.max_size {
        if file.size > max_size {
            let mut err = validator::ValidationError::new("attachment_too_large");
            err.message = Some(std::borrow::Cow::Owned(format!(
                "File exceeds maximum allowed size of {} bytes",
                max_size
            )));
            return Err(err);
        }
    }

    if rules.allowed.is_empty() {
        return Ok(());
    }
    let ct = file.content_type.to_ascii_lowercase();
    for rule in &rules.allowed {
        let rule_lower = rule.to_ascii_lowercase();
        if rule_lower == "*" || rule_lower == "*/*" || ct == rule_lower {
            return Ok(());
        }
        if rule_lower.starts_with('.') {
            if let Some(name) = file.filename.as_deref().map(|n| n.to_ascii_lowercase()) {
                if name.ends_with(&rule_lower) {
                    return Ok(());
                }
            }
        }
    }

    let mut err = validator::ValidationError::new("attachment_type_not_allowed");
    err.message = Some(std::borrow::Cow::Borrowed("Attachment type not allowed"));
    Err(err)
}

// ── Storage upload helper ────────────────────────────────────────────────────

impl FileUpload {
    /// Upload this file to storage and return an `AttachmentInput` ready for DB.
    /// `entity` and `field` form the storage key prefix (e.g. "banks", "logo").
    pub async fn upload(
        &self,
        storage: &dyn core_db::infra::storage::Storage,
        entity: &str,
        field: &str,
    ) -> anyhow::Result<core_db::platform::attachments::types::AttachmentInput> {
        let ext = self.extension();
        let key = core_db::platform::attachments::service::build_object_key(entity, field, ext);
        storage.put(&key, self.bytes.clone(), &self.content_type).await?;
        Ok(core_db::platform::attachments::types::AttachmentInput::new(
            key,
            self.content_type.clone(),
            self.bytes.len() as i64,
            None,
            None,
        ))
    }
}

// ── Global attachment rules provider ─────────────────────────────────────────

/// Trait for providing attachment rules at runtime.
/// Implemented by the `generated` crate, wrapping its `get_attachment_rules()` match.
pub trait AttachmentRulesProvider: Send + Sync + 'static {
    fn get_rules(&self, name: &str) -> Option<AttachmentRules>;
}

static PROVIDER: OnceLock<Box<dyn AttachmentRulesProvider>> = OnceLock::new();

/// Register the global attachment rules provider (call once at startup).
pub fn register_attachment_rules_provider(p: impl AttachmentRulesProvider) {
    let _ = PROVIDER.set(Box::new(p));
}

/// Look up attachment rules by type name, as defined in `configs.toml`
/// under `[attachment_type.<name>]`.
pub fn get_attachment_rules(name: &str) -> Option<AttachmentRules> {
    PROVIDER.get().and_then(|p| p.get_rules(name))
}
