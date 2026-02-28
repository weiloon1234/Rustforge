use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct Schema {
    #[serde(rename = "model")]
    pub models: BTreeMap<String, ModelSpec>,

    // Collect all top-level enum definitions
    #[serde(flatten)]
    pub extra_sections: BTreeMap<String, EnumOrOther>,
}

#[derive(Debug, Deserialize)]
pub struct ModelSpec {
    pub table: Option<String>,
    pub pk: Option<String>,
    pub pk_type: Option<String>,
    /// Primary key generation strategy.
    /// Supported values:
    /// - "snowflake" (default for i64 pk)
    /// - "manual" (caller sets id explicitly)
    pub id_strategy: Option<String>,
    pub multilang: Option<Vec<String>>,
    pub meta: Option<Vec<String>>,
    pub attachment: Option<Vec<String>>,
    pub attachments: Option<Vec<String>>,
    pub fields: Option<Vec<String>>,
    pub relations: Option<Vec<String>>,
    pub computed: Option<Vec<String>>,
    pub touch: Option<Vec<String>>,
    pub hidden: Option<Vec<String>>,
    #[serde(default)]
    pub soft_delete: bool,
    #[serde(default)]
    pub disable_id: bool,
    #[serde(default)]
    pub disable_timestamps: bool,
}

