use crate::config::{ConfigsFile, Locales};
use crate::schema::{parse_attachments, to_owner_type, to_snake, to_title_case, Schema};
use crate::template::{render_template, TemplateContext};
use std::collections::BTreeSet;
use std::error::Error;
use std::fs;

pub fn generate_localized(
    locales: &Locales,
    cfgs: &ConfigsFile,
    schema: &Schema,
    out_dir: &std::path::Path,
) -> Result<(), Box<dyn Error>> {
    let has_loader_functions = schema.models.values().any(|cfg| {
        cfg.localized.is_some() || cfg.meta.is_some() || !parse_attachments(cfg).is_empty()
    });
    let needs_resize_rule = cfgs
        .attachment_types
        .values()
        .any(|attachment_type| attachment_type.resize.is_some());

    let mut context = TemplateContext::new();
    context.insert(
        "imports",
        render_imports(has_loader_functions, needs_resize_rule),
    )?;
    context.insert("default_locale", locales.default.clone())?;
    context.insert(
        "default_timezone",
        locales
            .timezone
            .clone()
            .unwrap_or_else(|| "+00:00".to_string()),
    )?;
    context.insert("supported_locales", render_supported_locales(locales))?;
    context.insert("locale_variants", render_locale_variants(locales))?;
    context.insert(
        "loader_functions_section",
        render_loader_functions_section(has_loader_functions),
    )?;
    context.insert("locale_impls_section", render_locale_impls_section(locales))?;
    context.insert(
        "localized_text_section",
        render_localized_text_section(locales),
    )?;
    context.insert(
        "localized_input_section",
        render_localized_input_section(locales),
    )?;
    context.insert(
        "model_localized_section",
        render_model_localized_section(locales, schema),
    )?;
    context.insert(
        "attachment_rules_section",
        render_attachment_rules_section(cfgs),
    )?;
    context.insert("meta_helpers_section", render_meta_helpers_section(schema))?;
    context.insert(
        "attachment_helpers_section",
        render_attachment_helpers_section(schema),
    )?;

    let rendered = render_template("localized/file.rs.tpl", &context)?;
    fs::write(out_dir.join("localized.rs"), rendered)?;
    Ok(())
}

fn render_imports(has_loader_functions: bool, needs_resize_rule: bool) -> String {
    let mut out = String::new();
    if has_loader_functions {
        out.push_str("use anyhow::Result;\n");
        out.push_str("use std::collections::HashMap;\n");
        out.push_str("use core_db::common::sql::{generate_snowflake_i64, DbConn, Op};\n");
        out.push_str(
            "use crate::generated::models::attachment::{Attachment as AttachmentModel, AttachmentCol};\n",
        );
        out.push_str(
            "use crate::generated::models::localized::{Localized as LocalizedModel, LocalizedCol};\n",
        );
        out.push_str("use crate::generated::models::meta::{Meta as MetaModel, MetaCol};\n");
        out.push_str("use uuid::Uuid;\n");
    }
    if needs_resize_rule {
        out.push_str("use core_db::platform::attachments::types::{AttachmentRules, ResizeRule};\n");
    } else {
        out.push_str("use core_db::platform::attachments::types::AttachmentRules;\n");
    }
    if !out.is_empty() {
        out.push('\n');
    }
    out
}

