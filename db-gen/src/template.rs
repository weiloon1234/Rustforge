use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum TemplateError {
    MissingTemplate {
        path: PathBuf,
        source: std::io::Error,
    },
    DuplicateKey(String),
    MalformedPlaceholder {
        template: String,
        detail: String,
    },
    InvalidPlaceholderKey {
        template: String,
        key: String,
    },
    UnresolvedPlaceholder {
        template: String,
        key: String,
    },
}

impl fmt::Display for TemplateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingTemplate { path, source } => {
                write!(
                    f,
                    "failed to load template '{}': {}",
                    path.display(),
                    source
                )
            }
            Self::DuplicateKey(key) => write!(f, "duplicate template context key '{}'", key),
            Self::MalformedPlaceholder { template, detail } => write!(
                f,
                "malformed placeholder in template '{}': {}",
                template, detail
            ),
            Self::InvalidPlaceholderKey { template, key } => write!(
                f,
                "invalid placeholder key '{}' in template '{}'",
                key, template
            ),
            Self::UnresolvedPlaceholder { template, key } => write!(
                f,
                "unresolved placeholder '{}' in template '{}'",
                key, template
            ),
        }
    }
}

impl Error for TemplateError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::MissingTemplate { source, .. } => Some(source),
            _ => None,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct TemplateContext {
    values: BTreeMap<String, String>,
}

impl TemplateContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert<K, V>(&mut self, key: K, value: V) -> Result<(), TemplateError>
    where
        K: Into<String>,
        V: Into<String>,
    {
        let key = key.into();
        if self.values.contains_key(&key) {
            return Err(TemplateError::DuplicateKey(key));
        }
        self.values.insert(key, value.into());
        Ok(())
    }

    pub fn with<K, V>(mut self, key: K, value: V) -> Result<Self, TemplateError>
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.insert(key, value)?;
        Ok(self)
    }

    pub fn render_named(
        &self,
        template_name: &str,
        template: &str,
    ) -> Result<String, TemplateError> {
        let mut out = String::with_capacity(template.len());
        let mut idx = 0usize;
        let bytes = template.as_bytes();
        while idx < bytes.len() {
            if template[idx..].starts_with("{{") {
                let Some(close_rel) = template[idx + 2..].find("}}") else {
                    return Err(TemplateError::MalformedPlaceholder {
                        template: template_name.to_string(),
                        detail: format!("missing closing braces after byte {}", idx),
                    });
                };
                let close = idx + 2 + close_rel;
                let raw_key = &template[idx + 2..close];
                let key = raw_key.trim();
                if key.is_empty() {
                    return Err(TemplateError::MalformedPlaceholder {
                        template: template_name.to_string(),
                        detail: format!("empty placeholder at byte {}", idx),
                    });
                }
                if !is_valid_key(key) {
                    return Err(TemplateError::InvalidPlaceholderKey {
                        template: template_name.to_string(),
                        key: key.to_string(),
                    });
                }
                let value =
                    self.values
                        .get(key)
                        .ok_or_else(|| TemplateError::UnresolvedPlaceholder {
                            template: template_name.to_string(),
                            key: key.to_string(),
                        })?;
                out.push_str(value);
                idx = close + 2;
                continue;
            }
            let ch = template[idx..]
                .chars()
                .next()
                .expect("slice should contain a char");
            out.push(ch);
            idx += ch.len_utf8();
        }

        let referenced = extract_placeholders(template_name, template)?;
        let unresolved: BTreeSet<_> = referenced
            .into_iter()
            .filter(|key| !self.values.contains_key(key))
            .collect();
        if let Some(key) = unresolved.into_iter().next() {
            return Err(TemplateError::UnresolvedPlaceholder {
                template: template_name.to_string(),
                key,
            });
        }

        Ok(out)
    }
}

pub fn render_template(
    template_name: &str,
    context: &TemplateContext,
) -> Result<String, TemplateError> {
    let path = template_path(template_name);
    let template = fs::read_to_string(&path).map_err(|source| TemplateError::MissingTemplate {
        path: path.clone(),
        source,
    })?;
    context.render_named(template_name, &template)
}

pub fn template_path(template_name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("templates")
        .join(template_name)
}

fn extract_placeholders(
    template_name: &str,
    template: &str,
) -> Result<BTreeSet<String>, TemplateError> {
    let mut out = BTreeSet::new();
    let mut idx = 0usize;
    while idx < template.len() {
        if template[idx..].starts_with("{{") {
            let Some(close_rel) = template[idx + 2..].find("}}") else {
                return Err(TemplateError::MalformedPlaceholder {
                    template: template_name.to_string(),
                    detail: format!("missing closing braces after byte {}", idx),
                });
            };
            let close = idx + 2 + close_rel;
            let raw_key = &template[idx + 2..close];
            let key = raw_key.trim();
            if key.is_empty() {
                return Err(TemplateError::MalformedPlaceholder {
                    template: template_name.to_string(),
                    detail: format!("empty placeholder at byte {}", idx),
                });
            }
            if !is_valid_key(key) {
                return Err(TemplateError::InvalidPlaceholderKey {
                    template: template_name.to_string(),
                    key: key.to_string(),
                });
            }
            out.insert(key.to_string());
            idx = close + 2;
            continue;
        }
        let ch = template[idx..]
            .chars()
            .next()
            .expect("slice should contain a char");
        idx += ch.len_utf8();
    }
    Ok(out)
}

fn is_valid_key(key: &str) -> bool {
    key.chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.'))
}

#[cfg(test)]
mod tests {
    use super::{render_template, TemplateContext, TemplateError};

    #[test]
    fn render_named_replaces_placeholders() {
        let mut context = TemplateContext::new();
        context
            .insert("name", "World")
            .expect("context insert should succeed");
        let rendered = context
            .render_named("inline", "Hello {{name}}!")
            .expect("template render should succeed");
        assert_eq!(rendered, "Hello World!");
    }

    #[test]
    fn render_named_rejects_duplicate_context_keys() {
        let mut context = TemplateContext::new();
        context
            .insert("name", "first")
            .expect("first insert should succeed");
        let err = context
            .insert("name", "second")
            .expect_err("duplicate insert should fail");
        assert!(matches!(err, TemplateError::DuplicateKey(key) if key == "name"));
    }

    #[test]
    fn render_named_rejects_unresolved_placeholders() {
        let context = TemplateContext::new();
        let err = context
            .render_named("inline", "Hello {{name}}!")
            .expect_err("missing placeholder should fail");
        assert!(matches!(
            err,
            TemplateError::UnresolvedPlaceholder { template, key }
                if template == "inline" && key == "name"
        ));
    }

    #[test]
    fn render_template_fails_for_missing_file() {
        let err = render_template("missing/nope.tpl", &TemplateContext::new())
            .expect_err("missing template file should fail");
        assert!(matches!(err, TemplateError::MissingTemplate { .. }));
    }
}