#[derive(Debug, Clone)]
pub struct FieldSpec {
    pub name: String,
    pub ty: String,
    pub serde_attr: Option<&'static str>,
    pub special_type: Option<SpecialType>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpecialType {
    Hashed,
}

// Enum specification from TOML
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum EnumOrOther {
    Enum(EnumSpec),
    Other(serde_json::Value), // Ignore non-enum sections
}

#[derive(Debug, Clone, Deserialize)]
pub struct EnumSpec {
    #[serde(rename = "type")]
    pub type_name: String, // Should be "enum"
    pub storage: String, // "string", "i16", "i32", etc.
    pub variants: EnumVariants,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum EnumVariants {
    Simple(Vec<String>),        // ["Admin", "User"]
    Explicit(Vec<EnumVariant>), // [{name: "Pending", value: 0}]
}

#[derive(Debug, Clone, Deserialize)]
pub struct EnumVariant {
    pub name: String,
    pub value: serde_json::Value, // String or Number
}

#[derive(Debug, Clone)]
pub struct MetaFieldSpec {
    pub name: String,
    pub ty: MetaType,
}

#[derive(Debug, Clone)]
pub enum MetaType {
    String,
    Bool,
    I32,
    I64,
    F64,
    Json,
    DateTime,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct AttachmentFieldSpec {
    pub name: String,
    pub typ: String,
    pub multiple: bool,
}

#[derive(Debug, Clone)]
pub struct ComputedFieldSpec {
    pub name: String,
    pub ty: String,
}

#[derive(Debug, Clone)]
pub enum RelationKind {
    BelongsTo,
    HasMany,
}

#[derive(Debug, Clone)]
pub struct RelationSpec {
    pub name: String,
    pub kind: RelationKind,
    pub target_model: String,
    pub foreign_key: String,
    pub local_key: String,
    pub target_table: String,
    pub target_pk: String,
    pub target_pk_ty: String,
}

pub fn load(path_str: &str) -> Result<Schema, Box<dyn Error>> {
    let path = std::path::Path::new(path_str);
    if path.is_dir() {
        let mut master = Schema {
            models: BTreeMap::new(),
            extra_sections: BTreeMap::new(),
        };
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let p = entry.path();
            if p.is_file() && p.extension().map_or(false, |e| e == "toml") {
                let raw = fs::read_to_string(&p)?;
                let partial: Schema = toml::from_str(&raw)?;
                for (k, v) in partial.models {
                    if master.models.contains_key(&k) {
                        panic!("Duplicate model definition '{}' in {:?}", k, p);
                    }
                    master.models.insert(k, v);
                }
                // Merge enum definitions
                for (k, v) in partial.extra_sections {
                    if master.extra_sections.contains_key(&k) {
                        panic!("Duplicate enum definition '{}' in {:?}", k, p);
                    }
                    master.extra_sections.insert(k, v);
                }
            }
        }
        Ok(master)
    } else {
        let raw = fs::read_to_string(path)?;
        let schema: Schema = toml::from_str(&raw)?;
        Ok(schema)
    }
}

pub fn to_owner_type(s: &str) -> String {
    s.trim().to_lowercase()
}

pub fn to_snake(s: &str) -> String {
    let s = s.trim();
    let mut out = String::new();
    for (i, ch) in s.chars().enumerate() {
        if ch.is_ascii_uppercase() {
            if i != 0 {
                out.push('_');
            }
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push(ch);
        }
    }
    out
}

pub fn to_title_case(s: &str) -> String {
    let mut out = String::new();
    for part in s.split('_').filter(|p| !p.is_empty()) {
        let mut chars = part.chars();
        if let Some(first) = chars.next() {
            out.push(first.to_ascii_uppercase());
            out.push_str(&chars.as_str().to_ascii_lowercase());
        }
    }
    out
}

pub fn to_label(s: &str) -> String {
    s.split('_')
        .filter(|p| !p.is_empty())
        .map(|w| {
            if w.eq_ignore_ascii_case("id") {
                "ID".to_string()
            } else {
                let mut c = w.chars();
                match c.next() {
                    None => String::new(),
                    Some(f) => {
                        let mut out = f.to_ascii_uppercase().to_string();
                        out.push_str(c.as_str());
                        out
                    }
                }
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn parse_fields(cfg: &ModelSpec, pk: &str) -> Vec<FieldSpec> {
    let mut out = Vec::new();
    if let Some(list) = &cfg.fields {
        for raw in list {
            out.push(parse_field(raw));
        }
    }

    let mut seen: BTreeSet<String> = out.iter().map(|f| f.name.clone()).collect();

    if !cfg.disable_id && !seen.contains(pk) {
        let pk_ty = cfg
            .pk_type
            .as_deref()
            .map(normalize_type)
            .unwrap_or_else(|| "i64".to_string());
        out.insert(
            0,
            FieldSpec {
                name: pk.to_string(),
                ty: pk_ty,
                serde_attr: None,
                special_type: None,
            },
        );
        seen.insert(pk.to_string());
    }

    if !cfg.disable_timestamps {
        if !seen.contains("created_at") {
            out.push(FieldSpec {
                name: "created_at".into(),
                ty: "time::OffsetDateTime".into(),
                serde_attr: Some("#[serde(with = \"time::serde::rfc3339\")]"),
                special_type: None,
            });
            seen.insert("created_at".into());
        }
        if !seen.contains("updated_at") {
            out.push(FieldSpec {
                name: "updated_at".into(),
                ty: "time::OffsetDateTime".into(),
                serde_attr: Some("#[serde(with = \"time::serde::rfc3339\")]"),
                special_type: None,
            });
        }
    }

    if cfg.soft_delete && !seen.contains("deleted_at") {
        out.push(FieldSpec {
            name: "deleted_at".into(),
            ty: "Option<time::OffsetDateTime>".into(),
            serde_attr: Some("#[serde(with = \"time::serde::rfc3339::option\")]"),
            special_type: None,
        });
    }

    out
}

pub fn parse_meta(cfg: &ModelSpec) -> Vec<MetaFieldSpec> {
    let mut out = Vec::new();
    let Some(list) = &cfg.meta else {
        return out;
    };
    for raw in list {
        let parts = raw.splitn(2, ':').collect::<Vec<_>>();
        if parts.len() != 2 {
            panic!("Invalid meta spec '{}'. Use name:type", raw);
        }
        let name = to_snake(parts[0].trim());
        let ty_raw = parts[1].trim();
        let ty_lower = ty_raw.to_ascii_lowercase();
        let ty = match ty_lower.as_str() {
            "string" => MetaType::String,
            "bool" => MetaType::Bool,
            "i32" => MetaType::I32,
            "i64" => MetaType::I64,
            "f64" => MetaType::F64,
            "json" => MetaType::Json,
            "datetime" => MetaType::DateTime,
            _ => {
                let looks_like_type_path = ty_raw
                    .chars()
                    .next()
                    .map(|c| c.is_ascii_uppercase())
                    .unwrap_or(false)
                    || ty_raw.contains("::");
                if looks_like_type_path {
                    MetaType::Custom(ty_raw.to_string())
                } else {
                    panic!("Unknown meta type '{}' in '{}'", ty_raw, raw);
                }
            }
        };
        out.push(MetaFieldSpec { name, ty });
    }
    out
}

pub fn parse_field(raw: &str) -> FieldSpec {
    let mut parts = raw.splitn(2, ':').collect::<Vec<_>>();
    if parts.len() != 2 {
        panic!(
            "Invalid field spec '{}'. Use name:Type or name:datetime.",
            raw
        );
    }
    let name = parts.remove(0).trim().to_string();
    let ty_raw = parts.remove(0).trim();

    let (ty, serde_attr, special_type) = match ty_raw {
        "datetime" => (
            "time::OffsetDateTime".to_string(),
            Some("#[serde(with = \"time::serde::rfc3339\")]"),
            None,
        ),
        "uuid" | "Uuid" => ("uuid::Uuid".to_string(), None, None),
        "hashed" => ("String".to_string(), None, Some(SpecialType::Hashed)), // "hashed" -> String in Db, but logic added
        other => (normalize_type(other), None, None),
    };

    FieldSpec {
        name,
        ty,
        serde_attr,
        special_type,
    }
}

fn normalize_type(raw: &str) -> String {
    match raw.trim() {
        "uuid" | "Uuid" => "uuid::Uuid".to_string(),
        "string" => "String".to_string(),
        other => other.to_string(),
    }
}

pub fn parse_relations(
    schema: &Schema,
    cfg: &ModelSpec,
    model_name: &str,
    _fields: &[FieldSpec],
) -> Vec<RelationSpec> {
    let mut out = Vec::new();
    let Some(list) = &cfg.relations else {
        return out;
    };
    for raw in list {
        let parts: Vec<&str> = raw.split(':').collect();
        if parts.len() < 5 {
            panic!(
                "Invalid relation '{}'. Expected name:kind:target_model:foreign_key:local_key",
                raw
            );
        }
        let name = parts[0].trim().to_string();
        let kind = match parts[1].trim() {
            "belongs_to" => RelationKind::BelongsTo,
            "has_many" => RelationKind::HasMany,
            other => panic!("Unknown relation kind '{}' in '{}'", other, raw),
        };
        let target_model = parts[2].trim().to_string();
        let foreign_key = parts[3].trim().to_string();
        let local_key = parts[4].trim().to_string();

        let target_cfg = schema.models.get(&target_model).unwrap_or_else(|| {
            panic!(
                "Relation '{}' references unknown model '{}'",
                raw, target_model
            )
        });
        let target_table = target_cfg
            .table
            .as_ref()
            .map(|s| s.to_string())
            .unwrap_or_else(|| to_snake(&target_model));
        let target_pk = target_cfg
            .pk
            .as_ref()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "id".to_string());
        let target_fields = parse_fields(target_cfg, &target_pk);
        let target_pk_ty = target_fields
            .iter()
            .find(|f| f.name == target_pk)
            .map(|f| f.ty.clone())
            .unwrap_or_else(|| "i64".to_string());

        out.push(RelationSpec {
            name,
            kind,
            target_model,
            foreign_key,
            local_key,
            target_table,
            target_pk,
            target_pk_ty,
        });
    }

    // warn for obvious mistakes
    let mut seen: BTreeSet<String> = BTreeSet::new();
    for r in &out {
        if !seen.insert(r.name.clone()) {
            panic!(
                "Duplicate relation name '{}' on model '{}'",
                r.name, model_name
            );
        }
    }

    out
}

pub fn parse_attachments(cfg: &ModelSpec) -> Vec<AttachmentFieldSpec> {
    let mut out = Vec::new();
    if let Some(list) = &cfg.attachment {
        for raw in list {
            out.push(parse_attachment_field(raw, false));
        }
    }
    if let Some(list) = &cfg.attachments {
        for raw in list {
            out.push(parse_attachment_field(raw, true));
        }
    }
    out
}

fn parse_attachment_field(raw: &str, multiple: bool) -> AttachmentFieldSpec {
    let parts: Vec<&str> = raw.splitn(2, ':').collect();
    if parts.len() != 2 {
        panic!("Invalid attachment spec '{}'. Use name:type", raw);
    }
    let name = to_snake(parts[0].trim());
    let typ = parts[1].trim().to_string();
    AttachmentFieldSpec {
        name,
        typ,
        multiple,
    }
}

pub fn parse_computed(cfg: &ModelSpec) -> Vec<ComputedFieldSpec> {
    let mut out = Vec::new();
    let Some(list) = &cfg.computed else {
        return out;
    };
    for raw in list {
        let parts: Vec<&str> = raw.splitn(2, ':').collect();
        if parts.len() != 2 {
            panic!("Invalid computed spec '{}'. Use name:Type", raw);
        }
        let name = to_snake(parts[0].trim());
        let ty_raw = parts[1].trim();
        let val_type = normalize_type(ty_raw);
        out.push(ComputedFieldSpec { name, ty: val_type });
    }
    out
}