fn render_supported_locales(locales: &Locales) -> String {
    locales
        .supported
        .iter()
        .map(|lang| format!("    \"{lang}\","))
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_locale_variants(locales: &Locales) -> String {
    locales
        .supported
        .iter()
        .map(|lang| {
            let variant = to_title_case(lang);
            format!("    #[serde(rename = \"{lang}\")]\n    {variant},")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_upsert_localized_many_function() -> String {
    String::from(
        r#"pub async fn upsert_localized_many<'a>(
    db: DbConn<'a>,
    owner_type: &str,
    owner_id: i64,
    field: &str,
    values: &HashMap<String, String>,
) -> Result<()> {
    for (locale, value) in values {
        let current = LocalizedModel::new(db.clone(), None)
            .query()
            .where_owner_type(Op::Eq, owner_type.to_string())
            .where_owner_id(Op::Eq, owner_id)
            .where_field(Op::Eq, field.to_string())
            .where_locale(Op::Eq, locale.clone())
            .first()
            .await?;
        if let Some(current) = current {
            LocalizedModel::new(db.clone(), None)
                .update()
                .where_id(Op::Eq, current.id)
                .set_value(value.clone())
                .save()
                .await?;
        } else {
            LocalizedModel::new(db.clone(), None)
                .insert()
                .set_id(generate_snowflake_i64())
                .set_owner_type(owner_type.to_string())
                .set_owner_id(owner_id)
                .set_field(field.to_string())
                .set_locale(locale.clone())
                .set_value(value.clone())
                .save()
                .await?;
        }
    }
    Ok(())
}

"#,
    )
}

fn render_load_owner_localized_function() -> String {
    String::from(
        r#"pub async fn load_owner_localized<'a>(
    db: DbConn<'a>,
    owner_type: &str,
    ids: &[i64],
    fields: &[&str],
) -> Result<core_db::platform::localized::types::LocalizedMap> {
    if ids.is_empty() || fields.is_empty() {
        return Ok(core_db::platform::localized::types::LocalizedMap::default());
    }
    let owner_ids: Vec<i64> = ids.to_vec();
    let field_names: Vec<String> = fields.iter().map(|field| (*field).to_string()).collect();
    let rows = LocalizedModel::new(db, None)
        .query()
        .where_owner_type(Op::Eq, owner_type.to_string())
        .where_in(LocalizedCol::OwnerId, &owner_ids)
        .where_in(LocalizedCol::Field, &field_names)
        .get()
        .await?;
    let mut out: HashMap<String, HashMap<i64, HashMap<String, String>>> = HashMap::new();
    for row in rows {
        out.entry(row.field.clone())
            .or_default()
            .entry(row.owner_id)
            .or_default()
            .insert(row.locale.clone(), row.value.clone());
    }
    Ok(core_db::platform::localized::types::LocalizedMap::new(out))
}

"#,
    )
}

fn render_upsert_meta_many_function() -> String {
    String::from(
        r#"pub async fn upsert_meta_many<'a>(
    db: DbConn<'a>,
    owner_type: &str,
    owner_id: i64,
    values: &HashMap<String, serde_json::Value>,
) -> Result<()> {
    for (field, value) in values {
        let current = MetaModel::new(db.clone(), None)
            .query()
            .where_owner_type(Op::Eq, owner_type.to_string())
            .where_owner_id(Op::Eq, owner_id)
            .where_field(Op::Eq, field.clone())
            .first()
            .await?;
        if let Some(current) = current {
            MetaModel::new(db.clone(), None)
                .update()
                .where_id(Op::Eq, current.id)
                .set_value(value.clone())
                .save()
                .await?;
        } else {
            MetaModel::new(db.clone(), None)
                .insert()
                .set_id(generate_snowflake_i64())
                .set_owner_type(owner_type.to_string())
                .set_owner_id(owner_id)
                .set_field(field.clone())
                .set_value(value.clone())
                .save()
                .await?;
        }
    }
    Ok(())
}

"#,
    )
}

fn render_load_owner_meta_function() -> String {
    String::from(
        r#"pub async fn load_owner_meta<'a>(
    db: DbConn<'a>,
    owner_type: &str,
    ids: &[i64],
    fields: &[&str],
) -> Result<core_db::platform::meta::types::MetaMap> {
    if ids.is_empty() || fields.is_empty() {
        return Ok(core_db::platform::meta::types::MetaMap::default());
    }
    let owner_ids: Vec<i64> = ids.to_vec();
    let field_names: Vec<String> = fields.iter().map(|field| (*field).to_string()).collect();
    let rows = MetaModel::new(db, None)
        .query()
        .where_owner_type(Op::Eq, owner_type.to_string())
        .where_in(MetaCol::OwnerId, &owner_ids)
        .where_in(MetaCol::Field, &field_names)
        .get()
        .await?;
    let mut out: HashMap<String, HashMap<i64, serde_json::Value>> = HashMap::new();
    for row in rows {
        out.entry(row.field.clone())
            .or_default()
            .insert(row.owner_id, row.value.clone());
    }
    Ok(core_db::platform::meta::types::MetaMap::new(out))
}

"#,
    )
}

fn render_clear_attachment_field_function() -> String {
    String::from(
        r#"pub async fn clear_attachment_field<'a>(
    db: DbConn<'a>,
    owner_type: &str,
    owner_id: i64,
    field: &str,
) -> Result<()> {
    let rows = AttachmentModel::new(db.clone(), None)
        .query()
        .where_owner_type(Op::Eq, owner_type.to_string())
        .where_owner_id(Op::Eq, owner_id)
        .where_field(Op::Eq, field.to_string())
        .get()
        .await?;
    for row in rows {
        AttachmentModel::new(db.clone(), None)
            .query()
            .where_id(Op::Eq, row.id)
            .delete()
            .await?;
    }
    Ok(())
}

"#,
    )
}

fn render_add_attachments_function() -> String {
    String::from(
        r#"pub async fn add_attachments<'a>(
    db: DbConn<'a>,
    owner_type: &str,
    owner_id: i64,
    field: &str,
    values: &[core_db::platform::attachments::types::AttachmentInput],
) -> Result<()> {
    for value in values {
        let now = time::OffsetDateTime::now_utc();
        let attachment_id = value.id.unwrap_or_else(Uuid::new_v4);
        AttachmentModel::new(db.clone(), None)
            .insert()
            .set_id(attachment_id)
            .set_owner_type(owner_type.to_string())
            .set_owner_id(owner_id)
            .set_field(field.to_string())
            .set_path(value.path.clone())
            .set_content_type(value.content_type.clone())
            .set_size(value.size)
            .set_width(value.width)
            .set_height(value.height)
            .set_created_at(now)
            .set_updated_at(now)
            .save()
            .await?;
    }
    Ok(())
}

"#,
    )
}

fn render_replace_single_attachment_function() -> String {
    String::from(
        r#"pub async fn replace_single_attachment<'a>(
    db: DbConn<'a>,
    owner_type: &str,
    owner_id: i64,
    field: &str,
    value: &core_db::platform::attachments::types::AttachmentInput,
) -> Result<()> {
    clear_attachment_field(db.clone(), owner_type, owner_id, field).await?;
    add_attachments(db, owner_type, owner_id, field, std::slice::from_ref(value)).await
}

"#,
    )
}

fn render_delete_attachment_ids_function() -> String {
    String::from(
        r#"pub async fn delete_attachment_ids<'a>(
    db: DbConn<'a>,
    owner_type: &str,
    owner_id: i64,
    field: &str,
    ids: &[Uuid],
) -> Result<()> {
    for id in ids {
        AttachmentModel::new(db.clone(), None)
            .query()
            .where_id(Op::Eq, *id)
            .where_owner_type(Op::Eq, owner_type.to_string())
            .where_owner_id(Op::Eq, owner_id)
            .where_field(Op::Eq, field.to_string())
            .delete()
            .await?;
    }
    Ok(())
}

"#,
    )
}

fn render_load_owner_attachments_function() -> String {
    String::from(
        r#"pub async fn load_owner_attachments<'a>(
    db: DbConn<'a>,
    owner_type: &str,
    ids: &[i64],
    fields: &[&str],
) -> Result<core_db::platform::attachments::types::AttachmentMap> {
    if ids.is_empty() || fields.is_empty() {
        return Ok(core_db::platform::attachments::types::AttachmentMap::default());
    }
    let owner_ids: Vec<i64> = ids.to_vec();
    let field_names: Vec<String> = fields.iter().map(|field| (*field).to_string()).collect();
    let rows = AttachmentModel::new(db, None)
        .query()
        .where_owner_type(Op::Eq, owner_type.to_string())
        .where_in(AttachmentCol::OwnerId, &owner_ids)
        .where_in(AttachmentCol::Field, &field_names)
        .get()
        .await?;
    let mut out: HashMap<String, HashMap<i64, Vec<core_db::platform::attachments::types::Attachment>>> = HashMap::new();
    for wr in rows {
        let row = wr.into_row();
        out.entry(row.field.clone())
            .or_default()
            .entry(row.owner_id)
            .or_default()
            .push(core_db::platform::attachments::types::Attachment {
                id: row.id,
                path: row.path.clone(),
                url: core_db::platform::attachments::types::attachment_url(&row.path, None),
                content_type: row.content_type,
                size: row.size,
                width: row.width,
                height: row.height,
                created_at: row.created_at,
            });
    }
    Ok(core_db::platform::attachments::types::AttachmentMap::new(out))
}

"#,
    )
}

fn render_locale_match_arms(locales: &Locales) -> String {
    locales
        .supported
        .iter()
        .map(|lang| {
            let variant = to_title_case(lang);
            format!("            Locale::{variant} => \"{lang}\",")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_localized_text_fields(locales: &Locales) -> String {
    locales
        .supported
        .iter()
        .map(|lang| format!("    pub {lang}: String,"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_localized_text_get_arms(locales: &Locales) -> String {
    locales
        .supported
        .iter()
        .map(|lang| format!("            \"{lang}\" => &self.{lang},"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_localized_text_to_map_lines(locales: &Locales) -> String {
    locales
        .supported
        .iter()
        .map(|lang| format!("        out.insert(\"{lang}\".to_string(), self.{lang}.clone());"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_localized_text_from_map_fields(locales: &Locales) -> String {
    locales
        .supported
        .iter()
        .map(|lang| {
            format!("            {lang}: map.get(\"{lang}\").cloned().unwrap_or_default(),")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_localized_text_empty_fields(locales: &Locales) -> String {
    locales
        .supported
        .iter()
        .map(|lang| format!("            {lang}: String::new(),"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_localized_text_assign_arms(locales: &Locales) -> String {
    locales
        .supported
        .iter()
        .map(|lang| format!("                    \"{lang}\" => out.{lang} = v.clone(),"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_localized_text_default_fill_lines(locales: &Locales) -> String {
    locales
        .supported
        .iter()
        .map(|lang| {
            format!("            if out.{lang}.is_empty() {{ out.{lang} = default_val.clone(); }}")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_localized_input_section(locales: &Locales) -> String {
    let fields = locales
        .supported
        .iter()
        .map(|lang| format!("    #[serde(default)]\n    pub {lang}: Option<String>,"))
        .collect::<Vec<_>>()
        .join("\n");
    let to_map_lines = locales
        .supported
        .iter()
        .map(|lang| {
            format!(
                "        if let Some(v) = self.{lang} {{ out.insert(\"{lang}\".to_string(), v); }}"
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let is_empty_checks = locales
        .supported
        .iter()
        .map(|lang| format!("self.{lang}.is_none()"))
        .collect::<Vec<_>>()
        .join(" && ");
    let default_locale = &locales.default;
    format!(
        r#"#[derive(Debug, Clone, Default, serde::Deserialize, schemars::JsonSchema)]
pub struct LocalizedInput {{
{fields}
}}

impl LocalizedInput {{
    /// Convert to HashMap, keeping only non-None values.
    pub fn to_hashmap(self) -> std::collections::HashMap<String, String> {{
        let mut out = std::collections::HashMap::new();
{to_map_lines}
        out
    }}

    /// Returns true if all locale values are None.
    pub fn is_empty(&self) -> bool {{
        {is_empty_checks}
    }}
}}

impl validator::Validate for LocalizedInput {{
    fn validate(&self) -> Result<(), validator::ValidationErrors> {{
        let mut errors = validator::ValidationErrors::new();
        if self.{default_locale}.as_ref().map_or(true, |s| s.is_empty()) {{
            errors.add(
                "{default_locale}",
                validator::ValidationError::new("required")
                    .with_message(std::borrow::Cow::Borrowed("Default locale value is required.")),
            );
        }}
        if errors.is_empty() {{ Ok(()) }} else {{ Err(errors) }}
    }}
}}

impl ts_rs::TS for LocalizedInput {{
    type WithoutGenerics = Self;
    fn name() -> String {{ "LocalizedInput".to_string() }}
    fn inline() -> String {{ Self::name() }}
    fn inline_flattened() -> String {{ panic!("LocalizedInput cannot be flattened") }}
    fn decl() -> String {{ panic!("LocalizedInput declaration is provided by shared platform types") }}
    fn decl_concrete() -> String {{ Self::decl() }}
}}

"#,
        fields = fields,
        to_map_lines = to_map_lines,
        is_empty_checks = is_empty_checks,
        default_locale = default_locale,
    )
}

fn render_loader_functions_section(has_loader_functions: bool) -> String {
    if !has_loader_functions {
        return String::new();
    }

    let mut out = String::new();
    out.push_str(&render_upsert_localized_many_function());
    out.push_str(&render_load_owner_localized_function());
    out.push_str(&render_upsert_meta_many_function());
    out.push_str(&render_load_owner_meta_function());
    out.push_str(&render_clear_attachment_field_function());
    out.push_str(&render_add_attachments_function());
    out.push_str(&render_replace_single_attachment_function());
    out.push_str(&render_delete_attachment_ids_function());
    out.push_str(&render_load_owner_attachments_function());
    out
}

fn render_locale_impls_section(locales: &Locales) -> String {
    format!(
        "impl Locale {{\n    pub fn as_str(&self) -> &'static str {{\n        match self {{\n{match_arms}\n        }}\n    }}\n}}\n\nimpl From<Locale> for String {{\n    fn from(l: Locale) -> Self {{\n        l.as_str().to_string()\n    }}\n}}\n\nimpl std::fmt::Display for Locale {{\n    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {{\n        write!(f, \"{{}}\", self.as_str())\n    }}\n}}\n\n",
        match_arms = render_locale_match_arms(locales),
    )
}

fn render_localized_text_section(locales: &Locales) -> String {
    format!(
        "#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]\npub struct LocalizedText {{\n{fields}\n}}\n\nimpl LocalizedText {{\n    pub fn get(&self, locale: &str) -> &str {{\n        match locale {{\n{get_arms}\n            _ => &self.{default_locale},\n        }}\n    }}\n    pub fn to_map(&self) -> std::collections::BTreeMap<String, String> {{\n        let mut out = std::collections::BTreeMap::new();\n{to_map_lines}\n        out\n    }}\n    pub fn from_map(map: &std::collections::BTreeMap<String, String>) -> Self {{\n        Self {{\n{from_map_fields}\n        }}\n    }}\n}}\n\nimpl ts_rs::TS for LocalizedText {{\n    type WithoutGenerics = Self;\n    fn name() -> String {{ \"LocalizedText\".to_string() }}\n    fn inline() -> String {{ Self::name() }}\n    fn inline_flattened() -> String {{ panic!(\"LocalizedText cannot be flattened\") }}\n    fn decl() -> String {{ panic!(\"LocalizedText declaration is provided by shared platform types\") }}\n    fn decl_concrete() -> String {{ Self::decl() }}\n}}\n\npub trait LocalizedMapHelper {{\n    fn get_localized_text(&self, field: &str, owner_id: i64) -> Option<LocalizedText>;\n}}\n\nimpl LocalizedMapHelper for core_db::platform::localized::types::LocalizedMap {{\n    fn get_localized_text(&self, field: &str, owner_id: i64) -> Option<LocalizedText> {{\n        let by_owner = self.inner.get(field)?;\n        let by_locale = by_owner.get(&owner_id)?;\n        if by_locale.is_empty() {{ return None; }}\n        let mut out = LocalizedText {{\n{empty_fields}\n        }};\n        for &loc in SUPPORTED_LOCALES {{\n            if let Some(v) = by_locale.get(loc) {{\n                match loc {{\n{assign_arms}\n                    _ => {{}}\n                }}\n            }}\n        }}\n        if let Some(default_val) = by_locale.get(DEFAULT_LOCALE) {{\n{default_fill_lines}\n        }}\n        Some(out)\n    }}\n}}\n\n",
        fields = render_localized_text_fields(locales),
        get_arms = render_localized_text_get_arms(locales),
        default_locale = locales.default,
        to_map_lines = render_localized_text_to_map_lines(locales),
        from_map_fields = render_localized_text_from_map_fields(locales),
        empty_fields = render_localized_text_empty_fields(locales),
        assign_arms = render_localized_text_assign_arms(locales),
        default_fill_lines = render_localized_text_default_fill_lines(locales),
    )
}

fn render_model_localized_section(locales: &Locales, schema: &Schema) -> String {
    let mut out = String::new();
    let mut owner_consts: BTreeSet<String> = BTreeSet::new();

    for (name, cfg) in &schema.models {
        let Some(localized_fields) = &cfg.localized else {
            continue;
        };
        let model_snake = to_snake(name);
        let model_title = to_title_case(&model_snake);
        let model_const = model_snake.to_uppercase();
        let owner_type = to_owner_type(name);

        if owner_consts.insert(model_const.clone()) {
            out.push_str(&format!(
                "pub const {model_const}_OWNER_TYPE: &str = \"{owner_type}\";\n"
            ));
        }
        out.push_str(&format!("pub const {model_const}_FIELDS: &[&str] = &[\n"));
        for field in localized_fields {
            out.push_str(&format!("    \"{field}\",\n"));
        }
        out.push_str("];\n\n");
        out.push_str(&format!(
            "pub async fn load_{model_snake}_localized<'a>(db: DbConn<'a>, ids: &[i64]) -> Result<core_db::platform::localized::types::LocalizedMap> {{\n"
        ));
        out.push_str(&format!(
            "    load_owner_localized(db, {model_const}_OWNER_TYPE, ids, {model_const}_FIELDS).await\n"
        ));
        out.push_str("}\n\n");

        let trait_name = format!("{}Localized", model_title);
        out.push_str(&format!("pub trait {trait_name} {{\n"));
        for field in localized_fields {
            let field_snake = to_snake(field);
            out.push_str(&format!("    fn {model_snake}_{field_snake}_translations(&self, id: i64) -> Option<crate::generated::LocalizedText>;\n"));
            out.push_str(&format!(
                "    fn {model_snake}_{field_snake}(&self, id: i64) -> Option<String>;\n"
            ));
        }
        out.push_str("}\n\n");

        out.push_str(&format!(
            "impl {trait_name} for core_db::platform::localized::types::LocalizedMap {{\n"
        ));
        for field in localized_fields {
            let field_snake = to_snake(field);
            out.push_str(&format!(
                "    fn {model_snake}_{field_snake}_translations(&self, id: i64) -> Option<crate::generated::LocalizedText> {{\n"
            ));
            out.push_str(&format!(
                "        self.get_localized_text(\"{field}\", id)\n"
            ));
            out.push_str("    }\n");
            out.push_str(&format!(
                "    fn {model_snake}_{field_snake}(&self, id: i64) -> Option<String> {{\n"
            ));
            out.push_str("        let locale = core_i18n::current_locale();\n");
            out.push_str(&format!(
                "        self.get_value(\"{field}\", id, locale)\n"
            ));
            out.push_str("    }\n");
        }
        out.push_str("}\n\n");
    }

    if out.is_empty() {
        return out;
    }
    let _ = locales;
    out
}

fn render_attachment_rules_section(cfgs: &ConfigsFile) -> String {
    let mut out = String::new();
    out.push_str("pub fn get_attachment_rules(name: &str) -> Option<AttachmentRules> {\n");
    out.push_str("    match name {\n");
    for (name, attachment_type) in &cfgs.attachment_types {
        out.push_str(&format!("        \"{name}\" => Some(AttachmentRules {{\n"));
        out.push_str("            allowed: vec![\n");
        for allowed in &attachment_type.allowed {
            out.push_str(&format!("                \"{allowed}\".to_string(),\n"));
        }
        out.push_str("            ],\n");
        if let Some(resize) = &attachment_type.resize {
            out.push_str("            resize: Some(ResizeRule {\n");
            match resize.width {
                Some(width) => out.push_str(&format!("                width: Some({width}),\n")),
                None => out.push_str("                width: None,\n"),
            }
            match resize.height {
                Some(height) => out.push_str(&format!("                height: Some({height}),\n")),
                None => out.push_str("                height: None,\n"),
            }
            match resize.quality {
                Some(quality) => {
                    out.push_str(&format!("                quality: Some({quality}),\n"))
                }
                None => out.push_str("                quality: None,\n"),
            }
            out.push_str("            }),\n");
        } else {
            out.push_str("            resize: None,\n");
        }
        out.push_str("        }),\n");
    }
    out.push_str("        _ => None,\n");
    out.push_str("    }\n");
    out.push_str("}\n\n");
    out
}

fn render_meta_helpers_section(schema: &Schema) -> String {
    let mut out = String::new();
    let mut owner_consts: BTreeSet<String> = BTreeSet::new();
    for (name, cfg) in &schema.models {
        let Some(meta_fields) = &cfg.meta else {
            continue;
        };
        let model_snake = to_snake(name);
        let model_const = model_snake.to_uppercase();
        let owner_type = to_owner_type(name);
        if cfg.localized.is_none() && owner_consts.insert(model_const.clone()) {
            out.push_str(&format!(
                "pub const {model_const}_OWNER_TYPE: &str = \"{owner_type}\";\n"
            ));
        }
        out.push_str(&format!(
            "pub const {model_const}_META_FIELDS: &[&str] = &[\n"
        ));
        for field in meta_fields {
            let mut parts = field.splitn(2, ':');
            let field_name = parts.next().unwrap_or("");
            out.push_str(&format!("    \"{field_name}\",\n"));
        }
        out.push_str("];\n\n");
        out.push_str(&format!(
            "pub async fn load_{model_snake}_meta<'a>(db: DbConn<'a>, ids: &[i64]) -> Result<core_db::platform::meta::types::MetaMap> {{\n"
        ));
        out.push_str(&format!(
            "    load_owner_meta(db, {model_const}_OWNER_TYPE, ids, {model_const}_META_FIELDS).await\n"
        ));
        out.push_str("}\n\n");
    }
    out
}

fn render_attachment_helpers_section(schema: &Schema) -> String {
    let mut out = String::new();
    let mut owner_consts: BTreeSet<String> = BTreeSet::new();
    for (name, cfg) in &schema.models {
        let attachment_fields = parse_attachments(cfg);
        if attachment_fields.is_empty() {
            continue;
        }
        let model_snake = to_snake(name);
        let model_const = model_snake.to_uppercase();
        let owner_type = to_owner_type(name);
        if cfg.localized.is_none() && cfg.meta.is_none() && owner_consts.insert(model_const.clone())
        {
            out.push_str(&format!(
                "pub const {model_const}_OWNER_TYPE: &str = \"{owner_type}\";\n"
            ));
        }
        out.push_str(&format!(
            "pub const {model_const}_ATTACHMENT_FIELDS: &[&str] = &[\n"
        ));
        for field in &attachment_fields {
            out.push_str(&format!("    \"{}\",\n", field.name));
        }
        out.push_str("];\n\n");
        out.push_str(&format!(
            "pub async fn load_{model_snake}_attachments<'a>(db: DbConn<'a>, ids: &[i64]) -> Result<core_db::platform::attachments::types::AttachmentMap> {{\n"
        ));
        out.push_str(&format!(
            "    load_owner_attachments(db, {model_const}_OWNER_TYPE, ids, {model_const}_ATTACHMENT_FIELDS).await\n"
        ));
        out.push_str("}\n\n");
    }
    out
}
