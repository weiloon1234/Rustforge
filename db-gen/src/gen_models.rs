use crate::config::ConfigsFile;
use crate::schema::{
    parse_attachments, parse_computed, parse_fields, parse_meta, parse_relations, to_snake,
    to_title_case, AttachmentFieldSpec, EnumOrOther, EnumSpec, FieldSpec, MetaFieldSpec, MetaType,
    ModelSpec, RelationKind, RelationSpec, Schema, SpecialType,
};
use crate::template::{render_template, TemplateContext};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::Write;
use std::fs;
use std::path::Path;

const DATATABLE_REL_FILTER_MAX_DEPTH: usize = 2;

#[derive(Debug, Clone, Copy)]
pub struct GenerateModelsOptions {
    pub include_datatable: bool,
    pub include_extensions_imports: bool,
}

impl Default for GenerateModelsOptions {
    fn default() -> Self {
        Self {
            include_datatable: true,
            include_extensions_imports: true,
        }
    }
}

#[derive(Debug, Clone)]
struct RelationPathSpec {
    path: Vec<String>,
    target_model: String,
}

#[derive(Debug, Clone)]
struct EnumExplainedFieldSpec {
    name: String,
    explained_name: String,
    optional: bool,
}

fn parse_option_inner_type(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if !trimmed.starts_with("Option<") || !trimmed.ends_with('>') {
        return None;
    }
    let inner = trimmed
        .strip_prefix("Option<")
        .and_then(|value| value.strip_suffix('>'))?
        .trim();
    if inner.is_empty() {
        None
    } else {
        Some(inner.to_string())
    }
}

fn enum_explained_field_name(field_name: &str) -> String {
    let normalized = field_name.trim_end_matches('_');
    format!("{normalized}_explained")
}

fn enum_field_spec(
    field_name: &str,
    field_type: &str,
    enum_type_names: &BTreeSet<String>,
) -> Option<EnumExplainedFieldSpec> {
    if enum_type_names.contains(field_type.trim()) {
        return Some(EnumExplainedFieldSpec {
            name: field_name.to_string(),
            explained_name: enum_explained_field_name(field_name),
            optional: false,
        });
    }

    let inner = parse_option_inner_type(field_type)?;
    if !enum_type_names.contains(inner.as_str()) {
        return None;
    }

    Some(EnumExplainedFieldSpec {
        name: field_name.to_string(),
        explained_name: enum_explained_field_name(field_name),
        optional: true,
    })
}

fn create_input_ident(model_title: &str) -> String {
    format!("{model_title}Create")
}

fn update_changes_ident(model_title: &str) -> String {
    format!("{model_title}Changes")
}

/// Return the Rust type to use with `serde_json::from_value` for a given field type.
/// Enums are stored as strings, so we deserialize them as String (or Option<String>).
fn json_deser_type_for_field(ty: &str, enum_specs: &BTreeMap<String, EnumSpec>) -> String {
    let trimmed = ty.trim();
    if let Some(inner) = parse_option_inner_type(trimmed) {
        if enum_specs.contains_key(inner.as_str()) {
            return "Option<String>".to_string();
        }
        return format!("Option<{inner}>");
    }
    if enum_specs.contains_key(trimmed) {
        return "String".to_string();
    }
    trimmed.to_string()
}

fn bind_variant_for_type(ty: &str) -> Option<&'static str> {
    match ty.trim() {
        "i16" => Some("I16"),
        "i32" => Some("I32"),
        "i64" => Some("I64"),
        "f64" => Some("F64"),
        "rust_decimal::Decimal" => Some("Decimal"),
        "bool" => Some("Bool"),
        "String" => Some("String"),
        "Vec<String>" => Some("StringArray"),
        "time::OffsetDateTime" => Some("Time"),
        "uuid::Uuid" => Some("Uuid"),
        "serde_json::Value" => Some("Json"),
        _ => None,
    }
}

fn render_enum_bind_decode_expr(enum_name: &str, bind_expr: &str, optional: bool) -> String {
    if optional {
        format!(
            "match {bind_expr} {{
                BindValue::StringOpt(value) => match value {{
                    Some(raw) => {enum_name}::from_storage(raw)
                        .map(Some)
                        .ok_or_else(|| anyhow::anyhow!(\"invalid enum storage '{{}}' for type '{enum_name}'\", raw))?,
                    None => None,
                }},
                BindValue::I64Opt(value) => match value {{
                    Some(raw) => {{
                        let raw_str = raw.to_string();
                        {enum_name}::from_storage(&raw_str)
                            .map(Some)
                            .ok_or_else(|| anyhow::anyhow!(\"invalid enum storage '{{}}' for type '{enum_name}'\", raw))?
                    }}
                    None => None,
                }},
                other => anyhow::bail!(\"unexpected bind value '{{:?}}' for type 'Option<{enum_name}>'\", other),
            }}"
        )
    } else {
        format!(
            "match {bind_expr} {{
                BindValue::String(value) => {enum_name}::from_storage(value)
                    .ok_or_else(|| anyhow::anyhow!(\"invalid enum storage '{{}}' for type '{enum_name}'\", value))?,
                BindValue::I64(value) => {{
                    let raw = value.to_string();
                    {enum_name}::from_storage(&raw)
                        .ok_or_else(|| anyhow::anyhow!(\"invalid enum storage '{{}}' for type '{enum_name}'\", value))?
                }}
                other => anyhow::bail!(\"unexpected bind value '{{:?}}' for type '{enum_name}'\", other),
            }}"
        )
    }
}

fn render_bind_decode_expr(
    ty: &str,
    bind_expr: &str,
    enum_specs: &BTreeMap<String, EnumSpec>,
) -> String {
    let trimmed = ty.trim();
    if let Some(inner) = parse_option_inner_type(trimmed) {
        if enum_specs.contains_key(inner.as_str()) {
            return render_enum_bind_decode_expr(&inner, bind_expr, true);
        }

        let Some(variant) = bind_variant_for_type(&inner) else {
            panic!("unsupported lifecycle observer field type '{}'", ty);
        };
        return format!(
            "match {bind_expr} {{
                BindValue::{variant}Opt(value) => value.clone(),
                other => anyhow::bail!(\"unexpected bind value '{{:?}}' for type '{trimmed}'\", other),
            }}"
        );
    }

    if enum_specs.contains_key(trimmed) {
        return render_enum_bind_decode_expr(trimmed, bind_expr, false);
    }

    let Some(variant) = bind_variant_for_type(trimmed) else {
        panic!("unsupported lifecycle observer field type '{}'", ty);
    };
    format!(
        "match {bind_expr} {{
            BindValue::{variant}(value) => value.clone(),
            other => anyhow::bail!(\"unexpected bind value '{{:?}}' for type '{trimmed}'\", other),
        }}"
    )
}

fn relation_target_field_is_optional(schema: &Schema, rel: &RelationSpec) -> bool {
    let Some(target_cfg) = schema.models.get(&rel.target_model) else {
        return false;
    };

    let target_pk = target_cfg.pk.clone().unwrap_or_else(|| "id".to_string());
    parse_fields(target_cfg, &target_pk)
        .iter()
        .any(|field| field.name == rel.foreign_key && field.ty.starts_with("Option<"))
}

fn collect_relation_paths(
    schema: &Schema,
    model_name: &str,
    max_depth: usize,
) -> Vec<RelationPathSpec> {
    if max_depth == 0 {
        return Vec::new();
    }

    fn walk(
        schema: &Schema,
        current_model: &str,
        path: &mut Vec<String>,
        out: &mut Vec<RelationPathSpec>,
        seen: &mut BTreeSet<String>,
        visited_models: &mut BTreeSet<String>,
        max_depth: usize,
    ) {
        if path.len() >= max_depth {
            return;
        }

        let Some(cfg) = schema.models.get(current_model) else {
            return;
        };

        let pk = cfg.pk.clone().unwrap_or_else(|| "id".to_string());
        let fields = parse_fields(cfg, &pk);
        let relations = parse_relations(schema, cfg, current_model, &fields);

        for rel in relations {
            let rel_snake = to_snake(&rel.name);
            path.push(rel_snake);
            let key = path.join("__");

            if seen.insert(key) {
                out.push(RelationPathSpec {
                    path: path.clone(),
                    target_model: rel.target_model.clone(),
                });
            }

            // Only recurse if this model type hasn't been visited in the
            // current path — prevents combinatorial explosion from
            // self-referencing relations (e.g. User.introducer → User).
            if visited_models.insert(rel.target_model.clone()) {
                walk(
                    schema,
                    &rel.target_model,
                    path,
                    out,
                    seen,
                    visited_models,
                    max_depth,
                );
                visited_models.remove(&rel.target_model);
            }

            path.pop();
        }
    }

    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    let mut path = Vec::new();
    let mut visited_models = BTreeSet::new();
    visited_models.insert(model_name.to_string());
    walk(
        schema,
        model_name,
        &mut path,
        &mut out,
        &mut seen,
        &mut visited_models,
        max_depth,
    );
    out
}

fn build_nested_where_has_expr(
    schema: &Schema,
    root_model_name: &str,
    path: &[String],
    leaf_expr_template: &str,
    root_var: &str,
) -> String {
    fn render(
        schema: &Schema,
        current_model_name: &str,
        path: &[String],
        leaf_expr_template: &str,
        current_var: &str,
    ) -> String {
        if path.is_empty() {
            return leaf_expr_template.replace("{var}", current_var);
        }
        let rel = &path[0];
        let model_spec = schema.models.get(current_model_name).unwrap_or_else(|| {
            panic!(
                "Model '{}' not found while rendering relation path",
                current_model_name
            )
        });
        let relation = parse_relations(schema, model_spec, current_model_name, &[])
            .into_iter()
            .find(|candidate| candidate.name == *rel)
            .unwrap_or_else(|| {
                panic!(
                    "Relation '{}' not found on model '{}' while rendering relation path",
                    rel, current_model_name
                )
            });
        let rel_ident = format!("{}Rel", to_title_case(current_model_name));
        let rel_const = to_snake(rel).to_uppercase();
        let nested = render(
            schema,
            &relation.target_model,
            &path[1..],
            leaf_expr_template,
            "rq",
        );
        format!("{current_var}.where_has({rel_ident}::{rel_const}, |rq| {nested})")
    }

    render(schema, root_model_name, path, leaf_expr_template, root_var)
}

fn render_insert_field_setters(db_fields: &[FieldSpec], _col_ident: &str) -> String {
    let mut out = String::new();
    for f in db_fields {
        let fn_name = format!("set_{}", to_snake(&f.name));
        let col_sql = to_snake(&f.name);
        if let Some(SpecialType::Hashed) = f.special_type {
            writeln!(
                out,
                "    pub fn {fn_name}(mut self, val: &str) -> anyhow::Result<Self> {{",
            )
            .unwrap();
            writeln!(
                out,
                "        let hashed = core_db::common::auth::hash::hash_password(val)?;"
            )
            .unwrap();
            writeln!(
                out,
                "        self.state = self.state.set_col(\"{col_sql}\", hashed.into());"
            )
            .unwrap();
            writeln!(out, "        Ok(self)").unwrap();
            writeln!(out, "    }}").unwrap();

            let fn_name_raw = format!("{}_raw", fn_name);
            writeln!(
                out,
                "    pub fn {fn_name_raw}(mut self, val: String) -> Self {{",
            )
            .unwrap();
            writeln!(
                out,
                "        self.state = self.state.set_col(\"{col_sql}\", val.into());"
            )
            .unwrap();
            writeln!(out, "        self").unwrap();
            writeln!(out, "    }}").unwrap();
        } else {
            writeln!(
                out,
                "    pub fn {fn_name}(mut self, val: {typ}) -> Self {{",
                typ = f.ty
            )
            .unwrap();
            writeln!(
                out,
                "        self.state = self.state.set_col(\"{col_sql}\", val.into());"
            )
            .unwrap();
            writeln!(out, "        self").unwrap();
            writeln!(out, "    }}").unwrap();
        }
    }
    out
}

fn render_localized_setters(localized_fields: &[String], cfgs: &ConfigsFile) -> String {
    let mut out = String::new();
    for f in localized_fields {
        let fn_name = format!("set_{}_lang", to_snake(f));
        writeln!(
            out,
            "    pub fn {fn_name}(mut self, locale: localized::Locale, val: impl Into<String>) -> Self {{"
        )
        .unwrap();
        writeln!(
            out,
            "        self.translations.entry(\"{f}\").or_default().insert(locale.into(), val.into());"
        )
        .unwrap();
        writeln!(out, "        self").unwrap();
        writeln!(out, "    }}").unwrap();

        let fn_name_bulk = format!("set_{}_langs", to_snake(f));
        writeln!(
            out,
            "    pub fn {fn_name_bulk}(mut self, langs: localized::LocalizedText) -> Self {{"
        )
        .unwrap();
        for lang in &cfgs.languages.supported {
            let variant = to_title_case(lang);
            writeln!(
                out,
                "        if !langs.{lang}.is_empty() {{ self = self.{fn_name}(localized::Locale::{variant}, langs.{lang}); }}"
            )
            .unwrap();
        }
        writeln!(out, "        self").unwrap();
        writeln!(out, "    }}").unwrap();

        let fn_name_input = format!("set_{}_input", to_snake(f));
        writeln!(
            out,
            "    pub fn {fn_name_input}(mut self, input: Option<localized::LocalizedInput>) -> Self {{"
        )
        .unwrap();
        writeln!(
            out,
            "        let Some(input) = input else {{ return self; }};"
        )
        .unwrap();
        writeln!(out, "        if input.is_empty() {{ return self; }}").unwrap();
        writeln!(out, "        let map = input.to_hashmap();").unwrap();
        writeln!(out, "        for (locale, val) in map {{").unwrap();
        writeln!(
            out,
            "            self.translations.entry(\"{f}\").or_default().insert(locale, val);"
        )
        .unwrap();
        writeln!(out, "        }}").unwrap();
        writeln!(out, "        self").unwrap();
        writeln!(out, "    }}").unwrap();
    }
    out
}

fn render_meta_setters(meta_fields: &[MetaFieldSpec]) -> String {
    let mut out = String::new();
    for m in meta_fields {
        let fn_name = format!("set_meta_{}", m.name);
        match &m.ty {
            MetaType::String => {
                writeln!(
                    out,
                    "    pub fn {fn_name}(mut self, val: impl Into<String>) -> Self {{"
                )
                .unwrap();
                writeln!(
                    out,
                    "        self.meta.insert(\"{name}\".to_string(), JsonValue::String(val.into()));",
                    name = m.name
                )
                .unwrap();
                writeln!(out, "        self").unwrap();
                writeln!(out, "    }}").unwrap();
            }
            MetaType::Bool => {
                writeln!(out, "    pub fn {fn_name}(mut self, val: bool) -> Self {{").unwrap();
                writeln!(
                    out,
                    "        self.meta.insert(\"{name}\".to_string(), JsonValue::Bool(val));",
                    name = m.name
                )
                .unwrap();
                writeln!(out, "        self").unwrap();
                writeln!(out, "    }}").unwrap();
            }
            MetaType::I32 => {
                writeln!(out, "    pub fn {fn_name}(mut self, val: i32) -> Self {{").unwrap();
                writeln!(
                    out,
                    "        self.meta.insert(\"{name}\".to_string(), JsonValue::from(val));",
                    name = m.name
                )
                .unwrap();
                writeln!(out, "        self").unwrap();
                writeln!(out, "    }}").unwrap();
            }
            MetaType::I64 => {
                writeln!(out, "    pub fn {fn_name}(mut self, val: i64) -> Self {{").unwrap();
                writeln!(
                    out,
                    "        self.meta.insert(\"{name}\".to_string(), JsonValue::from(val));",
                    name = m.name
                )
                .unwrap();
                writeln!(out, "        self").unwrap();
                writeln!(out, "    }}").unwrap();
            }
            MetaType::F64 => {
                writeln!(out, "    pub fn {fn_name}(mut self, val: f64) -> Self {{").unwrap();
                writeln!(
                    out,
                    "        self.meta.insert(\"{name}\".to_string(), JsonValue::from(val));",
                    name = m.name
                )
                .unwrap();
                writeln!(out, "        self").unwrap();
                writeln!(out, "    }}").unwrap();
            }
            MetaType::Decimal => {
                writeln!(
                    out,
                    "    pub fn {fn_name}(mut self, val: rust_decimal::Decimal) -> Self {{"
                )
                .unwrap();
                writeln!(
                    out,
                    "        self.meta.insert(\"{name}\".to_string(), JsonValue::from(val.to_string()));",
                    name = m.name
                )
                .unwrap();
                writeln!(out, "        self").unwrap();
                writeln!(out, "    }}").unwrap();
            }
            MetaType::Json => {
                writeln!(
                    out,
                    "    pub fn {fn_name}(mut self, val: JsonValue) -> Self {{"
                )
                .unwrap();
                writeln!(
                    out,
                    "        self.meta.insert(\"{name}\".to_string(), val);",
                    name = m.name
                )
                .unwrap();
                writeln!(out, "        self").unwrap();
                writeln!(out, "    }}").unwrap();
                let typed_fn_name = format!("set_meta_{}_as", m.name);
                writeln!(
                    out,
                    "    pub fn {typed_fn_name}<T: serde::Serialize>(mut self, val: &T) -> anyhow::Result<Self> {{"
                )
                .unwrap();
                writeln!(
                    out,
                    "        self.meta.insert(\"{name}\".to_string(), serde_json::to_value(val)?);",
                    name = m.name
                )
                .unwrap();
                writeln!(out, "        Ok(self)").unwrap();
                writeln!(out, "    }}").unwrap();
            }
            MetaType::DateTime => {
                writeln!(
                    out,
                    "    pub fn {fn_name}(mut self, val: time::OffsetDateTime) -> Self {{"
                )
                .unwrap();
                writeln!(
                    out,
                    "        self.meta.insert(\"{name}\".to_string(), JsonValue::String(val.to_string()));",
                    name = m.name
                )
                .unwrap();
                writeln!(out, "        self").unwrap();
                writeln!(out, "    }}").unwrap();
            }
            MetaType::Custom(ty) => {
                writeln!(
                    out,
                    "    pub fn {fn_name}(mut self, val: &{ty}) -> anyhow::Result<Self> {{",
                    ty = ty
                )
                .unwrap();
                writeln!(
                    out,
                    "        self.meta.insert(\"{name}\".to_string(), serde_json::to_value(val)?);",
                    name = m.name
                )
                .unwrap();
                writeln!(out, "        Ok(self)").unwrap();
                writeln!(out, "    }}").unwrap();
            }
        }
    }
    out
}

fn render_insert_attachment_setters(
    single_attachments: &[AttachmentFieldSpec],
    multi_attachments: &[AttachmentFieldSpec],
) -> String {
    let mut out = String::new();
    for a in single_attachments {
        let fn_name = format!("set_attachment_{}", to_snake(&a.name));
        writeln!(
            out,
            "    pub fn {fn_name}(mut self, att: AttachmentInput) -> Self {{"
        )
        .unwrap();
        writeln!(
            out,
            "        self.attachments_single.insert(\"{name}\", att);",
            name = a.name
        )
        .unwrap();
        writeln!(out, "        self").unwrap();
        writeln!(out, "    }}").unwrap();
    }
    for a in multi_attachments {
        let fn_name = format!("add_attachment_{}", to_snake(&a.name));
        writeln!(
            out,
            "    pub fn {fn_name}(mut self, att: AttachmentInput) -> Self {{"
        )
        .unwrap();
        writeln!(
            out,
            "        self.attachments_multi.entry(\"{name}\").or_default().push(att);",
            name = a.name
        )
        .unwrap();
        writeln!(out, "        self").unwrap();
        writeln!(out, "    }}").unwrap();
    }
    out
}

fn is_incrementable_type(ty: &str) -> bool {
    matches!(ty, "i16" | "i32" | "i64" | "f64" | "rust_decimal::Decimal")
}

fn render_public_column_namespace(
    model_title: &str,
    model_snake: &str,
    col_ident: &str,
    public_col_ident: &str,
    db_fields: &[FieldSpec],
) -> String {
    let mut out = String::new();
    let resolver_ident = format!("resolve_{}_db_col", model_snake);
    writeln!(out, "#[derive(Debug, Clone, Copy, Default)]").unwrap();
    writeln!(out, "pub struct {public_col_ident};").unwrap();
    writeln!(out, "impl {public_col_ident} {{").unwrap();
    for field in db_fields {
        writeln!(
            out,
            "    pub const {variant}: Column<{model_title}Model, {ty}> = Column::new(\"{name}\");",
            variant = field.name.to_ascii_uppercase(),
            ty = field.ty,
            name = field.name
        )
        .unwrap();
    }
    writeln!(out, "}}\n").unwrap();
    writeln!(
        out,
        "fn {resolver_ident}(sql: &str) -> Option<{col_ident}> {{"
    )
    .unwrap();
    writeln!(out, "    match sql {{").unwrap();
    for field in db_fields {
        writeln!(
            out,
            "        \"{name}\" => Some({col_ident}::{variant}),",
            name = field.name,
            variant = to_title_case(&field.name)
        )
        .unwrap();
    }
    writeln!(out, "        _ => None,").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(out, "}}\n").unwrap();
    out
}

fn render_update_field_setters(db_fields: &[FieldSpec], col_ident: &str) -> String {
    let mut out = String::new();
    for f in db_fields {
        let fn_name = format!("set_{}", to_snake(&f.name));
        let col_variant = to_title_case(&f.name);
        if let Some(SpecialType::Hashed) = f.special_type {
            writeln!(
                out,
                "    pub fn {fn_name}(mut self, val: &str) -> anyhow::Result<Self> {{",
            )
            .unwrap();
            writeln!(
                out,
                "        let hashed = core_db::common::auth::hash::hash_password(val)?;"
            )
            .unwrap();
            writeln!(
                out,
                "        self.state = self.state.assign_col({col_ident}::{col_variant}.as_sql(), hashed.into());",
            )
            .unwrap();
            writeln!(out, "        Ok(self)").unwrap();
            writeln!(out, "    }}").unwrap();

            let fn_name_raw = format!("{}_raw", fn_name);
            writeln!(
                out,
                "    pub fn {fn_name_raw}(mut self, val: String) -> Self {{",
            )
            .unwrap();
            writeln!(
                out,
                "        self.state = self.state.assign_col({col_ident}::{col_variant}.as_sql(), val.into());",
            )
            .unwrap();
            writeln!(out, "        self").unwrap();
            writeln!(out, "    }}").unwrap();
        } else {
            writeln!(
                out,
                "    pub fn {fn_name}(mut self, val: {typ}) -> Self {{",
                typ = f.ty
            )
            .unwrap();
            writeln!(
                out,
                "        self.state = self.state.assign_col({col_ident}::{col_variant}.as_sql(), val.into());",
            )
            .unwrap();
            writeln!(out, "        self").unwrap();
            writeln!(out, "    }}").unwrap();

            // Generate increment/decrement for non-optional numeric fields
            if is_incrementable_type(&f.ty) {
                let snake = to_snake(&f.name);
                let inc_fn = format!("increment_{snake}");
                let dec_fn = format!("decrement_{snake}");
                writeln!(
                    out,
                    "    pub fn {inc_fn}(mut self, val: {typ}) -> Self {{",
                    typ = f.ty
                )
                .unwrap();
                writeln!(
                    out,
                    "        self.state = self.state.increment_col({col_ident}::{col_variant}.as_sql(), val.into());",
                )
                .unwrap();
                writeln!(out, "        self").unwrap();
                writeln!(out, "    }}").unwrap();

                writeln!(
                    out,
                    "    pub fn {dec_fn}(mut self, val: {typ}) -> Self {{",
                    typ = f.ty
                )
                .unwrap();
                writeln!(
                    out,
                    "        self.state = self.state.decrement_col({col_ident}::{col_variant}.as_sql(), val.into());",
                )
                .unwrap();
                writeln!(out, "        self").unwrap();
                writeln!(out, "    }}").unwrap();
            }
        }
    }
    out
}

fn render_update_attachment_setters(
    single_attachments: &[AttachmentFieldSpec],
    multi_attachments: &[AttachmentFieldSpec],
) -> String {
    let mut out = String::new();
    for a in single_attachments {
        let fn_name = format!("set_attachment_{}", to_snake(&a.name));
        writeln!(
            out,
            "    pub fn {fn_name}(mut self, att: AttachmentInput) -> Self {{"
        )
        .unwrap();
        writeln!(
            out,
            "        self.attachments_single.insert(\"{name}\", att);",
            name = a.name
        )
        .unwrap();
        writeln!(out, "        self").unwrap();
        writeln!(out, "    }}").unwrap();

        let clear_fn = format!("clear_attachment_{}", to_snake(&a.name));
        writeln!(out, "    pub fn {clear_fn}(mut self) -> Self {{").unwrap();
        writeln!(
            out,
            "        if !self.attachments_clear_single.contains(&\"{name}\") {{ self.attachments_clear_single.push(\"{name}\"); }}",
            name = a.name
        )
        .unwrap();
        writeln!(out, "        self").unwrap();
        writeln!(out, "    }}").unwrap();
    }
    for a in multi_attachments {
        let add_fn = format!("add_attachment_{}", to_snake(&a.name));
        writeln!(
            out,
            "    pub fn {add_fn}(mut self, att: AttachmentInput) -> Self {{"
        )
        .unwrap();
        writeln!(
            out,
            "        self.attachments_multi.entry(\"{name}\").or_default().push(att);",
            name = a.name
        )
        .unwrap();
        writeln!(out, "        self").unwrap();
        writeln!(out, "    }}").unwrap();

        let del_fn = format!("delete_attachment_{}", to_snake(&a.name));
        writeln!(
            out,
            "    pub fn {del_fn}(mut self, ids: impl IntoIterator<Item = Uuid>) -> Self {{"
        )
        .unwrap();
        writeln!(
            out,
            "        self.attachments_delete_multi.entry(\"{name}\").or_default().extend(ids);",
            name = a.name
        )
        .unwrap();
        writeln!(out, "        self").unwrap();
        writeln!(out, "    }}").unwrap();
    }
    out
}

fn render_support_data_loaders(
    model_snake: &str,
    pk: &str,
    parent_pk_ty: &str,
    localized_fields: &[String],
    has_meta: bool,
    has_attachments: bool,
    rows_ident: &str,
    db_expr: &str,
) -> String {
    let mut out = String::new();
    writeln!(
        out,
        "        let ids: Vec<{parent_pk_ty}> = {rows_ident}.iter().map(|r| r.{pk}.clone()).collect();"
    )
    .unwrap();
    if localized_fields.is_empty() {
        writeln!(out, "        let localized = LocalizedMap::default();").unwrap();
    } else {
        writeln!(
            out,
            "        let localized = localized::load_{model_snake}_localized({db_expr}, &ids).await?;"
        )
        .unwrap();
    }
    if has_meta {
        writeln!(
            out,
            "        let meta_map = localized::load_{model_snake}_meta({db_expr}, &ids).await?;"
        )
        .unwrap();
    }
    if has_attachments {
        writeln!(
            out,
            "        let attachments = localized::load_{model_snake}_attachments({db_expr}, &ids).await?;"
        )
        .unwrap();
    }
    out
}

fn build_hydrate_record_expr(
    row_expr: &str,
    localized_fields: &[String],
    has_meta: bool,
    has_attachments: bool,
    base_url_expr: &str,
) -> String {
    match (has_meta, has_attachments) {
        (true, true) => format!(
            "hydrate_record({row_expr}, &localized, &meta_map, &attachments, {base_url_expr})"
        ),
        (true, false) => {
            format!("hydrate_record({row_expr}, &localized, &meta_map, {base_url_expr})")
        }
        (false, true) => {
            format!("hydrate_record({row_expr}, &localized, &attachments, {base_url_expr})")
        }
        (false, false) => {
            if localized_fields.is_empty() {
                format!("hydrate_record({row_expr}, &LocalizedMap::default(), {base_url_expr})")
            } else {
                format!("hydrate_record({row_expr}, &localized, {base_url_expr})")
            }
        }
    }
}

fn render_record_collection_build_no_relations(
    out_ident: &str,
    row_var: &str,
    rows_ident: &str,
    localized_fields: &[String],
    has_meta: bool,
    has_attachments: bool,
    base_url_expr: &str,
) -> String {
    let mut out = String::new();
    writeln!(
        out,
        "        let mut {out_ident} = Vec::with_capacity({rows_ident}.len());"
    )
    .unwrap();
    writeln!(out, "        for {row_var} in {rows_ident} {{").unwrap();
    writeln!(
        out,
        "            {out_ident}.push({});",
        build_hydrate_record_expr(
            row_var,
            localized_fields,
            has_meta,
            has_attachments,
            base_url_expr
        )
    )
    .unwrap();
    writeln!(out, "        }}").unwrap();
    out
}

fn render_record_collection_build(
    relations: &[RelationSpec],
    _model_snake: &str,
    pk: &str,
    rows_ident: &str,
    row_var: &str,
    out_ident: &str,
    localized_fields: &[String],
    has_meta: bool,
    has_attachments: bool,
    base_url_expr: &str,
) -> String {
    let mut out = String::new();
    writeln!(
        out,
        "        let mut {out_ident} = Vec::with_capacity({rows_ident}.len());"
    )
    .unwrap();
    writeln!(out, "        for {row_var} in {rows_ident} {{").unwrap();
    writeln!(out, "            let key = {row_var}.{pk}.clone();").unwrap();
    writeln!(
        out,
        "            let mut record = {};",
        build_hydrate_record_expr(
            &format!("{row_var}.clone()"),
            localized_fields,
            has_meta,
            has_attachments,
            base_url_expr
        )
    )
    .unwrap();
    for rel in relations {
        let field = to_snake(&rel.name);
        match rel.kind {
            RelationKind::HasMany => {
                writeln!(
                    out,
                    "            record.{field} = {field}.get(&key).cloned().unwrap_or_default();"
                )
                .unwrap();
            }
            RelationKind::BelongsTo => {
                writeln!(
                    out,
                    "            record.{field} = {field}.get(&key).cloned().unwrap_or(None).map(Box::new);"
                )
                .unwrap();
            }
        }
    }
    writeln!(out, "            {out_ident}.push(record);").unwrap();
    writeln!(out, "        }}").unwrap();
    out
}

/// Generate the `let __profiler_start = ...` line.
/// When `skip` is true (model has `profile = false`), returns empty.
fn render_profiler_start(skip: bool) -> String {
    if skip {
        return String::new();
    }
    "        let __profiler_start = std::time::Instant::now();\n".to_string()
}

/// Generate the profiler log block. `sql_var` is the variable name holding the SQL string,
/// `binds_expr` is the expression for the bind values to display.
/// When `skip` is true (model has `profile = false`), returns empty.
fn render_profiler_log(
    table: &str,
    op: &str,
    sql_var: &str,
    binds_expr: &str,
    skip: bool,
) -> String {
    if skip {
        return String::new();
    }
    let mut out = String::new();
    writeln!(out, "        record_profiled_query(\"{table}\", \"{op}\", {sql_var}, {binds_expr}, __profiler_start.elapsed());").unwrap();
    out
}

/// Generate the body of `query_all` that uses QueryState::to_select_sql + hydration.
/// Returns the code inside the `Box::pin(async move { ... })` block.
fn render_query_all_body(
    model_title: &str,
    row_ident: &str,
    has_soft_delete: bool,
    table: &str,
    model_snake: &str,
    pk: &str,
    parent_pk_ty: &str,
    relations: &[RelationSpec],
    localized_fields: &[String],
    has_meta: bool,
    has_attachments: bool,
    skip_profiler: bool,
) -> String {
    let mut out = String::new();
    let _soft_delete_col = if has_soft_delete { "deleted_at" } else { "" };
    writeln!(
        out,
        "            let (sql, binds) = state.to_select_sql(Self::TABLE, Self::HAS_SOFT_DELETE, Self::SOFT_DELETE_COL);"
    )
    .unwrap();
    if !skip_profiler {
        writeln!(out, "            let __profiler_binds = if is_sql_profiler_enabled() {{ binds.iter().map(|b| format!(\"{{}}\", b)).collect::<Vec<_>>().join(\", \") }} else {{ String::new() }};").unwrap();
        writeln!(
            out,
            "            let __profiler_start = std::time::Instant::now();"
        )
        .unwrap();
    }
    writeln!(
        out,
        "            let mut q = sqlx::query_as::<_, {row_ident}>(&sql);"
    )
    .unwrap();
    writeln!(out, "            for b in binds {{ q = bind(q, b); }}").unwrap();
    writeln!(out, "            let rows = state.db.fetch_all(q).await?;").unwrap();
    if !skip_profiler {
        writeln!(out, "            record_profiled_query(\"{table}\", \"SELECT\", &sql, &__profiler_binds, __profiler_start.elapsed());").unwrap();
    }
    if !relations.is_empty() {
        // Relation loader bindings use `db.clone()` - we need a db ref
        writeln!(out, "            let db = state.db.clone();").unwrap();
        for rel in relations {
            let rel_name = to_snake(&rel.name);
            writeln!(
                out,
                "            let {rel_name} = load_{rel_name}(db.clone(), &rows, state.base_url.as_deref()).await?;"
            )
            .unwrap();
        }
    }
    // Support data loaders (localized, meta, attachments)
    let db_expr = if relations.is_empty() {
        "state.db.clone()"
    } else {
        "db.clone()"
    };
    // Indent: render_support_data_loaders generates 8-space indented code, we need 12
    let support_loaders = render_support_data_loaders(
        &model_snake,
        &pk,
        &parent_pk_ty,
        &localized_fields,
        has_meta,
        has_attachments,
        "rows",
        db_expr,
    );
    // Re-indent from 8 to 12 spaces
    for line in support_loaders.lines() {
        if line.trim().is_empty() {
            writeln!(out).unwrap();
        } else {
            writeln!(out, "    {}", line).unwrap();
        }
    }
    // Record collection build
    let base_url_expr = "state.base_url.as_deref()";
    if !relations.is_empty() {
        let build = render_record_collection_build(
            relations,
            model_snake,
            pk,
            "rows",
            "r",
            "out_vec",
            localized_fields,
            has_meta,
            has_attachments,
            base_url_expr,
        );
        for line in build.lines() {
            if line.trim().is_empty() {
                writeln!(out).unwrap();
            } else {
                writeln!(out, "    {}", line).unwrap();
            }
        }
    } else {
        let build = render_record_collection_build_no_relations(
            "out_vec",
            "r",
            "rows",
            localized_fields,
            has_meta,
            has_attachments,
            base_url_expr,
        );
        for line in build.lines() {
            if line.trim().is_empty() {
                writeln!(out).unwrap();
            } else {
                writeln!(out, "    {}", line).unwrap();
            }
        }
        writeln!(
            out,
            "            let out_vec: Vec<{model_title}Record> = out_vec;"
        )
        .unwrap();
    }
    // Relation counts: execute count queries for with_count() relations
    let has_many_rels: Vec<_> = relations
        .iter()
        .filter(|r| matches!(r.kind, RelationKind::HasMany))
        .collect();
    if !has_many_rels.is_empty() {
        writeln!(out, "            if !state.count_relations.is_empty() {{").unwrap();
        writeln!(out, "                let parent_ids: Vec<core_db::common::sql::BindValue> = rows.iter().map(|r| r.{pk}.clone().into()).collect();").unwrap();
        let db_ref = if relations.is_empty() { "state.db" } else { "db" };
        writeln!(out, "                let counts = core_db::common::model_api::execute_relation_counts(&{db_ref}, &parent_ids, &state.count_relations).await?;").unwrap();
        writeln!(out, "                for record in &mut out_vec {{").unwrap();
        writeln!(out, "                    for (rel_name, by_fk) in &counts {{").unwrap();
        writeln!(out, "                        if let Some(&cnt) = by_fk.get(&record.{pk}) {{").unwrap();
        writeln!(out, "                            record.__relation_counts.insert(rel_name.clone(), cnt);").unwrap();
        writeln!(out, "                        }}").unwrap();
        writeln!(out, "                    }}").unwrap();
        writeln!(out, "                }}").unwrap();
        writeln!(out, "            }}").unwrap();
    }
    writeln!(out, "            Ok(out_vec)").unwrap();
    out
}

/// Generate the body of `query_count` using QueryState::to_count_sql.
fn render_query_count_body(has_soft_delete: bool, table: &str, skip_profiler: bool) -> String {
    let mut out = String::new();
    let _soft_delete_col = if has_soft_delete { "deleted_at" } else { "" };
    writeln!(
        out,
        "            let (sql, binds) = state.to_count_sql(Self::TABLE, Self::HAS_SOFT_DELETE, Self::SOFT_DELETE_COL);"
    )
    .unwrap();
    if !skip_profiler {
        writeln!(out, "            let __profiler_binds = if is_sql_profiler_enabled() {{ binds.iter().map(|b| format!(\"{{}}\", b)).collect::<Vec<_>>().join(\", \") }} else {{ String::new() }};").unwrap();
        writeln!(
            out,
            "            let __profiler_start = std::time::Instant::now();"
        )
        .unwrap();
    }
    writeln!(
        out,
        "            let mut q = sqlx::query_scalar::<_, i64>(&sql);"
    )
    .unwrap();
    writeln!(
        out,
        "            for b in binds {{ q = bind_scalar(q, b); }}"
    )
    .unwrap();
    writeln!(
        out,
        "            let count = state.db.fetch_scalar(q).await?;"
    )
    .unwrap();
    if !skip_profiler {
        writeln!(out, "            record_profiled_query(\"{table}\", \"COUNT\", &sql, &__profiler_binds, __profiler_start.elapsed());").unwrap();
    }
    writeln!(out, "            Ok(count)").unwrap();
    out
}

/// Generate the body of `query_paginate` using QueryState.
fn render_query_paginate_body(
    model_title: &str,
    row_ident: &str,
    has_soft_delete: bool,
    table: &str,
    model_snake: &str,
    pk: &str,
    parent_pk_ty: &str,
    relations: &[RelationSpec],
    localized_fields: &[String],
    has_meta: bool,
    has_attachments: bool,
    skip_profiler: bool,
) -> String {
    let mut out = String::new();
    let _soft_delete_col = if has_soft_delete { "deleted_at" } else { "" };
    writeln!(
        out,
        "            let page = if page < 1 {{ 1 }} else {{ page }};"
    )
    .unwrap();
    writeln!(
        out,
        "            let per_page = resolve_per_page(per_page);"
    )
    .unwrap();
    // Count query
    writeln!(
        out,
        "            let (count_sql, count_binds) = state.to_count_sql(Self::TABLE, Self::HAS_SOFT_DELETE, Self::SOFT_DELETE_COL);"
    )
    .unwrap();
    if !skip_profiler {
        writeln!(out, "            let __profiler_binds = if is_sql_profiler_enabled() {{ count_binds.iter().map(|b| format!(\"{{}}\", b)).collect::<Vec<_>>().join(\", \") }} else {{ String::new() }};").unwrap();
        writeln!(
            out,
            "            let __profiler_start = std::time::Instant::now();"
        )
        .unwrap();
    }
    writeln!(
        out,
        "            let mut count_q = sqlx::query_scalar::<_, i64>(&count_sql);"
    )
    .unwrap();
    writeln!(
        out,
        "            for b in count_binds {{ count_q = bind_scalar(count_q, b); }}"
    )
    .unwrap();
    writeln!(
        out,
        "            let total: i64 = state.db.fetch_scalar(count_q).await?;"
    )
    .unwrap();
    if !skip_profiler {
        writeln!(out, "            record_profiled_query(\"{table}\", \"COUNT\", &count_sql, &__profiler_binds, __profiler_start.elapsed());").unwrap();
    }
    writeln!(
        out,
        "            let last_page = ((total + per_page - 1) / per_page).max(1);"
    )
    .unwrap();
    writeln!(out, "            let current_page = page.min(last_page);").unwrap();
    writeln!(
        out,
        "            let offset_val = (current_page - 1) * per_page;"
    )
    .unwrap();
    // Data query - override offset/limit on state
    writeln!(out, "            let mut state = state;").unwrap();
    writeln!(out, "            state.offset = Some(offset_val);").unwrap();
    writeln!(out, "            state.limit = Some(per_page);").unwrap();
    writeln!(
        out,
        "            let (sql, binds) = state.to_select_sql(Self::TABLE, Self::HAS_SOFT_DELETE, Self::SOFT_DELETE_COL);"
    )
    .unwrap();
    if !skip_profiler {
        writeln!(
            out,
            "            let __profiler_start = std::time::Instant::now();"
        )
        .unwrap();
    }
    writeln!(
        out,
        "            let mut q = sqlx::query_as::<_, {row_ident}>(&sql);"
    )
    .unwrap();
    writeln!(out, "            for b in binds {{ q = bind(q, b); }}").unwrap();
    writeln!(out, "            let rows = state.db.fetch_all(q).await?;").unwrap();
    if !skip_profiler {
        writeln!(out, "            record_profiled_query(\"{table}\", \"SELECT\", &sql, &__profiler_binds, __profiler_start.elapsed());").unwrap();
    }
    // Hydration (same as query_all)
    if !relations.is_empty() {
        writeln!(out, "            let db = state.db.clone();").unwrap();
        for rel in relations {
            let rel_name = to_snake(&rel.name);
            writeln!(
                out,
                "            let {rel_name} = load_{rel_name}(db.clone(), &rows, state.base_url.as_deref()).await?;"
            )
            .unwrap();
        }
    }
    let db_expr = if relations.is_empty() {
        "state.db.clone()"
    } else {
        "db.clone()"
    };
    let support_loaders = render_support_data_loaders(
        &model_snake,
        &pk,
        &parent_pk_ty,
        &localized_fields,
        has_meta,
        has_attachments,
        "rows",
        db_expr,
    );
    for line in support_loaders.lines() {
        if line.trim().is_empty() {
            writeln!(out).unwrap();
        } else {
            writeln!(out, "    {}", line).unwrap();
        }
    }
    let base_url_expr = "state.base_url.as_deref()";
    if !relations.is_empty() {
        let build = render_record_collection_build(
            relations,
            model_snake,
            pk,
            "rows",
            "r",
            "data",
            localized_fields,
            has_meta,
            has_attachments,
            base_url_expr,
        );
        for line in build.lines() {
            if line.trim().is_empty() {
                writeln!(out).unwrap();
            } else {
                writeln!(out, "    {}", line).unwrap();
            }
        }
    } else {
        let build = render_record_collection_build_no_relations(
            "data",
            "r",
            "rows",
            localized_fields,
            has_meta,
            has_attachments,
            base_url_expr,
        );
        for line in build.lines() {
            if line.trim().is_empty() {
                writeln!(out).unwrap();
            } else {
                writeln!(out, "    {}", line).unwrap();
            }
        }
        writeln!(
            out,
            "            let data: Vec<{model_title}Record> = data;"
        )
        .unwrap();
    }
    writeln!(
        out,
        "            Ok(core_db::common::model_api::Page {{ data, total, per_page, current_page, last_page }})"
    )
    .unwrap();
    out
}

/// Generate the body of `query_delete` using state fields directly.
/// Handles observer hooks and soft-delete logic.
fn render_query_delete_body(
    table: &str,
    model_key: &str,
    col_ident: &str,
    has_soft_delete: bool,
    emit_hooks: bool,
    row_ident: &str,
    pk_snake: &str,
    skip_profiler: bool,
    parent_pk_ty: &str,
) -> String {
    let mut out = String::new();
    writeln!(out, "            if state.limit.is_some() {{").unwrap();
    writeln!(
        out,
        "                anyhow::bail!(\"delete() does not support limit; add where clauses\");"
    )
    .unwrap();
    writeln!(out, "            }}").unwrap();
    writeln!(out, "            let db = state.db;").unwrap();
    writeln!(out, "            let mut where_sql = state.where_sql;").unwrap();
    writeln!(out, "            let binds = state.binds;").unwrap();
    if has_soft_delete {
        writeln!(out, "            let with_deleted = state.with_deleted;").unwrap();
        writeln!(out, "            let only_deleted = state.only_deleted;").unwrap();
    }
    writeln!(
        out,
        "            if where_sql.is_empty() {{ anyhow::bail!(\"delete(): no conditions set\"); }}"
    )
    .unwrap();
    writeln!(out, "            let __profiler_binds = if is_sql_profiler_enabled() {{ binds.iter().map(|b| format!(\"{{}}\", b)).collect::<Vec<_>>().join(\", \") }} else {{ String::new() }};").unwrap();
    if emit_hooks {
        writeln!(
            out,
            "            let __observer_active = try_get_observer().is_some();"
        )
        .unwrap();
        writeln!(
            out,
            "            let __old_rows: Vec<{row_ident}> = if __observer_active {{"
        )
        .unwrap();
        writeln!(out, "                let select_sql = format!(\"SELECT * FROM {table} WHERE {{}}\", where_sql.join(\" AND \"));").unwrap();
        writeln!(
            out,
            "                let mut fq = sqlx::query_as::<_, {row_ident}>(&select_sql);"
        )
        .unwrap();
        writeln!(
            out,
            "                for b in &binds {{ fq = bind(fq, b.clone()); }}"
        )
        .unwrap();
        writeln!(
            out,
            "                let rows: Vec<{row_ident}> = db.fetch_all(fq).await.unwrap_or_default();"
        )
        .unwrap();
        writeln!(out, "                rows").unwrap();
        writeln!(out, "            }} else {{").unwrap();
        writeln!(out, "                Vec::new()").unwrap();
        writeln!(out, "            }};").unwrap();
        writeln!(out, "            if !__old_rows.is_empty() {{").unwrap();
        writeln!(
            out,
            "                if let Some(observer) = try_get_observer() {{"
        )
        .unwrap();
        writeln!(
            out,
            "                    let old_data = serde_json::to_value(&__old_rows)?;"
        )
        .unwrap();
        writeln!(
            out,
            "                    let event = ModelEvent {{ model: \"{model_key}\", table: \"{table}\", record_key: None }};"
        )
        .unwrap();
        writeln!(
            out,
            "                    let action = observer.on_deleting(&event, &old_data).await?;"
        )
        .unwrap();
        writeln!(out, "                    match action {{").unwrap();
        writeln!(
            out,
            "                        ObserverAction::Prevent(err) => return Err(err),"
        )
        .unwrap();
        writeln!(
            out,
            "                        ObserverAction::Modify(overrides) => {{"
        )
        .unwrap();
        writeln!(
            out,
            "                            let ids: Vec<{parent_pk_ty}> = __old_rows.iter().map(|r| r.{pk_snake}.clone()).collect();"
        )
        .unwrap();
        writeln!(
            out,
            "                            let affected = Self::convert_delete_to_update(&db, &ids, overrides).await?;"
        )
        .unwrap();
        writeln!(out, "                            return Ok(affected);").unwrap();
        writeln!(out, "                        }}").unwrap();
        writeln!(
            out,
            "                        ObserverAction::Continue => {{}}"
        )
        .unwrap();
        writeln!(out, "                    }}").unwrap();
        writeln!(out, "                }}").unwrap();
        writeln!(out, "            }}").unwrap();
    }
    if has_soft_delete {
        writeln!(out, "            if HAS_SOFT_DELETE {{").unwrap();
        writeln!(out, "                if only_deleted {{").unwrap();
        writeln!(
            out,
            "                    where_sql.push(format!(\"{{}} IS NOT NULL\", {col_ident}::DeletedAt.as_sql()));"
        )
        .unwrap();
        writeln!(out, "                }} else if !with_deleted {{").unwrap();
        writeln!(
            out,
            "                    where_sql.push(format!(\"{{}} IS NULL\", {col_ident}::DeletedAt.as_sql()));"
        )
        .unwrap();
        writeln!(out, "                }}").unwrap();
        writeln!(out, "                let idx = binds.len() + 1;").unwrap();
        writeln!(
            out,
            "                let mut sql = format!(\"UPDATE {table} SET {{}} = ${{}}\", {col_ident}::DeletedAt.as_sql(), idx);"
        )
        .unwrap();
        writeln!(out, "                if !where_sql.is_empty() {{").unwrap();
        writeln!(out, "                    sql.push_str(\" WHERE \");").unwrap();
        writeln!(
            out,
            "                    sql.push_str(&where_sql.join(\" AND \"));"
        )
        .unwrap();
        writeln!(out, "                }}").unwrap();
        if !skip_profiler {
            writeln!(
                out,
                "                let __profiler_start = std::time::Instant::now();"
            )
            .unwrap();
        }
        writeln!(out, "                let mut q = sqlx::query(&sql);").unwrap();
        writeln!(
            out,
            "                for b in binds {{ q = bind_query(q, b); }}"
        )
        .unwrap();
        writeln!(
            out,
            "                q = bind_query(q, time::OffsetDateTime::now_utc().into());"
        )
        .unwrap();
        writeln!(out, "                let res = db.execute(q).await?;").unwrap();
        if !skip_profiler {
            writeln!(out, "                record_profiled_query(\"{table}\", \"UPDATE\", &sql, &__profiler_binds, __profiler_start.elapsed());").unwrap();
        }
        if emit_hooks {
            writeln!(
                out,
                "                if !__old_rows.is_empty() && res.rows_affected() > 0 {{"
            )
            .unwrap();
            writeln!(
                out,
                "                    if let Some(observer) = try_get_observer() {{"
            )
            .unwrap();
            writeln!(out, "                        for old_row in &__old_rows {{").unwrap();
            writeln!(
                out,
                "                            let event = ModelEvent {{ model: \"{model_key}\", table: \"{table}\", record_key: Some(format!(\"{{}}\", old_row.{pk_snake})) }};"
            )
            .unwrap();
            writeln!(
                out,
                "                            match serde_json::to_value(old_row) {{"
            )
            .unwrap();
            writeln!(out, "                                Ok(old_data) => {{").unwrap();
            writeln!(out, "                                    if let Err(err) = observer.on_deleted(&event, &old_data).await {{").unwrap();
            writeln!(out, "                                        log_observer_error(\"deleted\", \"{model_key}\", &err);").unwrap();
            writeln!(out, "                                    }}").unwrap();
            writeln!(out, "                                }}").unwrap();
            writeln!(out, "                                Err(err) => log_observer_error(\"deleted\", \"{model_key}\", &err),").unwrap();
            writeln!(out, "                            }}").unwrap();
            writeln!(out, "                        }}").unwrap();
            writeln!(out, "                    }}").unwrap();
            writeln!(out, "                }}").unwrap();
        }
        writeln!(out, "                return Ok(res.rows_affected());").unwrap();
        writeln!(out, "            }}").unwrap();
    }
    // Hard delete path
    writeln!(
        out,
        "            let mut sql = String::from(\"DELETE FROM {table}\");"
    )
    .unwrap();
    writeln!(out, "            if !where_sql.is_empty() {{").unwrap();
    writeln!(out, "                sql.push_str(\" WHERE \");").unwrap();
    writeln!(
        out,
        "                sql.push_str(&where_sql.join(\" AND \"));"
    )
    .unwrap();
    writeln!(out, "            }}").unwrap();
    if !skip_profiler {
        writeln!(
            out,
            "            let __profiler_start = std::time::Instant::now();"
        )
        .unwrap();
    }
    writeln!(out, "            let mut q = sqlx::query(&sql);").unwrap();
    writeln!(
        out,
        "            for b in binds {{ q = bind_query(q, b); }}"
    )
    .unwrap();
    writeln!(out, "            let res = db.execute(q).await?;").unwrap();
    if !skip_profiler {
        writeln!(out, "            record_profiled_query(\"{table}\", \"DELETE\", &sql, &__profiler_binds, __profiler_start.elapsed());").unwrap();
    }
    if emit_hooks {
        writeln!(
            out,
            "            if !__old_rows.is_empty() && res.rows_affected() > 0 {{"
        )
        .unwrap();
        writeln!(
            out,
            "                if let Some(observer) = try_get_observer() {{"
        )
        .unwrap();
        writeln!(out, "                    for old_row in &__old_rows {{").unwrap();
        writeln!(
            out,
            "                        let event = ModelEvent {{ model: \"{model_key}\", table: \"{table}\", record_key: Some(format!(\"{{}}\", old_row.{pk_snake})) }};"
        )
        .unwrap();
        writeln!(
            out,
            "                        match serde_json::to_value(old_row) {{"
        )
        .unwrap();
        writeln!(out, "                            Ok(old_data) => {{").unwrap();
        writeln!(out, "                                if let Err(err) = observer.on_deleted(&event, &old_data).await {{").unwrap();
        writeln!(out, "                                    log_observer_error(\"deleted\", \"{model_key}\", &err);").unwrap();
        writeln!(out, "                                }}").unwrap();
        writeln!(out, "                            }}").unwrap();
        writeln!(out, "                            Err(err) => log_observer_error(\"deleted\", \"{model_key}\", &err),").unwrap();
        writeln!(out, "                        }}").unwrap();
        writeln!(out, "                    }}").unwrap();
        writeln!(out, "                }}").unwrap();
        writeln!(out, "            }}").unwrap();
    }
    writeln!(out, "            Ok(res.rows_affected())").unwrap();
    out
}

fn render_create_model_impl(
    model_title: &str,
    insert_ident: &str,
    _query_ident: &str,
    pk: &str,
    table: &str,
) -> String {
    let mut out = String::new();
    writeln!(
        out,
        "impl core_db::common::model_api::CreateModel for {model_title}Model {{"
    )
    .unwrap();
    writeln!(
        out,
        "    fn create_save<'db>(state: CreateState<'db>) -> core_db::common::model_api::BoxModelFuture<'db, Self::Record> {{"
    )
    .unwrap();
    writeln!(out, "        Box::pin(async move {{").unwrap();
    writeln!(
        out,
        "            let builder = {insert_ident}::from_state(state);"
    )
    .unwrap();
    writeln!(out, "            let db = builder.state.db.clone();").unwrap();
    writeln!(
        out,
        "            let base_url = builder.state.base_url.clone();"
    )
    .unwrap();
    writeln!(out, "            let created = builder.save().await?;").unwrap();
    writeln!(
        out,
        "            Query::<{model_title}Model>::new_with_base_url(db, base_url).find(created.{pk}.clone()).await?.ok_or_else(|| anyhow::anyhow!(\"{table}: created record not found\"))"
    )
    .unwrap();
    writeln!(out, "        }})").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(
        out,
        "    fn transform_create_value(col: &str, value: BindValue) -> anyhow::Result<BindValue> {{"
    )
    .unwrap();
    writeln!(out, "        Self::_transform_create_value(col, value)").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(out, "}}\n").unwrap();
    out
}

fn render_create_field_impl(
    model_title: &str,
    col_ident: &str,
    _db_fields: &[FieldSpec],
) -> String {
    let mut out = String::new();
    writeln!(
        out,
        "impl core_db::common::model_api::CreateField<{model_title}Model> for {col_ident} {{"
    )
    .unwrap();
    writeln!(out, "    type Value = BindValue;").unwrap();
    writeln!(
        out,
        "    fn set<'db>(field: Self, state: CreateState<'db>, value: BindValue) -> anyhow::Result<CreateState<'db>> {{"
    )
    .unwrap();
    writeln!(out, "        let value = <{model_title}Model as core_db::common::model_api::CreateModel>::transform_create_value(field.as_sql(), value)?;").unwrap();
    writeln!(out, "        Ok(state.set_col(field.as_sql(), value))").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(out, "}}\n").unwrap();
    out
}

fn render_create_conflict_field_impl(model_title: &str, col_ident: &str) -> String {
    let mut out = String::new();
    writeln!(
        out,
        "impl core_db::common::model_api::CreateConflictField<{model_title}Model> for {col_ident} {{"
    )
    .unwrap();
    writeln!(
        out,
        "    fn on_conflict_do_nothing<'db>(state: CreateState<'db>, fields: &[Self]) -> CreateState<'db> {{"
    )
    .unwrap();
    writeln!(
        out,
        "        let cols: Vec<&'static str> = fields.iter().map(|f| f.as_sql()).collect();"
    )
    .unwrap();
    writeln!(out, "        state.on_conflict_do_nothing(&cols)").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(
        out,
        "    fn on_conflict_update<'db>(state: CreateState<'db>, fields: &[Self]) -> CreateState<'db> {{"
    )
    .unwrap();
    writeln!(
        out,
        "        let cols: Vec<&'static str> = fields.iter().map(|f| f.as_sql()).collect();"
    )
    .unwrap();
    writeln!(out, "        state.on_conflict_update(&cols)").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(out, "}}\n").unwrap();
    out
}

fn render_patch_model_impl(
    model_title: &str,
    _query_ident: &str,
    update_ident: &str,
    col_ident: &str,
    pk_col_variant: &str,
    parent_pk_ty: &str,
    table: &str,
    has_soft_delete: bool,
) -> String {
    let mut out = String::new();
    writeln!(
        out,
        "impl core_db::common::model_api::PatchModel for {model_title}Model {{"
    )
    .unwrap();
    writeln!(
        out,
        "    fn patch_from_query<'db>(mut state: QueryState<'db>) -> PatchState<'db> {{"
    )
    .unwrap();
    writeln!(out, "        let db = state.db.clone();").unwrap();
    writeln!(out, "        let base_url = state.base_url.clone();").unwrap();
    writeln!(
        out,
        "        state.select_sql = Some({col_ident}::{pk_col_variant}.as_sql().to_string());"
    )
    .unwrap();
    writeln!(out, "        let (scope_sql, binds) = state.to_sql();").unwrap();
    writeln!(
        out,
        "        let mut ps = PatchState::new(db, base_url, \"{table}\");"
    )
    .unwrap();
    writeln!(
        out,
        "        ps.where_sql.push(format!(\"{{}} IN ({{}})\", {col_ident}::{pk_col_variant}.as_sql(), scope_sql));"
    )
    .unwrap();
    writeln!(out, "        ps.where_binds = binds;").unwrap();
    writeln!(out, "        ps").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(
        out,
        "    fn patch_save<'db>(state: PatchState<'db>) -> core_db::common::model_api::BoxModelFuture<'db, u64> {{"
    )
    .unwrap();
    writeln!(out, "        Box::pin(async move {{").unwrap();
    writeln!(
        out,
        "            let builder = {update_ident}::from_state(state);"
    )
    .unwrap();
    writeln!(out, "            builder.save().await").unwrap();
    writeln!(out, "        }})").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(
        out,
        "    fn patch_fetch<'db>(state: PatchState<'db>) -> core_db::common::model_api::BoxModelFuture<'db, Vec<Self::Record>> {{"
    )
    .unwrap();
    writeln!(out, "        Box::pin(async move {{").unwrap();
    writeln!(out, "            if state.where_sql.is_empty() {{").unwrap();
    writeln!(
        out,
        "                anyhow::bail!(\"update: no conditions set\");"
    )
    .unwrap();
    writeln!(out, "            }}").unwrap();
    writeln!(out, "            let db = state.db.clone();").unwrap();
    writeln!(out, "            let base_url = state.base_url.clone();").unwrap();
    writeln!(
        out,
        "            let mut select_sql = format!(\"SELECT {{}} FROM {table}\", {col_ident}::{pk_col_variant}.as_sql());"
    )
    .unwrap();
    writeln!(
        out,
        "            select_sql.push_str(&format!(\" WHERE {{}}\", state.where_sql.join(\" AND \")));"
    )
    .unwrap();
    writeln!(
        out,
        "            let mut select_q = sqlx::query_scalar::<_, {parent_pk_ty}>(&select_sql);"
    )
    .unwrap();
    writeln!(
        out,
        "            for bind_value in &state.where_binds {{ select_q = bind_scalar(select_q, bind_value.clone()); }}"
    )
    .unwrap();
    writeln!(
        out,
        "            let target_ids = db.fetch_all_scalar(select_q).await?;"
    )
    .unwrap();
    writeln!(
        out,
        "            let builder = {update_ident}::from_state(state);"
    )
    .unwrap();
    writeln!(out, "            builder.save().await?;").unwrap();
    writeln!(out, "            if target_ids.is_empty() {{").unwrap();
    writeln!(out, "                return Ok(Vec::new());").unwrap();
    writeln!(out, "            }}").unwrap();
    writeln!(
        out,
        "            let query = Query::<{model_title}Model>::new_with_base_url(db, base_url);"
    )
    .unwrap();
    if has_soft_delete {
        writeln!(
            out,
            "            let mut state = query.into_inner().with_deleted();"
        )
        .unwrap();
        writeln!(out, "            let binds: Vec<BindValue> = target_ids.iter().cloned().map(Into::into).collect();").unwrap();
        writeln!(
            out,
            "            state = state.where_in_str({col_ident}::{pk_col_variant}.as_sql(), &binds);"
        )
        .unwrap();
        writeln!(
            out,
            "            <Self as core_db::common::model_api::QueryModel>::query_all(state).await"
        )
        .unwrap();
    } else {
        writeln!(out, "            let binds: Vec<BindValue> = target_ids.iter().cloned().map(Into::into).collect();").unwrap();
        writeln!(out, "            let state = query.into_inner().where_in_str({col_ident}::{pk_col_variant}.as_sql(), &binds);").unwrap();
        writeln!(
            out,
            "            <Self as core_db::common::model_api::QueryModel>::query_all(state).await"
        )
        .unwrap();
    }
    writeln!(out, "        }})").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(
        out,
        "    fn transform_patch_value(col: &str, value: BindValue) -> anyhow::Result<BindValue> {{"
    )
    .unwrap();
    writeln!(out, "        Self::_transform_patch_value(col, value)").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(out, "}}\n").unwrap();
    out
}

fn render_patch_assign_field_impl(
    model_title: &str,
    col_ident: &str,
    _db_fields: &[FieldSpec],
) -> String {
    let mut out = String::new();
    writeln!(
        out,
        "impl core_db::common::model_api::PatchAssignField<{model_title}Model> for {col_ident} {{"
    )
    .unwrap();
    writeln!(out, "    type Value = BindValue;").unwrap();
    writeln!(
        out,
        "    fn assign<'db>(field: Self, state: PatchState<'db>, value: BindValue) -> anyhow::Result<PatchState<'db>> {{"
    )
    .unwrap();
    writeln!(out, "        let value = <{model_title}Model as core_db::common::model_api::PatchModel>::transform_patch_value(field.as_sql(), value)?;").unwrap();
    writeln!(out, "        Ok(state.assign_col(field.as_sql(), value))").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(out, "}}\n").unwrap();
    out
}

fn render_patch_numeric_field_impl(
    model_title: &str,
    col_ident: &str,
    db_fields: &[FieldSpec],
) -> String {
    let numeric_fields: Vec<&FieldSpec> = db_fields
        .iter()
        .filter(|field| !field.ty.starts_with("Option<") && is_incrementable_type(&field.ty))
        .collect();
    if numeric_fields.is_empty() {
        return String::new();
    }

    let mut out = String::new();
    writeln!(
        out,
        "impl core_db::common::model_api::PatchNumericField<{model_title}Model> for {col_ident} {{"
    )
    .unwrap();
    writeln!(
        out,
        "    fn increment<'db>(field: Self, state: PatchState<'db>, value: BindValue) -> anyhow::Result<PatchState<'db>> {{"
    )
    .unwrap();
    writeln!(out, "        match field {{").unwrap();
    for field in &numeric_fields {
        let col_variant = to_title_case(&field.name);
        writeln!(
            out,
            "            {col_ident}::{col_variant} => Ok(state.increment_col(field.as_sql(), value)),"
        )
        .unwrap();
    }
    writeln!(
        out,
        "            _ => anyhow::bail!(\"column '{{}}' does not support increment\", field.as_sql()),"
    )
    .unwrap();
    writeln!(out, "        }}").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(
        out,
        "    fn decrement<'db>(field: Self, state: PatchState<'db>, value: BindValue) -> anyhow::Result<PatchState<'db>> {{"
    )
    .unwrap();
    writeln!(out, "        match field {{").unwrap();
    for field in &numeric_fields {
        let col_variant = to_title_case(&field.name);
        writeln!(
            out,
            "            {col_ident}::{col_variant} => Ok(state.decrement_col(field.as_sql(), value)),"
        )
        .unwrap();
    }
    writeln!(
        out,
        "            _ => anyhow::bail!(\"column '{{}}' does not support decrement\", field.as_sql()),"
    )
    .unwrap();
    writeln!(out, "        }}").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(out, "}}\n").unwrap();
    out
}

pub fn generate_models(
    schema: &Schema,
    cfgs: &ConfigsFile,
    out_dir: &Path,
) -> Result<(), Box<dyn Error>> {
    generate_models_with_options(schema, cfgs, out_dir, GenerateModelsOptions::default())
}

pub fn generate_models_with_options(
    schema: &Schema,
    cfgs: &ConfigsFile,
    out_dir: &Path,
    options: GenerateModelsOptions,
) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(out_dir)?;

    let mut model_module_exports = String::new();

    for (name, cfg) in &schema.models {
        let file_stem = to_snake(name);
        let model_title = to_title_case(&file_stem);
        let code = render_model(name, cfg, schema, cfgs, options);
        crate::write_if_changed(&out_dir.join(format!("{file_stem}.rs")), code)?;

        let pk = cfg.pk.clone().unwrap_or_else(|| "id".to_string());
        let fields = parse_fields(cfg, &pk);
        let has_rel = !parse_relations(schema, cfg, name, &fields).is_empty();

        let mut exports = vec![
            format!("{model_title}Model"),
            format!("{model_title}Record"),
            format!("{model_title}Create"),
            format!("{model_title}Changes"),
            format!("{model_title}Col"),
        ];
        if options.include_datatable {
            exports.push(format!("{model_title}TableAdapter"));
            exports.push(format!("{model_title}DataTable"));
            exports.push(format!("{model_title}DataTableConfig"));
            exports.push(format!("{model_title}DataTableHooks"));
            exports.push(format!("{model_title}DefaultDataTableHooks"));
        }
        if has_rel {
            exports.push(format!("{model_title}Rel"));
        }

        writeln!(model_module_exports, "pub(crate) mod {};", file_stem)?;
        writeln!(
            model_module_exports,
            "pub use {}::{{{}}};",
            file_stem,
            exports.join(", ")
        )?;
    }

    crate::write_if_changed(&out_dir.join("common.rs"), generate_common())?;
    let mut mod_context = TemplateContext::new();
    mod_context.insert(
        "model_module_exports",
        model_module_exports.trim_end().to_string(),
    )?;
    crate::write_if_changed(
        &out_dir.join("mod.rs"),
        render_template("models/mod.rs.tpl", &mod_context)?,
    )?;
    Ok(())
}

fn render_model(
    name: &str,
    cfg: &ModelSpec,
    schema: &Schema,
    cfgs: &ConfigsFile,
    options: GenerateModelsOptions,
) -> String {
    let model_snake = to_snake(name);
    let model_title = to_title_case(&model_snake);
    let row_ident = format!("{}Row", model_title);
    let record_ident = format!("{}Record", model_title);
    let col_ident = format!("{}DbCol", model_title);
    let public_col_ident = format!("{}Col", model_title);
    let query_ident = format!("{}QueryInner", model_title);
    let insert_ident = format!("{}CreateInner", model_title);
    let update_ident = format!("{}PatchInner", model_title);
    let model_snake_upper = model_snake.to_uppercase();

    let table = cfg.table.as_deref().unwrap_or(&model_snake).to_string();
    let pk = cfg.pk.clone().unwrap_or_else(|| "id".to_string());
    let pk_col_variant = to_title_case(&pk);

    let fields = parse_fields(cfg, &pk);
    let meta_fields = parse_meta(cfg);
    let has_meta = !meta_fields.is_empty();
    let attachment_fields = parse_attachments(cfg);
    for att in &attachment_fields {
        if !cfgs.attachment_types.contains_key(att.typ.as_str()) {
            panic!(
                "Attachment type '{}' on model '{}' field '{}' is not defined in settings.toml [attachment_type.*]",
                att.typ, name, att.name
            );
        }
    }
    let single_attachments: Vec<AttachmentFieldSpec> = attachment_fields
        .iter()
        .cloned()
        .filter(|a| !a.multiple)
        .collect();
    let multi_attachments: Vec<AttachmentFieldSpec> = attachment_fields
        .iter()
        .cloned()
        .filter(|a| a.multiple)
        .collect();
    let has_attachments = !attachment_fields.is_empty();
    let localized_fields: Vec<String> = cfg
        .localized
        .clone()
        .unwrap_or_default()
        .into_iter()
        .map(|s| to_snake(&s))
        .collect();
    let localized_set: BTreeSet<String> = localized_fields.iter().cloned().collect();
    let db_fields: Vec<FieldSpec> = fields
        .iter()
        .cloned()
        .filter(|f| !localized_set.contains(&f.name))
        .collect();
    let enum_specs: BTreeMap<String, EnumSpec> = schema
        .extra_sections
        .iter()
        .filter_map(|(name, section)| match section {
            EnumOrOther::Enum(spec) if spec.type_name == "enum" => {
                Some((name.clone(), spec.clone()))
            }
            _ => None,
        })
        .collect();
    let enum_type_names: BTreeSet<String> = enum_specs.keys().cloned().collect();
    let enum_explained_fields: Vec<EnumExplainedFieldSpec> = db_fields
        .iter()
        .filter_map(|field| enum_field_spec(&field.name, &field.ty, &enum_type_names))
        .collect();

    let parent_pk_ty = fields
        .iter()
        .find(|f| f.name == pk)
        .map(|f| f.ty.clone())
        .unwrap_or_else(|| "i64".to_string());
    let id_strategy = cfg
        .id_strategy
        .as_deref()
        .unwrap_or(if parent_pk_ty == "i64" {
            "snowflake"
        } else {
            "manual"
        });
    if id_strategy != "snowflake" && id_strategy != "manual" {
        panic!(
            "Model '{}' has invalid id_strategy '{}'. Supported: snowflake, manual",
            name, id_strategy
        );
    }
    if id_strategy == "snowflake" && parent_pk_ty != "i64" {
        panic!(
            "Model '{}' uses id_strategy='snowflake' but pk_type='{}'. Snowflake currently requires i64 PK.",
            name, parent_pk_ty
        );
    }
    let use_snowflake_id = !cfg.disable_id && id_strategy == "snowflake" && parent_pk_ty == "i64";
    // Emit lifecycle hooks for observed models.
    let emit_hooks = cfg.observe;
    // Skip profiler instrumentation for models with `profile = false`
    let skip_profiler = !cfg.profile;
    let has_created_at = fields.iter().any(|f| f.name == "created_at");
    let has_updated_at = fields.iter().any(|f| f.name == "updated_at");
    let has_soft_delete = fields.iter().any(|f| f.name == "deleted_at");
    let relations = parse_relations(schema, cfg, name, &fields);
    let has_many_rels: Vec<_> = relations
        .iter()
        .filter(|relation| matches!(relation.kind, RelationKind::HasMany))
        .collect();
    let max_rel_depth = cfg
        .datatable_rel_depth
        .unwrap_or(DATATABLE_REL_FILTER_MAX_DEPTH);
    let relation_paths = collect_relation_paths(schema, name, max_rel_depth);
    let computed_fields = parse_computed(cfg);
    let hidden_fields: BTreeSet<String> = cfg
        .hidden
        .clone()
        .unwrap_or_default()
        .into_iter()
        .map(|s| to_snake(&s))
        .collect();

    // Parse touch relations
    let mut touch_targets: Vec<(String, String, String, String, String)> = Vec::new(); // (fk_col, target_model_snake, target_model_title, target_pk, target_pk_ty)
    if let Some(touches) = &cfg.touch {
        for rel_name in touches {
            if let Some(rel) = relations
                .iter()
                .find(|r| &r.name == rel_name && matches!(r.kind, RelationKind::BelongsTo))
            {
                let target_snake = to_snake(&rel.target_model);
                let target_title = to_title_case(&rel.target_model);
                touch_targets.push((
                    rel.foreign_key.clone(),
                    target_snake,
                    target_title,
                    rel.target_pk.clone(),
                    rel.target_pk_ty.clone(),
                ));
            } else {
                panic!(
                    "Model '{}' configures touch='{}' but no such belongs_to relation found.",
                    name, rel_name
                );
            }
        }
    }
    let base_select = fields
        .iter()
        .map(|f| f.name.clone())
        .collect::<Vec<_>>()
        .join(", ");

    let mut imports = String::new();
    writeln!(imports, "use anyhow::Result;").unwrap();
    if !relations.is_empty() || !localized_fields.is_empty() || has_meta || has_attachments {
        writeln!(imports, "use std::collections::HashMap;").unwrap();
    }
    if has_meta {
        writeln!(imports, "use serde_json::Value as JsonValue;").unwrap();
    }
    writeln!(imports, "use serde::{{Deserialize, Serialize}};").unwrap();
    writeln!(imports, "use schemars::JsonSchema;").unwrap();

    writeln!(imports, "use sqlx::FromRow;").unwrap();
    if use_snowflake_id {
        writeln!(
            imports,
            "use core_db::common::sql::{{BindValue, Op, OrderDir, SetMode, bind, bind_query, bind_scalar, generate_snowflake_i64, is_sql_profiler_enabled, format_duration, record_profiled_query, DbConn}};"
        )
        .unwrap();
    } else {
        writeln!(
            imports,
            "use core_db::common::sql::{{BindValue, Op, OrderDir, SetMode, bind, bind_query, bind_scalar, is_sql_profiler_enabled, format_duration, record_profiled_query, DbConn}};"
        )
        .unwrap();
    }
    writeln!(
        imports,
        "use core_db::common::pagination::resolve_per_page;"
    )
    .unwrap();
    if options.include_datatable {
        writeln!(
            imports,
            "use core_datatable::{{AutoDataTable, BoxFuture, DataTableColumnDescriptor, DataTableContext, DataTableInput, DataTableRelationColumnDescriptor, GeneratedTableAdapter, ParsedFilter, SortDirection}};"
        )
        .unwrap();
    }
    if has_attachments {
        writeln!(imports, "use core_db::platform::attachments::types::{{Attachment, AttachmentInput, AttachmentMap}};").unwrap();
        writeln!(imports, "use uuid::Uuid;").unwrap();
    }
    writeln!(
        imports,
        "use core_db::platform::localized::types::LocalizedMap;"
    )
    .unwrap();
    writeln!(
        imports,
        "use crate::generated::models::common::{{FieldChange, FieldInput, Page, log_observer_error, renumber_placeholders}};"
    )
    .unwrap();
    writeln!(
        imports,
        "use core_db::common::model_api::{{ColExpr, Column, Create, CreateState, ManyRelation, ModelDef, OneRelation, Patch, PatchState, Query, QueryState}};"
    )
    .unwrap();
    if has_meta {
        writeln!(imports, "use core_db::platform::meta::types::MetaMap;").unwrap();
    }
    if !localized_fields.is_empty() || has_meta || has_attachments {
        writeln!(imports, "use crate::generated::localized;").unwrap();
    }
    if !localized_fields.is_empty() {
        writeln!(imports, "use core_i18n::current_locale;").unwrap();
    }
    {
        let mut imported_models = std::collections::BTreeSet::new();
        for rel in &relations {
            let target_mod = to_snake(&rel.target_model);
            if target_mod != model_snake && imported_models.insert(target_mod.clone()) {
                let target_title = to_title_case(&rel.target_model);
                let target_relations = schema
                    .models
                    .get(&rel.target_model)
                    .map(|model| parse_relations(schema, model, &rel.target_model, &[]))
                    .unwrap_or_default();
                let target_rel_import = if target_relations.is_empty() {
                    String::new()
                } else {
                    format!(", {}Rel", target_title)
                };
                writeln!(
                    imports,
                    "use crate::generated::models::{}::{{{}DbCol, {}Model, {}Record, {}Row{}}};",
                    target_mod, target_title, target_title, target_title, target_title, target_rel_import
                )
                .unwrap();
            }
        }
        // Import column types for nested relation paths used in datatable filters
        for rel_path in &relation_paths {
            let target_mod = to_snake(&rel_path.target_model);
            if target_mod != model_snake && imported_models.insert(target_mod.clone()) {
                let target_title = to_title_case(&rel_path.target_model);
                let target_relations = schema
                    .models
                    .get(&rel_path.target_model)
                    .map(|model| parse_relations(schema, model, &rel_path.target_model, &[]))
                    .unwrap_or_default();
                let target_rel_import = if target_relations.is_empty() {
                    String::new()
                } else {
                    format!(", {}Rel", target_title)
                };
                writeln!(
                    imports,
                    "use crate::generated::models::{}::{{{}DbCol, {}Model, {}Record, {}Row{}}};",
                    target_mod, target_title, target_title, target_title, target_title, target_rel_import
                )
                .unwrap();
            }
        }
    }
    if !localized_fields.is_empty() {
        writeln!(
            imports,
            "use crate::generated::localized::LocalizedMapHelper;"
        )
        .unwrap();
    }
    if !enum_explained_fields.is_empty() {
        writeln!(imports, "use super::enums::*;").unwrap();
    }
    if emit_hooks {
        writeln!(
            imports,
            "use core_db::common::model_observer::{{ModelEvent, ObserverAction, try_get_observer}};"
        )
        .unwrap();
    }

    let mut constants = String::new();
    writeln!(
        constants,
        "const HAS_CREATED_AT: bool = {};",
        has_created_at
    )
    .unwrap();
    writeln!(
        constants,
        "const HAS_UPDATED_AT: bool = {};",
        has_updated_at
    )
    .unwrap();
    writeln!(
        constants,
        "const HAS_SOFT_DELETE: bool = {};",
        has_soft_delete
    )
    .unwrap();

    let create_input_struct_ident = create_input_ident(&model_title);
    let update_changes_struct_ident = update_changes_ident(&model_title);
    let mut out = String::new();

    if !cfg.helper_items.is_empty() {
        writeln!(out, "{}", cfg.helper_items.join("\n")).unwrap();
        writeln!(out).unwrap();
    }

    writeln!(
        out,
        "#[derive(Debug, Clone, Default, Serialize, Deserialize)]"
    )
    .unwrap();
    writeln!(out, "#[doc(hidden)]").unwrap();
    writeln!(out, "pub struct {create_input_struct_ident} {{").unwrap();
    for f in &db_fields {
        writeln!(out, "    pub {}: FieldInput<{}>,", f.name, f.ty).unwrap();
    }
    writeln!(out, "}}\n").unwrap();

    writeln!(
        out,
        "#[derive(Debug, Clone, Default, Serialize, Deserialize)]"
    )
    .unwrap();
    writeln!(out, "#[doc(hidden)]").unwrap();
    writeln!(out, "pub struct {update_changes_struct_ident} {{").unwrap();
    for f in &db_fields {
        writeln!(out, "    pub {}: Option<FieldChange<{}>>,", f.name, f.ty).unwrap();
    }
    writeln!(out, "}}\n").unwrap();

    // Row
    writeln!(
        out,
        "#[derive(Debug, Clone, FromRow, Serialize, Deserialize, JsonSchema)]"
    )
    .unwrap();
    writeln!(out, "#[doc(hidden)]").unwrap();
    writeln!(out, "pub struct {row_ident} {{").unwrap();
    for f in &db_fields {
        if let Some(attr) = f.serde_attr {
            writeln!(out, "    {attr}").unwrap();
        }
        if f.ty.contains("OffsetDateTime") {
            writeln!(out, "    #[schemars(with = \"String\")]").unwrap();
        }
        writeln!(out, "    pub {}: {},", f.name, f.ty).unwrap();
    }
    writeln!(out, "}}\n").unwrap();

    let mut record_fields: Vec<String> = Vec::new();
    for f in &db_fields {
        if let Some(attr) = f.serde_attr {
            record_fields.push(format!("    {attr}"));
        }
        if f.ty.contains("OffsetDateTime") {
            record_fields.push("    #[schemars(with = \"String\")]".to_string());
        }
        record_fields.push(format!("    pub {}: {},", f.name, f.ty));
    }
    for enum_field in &enum_explained_fields {
        record_fields.push("    #[serde(default)]".to_string());
        if enum_field.optional {
            record_fields.push(format!(
                "    pub {}: Option<String>,",
                enum_field.explained_name
            ));
        } else {
            record_fields.push(format!("    pub {}: String,", enum_field.explained_name));
        }
    }
    for f in &localized_fields {
        record_fields.push("    #[serde(default)]".to_string());
        record_fields.push(format!("    pub {}: Option<String>,", f));
        record_fields.push("    #[serde(default)]".to_string());
        record_fields.push(format!(
            "    pub {f}_translations: Option<localized::LocalizedText>,"
        ));
    }
    for a in &single_attachments {
        record_fields.push("    #[serde(default)]".to_string());
        record_fields.push(format!("    pub {}: Option<Attachment>,", a.name));
        record_fields.push("    #[serde(default)]".to_string());
        record_fields.push(format!(
            "    pub {name}_url: Option<String>,",
            name = a.name
        ));
    }
    for a in &multi_attachments {
        record_fields.push("    #[serde(default)]".to_string());
        record_fields.push(format!("    pub {}: Vec<Attachment>,", a.name));
        record_fields.push("    #[serde(default)]".to_string());
        record_fields.push(format!("    pub {name}_urls: Vec<String>,", name = a.name));
    }
    if has_meta {
        record_fields.push("    #[serde(default)]".to_string());
        record_fields
            .push("    pub meta: std::collections::HashMap<String, JsonValue>,".to_string());
    }
    for rel in &relations {
        let rel_field = to_snake(&rel.name);
        let target_title = to_title_case(&rel.target_model);
        let target_record = format!("{}Record", target_title);
        record_fields.push("    #[serde(default)]".to_string());
        match rel.kind {
            RelationKind::HasMany => {
                record_fields.push(format!("    pub {rel_field}: Vec<{target_record}>,"));
            }
            RelationKind::BelongsTo => {
                record_fields.push(format!(
                    "    pub {rel_field}: Option<Box<{target_record}>>,",
                ));
            }
        }
    }
    if !has_many_rels.is_empty() {
        record_fields.push("    #[serde(skip)]".to_string());
        record_fields.push("    #[schemars(skip)]".to_string());
        record_fields
            .push("    pub __relation_counts: std::collections::HashMap<String, i64>,".to_string());
    }
    writeln!(
        out,
        "#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]"
    )
    .unwrap();
    writeln!(out, "pub struct {record_ident} {{").unwrap();
    for line in record_fields {
        writeln!(out, "{}", line).unwrap();
    }
    writeln!(out, "}}\n").unwrap();

    writeln!(out, "impl {record_ident} {{").unwrap();
    writeln!(
        out,
        "    pub fn update<'db>(&self, db: impl Into<DbConn<'db>>) -> Patch<'db, {model_title}Model> {{"
    )
    .unwrap();
    writeln!(
        out,
        "        {model_title}Model::query(db.into()).where_col({col_ident}::{pk_variant}, Op::Eq, self.{pk}.clone()).patch()",
        pk = to_snake(&pk),
        pk_variant = to_title_case(&pk)
    )
    .unwrap();
    writeln!(out, "    }}").unwrap();
    if has_meta && !hidden_fields.contains("meta") {
        for m in &meta_fields {
            match &m.ty {
                MetaType::String => {
                    writeln!(
                        out,
                        "    pub fn meta_{name}(&self) -> Option<String> {{ self.meta.get(\"{name}\").and_then(|v| v.as_str().map(|s| s.to_string())) }}",
                        name = m.name
                    )
                    .unwrap();
                }
                MetaType::Bool => {
                    writeln!(
                        out,
                        "    pub fn meta_{name}(&self) -> Option<bool> {{ self.meta.get(\"{name}\").and_then(|v| v.as_bool()) }}",
                        name = m.name
                    )
                    .unwrap();
                }
                MetaType::I32 => {
                    writeln!(
                        out,
                        "    pub fn meta_{name}(&self) -> Option<i32> {{ self.meta.get(\"{name}\").and_then(|v| v.as_i64()).and_then(|n| i32::try_from(n).ok()) }}",
                        name = m.name
                    )
                    .unwrap();
                }
                MetaType::I64 => {
                    writeln!(
                        out,
                        "    pub fn meta_{name}(&self) -> Option<i64> {{ self.meta.get(\"{name}\").and_then(|v| v.as_i64()) }}",
                        name = m.name
                    )
                    .unwrap();
                }
                MetaType::F64 => {
                    writeln!(
                        out,
                        "    pub fn meta_{name}(&self) -> Option<f64> {{ self.meta.get(\"{name}\").and_then(|v| v.as_f64()) }}",
                        name = m.name
                    )
                    .unwrap();
                }
                MetaType::Decimal => {
                    writeln!(
                        out,
                        "    pub fn meta_{name}(&self) -> Option<rust_decimal::Decimal> {{ self.meta.get(\"{name}\").and_then(|v| v.as_str()).and_then(|s| s.parse().ok()) }}",
                        name = m.name
                    )
                    .unwrap();
                }
                MetaType::DateTime => {
                    writeln!(
                        out,
                        "    pub fn meta_{name}(&self) -> Option<time::OffsetDateTime> {{ self.meta.get(\"{name}\").and_then(|v| v.as_str()).and_then(|s| time::OffsetDateTime::parse(s, &time::format_description::well_known::Rfc3339).ok()) }}",
                        name = m.name
                    )
                    .unwrap();
                }
                MetaType::Json => {
                    writeln!(
                        out,
                        "    pub fn meta_{name}_as<T: serde::de::DeserializeOwned>(&self) -> anyhow::Result<Option<T>> {{ match self.meta.get(\"{name}\") {{ None => Ok(None), Some(v) => Ok(Some(serde_json::from_value(v.clone())?)), }} }}",
                        name = m.name
                    )
                    .unwrap();
                }
                MetaType::Custom(ty) => {
                    writeln!(
                        out,
                        "    pub fn meta_{name}(&self) -> anyhow::Result<Option<{ty}>> {{ match self.meta.get(\"{name}\") {{ None => Ok(None), Some(v) => Ok(Some(serde_json::from_value(v.clone())?)), }} }}",
                        name = m.name,
                        ty = ty
                    )
                    .unwrap();
                }
            }
        }
    }
    if !localized_fields.is_empty() {
        let model_snake_upper = model_snake.to_uppercase();
        for f in &localized_fields {
            let fn_upsert = format!("upsert_{}", to_snake(f));
            writeln!(
                out,
                "    pub async fn {fn_upsert}<'a>(&self, db: DbConn<'a>, input: Option<localized::LocalizedInput>) -> Result<()> {{"
            )
            .unwrap();
            writeln!(
                out,
                "        let Some(input) = input else {{ return Ok(()); }};"
            )
            .unwrap();
            writeln!(out, "        if input.is_empty() {{ return Ok(()); }}").unwrap();
            writeln!(out, "        let map = input.to_hashmap();").unwrap();
            writeln!(
                out,
                "        localized::upsert_localized_many(db, localized::{model_snake_upper}_OWNER_TYPE, self.{pk}, \"{f}\", &map).await",
                pk = to_snake(&pk)
            )
            .unwrap();
            writeln!(out, "    }}").unwrap();

            let fn_clear = format!("clear_{}", to_snake(f));
            writeln!(
                out,
                "    pub async fn {fn_clear}<'a>(&self, db: DbConn<'a>) -> Result<()> {{"
            )
            .unwrap();
            writeln!(
                out,
                "        localized::delete_localized_field(db, localized::{model_snake_upper}_OWNER_TYPE, self.{pk}, \"{f}\").await",
                pk = to_snake(&pk)
            )
            .unwrap();
            writeln!(out, "    }}").unwrap();
        }
    }
    writeln!(out, "}}\n").unwrap();

    // Hydrate helper: combine DB row + localized/meta/attachments maps into view
    let loc_ident = if !localized_fields.is_empty() {
        "loc"
    } else {
        "_loc"
    };
    let base_url_ident = if has_attachments {
        "base_url"
    } else {
        "_base_url"
    };
    if has_meta && has_attachments {
        writeln!(
            out,
            "pub(crate) fn hydrate_record(row: {row_ident}, {loc_ident}: &LocalizedMap, meta: &MetaMap, attachments: &AttachmentMap, {base_url_ident}: Option<&str>) -> {record_ident} {{"
        )
        .unwrap();
    } else if has_meta {
        writeln!(
            out,
            "pub(crate) fn hydrate_record(row: {row_ident}, {loc_ident}: &LocalizedMap, meta: &MetaMap, {base_url_ident}: Option<&str>) -> {record_ident} {{"
        )
        .unwrap();
    } else if has_attachments {
        writeln!(
            out,
            "pub(crate) fn hydrate_record(row: {row_ident}, {loc_ident}: &LocalizedMap, attachments: &AttachmentMap, {base_url_ident}: Option<&str>) -> {record_ident} {{"
        )
        .unwrap();
    } else {
        writeln!(
            out,
            "pub(crate) fn hydrate_record(row: {row_ident}, {loc_ident}: &LocalizedMap, {base_url_ident}: Option<&str>) -> {record_ident} {{"
        )
        .unwrap();
    }
    if !localized_fields.is_empty() {
        writeln!(out, "    let locale = current_locale();").unwrap();
    }
    if !localized_fields.is_empty() || has_meta || has_attachments {
        writeln!(out, "    let mut record = {record_ident} {{").unwrap();
    } else {
        writeln!(out, "    let mut record = {record_ident} {{").unwrap();
    }
    for f in &db_fields {
        writeln!(out, "        {}: row.{},", f.name, f.name).unwrap();
    }
    for enum_field in &enum_explained_fields {
        if enum_field.optional {
            writeln!(
                out,
                "        {}: row.{}.map(|value| value.explained_label()),",
                enum_field.explained_name, enum_field.name
            )
            .unwrap();
        } else {
            writeln!(
                out,
                "        {}: row.{}.explained_label(),",
                enum_field.explained_name, enum_field.name
            )
            .unwrap();
        }
    }
    for f in &localized_fields {
        writeln!(out, "        {f}: None,").unwrap();
        writeln!(out, "        {f}_translations: None,").unwrap();
    }
    for a in &single_attachments {
        writeln!(out, "        {}: None,", a.name).unwrap();
        writeln!(out, "        {name}_url: None,", name = a.name).unwrap();
    }
    for a in &multi_attachments {
        writeln!(out, "        {}: Vec::new(),", a.name).unwrap();
        writeln!(out, "        {name}_urls: Vec::new(),", name = a.name).unwrap();
    }
    if has_meta {
        writeln!(out, "        meta: HashMap::new(),").unwrap();
    }
    for rel in &relations {
        let rel_field = to_snake(&rel.name);
        match rel.kind {
            RelationKind::HasMany => {
                writeln!(out, "        {rel_field}: Vec::new(),").unwrap();
            }
            RelationKind::BelongsTo => {
                writeln!(out, "        {rel_field}: None,").unwrap();
            }
        }
    }
    if !has_many_rels.is_empty() {
        writeln!(out, "        __relation_counts: HashMap::new(),").unwrap();
    }
    writeln!(out, "    }};").unwrap();
    for f in &localized_fields {
        writeln!(
            out,
            "    let ml_{f} = {loc_ident}.get_localized_text(\"{f}\", record.id);"
        )
        .unwrap();
        writeln!(out, "    if let Some(ref ml) = ml_{f} {{").unwrap();
        writeln!(
            out,
            "        record.{f} = Some(ml.get(locale).to_string());"
        )
        .unwrap();
        writeln!(out, "    }}").unwrap();
        writeln!(out, "    record.{f}_translations = ml_{f};").unwrap();
    }
    if has_meta {
        writeln!(out, "    record.meta = meta.get_all_for_owner(record.id);").unwrap();
    }
    if has_attachments {
        for a in &single_attachments {
            writeln!(
                out,
                "    record.{name} = attachments.get_single(\"{name}\", record.id);",
                name = a.name
            )
            .unwrap();
            writeln!(
                out,
                "    record.{name}_url = record.{name}.as_ref().map(|a| a.url_with_base({base_url_ident}));",
                name = a.name
            )
            .unwrap();
        }
        for a in &multi_attachments {
            writeln!(
                out,
                "    record.{name} = attachments.get_many(\"{name}\", record.id);",
                name = a.name
            )
            .unwrap();
            writeln!(
                out,
                "    record.{name}_urls = record.{name}.iter().map(|a| a.url_with_base({base_url_ident})).collect();",
                name = a.name
            )
            .unwrap();
        }
    }
    writeln!(out, "    record").unwrap();
    writeln!(out, "}}\n").unwrap();

    let support_loaders = render_support_data_loaders(
        &model_snake,
        &pk,
        &parent_pk_ty,
        &localized_fields,
        has_meta,
        has_attachments,
        "rows",
        "db.clone()",
    );
    writeln!(
        out,
        "pub(crate) async fn hydrate_records<'db>(db: DbConn<'db>, rows: &[{row_ident}], base_url: Option<&str>) -> Result<Vec<{record_ident}>> {{"
    )
    .unwrap();
    writeln!(out, "    if rows.is_empty() {{ return Ok(Vec::new()); }}").unwrap();
    for line in support_loaders.lines() {
        if line.trim().is_empty() {
            writeln!(out).unwrap();
        } else {
            writeln!(out, "{}", line).unwrap();
        }
    }
    writeln!(out, "    let mut records = Vec::with_capacity(rows.len());").unwrap();
    writeln!(out, "    for row in rows {{").unwrap();
    writeln!(
        out,
        "        records.push({});",
        build_hydrate_record_expr(
            "row.clone()",
            &localized_fields,
            has_meta,
            has_attachments,
            "base_url",
        )
    )
    .unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(out, "    Ok(records)").unwrap();
    writeln!(out, "}}\n").unwrap();

    if !relations.is_empty() {
        writeln!(out, "impl {record_ident} {{").unwrap();
        writeln!(
            out,
            "    pub fn one<R>(&self, relation: R) -> Option<&R::Target>"
        )
        .unwrap();
        writeln!(out, "    where").unwrap();
        writeln!(
            out,
            "        R: core_db::common::model_api::RecordOneRelation<{model_title}Model>,"
        )
        .unwrap();
        writeln!(out, "    {{").unwrap();
        writeln!(out, "        R::get(relation, self)").unwrap();
        writeln!(out, "    }}").unwrap();
        writeln!(
            out,
            "    pub fn many<R>(&self, relation: R) -> &[R::Target]"
        )
        .unwrap();
        writeln!(out, "    where").unwrap();
        writeln!(
            out,
            "        R: core_db::common::model_api::RecordManyRelation<{model_title}Model>,"
        )
        .unwrap();
        writeln!(out, "    {{").unwrap();
        writeln!(out, "        R::get(relation, self)").unwrap();
        writeln!(out, "    }}").unwrap();
        if !has_many_rels.is_empty() {
            writeln!(
                out,
                "    pub fn count<R>(&self, relation: R) -> Option<i64>"
            )
            .unwrap();
            writeln!(out, "    where").unwrap();
            writeln!(
                out,
                "        R: core_db::common::model_api::CountRelation<{model_title}Model>,"
            )
            .unwrap();
            writeln!(out, "    {{").unwrap();
            writeln!(
                out,
                "        self.__relation_counts.get(R::name(relation)).copied()"
            )
            .unwrap();
            writeln!(out, "    }}").unwrap();
        }
        writeln!(out, "}}\n").unwrap();
    }

    if !cfg.record_impl_items.is_empty() {
        out.push_str(&render_custom_impl_block(
            &record_ident,
            &cfg.record_impl_items,
        ));
    }

    let row_view_json_section = out;
    let mut out = String::new();

    out.push_str(&render_public_column_namespace(
        &model_title,
        &model_snake,
        &col_ident,
        &public_col_ident,
        &db_fields,
    ));

    // Internal DB col enum
    writeln!(out, "#[derive(Debug, Clone, Copy, JsonSchema)]").unwrap();
    writeln!(out, "pub enum {col_ident} {{").unwrap();
    for f in &db_fields {
        writeln!(out, "    {},", to_title_case(&f.name)).unwrap();
    }
    writeln!(out, "}}\n").unwrap();

    writeln!(out, "impl {col_ident} {{").unwrap();
    writeln!(out, "    pub const fn all() -> &'static [{col_ident}] {{").unwrap();
    writeln!(
        out,
        "        &[{}]",
        db_fields
            .iter()
            .map(|f| format!("{col_ident}::{}", to_title_case(&f.name)))
            .collect::<Vec<_>>()
            .join(", ")
    )
    .unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(out, "    pub const fn as_sql(self) -> &'static str {{").unwrap();
    writeln!(out, "        match self {{").unwrap();
    for f in &db_fields {
        writeln!(
            out,
            "            {col_ident}::{} => \"{}\",",
            to_title_case(&f.name),
            f.name
        )
        .unwrap();
    }
    writeln!(out, "        }}").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(out, "}}\n").unwrap();

    if !relations.is_empty() {
        let rel_ident = format!("{}Rel", model_title);
        writeln!(out, "#[derive(Debug, Clone, Copy, Default)]").unwrap();
        writeln!(out, "pub struct {rel_ident};").unwrap();
        writeln!(out, "impl {rel_ident} {{").unwrap();
        for (rel_idx, rel) in relations.iter().enumerate() {
            let rel_const = to_snake(&rel.name).to_uppercase();
            let target_title = to_title_case(&rel.target_model);
            let target_record_ident = format!("{target_title}Record");
            let rel_ty = match rel.kind {
                RelationKind::BelongsTo => {
                    format!("OneRelation<{model_title}Model, {target_record_ident}, {rel_idx}>")
                }
                RelationKind::HasMany => {
                    format!("ManyRelation<{model_title}Model, {target_record_ident}, {rel_idx}>")
                }
            };
            let rel_value = match rel.kind {
                RelationKind::BelongsTo => format!(
                    "OneRelation::<{model_title}Model, {target_record_ident}, {rel_idx}>::new(\"{rel_name}\")",
                    rel_name = rel.name
                ),
                RelationKind::HasMany => {
                    let target_soft_delete = schema
                        .models
                        .get(&rel.target_model)
                        .map(|m| m.soft_delete)
                        .unwrap_or(false);
                    let ctor = if target_soft_delete {
                        "new_with_soft_delete"
                    } else {
                        "new"
                    };
                    format!(
                        "ManyRelation::<{model_title}Model, {target_record_ident}, {rel_idx}>::{ctor}(\"{rel_name}\", \"{target_table}\", \"{target_pk}\", \"{foreign_key}\")",
                        rel_name = rel.name,
                        target_table = rel.target_table,
                        target_pk = rel.target_pk,
                        foreign_key = rel.foreign_key
                    )
                },
            };
            writeln!(out, "    pub const {rel_const}: {rel_ty} = {rel_value};",).unwrap();
        }
        writeln!(out, "}}\n").unwrap();
    }

    // localized setters are provided on insert/update builders via *_lang methods
    for rel in &relations {
        let fn_name = format!("load_{}", to_snake(&rel.name));
        let target_title = to_title_case(&rel.target_model);
        let target_row = format!("{}Row", target_title);
        let target_record = format!("{}Record", target_title);
        let target_model_snake = to_snake(&rel.target_model);
        let target_hydrate_records_expr = if rel.target_model == name {
            "hydrate_records(db.clone(), &rows, base_url).await?".to_string()
        } else {
            format!(
                "crate::generated::models::{target_model_snake}::hydrate_records(db.clone(), &rows, base_url).await?"
            )
        };
        match rel.kind {
            RelationKind::HasMany => {
                let target_fk_optional = relation_target_field_is_optional(schema, rel);
                writeln!(out, "async fn {fn_name}<'db>(db: DbConn<'db>, parents: &[{row_ident}], base_url: Option<&str>) -> Result<HashMap<{parent_pk_ty}, Vec<{target_record}>>> {{").unwrap();
                writeln!(
                    out,
                    "        if parents.is_empty() {{ return Ok(HashMap::new()); }}"
                )
                .unwrap();
                writeln!(out, "        let ids: Vec<{parent_pk_ty}> = parents.iter().map(|p| p.{pk}.clone()).collect();").unwrap();
                writeln!(out, "        let placeholders: Vec<String> = (1..=ids.len()).map(|i| format!(\"${{}}\", i)).collect();").unwrap();
                writeln!(out, "        let sql = format!(\"SELECT * FROM {rel_table} WHERE {fk} IN ({{}})\", placeholders.join(\", \"));", rel_table = rel.target_table, fk = rel.foreign_key).unwrap();
                writeln!(
                    out,
                    "        let mut q = sqlx::query_as::<_, {target_row}>(&sql);"
                )
                .unwrap();
                writeln!(out, "        for id in ids {{ q = bind(q, id.into()); }}").unwrap();
                writeln!(out, "        let rows = db.fetch_all(q).await?;").unwrap();
                writeln!(
                    out,
                    "        let records = {target_hydrate_records_expr};"
                )
                .unwrap();
                writeln!(out, "        let mut map: HashMap<{parent_pk_ty}, Vec<{target_record}>> = HashMap::new();").unwrap();
                writeln!(out, "        for record in records {{").unwrap();
                if target_fk_optional {
                    writeln!(
                        out,
                        "            if let Some(fk_val) = record.{fk}.clone() {{ map.entry(fk_val).or_default().push(record); }}",
                        fk = rel.foreign_key
                    )
                    .unwrap();
                } else {
                    writeln!(
                        out,
                        "            let fk_val = record.{fk}.clone();",
                        fk = rel.foreign_key
                    )
                    .unwrap();
                    writeln!(
                        out,
                        "            map.entry(fk_val).or_default().push(record);"
                    )
                    .unwrap();
                }
                writeln!(out, "        }}").unwrap();
                writeln!(out, "        Ok(map)").unwrap();
                writeln!(out, "    }}").unwrap();
            }
            RelationKind::BelongsTo => {
                let is_fk_optional = fields
                    .iter()
                    .any(|f| f.name == rel.foreign_key && f.ty.starts_with("Option<"));
                writeln!(out, "async fn {fn_name}<'db>(db: DbConn<'db>, parents: &[{row_ident}], base_url: Option<&str>) -> Result<HashMap<{parent_pk_ty}, Option<{target_record}>>> {{").unwrap();
                writeln!(
                    out,
                    "        if parents.is_empty() {{ return Ok(HashMap::new()); }}"
                )
                .unwrap();
                writeln!(out, "        let mut fk_vals = Vec::new();").unwrap();
                writeln!(out, "        let mut parent_pairs = Vec::new();").unwrap();
                writeln!(out, "        for p in parents {{").unwrap();
                if is_fk_optional {
                    writeln!(
                        out,
                        "            if let Some(fk_val) = p.{fk}.clone() {{ fk_vals.push(fk_val); parent_pairs.push((p.{pk}.clone(), Some(fk_val))); }} else {{ parent_pairs.push((p.{pk}.clone(), None)); }}",
                        fk = rel.foreign_key,
                        pk = pk
                    )
                    .unwrap();
                } else {
                    writeln!(
                        out,
                        "            fk_vals.push(p.{fk}.clone());",
                        fk = rel.foreign_key
                    )
                    .unwrap();
                    writeln!(
                        out,
                        "            parent_pairs.push((p.{pk}.clone(), Some(p.{fk}.clone())));",
                        pk = pk,
                        fk = rel.foreign_key
                    )
                    .unwrap();
                }
                writeln!(out, "        }}").unwrap();
                writeln!(
                    out,
                    "        if fk_vals.is_empty() {{ return Ok(HashMap::new()); }}"
                )
                .unwrap();
                writeln!(out, "        let placeholders: Vec<String> = (1..=fk_vals.len()).map(|i| format!(\"${{}}\", i)).collect();").unwrap();
                writeln!(out, "        let sql = format!(\"SELECT * FROM {rel_table} WHERE {target_pk} IN ({{}})\", placeholders.join(\", \"));", rel_table = rel.target_table, target_pk = rel.target_pk).unwrap();
                writeln!(
                    out,
                    "        let mut q = sqlx::query_as::<_, {target_row}>(&sql);"
                )
                .unwrap();
                writeln!(
                    out,
                    "        for fk in fk_vals {{ q = bind(q, fk.into()); }}"
                )
                .unwrap();
                writeln!(out, "        let rows = db.fetch_all(q).await?;").unwrap();
                writeln!(
                    out,
                    "        let records = {target_hydrate_records_expr};"
                )
                .unwrap();
                writeln!(out, "        let mut by_pk: HashMap<{target_pk_ty}, {target_record}> = HashMap::new();", target_pk_ty = rel.target_pk_ty).unwrap();
                writeln!(out, "        for record in records {{").unwrap();
                writeln!(
                    out,
                    "            let key = record.{target_pk}.clone();",
                    target_pk = rel.target_pk
                )
                .unwrap();
                writeln!(out, "            by_pk.insert(key, record);").unwrap();
                writeln!(out, "        }}").unwrap();
                writeln!(out, "        let mut out = HashMap::new();").unwrap();
                writeln!(out, "        for (pid, fk) in parent_pairs {{").unwrap();
                writeln!(
                    out,
                    "            out.insert(pid, fk.and_then(|k| by_pk.get(&k).cloned()));"
                )
                .unwrap();
                writeln!(out, "        }}").unwrap();
                writeln!(out, "        Ok(out)").unwrap();
                writeln!(out, "    }}").unwrap();
            }
        }
    }
    writeln!(out).unwrap();

    let column_model_section = out;

    // Query inner type (XxxQueryInner) is no longer generated.
    // InnerQuery<'db> = QueryState<'db> — all SQL builder and terminal method logic
    // is in the QueryModel trait impl and QueryState methods in core_db::common::model_api.
    let query_struct_section = String::new();
    let query_builder_methods_section = String::new();
    let query_terminal_methods_section = String::new();
    let unsafe_query_section = String::new();
    let mut query_context = TemplateContext::new();
    query_context
        .insert(
            "query_struct_section",
            query_struct_section.trim_start().to_string(),
        )
        .unwrap();
    query_context
        .insert(
            "query_builder_methods_section",
            query_builder_methods_section.trim_start().to_string(),
        )
        .unwrap();
    query_context
        .insert(
            "query_terminal_methods_section",
            query_terminal_methods_section.trim_start().to_string(),
        )
        .unwrap();
    query_context
        .insert("unsafe_query_section", unsafe_query_section)
        .unwrap();
    let query_section = render_template("models/query.rs.tpl", &query_context).unwrap();
    let mut out = String::new();

    // Insert builder — wraps CreateState for SQL, adds model-specific extension fields
    writeln!(out, "pub struct {insert_ident}<'db> {{").unwrap();
    writeln!(
        out,
        "    pub(crate) state: core_db::common::model_api::CreateState<'db>,"
    )
    .unwrap();
    if !localized_fields.is_empty() {
        writeln!(
            out,
            "    translations: HashMap<&'static str, HashMap<String, String>>,"
        )
        .unwrap();
    }
    if has_meta {
        writeln!(out, "    meta: HashMap<String, JsonValue>,").unwrap();
    }
    if has_attachments {
        writeln!(
            out,
            "    attachments_single: HashMap<&'static str, AttachmentInput>,"
        )
        .unwrap();
        writeln!(
            out,
            "    attachments_multi: HashMap<&'static str, Vec<AttachmentInput>>,"
        )
        .unwrap();
    }
    writeln!(out, "}}\n").unwrap();

    writeln!(out, "impl<'db> {insert_ident}<'db> {{").unwrap();
    writeln!(
        out,
        "    pub fn new(db: DbConn<'db>, base_url: Option<String>) -> Self {{"
    )
    .unwrap();
    writeln!(out, "        Self {{").unwrap();
    writeln!(out, "            state: core_db::common::model_api::CreateState::new(db, base_url, \"{table}\"),").unwrap();
    if !localized_fields.is_empty() {
        writeln!(out, "            translations: HashMap::new(),").unwrap();
    }
    if has_meta {
        writeln!(out, "            meta: HashMap::new(),").unwrap();
    }
    if has_attachments {
        writeln!(out, "            attachments_single: HashMap::new(),").unwrap();
        writeln!(out, "            attachments_multi: HashMap::new(),").unwrap();
    }
    writeln!(out, "        }}").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(
        out,
        "    pub fn from_state(state: core_db::common::model_api::CreateState<'db>) -> Self {{"
    )
    .unwrap();
    writeln!(out, "        Self {{").unwrap();
    writeln!(out, "            state,").unwrap();
    if !localized_fields.is_empty() {
        writeln!(out, "            translations: HashMap::new(),").unwrap();
    }
    if has_meta {
        writeln!(out, "            meta: HashMap::new(),").unwrap();
    }
    if has_attachments {
        writeln!(out, "            attachments_single: HashMap::new(),").unwrap();
        writeln!(out, "            attachments_multi: HashMap::new(),").unwrap();
    }
    writeln!(out, "        }}").unwrap();
    writeln!(out, "    }}").unwrap();

    let insert_struct_section = out;
    let mut out = String::new();

    out.push_str(&render_insert_field_setters(&db_fields, &col_ident));
    if !localized_fields.is_empty() {
        out.push_str(&render_localized_setters(&localized_fields, cfgs));
    }
    if has_meta {
        out.push_str(&render_meta_setters(&meta_fields));
    }
    if has_attachments {
        out.push_str(&render_insert_attachment_setters(
            &single_attachments,
            &multi_attachments,
        ));
    }

    // on_conflict_do_nothing - INSERT ... ON CONFLICT DO NOTHING
    writeln!(
        out,
        "    pub fn on_conflict_do_nothing(mut self, conflict_cols: &[{col_ident}]) -> Self {{"
    )
    .unwrap();
    writeln!(out, "        self.state = self.state.on_conflict_do_nothing(&conflict_cols.iter().map(|c| c.as_sql()).collect::<Vec<_>>());").unwrap();
    writeln!(out, "        self").unwrap();
    writeln!(out, "    }}").unwrap();

    // on_conflict_update - INSERT ... ON CONFLICT (cols) DO UPDATE SET ...
    writeln!(
        out,
        "    pub fn on_conflict_update(mut self, conflict_cols: &[{col_ident}]) -> Self {{"
    )
    .unwrap();
    writeln!(out, "        self.state = self.state.on_conflict_update(&conflict_cols.iter().map(|c| c.as_sql()).collect::<Vec<_>>());").unwrap();
    writeln!(out, "        self").unwrap();
    writeln!(out, "    }}").unwrap();

    writeln!(
        out,
        "    fn to_create_input(&self) -> Result<{create_input_struct_ident}> {{"
    )
    .unwrap();
    writeln!(
        out,
        "        let mut input = {create_input_struct_ident}::default();"
    )
    .unwrap();
    writeln!(
        out,
        "        for (col_name, bind) in self.state.col_names.iter().zip(self.state.binds.iter()) {{"
    )
    .unwrap();
    writeln!(out, "            match *col_name {{").unwrap();
    for f in &db_fields {
        let decode_expr = render_bind_decode_expr(&f.ty, "bind", &enum_specs);
        writeln!(out, "                \"{}\" => {{", f.name).unwrap();
        writeln!(out, "                    let value = {decode_expr};").unwrap();
        writeln!(
            out,
            "                    input.{} = FieldInput::Set(value);",
            f.name
        )
        .unwrap();
        writeln!(out, "                }}").unwrap();
    }
    writeln!(out, "                _ => {{}}").unwrap();
    writeln!(out, "            }}").unwrap();
    writeln!(out, "        }}").unwrap();
    writeln!(out, "        Ok(input)").unwrap();
    writeln!(out, "    }}").unwrap();

    if emit_hooks {
        writeln!(out).unwrap();
        writeln!(
            out,
            "    fn apply_create_overrides(mut state: CreateState<'_>, overrides: serde_json::Value) -> Result<CreateState<'_>> {{"
        )
        .unwrap();
        writeln!(out, "        let map = overrides.as_object()").unwrap();
        writeln!(
            out,
            "            .ok_or_else(|| anyhow::anyhow!(\"observer overrides must be a JSON object\"))?;"
        )
        .unwrap();
        writeln!(out, "        for (key, val) in map {{").unwrap();
        writeln!(out, "            match key.as_str() {{").unwrap();
        for f in &db_fields {
            let deser_ty = json_deser_type_for_field(&f.ty, &enum_specs);
            writeln!(out, "                \"{}\" => {{", f.name).unwrap();
            writeln!(
                out,
                "                    let v: {deser_ty} = serde_json::from_value(val.clone())?;"
            )
            .unwrap();
            writeln!(
                out,
                "                    state = state.set_col(\"{}\", v.into());",
                f.name
            )
            .unwrap();
            writeln!(out, "                }}").unwrap();
        }
        writeln!(
            out,
            "                other => anyhow::bail!(\"unknown column '{{}}' in observer create overrides\", other),"
        )
        .unwrap();
        writeln!(out, "            }}").unwrap();
        writeln!(out, "        }}").unwrap();
        writeln!(out, "        Ok(state)").unwrap();
        writeln!(out, "    }}").unwrap();
    }

    let insert_builder_methods_section = out;
    let mut out = String::new();

    writeln!(
        out,
        "    pub async fn save(mut self) -> Result<{record_ident}> {{"
    )
    .unwrap();
    if emit_hooks {
        writeln!(
            out,
            "        let __create_input = if try_get_observer().is_some() {{"
        )
        .unwrap();
        writeln!(out, "            Some(self.to_create_input()?)").unwrap();
        writeln!(out, "        }} else {{").unwrap();
        writeln!(out, "            None").unwrap();
        writeln!(out, "        }};").unwrap();
        writeln!(out, "        if let Some(observer) = try_get_observer() {{").unwrap();
        writeln!(
            out,
            "            if let Some(create_input) = __create_input.as_ref() {{"
        )
        .unwrap();
        writeln!(
            out,
            "                let event = ModelEvent {{ model: \"{model_snake}\", table: \"{table}\", record_key: None }};"
        )
        .unwrap();
        writeln!(
            out,
            "                let data = serde_json::to_value(create_input)?;"
        )
        .unwrap();
        writeln!(
            out,
            "                let action = observer.on_creating(&event, &data).await?;"
        )
        .unwrap();
        writeln!(out, "                match action {{").unwrap();
        writeln!(
            out,
            "                    ObserverAction::Prevent(err) => return Err(err),"
        )
        .unwrap();
        writeln!(
            out,
            "                    ObserverAction::Modify(overrides) => {{"
        )
        .unwrap();
        writeln!(out, "                        self.state = Self::apply_create_overrides(self.state, overrides)?;").unwrap();
        writeln!(out, "                    }}").unwrap();
        writeln!(out, "                    ObserverAction::Continue => {{}}").unwrap();
        writeln!(out, "                }}").unwrap();
        writeln!(out, "            }}").unwrap();
        writeln!(out, "        }}").unwrap();
    }
    writeln!(out, "        let db_conn = self.state.db.clone();").unwrap();
    writeln!(out, "        match db_conn {{").unwrap();
    writeln!(out, "            DbConn::Pool(pool) => {{").unwrap();
    writeln!(out, "                let tx = pool.begin().await?;").unwrap();
    writeln!(
        out,
        "                let tx_lock = std::sync::Arc::new(tokio::sync::Mutex::new(tx));"
    )
    .unwrap();
    writeln!(out, "                let (record, row) = {{").unwrap();
    writeln!(
        out,
        "                    let db = DbConn::tx(tx_lock.clone());"
    )
    .unwrap();
    writeln!(out, "                    self.save_with_db(db).await?").unwrap();
    writeln!(out, "                }};").unwrap();
    writeln!(
        out,
        "                let tx = std::sync::Arc::try_unwrap(tx_lock)"
    )
    .unwrap();
    writeln!(out, "                    .map_err(|_| anyhow::anyhow!(\"transaction scope still has active handles\"))?")
        .unwrap();
    writeln!(out, "                    .into_inner();").unwrap();
    writeln!(out, "                tx.commit().await?;").unwrap();
    if emit_hooks {
        writeln!(
            out,
            "                if let Some(observer) = try_get_observer() {{"
        )
        .unwrap();
        writeln!(
            out,
            "                    let event = ModelEvent {{ model: \"{model_snake}\", table: \"{table}\", record_key: Some(format!(\"{{}}\", row.{pk_snake})) }};",
            pk_snake = to_snake(&pk)
        )
        .unwrap();
        writeln!(
            out,
            "                    match serde_json::to_value(&row) {{"
        )
        .unwrap();
        writeln!(out, "                        Ok(data) => {{").unwrap();
        writeln!(
            out,
            "                            if let Err(err) = observer.on_created(&event, &data).await {{"
        )
        .unwrap();
        writeln!(
            out,
            "                                log_observer_error(\"created\", \"{model_snake}\", &err);"
        )
        .unwrap();
        writeln!(out, "                            }}").unwrap();
        writeln!(out, "                        }}").unwrap();
        writeln!(
            out,
            "                        Err(err) => log_observer_error(\"created\", \"{model_snake}\", &err),"
        )
        .unwrap();
        writeln!(out, "                    }}").unwrap();
        writeln!(out, "                }}").unwrap();
    }
    writeln!(out, "                Ok(record)").unwrap();
    writeln!(out, "            }}").unwrap();
    writeln!(out, "            DbConn::Tx(_) => {{").unwrap();
    writeln!(
        out,
        "                let (record, row) = self.save_with_db(db_conn).await?;"
    )
    .unwrap();
    if emit_hooks {
        writeln!(
            out,
            "                if let Some(observer) = try_get_observer() {{"
        )
        .unwrap();
        writeln!(
            out,
            "                    let event = ModelEvent {{ model: \"{model_snake}\", table: \"{table}\", record_key: Some(format!(\"{{}}\", row.{pk_snake})) }};",
            pk_snake = to_snake(&pk)
        )
        .unwrap();
        writeln!(
            out,
            "                    match serde_json::to_value(&row) {{"
        )
        .unwrap();
        writeln!(out, "                        Ok(data) => {{").unwrap();
        writeln!(
            out,
            "                            if let Err(err) = observer.on_created(&event, &data).await {{"
        )
        .unwrap();
        writeln!(
            out,
            "                                log_observer_error(\"created\", \"{model_snake}\", &err);"
        )
        .unwrap();
        writeln!(out, "                            }}").unwrap();
        writeln!(out, "                        }}").unwrap();
        writeln!(
            out,
            "                        Err(err) => log_observer_error(\"created\", \"{model_snake}\", &err),"
        )
        .unwrap();
        writeln!(out, "                    }}").unwrap();
        writeln!(out, "                }}").unwrap();
    }
    writeln!(out, "                Ok(record)").unwrap();
    writeln!(out, "            }}").unwrap();
    writeln!(out, "        }}").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(out).unwrap();
    writeln!(
        out,
        "    async fn save_with_db<'tx>(mut self, db: DbConn<'tx>) -> Result<({record_ident}, {row_ident})> {{"
    )
    .unwrap();
    if use_snowflake_id {
        writeln!(
            out,
            "        if !self.state.col_names.contains(&\"{pk}\") {{"
        )
        .unwrap();
        writeln!(
            out,
            "            self.state = self.state.set_col(\"{pk}\", generate_snowflake_i64().into());"
        )
        .unwrap();
        writeln!(out, "        }}").unwrap();
    }
    if has_created_at {
        writeln!(
            out,
            "        if HAS_CREATED_AT && !self.state.col_names.contains(&\"created_at\") {{"
        )
        .unwrap();
        writeln!(
            out,
            "            self.state = self.state.set_col(\"created_at\", time::OffsetDateTime::now_utc().into());"
        )
        .unwrap();
        writeln!(out, "        }}").unwrap();
    }
    if has_updated_at {
        writeln!(
            out,
            "        if HAS_UPDATED_AT && !self.state.col_names.contains(&\"updated_at\") {{"
        )
        .unwrap();
        writeln!(
            out,
            "            self.state = self.state.set_col(\"updated_at\", time::OffsetDateTime::now_utc().into());"
        )
        .unwrap();
        writeln!(out, "        }}").unwrap();
    }
    writeln!(out, "        if self.state.col_names.is_empty() {{").unwrap();
    writeln!(
        out,
        "            anyhow::bail!(\"insert: no columns set\");"
    )
    .unwrap();
    writeln!(out, "        }}").unwrap();
    writeln!(out, "        let base_url = self.state.base_url.clone();").unwrap();
    writeln!(
        out,
        "        let (sql, binds) = self.state.build_insert_sql();"
    )
    .unwrap();
    writeln!(out, "        let __profiler_binds = if is_sql_profiler_enabled() {{ binds.iter().map(|b| format!(\"{{}}\", b)).collect::<Vec<_>>().join(\", \") }} else {{ String::new() }};").unwrap();
    out.push_str(&render_profiler_start(skip_profiler));
    writeln!(
        out,
        "        let mut q = sqlx::query_as::<_, {row_ident}>(&sql);"
    )
    .unwrap();
    writeln!(out, "        for b in binds {{").unwrap();
    writeln!(out, "            q = bind(q, b);").unwrap();
    writeln!(out, "        }}").unwrap();
    writeln!(out, "        let row = db.fetch_one(q).await?;").unwrap();
    out.push_str(&render_profiler_log(
        &table,
        "INSERT",
        "&sql",
        "&__profiler_binds",
        skip_profiler,
    ));
    if !localized_fields.is_empty() {
        writeln!(out, "        if !self.translations.is_empty() {{").unwrap();
        writeln!(
            out,
            "            let supported = localized::SUPPORTED_LOCALES;"
        )
        .unwrap();
        for f in &localized_fields {
            writeln!(
                out,
                "            if let Some(map) = self.translations.get(\"{f}\") {{"
            )
            .unwrap();
            writeln!(out, "                let mut filtered = HashMap::new();").unwrap();
            writeln!(out, "                for (loc, val) in map {{").unwrap();
            writeln!(out, "                    if supported.contains(&loc.as_str()) {{ filtered.insert(loc.clone(), val.clone()); }}").unwrap();
            writeln!(out, "                }}").unwrap();
            writeln!(out, "                if !filtered.is_empty() {{").unwrap();
            writeln!(out, "                    localized::upsert_localized_many(db.clone(), localized::{}_OWNER_TYPE, row.{pk}, \"{f}\", &filtered).await?;", model_snake.to_uppercase()).unwrap();
            writeln!(out, "                }}").unwrap();
            writeln!(out, "            }}").unwrap();
        }
        writeln!(out, "        }}").unwrap();
    }
    if has_meta {
        writeln!(out, "        if !self.meta.is_empty() {{").unwrap();
        writeln!(out, "            localized::upsert_meta_many(db.clone(), localized::{model_snake_upper}_OWNER_TYPE, row.{pk}, &self.meta).await?;", model_snake_upper = model_snake.to_uppercase()).unwrap();
        writeln!(out, "        }}").unwrap();
    }
    if has_attachments {
        writeln!(out, "        if !self.attachments_single.is_empty() || !self.attachments_multi.is_empty() {{").unwrap();
        writeln!(
            out,
            "            for (field, att) in &self.attachments_single {{"
        )
        .unwrap();
        writeln!(out, "                localized::replace_single_attachment(db.clone(), localized::{}_OWNER_TYPE, row.{pk}, field, att).await?;", model_snake_upper).unwrap();
        writeln!(out, "            }}").unwrap();
        writeln!(
            out,
            "            for (field, list) in &self.attachments_multi {{"
        )
        .unwrap();
        writeln!(out, "                localized::add_attachments(db.clone(), localized::{}_OWNER_TYPE, row.{pk}, field, list).await?;", model_snake_upper).unwrap();
        writeln!(out, "            }}").unwrap();
        writeln!(out, "        }}").unwrap();
    }
    // touch parent timestamps
    for (fk, target_snake, target_title, target_pk, _target_pk_ty) in &touch_targets {
        writeln!(out, "        if let Some(parent_id) = row.{} {{", fk).unwrap();
        writeln!(
            out,
            "            crate::generated::models::{}::{}::new(db.clone()).update()",
            target_snake, target_title
        )
        .unwrap();
        writeln!(
            out,
            "                .where_{target_pk}(Op::Eq, parent_id)",
            target_pk = target_pk
        )
        .unwrap();
        writeln!(
            out,
            "                .set_updated_at(time::OffsetDateTime::now_utc())"
        )
        .unwrap();
        writeln!(out, "                .save().await?;").unwrap();
        writeln!(out, "        }}").unwrap();
    }
    if localized_fields.is_empty() {
        writeln!(out, "        let localized = LocalizedMap::default();").unwrap();
    } else {
        writeln!(out, "        let localized = localized::load_{model_snake}_localized(db, &[row.{pk}]).await?;").unwrap();
    }
    if has_attachments {
        writeln!(out, "        let attachments = localized::load_{model_snake}_attachments(db, &[row.{pk}]).await?;").unwrap();
    }
    if has_meta {
        writeln!(
            out,
            "        let meta_map = localized::load_{model_snake}_meta(db, &[row.{pk}]).await?;"
        )
        .unwrap();
        match has_attachments {
            true => writeln!(out, "        let record = hydrate_record(row.clone(), &localized, &meta_map, &attachments, base_url.as_deref());").unwrap(),
            false => writeln!(out, "        let record = hydrate_record(row.clone(), &localized, &meta_map, base_url.as_deref());").unwrap(),
        }
    } else {
        match has_attachments {
            true => writeln!(
                out,
                "        let record = hydrate_record(row.clone(), &localized, &attachments, base_url.as_deref());"
            )
            .unwrap(),
            false => {
                if localized_fields.is_empty() {
                    writeln!(out, "        let record = hydrate_record(row.clone(), &LocalizedMap::default(), base_url.as_deref());").unwrap();
                } else {
                    writeln!(
                        out,
                        "        let record = hydrate_record(row.clone(), &localized, base_url.as_deref());"
                    )
                    .unwrap();
                }
            }
        }
    }
    writeln!(out, "        Ok((record, row))").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(out, "}}").unwrap();

    let insert_save_methods_section = out;
    let mut insert_context = TemplateContext::new();
    insert_context
        .insert(
            "insert_struct_section",
            insert_struct_section.trim_start().to_string(),
        )
        .unwrap();
    insert_context
        .insert(
            "insert_builder_methods_section",
            insert_builder_methods_section.trim_start().to_string(),
        )
        .unwrap();
    insert_context
        .insert(
            "insert_save_methods_section",
            insert_save_methods_section.trim_start().to_string(),
        )
        .unwrap();
    let insert_section = render_template("models/insert.rs.tpl", &insert_context).unwrap();
    let mut out = String::new();

    // Update builder — wraps PatchState for SQL, adds model-specific extension fields
    writeln!(out, "pub struct {update_ident}<'db> {{").unwrap();
    writeln!(
        out,
        "    pub(crate) state: core_db::common::model_api::PatchState<'db>,"
    )
    .unwrap();
    if !localized_fields.is_empty() {
        writeln!(
            out,
            "    translations: HashMap<&'static str, HashMap<String, String>>,"
        )
        .unwrap();
    }
    if has_meta {
        writeln!(out, "    meta: HashMap<String, JsonValue>,").unwrap();
    }
    if has_attachments {
        writeln!(
            out,
            "    attachments_single: HashMap<&'static str, AttachmentInput>,"
        )
        .unwrap();
        writeln!(
            out,
            "    attachments_multi: HashMap<&'static str, Vec<AttachmentInput>>,"
        )
        .unwrap();
        writeln!(out, "    attachments_clear_single: Vec<&'static str>,").unwrap();
        writeln!(
            out,
            "    attachments_delete_multi: HashMap<&'static str, Vec<Uuid>>,"
        )
        .unwrap();
    }
    writeln!(out, "}}\n").unwrap();

    writeln!(out, "impl<'db> {update_ident}<'db> {{").unwrap();
    writeln!(
        out,
        "    pub fn new(db: DbConn<'db>, base_url: Option<String>) -> Self {{"
    )
    .unwrap();
    writeln!(out, "        Self {{").unwrap();
    writeln!(out, "            state: core_db::common::model_api::PatchState::new(db, base_url, \"{table}\"),").unwrap();
    if !localized_fields.is_empty() {
        writeln!(out, "            translations: HashMap::new(),").unwrap();
    }
    if has_meta {
        writeln!(out, "            meta: HashMap::new(),").unwrap();
    }
    if has_attachments {
        writeln!(out, "            attachments_single: HashMap::new(),").unwrap();
        writeln!(out, "            attachments_multi: HashMap::new(),").unwrap();
        writeln!(out, "            attachments_clear_single: Vec::new(),").unwrap();
        writeln!(out, "            attachments_delete_multi: HashMap::new(),").unwrap();
    }
    writeln!(out, "        }}").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(
        out,
        "    pub fn from_state(state: core_db::common::model_api::PatchState<'db>) -> Self {{"
    )
    .unwrap();
    writeln!(out, "        Self {{").unwrap();
    writeln!(out, "            state,").unwrap();
    if !localized_fields.is_empty() {
        writeln!(out, "            translations: HashMap::new(),").unwrap();
    }
    if has_meta {
        writeln!(out, "            meta: HashMap::new(),").unwrap();
    }
    if has_attachments {
        writeln!(out, "            attachments_single: HashMap::new(),").unwrap();
        writeln!(out, "            attachments_multi: HashMap::new(),").unwrap();
        writeln!(out, "            attachments_clear_single: Vec::new(),").unwrap();
        writeln!(out, "            attachments_delete_multi: HashMap::new(),").unwrap();
    }
    writeln!(out, "        }}").unwrap();
    writeln!(out, "    }}").unwrap();
    let update_struct_section = out;
    let mut out = String::new();

    out.push_str(&render_update_field_setters(&db_fields, &col_ident));
    if !localized_fields.is_empty() {
        out.push_str(&render_localized_setters(&localized_fields, cfgs));
    }
    if has_meta {
        out.push_str(&render_meta_setters(&meta_fields));
    }
    if has_attachments {
        out.push_str(&render_update_attachment_setters(
            &single_attachments,
            &multi_attachments,
        ));
    }

    for f in &db_fields {
        let fn_name = format!("where_{}", to_snake(&f.name));
        writeln!(
            out,
            "    pub fn {fn_name}(mut self, op: Op, val: {typ}) -> Self {{",
            typ = f.ty
        )
        .unwrap();
        writeln!(out, "        let idx = self.state.where_binds.len() + 1;").unwrap();
        writeln!(
            out,
            "        self.state.where_sql.push(format!(\"{{}} {{}} ${{}}\", {col_ident}::{}.as_sql(), op.as_sql(), idx));",
            to_title_case(&f.name)
        )
        .unwrap();
        writeln!(out, "        self.state.where_binds.push(val.into());").unwrap();
        writeln!(out, "        self").unwrap();
        writeln!(out, "    }}").unwrap();
    }

    writeln!(
        out,
        "    pub fn where_col<T: Into<BindValue>>(mut self, col: {col_ident}, op: Op, val: T) -> Self {{"
    )
    .unwrap();
    writeln!(out, "        let idx = self.state.where_binds.len() + 1;").unwrap();
    writeln!(
        out,
        "        self.state.where_sql.push(format!(\"{{}} {{}} ${{}}\", col.as_sql(), op.as_sql(), idx));"
    )
    .unwrap();
    writeln!(out, "        self.state.where_binds.push(val.into());").unwrap();
    writeln!(out, "        self").unwrap();
    writeln!(out, "    }}").unwrap();

    writeln!(
        out,
        "    fn where_raw<T: Into<BindValue>>(mut self, clause: impl Into<String>, binds: impl IntoIterator<Item = T>) -> Self {{"
    )
    .unwrap();
    writeln!(out, "        let mut clause = clause.into();").unwrap();
    writeln!(
        out,
        "        let incoming: Vec<BindValue> = binds.into_iter().map(Into::into).collect();"
    )
    .unwrap();
    writeln!(
        out,
        "        let mut idx = self.state.where_binds.len() + 1;"
    )
    .unwrap();
    writeln!(out, "        while let Some(pos) = clause.find('?') {{").unwrap();
    writeln!(out, "            let ph = format!(\"${{}}\", idx);").unwrap();
    writeln!(out, "            clause.replace_range(pos..pos + 1, &ph);").unwrap();
    writeln!(out, "            idx += 1;").unwrap();
    writeln!(out, "        }}").unwrap();
    writeln!(out, "        self.state.where_sql.push(clause);").unwrap();
    writeln!(out, "        self.state.where_binds.extend(incoming);").unwrap();
    writeln!(out, "        self").unwrap();
    writeln!(out, "    }}").unwrap();

    writeln!(
        out,
        "    fn to_update_changes(&self) -> Result<{update_changes_struct_ident}> {{"
    )
    .unwrap();
    writeln!(
        out,
        "        let mut changes = {update_changes_struct_ident}::default();"
    )
    .unwrap();
    writeln!(out, "        for ((col, bind), mode) in self.state.set_cols.iter().zip(self.state.set_binds.iter()).zip(self.state.set_modes.iter()) {{").unwrap();
    writeln!(out, "            match *col {{").unwrap();
    for f in &db_fields {
        let decode_expr = render_bind_decode_expr(&f.ty, "bind", &enum_specs);
        writeln!(out, "                \"{}\" => {{", f.name).unwrap();
        writeln!(out, "                    let value = {decode_expr};").unwrap();
        writeln!(
            out,
            "                    changes.{} = Some(match mode {{",
            f.name
        )
        .unwrap();
        writeln!(
            out,
            "                        SetMode::Assign => FieldChange::Assign(value),"
        )
        .unwrap();
        writeln!(
            out,
            "                        SetMode::Increment => FieldChange::Increment(value),"
        )
        .unwrap();
        writeln!(
            out,
            "                        SetMode::Decrement => FieldChange::Decrement(value),"
        )
        .unwrap();
        writeln!(out, "                    }});").unwrap();
        writeln!(out, "                }}").unwrap();
    }
    writeln!(out, "                _ => {{}}").unwrap();
    writeln!(out, "            }}").unwrap();
    writeln!(out, "        }}").unwrap();
    writeln!(out, "        Ok(changes)").unwrap();
    writeln!(out, "    }}").unwrap();

    if emit_hooks {
        writeln!(out).unwrap();
        writeln!(
            out,
            "    fn apply_update_overrides(mut state: PatchState<'_>, overrides: serde_json::Value) -> Result<PatchState<'_>> {{"
        )
        .unwrap();
        writeln!(out, "        let map = overrides.as_object()").unwrap();
        writeln!(
            out,
            "            .ok_or_else(|| anyhow::anyhow!(\"observer overrides must be a JSON object\"))?;"
        )
        .unwrap();
        writeln!(out, "        for (key, val) in map {{").unwrap();
        writeln!(out, "            match key.as_str() {{").unwrap();
        for f in &db_fields {
            let deser_ty = json_deser_type_for_field(&f.ty, &enum_specs);
            writeln!(out, "                \"{}\" => {{", f.name).unwrap();
            writeln!(
                out,
                "                    let v: {deser_ty} = serde_json::from_value(val.clone())?;"
            )
            .unwrap();
            writeln!(
                out,
                "                    state = state.assign_col(\"{}\", v.into());",
                f.name
            )
            .unwrap();
            writeln!(out, "                }}").unwrap();
        }
        writeln!(
            out,
            "                other => anyhow::bail!(\"unknown column '{{}}' in observer update overrides\", other),"
        )
        .unwrap();
        writeln!(out, "            }}").unwrap();
        writeln!(out, "        }}").unwrap();
        writeln!(out, "        Ok(state)").unwrap();
        writeln!(out, "    }}").unwrap();
    }

    let update_builder_methods_section = out;
    let mut out = String::new();

    writeln!(out, "    pub async fn save(self) -> Result<u64> {{").unwrap();
    writeln!(
        out,
        "        if self.state.set_cols.is_empty() {{ anyhow::bail!(\"update: no columns set\"); }}"
    )
    .unwrap();
    writeln!(
        out,
        "        if self.state.where_sql.is_empty() {{ anyhow::bail!(\"update: no conditions set\"); }}"
    )
    .unwrap();
    if emit_hooks {
        writeln!(
            out,
            "        let observer_changes = if try_get_observer().is_some() {{"
        )
        .unwrap();
        writeln!(out, "            Some(self.to_update_changes()?)").unwrap();
        writeln!(out, "        }} else {{").unwrap();
        writeln!(out, "            None").unwrap();
        writeln!(out, "        }};").unwrap();
    }
    writeln!(out, "        let db_conn = self.state.db.clone();").unwrap();
    writeln!(out, "        match db_conn {{").unwrap();
    writeln!(out, "            DbConn::Pool(pool) => {{").unwrap();
    writeln!(out, "                let tx = pool.begin().await?;").unwrap();
    writeln!(
        out,
        "                let tx_lock = std::sync::Arc::new(tokio::sync::Mutex::new(tx));"
    )
    .unwrap();
    writeln!(out, "                let affected = {{").unwrap();
    writeln!(
        out,
        "                    let db = DbConn::tx(tx_lock.clone());"
    )
    .unwrap();
    if emit_hooks {
        writeln!(
            out,
            "                    self.save_with_db(db, observer_changes.clone()).await?"
        )
        .unwrap();
    } else {
        writeln!(
            out,
            "                    self.save_with_db(db, None).await?"
        )
        .unwrap();
    }
    writeln!(out, "                }};").unwrap();
    writeln!(
        out,
        "                let tx = std::sync::Arc::try_unwrap(tx_lock)"
    )
    .unwrap();
    writeln!(out, "                    .map_err(|_| anyhow::anyhow!(\"transaction scope still has active handles\"))?")
        .unwrap();
    writeln!(out, "                    .into_inner();").unwrap();
    writeln!(out, "                tx.commit().await?;").unwrap();
    writeln!(out, "                Ok(affected)").unwrap();
    writeln!(out, "            }}").unwrap();
    writeln!(
        out,
        "            DbConn::Tx(_) => {tx_body},",
        tx_body = if emit_hooks {
            "self.save_with_db(db_conn, observer_changes).await"
        } else {
            "self.save_with_db(db_conn, None).await"
        }
    )
    .unwrap();
    writeln!(out, "        }}").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(out).unwrap();
    writeln!(
        out,
        "    async fn save_with_db<'tx>(self, db: DbConn<'tx>, observer_changes: Option<{update_changes_struct_ident}>) -> Result<u64> {{"
    )
    .unwrap();
    writeln!(out, "        let mut state = self.state;").unwrap();
    if has_updated_at {
        writeln!(out, "        if HAS_UPDATED_AT && !state.set_cols.contains(&{col_ident}::UpdatedAt.as_sql()) {{").unwrap();
        writeln!(
            out,
            "            let now = time::OffsetDateTime::now_utc();"
        )
        .unwrap();
        writeln!(
            out,
            "            state = state.assign_col({col_ident}::UpdatedAt.as_sql(), now.into());"
        )
        .unwrap();
        writeln!(out, "        }}").unwrap();
    }
    writeln!(out, "        // find target ids for localized updates").unwrap();
    writeln!(out, "        let select_sql = format!(\"SELECT {pk} FROM {table} WHERE {{}}\", state.where_sql.join(\" AND \"));").unwrap();
    writeln!(
        out,
        "        let mut select_q = sqlx::query_scalar::<_, {parent_pk_ty}>(&select_sql);"
    )
    .unwrap();
    writeln!(
        out,
        "        for b in &state.where_binds {{ select_q = bind_scalar(select_q, b.clone()); }}"
    )
    .unwrap();
    writeln!(
        out,
        "        let target_ids = db.fetch_all_scalar(select_q).await?;"
    )
    .unwrap();
    if emit_hooks {
        writeln!(
            out,
            "        let __observer_active = try_get_observer().is_some();"
        )
        .unwrap();
        writeln!(out, "        let __old_rows: Vec<{row_ident}> = if __observer_active && !target_ids.is_empty() {{").unwrap();
        writeln!(out, "            let phs: Vec<String> = (1..=target_ids.len()).map(|i| format!(\"${{}}\", i)).collect();").unwrap();
        writeln!(out, "            let fetch_sql = format!(\"SELECT * FROM {table} WHERE {pk} IN ({{}})\", phs.join(\", \"));").unwrap();
        writeln!(
            out,
            "            let mut fq = sqlx::query_as::<_, {row_ident}>(&fetch_sql);"
        )
        .unwrap();
        writeln!(
            out,
            "            for id in &target_ids {{ fq = fq.bind(id); }}"
        )
        .unwrap();
        writeln!(
            out,
            "            let rows: Vec<{row_ident}> = db.fetch_all(fq).await.unwrap_or_default();"
        )
        .unwrap();
        writeln!(out, "            rows").unwrap();
        writeln!(out, "        }} else {{").unwrap();
        writeln!(out, "            Vec::new()").unwrap();
        writeln!(out, "        }};").unwrap();
        writeln!(out, "        if !__old_rows.is_empty() {{").unwrap();
        writeln!(
            out,
            "            if let Some(observer) = try_get_observer() {{"
        )
        .unwrap();
        writeln!(
            out,
            "                if let Some(changes) = observer_changes.as_ref() {{"
        )
        .unwrap();
        writeln!(
            out,
            "                    let changes_data = serde_json::to_value(changes)?;"
        )
        .unwrap();
        writeln!(
            out,
            "                    let old_data = serde_json::to_value(&__old_rows)?;"
        )
        .unwrap();
        writeln!(
            out,
            "                    let event = ModelEvent {{ model: \"{model_snake}\", table: \"{table}\", record_key: None }};"
        )
        .unwrap();
        writeln!(
            out,
            "                    let action = observer.on_updating(&event, &old_data, &changes_data).await?;"
        )
        .unwrap();
        writeln!(out, "                    match action {{").unwrap();
        writeln!(
            out,
            "                        ObserverAction::Prevent(err) => return Err(err),"
        )
        .unwrap();
        writeln!(
            out,
            "                        ObserverAction::Modify(overrides) => {{"
        )
        .unwrap();
        writeln!(
            out,
            "                            state = Self::apply_update_overrides(state, overrides)?;"
        )
        .unwrap();
        writeln!(out, "                        }}").unwrap();
        writeln!(
            out,
            "                        ObserverAction::Continue => {{}}"
        )
        .unwrap();
        writeln!(out, "                    }}").unwrap();
        writeln!(out, "                }}").unwrap();
        writeln!(out, "            }}").unwrap();
        writeln!(out, "        }}").unwrap();
    }
    writeln!(
        out,
        "        let (sql, all_binds) = state.build_update_sql();"
    )
    .unwrap();
    writeln!(out, "        let set_binds = &state.set_binds;").unwrap();
    writeln!(out, "        let mut q = sqlx::query(&sql);").unwrap();
    // touch parent timestamps
    if !touch_targets.is_empty() {
        writeln!(out, "        if !target_ids.is_empty() {{").unwrap();
        writeln!(out, "            fn to_params(len: usize) -> String {{ (1..=len).map(|i| format!(\"${{}}\", i)).collect::<Vec<_>>().join(\", \") }}").unwrap();
        for (fk, target_snake, target_title, target_pk, target_pk_ty) in &touch_targets {
            writeln!(
                out,
                "            let placeholders = to_params(target_ids.len());"
            )
            .unwrap();
            writeln!(out, "            let sql = format!(\"SELECT DISTINCT {} FROM {} WHERE {{}} IN ({{}})\", \"{pk}\", placeholders);", fk, table).unwrap();
            writeln!(
                out,
                "            let mut q = sqlx::query_scalar::<_, {target_pk_ty}>(&sql);"
            )
            .unwrap();
            writeln!(
                out,
                "            for id in &target_ids {{ q = bind_scalar(q, id.clone()); }}"
            )
            .unwrap();
            writeln!(
                out,
                "            let parent_ids: Vec<{target_pk_ty}> = db.fetch_all_scalar(q).await?;"
            )
            .unwrap();
            writeln!(out, "            for pid in parent_ids {{").unwrap();
            writeln!(
                out,
                "                crate::generated::models::{}::{}::new(db.clone()).update()",
                target_snake, target_title
            )
            .unwrap();
            writeln!(
                out,
                "                    .where_{target_pk}(Op::Eq, pid)",
                target_pk = target_pk
            )
            .unwrap();
            writeln!(
                out,
                "                    .set_updated_at(time::OffsetDateTime::now_utc())"
            )
            .unwrap();
            writeln!(out, "                    .save().await?;").unwrap();
            writeln!(out, "            }}").unwrap();
        }
        writeln!(out, "        }}").unwrap();
    }
    writeln!(out, "        let __profiler_binds = if is_sql_profiler_enabled() {{ all_binds.iter().map(|b| format!(\"{{}}\", b)).collect::<Vec<_>>().join(\", \") }} else {{ String::new() }};").unwrap();
    out.push_str(&render_profiler_start(skip_profiler));
    writeln!(
        out,
        "        for b in &all_binds {{ q = bind_query(q, b.clone()); }}"
    )
    .unwrap();
    writeln!(out, "        let res = db.execute(q).await?;").unwrap();
    out.push_str(&render_profiler_log(
        &table,
        "UPDATE",
        "&sql",
        "&__profiler_binds",
        skip_profiler,
    ));
    if !localized_fields.is_empty() {
        writeln!(out, "        if res.rows_affected() > 0 && !self.translations.is_empty() && !target_ids.is_empty() {{").unwrap();
        writeln!(
            out,
            "            let supported = localized::SUPPORTED_LOCALES;"
        )
        .unwrap();
        for f in &localized_fields {
            writeln!(
                out,
                "            if let Some(map) = self.translations.get(\"{f}\") {{"
            )
            .unwrap();
            writeln!(out, "                let mut filtered = HashMap::new();").unwrap();
            writeln!(out, "                for (loc, val) in map {{").unwrap();
            writeln!(out, "                    if supported.contains(&loc.as_str()) {{ filtered.insert(loc.clone(), val.clone()); }}").unwrap();
            writeln!(out, "                }}").unwrap();
            writeln!(out, "                if !filtered.is_empty() {{").unwrap();
            writeln!(out, "                    for id in &target_ids {{").unwrap();
            writeln!(out, "                        localized::upsert_localized_many(db.clone(), localized::{}_OWNER_TYPE, id.clone(), \"{f}\", &filtered).await?;", model_snake.to_uppercase()).unwrap();
            writeln!(out, "                    }}").unwrap();
            writeln!(out, "                }}").unwrap();
            writeln!(out, "            }}").unwrap();
        }
        writeln!(out, "        }}").unwrap();
    }
    if has_meta {
        writeln!(out, "        if res.rows_affected() > 0 && !self.meta.is_empty() && !target_ids.is_empty() {{").unwrap();
        writeln!(out, "            for id in &target_ids {{").unwrap();
        writeln!(out, "                localized::upsert_meta_many(db.clone(), localized::{model_snake_upper}_OWNER_TYPE, id.clone(), &self.meta).await?;", model_snake_upper = model_snake.to_uppercase()).unwrap();
        writeln!(out, "            }}").unwrap();
        writeln!(out, "        }}").unwrap();
    }
    if has_attachments {
        writeln!(
            out,
            "        if res.rows_affected() > 0 && !target_ids.is_empty() {{"
        )
        .unwrap();
        writeln!(out, "            for id in &target_ids {{").unwrap();
        writeln!(
            out,
            "                for field in &self.attachments_clear_single {{"
        )
        .unwrap();
        writeln!(
            out,
            "                    localized::clear_attachment_field(db.clone(), localized::{}_OWNER_TYPE, id.clone(), field).await?;",
            model_snake_upper
        )
        .unwrap();
        writeln!(out, "                }}").unwrap();
        writeln!(
            out,
            "                for (field, att) in &self.attachments_single {{"
        )
        .unwrap();
        writeln!(out, "                    localized::replace_single_attachment(db.clone(), localized::{}_OWNER_TYPE, id.clone(), field, att).await?;", model_snake_upper).unwrap();
        writeln!(out, "                }}").unwrap();
        writeln!(
            out,
            "                for (field, list) in &self.attachments_multi {{"
        )
        .unwrap();
        writeln!(
            out,
            "                    localized::add_attachments(db.clone(), localized::{}_OWNER_TYPE, id.clone(), field, list).await?;",
            model_snake_upper
        )
        .unwrap();
        writeln!(out, "                }}").unwrap();
        writeln!(
            out,
            "                for (field, ids) in &self.attachments_delete_multi {{"
        )
        .unwrap();
        writeln!(out, "                    localized::delete_attachment_ids(db.clone(), localized::{}_OWNER_TYPE, id.clone(), field, ids).await?;", model_snake_upper).unwrap();
        writeln!(out, "                }}").unwrap();
        writeln!(out, "            }}").unwrap();
        writeln!(out, "        }}").unwrap();
    }
    if emit_hooks {
        let pk_snake = to_snake(&pk);
        writeln!(
            out,
            "        if !__old_rows.is_empty() && res.rows_affected() > 0 {{"
        )
        .unwrap();
        writeln!(
            out,
            "            if let Some(observer) = try_get_observer() {{"
        )
        .unwrap();
        writeln!(out, "                for old_row in &__old_rows {{").unwrap();
        writeln!(out, "                    let fetch_sql = format!(\"SELECT * FROM {table} WHERE {pk} = $1\");").unwrap();
        writeln!(out, "                    match db.fetch_optional(sqlx::query_as::<_, {row_ident}>(&fetch_sql).bind(old_row.{pk_snake}.clone())).await {{").unwrap();
        writeln!(out, "                        Ok(Some(new_row)) => {{").unwrap();
        writeln!(out, "                            match (serde_json::to_value(old_row), serde_json::to_value(&new_row)) {{").unwrap();
        writeln!(
            out,
            "                                (Ok(old_data), Ok(new_data)) => {{"
        )
        .unwrap();
        writeln!(out, "                                    let event = ModelEvent {{ model: \"{model_snake}\", table: \"{table}\", record_key: Some(format!(\"{{}}\", old_row.{pk_snake})) }};").unwrap();
        writeln!(out, "                                    if let Err(err) = observer.on_updated(&event, &old_data, &new_data).await {{").unwrap();
        writeln!(out, "                                        log_observer_error(\"updated\", \"{model_snake}\", &err);").unwrap();
        writeln!(out, "                                    }}").unwrap();
        writeln!(out, "                                }}").unwrap();
        writeln!(out, "                                (Err(err), _) | (_, Err(err)) => log_observer_error(\"updated\", \"{model_snake}\", &err),").unwrap();
        writeln!(out, "                            }}").unwrap();
        writeln!(out, "                        }}").unwrap();
        writeln!(out, "                        Ok(None) => {{}}").unwrap();
        writeln!(out, "                        Err(err) => log_observer_error(\"updated\", \"{model_snake}\", &err),").unwrap();
        writeln!(out, "                    }}").unwrap();
        writeln!(out, "                }}").unwrap();
        writeln!(out, "            }}").unwrap();
        writeln!(out, "        }}").unwrap();
    }
    writeln!(out, "        Ok(res.rows_affected())").unwrap();
    writeln!(out, "    }}").unwrap();

    // returning_row/returning_view intentionally omitted.
    // save() is the single mutation path with side-effect parity and atomic behavior.

    writeln!(out, "}}").unwrap();

    let update_save_methods_section = out;
    let unsafe_update_section = String::new();
    let mut update_context = TemplateContext::new();
    update_context
        .insert(
            "update_struct_section",
            update_struct_section.trim_start().to_string(),
        )
        .unwrap();
    update_context
        .insert(
            "update_builder_methods_section",
            update_builder_methods_section.trim_start().to_string(),
        )
        .unwrap();
    update_context
        .insert(
            "update_save_methods_section",
            update_save_methods_section.trim_start().to_string(),
        )
        .unwrap();
    update_context
        .insert("unsafe_update_section", unsafe_update_section)
        .unwrap();
    let update_section = render_template("models/update.rs.tpl", &update_context).unwrap();
    let mut out = String::new();

    if options.include_datatable {
        let table_adapter_ident = format!("{}TableAdapter", model_title);
        let sortable_cols_lit = db_fields
            .iter()
            .filter(|f| !f.ty.contains("serde_json"))
            .map(|f| format!("\"{}\"", f.name))
            .collect::<Vec<_>>()
            .join(", ");
        let timestamp_cols_lit = db_fields
            .iter()
            .filter(|f| f.ty.contains("OffsetDateTime"))
            .map(|f| format!("\"{}\"", f.name))
            .collect::<Vec<_>>()
            .join(", ");

        let column_filter_ops_lit = |f: &FieldSpec| -> &'static str {
            if f.ty.contains("String") {
                "&[\"eq\", \"like\", \"gte\", \"lte\"]"
            } else if f.ty.contains("OffsetDateTime") {
                "&[\"eq\", \"gte\", \"lte\", \"date_from\", \"date_to\"]"
            } else {
                "&[\"eq\", \"gte\", \"lte\"]"
            }
        };

        writeln!(out, "pub struct {table_adapter_ident};").unwrap();
        writeln!(out, "impl {table_adapter_ident} {{").unwrap();
        writeln!(
            out,
            "    fn parse_col(name: &str) -> Option<{col_ident}> {{"
        )
        .unwrap();
        writeln!(out, "        match name {{").unwrap();
        for f in &db_fields {
            writeln!(
                out,
                "            \"{name}\" => Some({col_ident}::{variant}),",
                name = f.name,
                variant = to_title_case(&f.name)
            )
            .unwrap();
        }
        writeln!(out, "            _ => None,").unwrap();
        writeln!(out, "        }}").unwrap();
        writeln!(out, "    }}").unwrap();

        // -- per-target-model parse_locale_field helpers (deduplicated) --
        {
            let mut unique_targets: BTreeMap<String, Vec<&RelationPathSpec>> = BTreeMap::new();
            for rel_path in &relation_paths {
                unique_targets
                    .entry(rel_path.target_model.clone())
                    .or_default()
                    .push(rel_path);
            }
            for (target_model, _) in &unique_targets {
                let target_cfg = schema
                    .models
                    .get(target_model)
                    .unwrap_or_else(|| panic!("Target model '{}' not found", target_model));
                let target_localized_fields: Vec<String> = target_cfg
                    .localized
                    .clone()
                    .unwrap_or_default()
                    .into_iter()
                    .map(|s| to_snake(&s))
                    .collect();
                if target_localized_fields.is_empty() {
                    continue;
                }
                let fn_name = format!("parse_locale_field_for_{}_cols", to_snake(target_model));
                writeln!(
                    out,
                    "    fn {fn_name}(column: &str) -> Option<&'static str> {{"
                )
                .unwrap();
                writeln!(out, "        match column {{").unwrap();
                for tf in &target_localized_fields {
                    writeln!(out, "            \"{tf}\" => Some(\"{tf}\"),").unwrap();
                }
                writeln!(out, "            _ => None,").unwrap();
                writeln!(out, "        }}").unwrap();
                writeln!(out, "    }}").unwrap();
            }

            writeln!(
                out,
                "    fn parse_locale_field_for_relation(relation: &str, column: &str) -> Option<&'static str> {{"
            )
            .unwrap();
            writeln!(out, "        match relation {{").unwrap();
            for (target_model, rel_paths) in &unique_targets {
                let target_cfg = schema
                    .models
                    .get(target_model)
                    .unwrap_or_else(|| panic!("Target model '{}' not found", target_model));
                let target_localized_fields: Vec<String> = target_cfg
                    .localized
                    .clone()
                    .unwrap_or_default()
                    .into_iter()
                    .map(|s| to_snake(&s))
                    .collect();
                if target_localized_fields.is_empty() {
                    continue;
                }
                let helper_fn = format!("parse_locale_field_for_{}_cols", to_snake(target_model));
                for rel_path in rel_paths {
                    let rel_key = rel_path.path.join("__");
                    writeln!(
                        out,
                        "            \"{rel}\" => Self::{helper_fn}(column),",
                        rel = rel_key
                    )
                    .unwrap();
                }
            }
            writeln!(out, "            _ => None,").unwrap();
            writeln!(out, "        }}").unwrap();
            writeln!(out, "    }}").unwrap();
        }

        writeln!(
            out,
            "    fn parse_locale_field(name: &str) -> Option<&'static str> {{"
        )
        .unwrap();
        writeln!(out, "        match name {{").unwrap();
        for f in &localized_fields {
            writeln!(out, "            \"{f}\" => Some(\"{f}\"),").unwrap();
        }
        writeln!(out, "            _ => None,").unwrap();
        writeln!(out, "        }}").unwrap();
        writeln!(out, "    }}").unwrap();

        writeln!(
            out,
            "    fn parse_like_col(name: &str) -> Option<{col_ident}> {{"
        )
        .unwrap();
        writeln!(out, "        match name {{").unwrap();
        for f in &db_fields {
            if f.ty.contains("String") {
                writeln!(
                    out,
                    "            \"{name}\" => Some({col_ident}::{variant}),",
                    name = f.name,
                    variant = to_title_case(&f.name)
                )
                .unwrap();
            }
        }
        writeln!(out, "            _ => None,").unwrap();
        writeln!(out, "        }}").unwrap();
        writeln!(out, "    }}").unwrap();

        let parse_bind_expr = |ty: &str, raw: &str| -> String {
            match ty {
                "String" => format!("Some({raw}.trim().to_string().into())"),
                "bool" => format!("{raw}.trim().parse::<bool>().ok().map(Into::into)"),
                "i8" => format!("{raw}.trim().parse::<i8>().ok().map(|v| (v as i64).into())"),
                "i16" => format!("{raw}.trim().parse::<i16>().ok().map(Into::into)"),
                "i32" => format!("{raw}.trim().parse::<i32>().ok().map(Into::into)"),
                "i64" => format!("{raw}.trim().parse::<i64>().ok().map(Into::into)"),
                "u8" => format!("{raw}.trim().parse::<u8>().ok().map(|v| (v as i64).into())"),
                "u16" => format!("{raw}.trim().parse::<u16>().ok().map(|v| (v as i64).into())"),
                "u32" => format!("{raw}.trim().parse::<u32>().ok().map(|v| (v as i64).into())"),
                "u64" => format!("{raw}.trim().parse::<u64>().ok().map(|v| (v as i64).into())"),
                "f32" => format!("{raw}.trim().parse::<f32>().ok().map(|v| (v as f64).into())"),
                "f64" => format!("{raw}.trim().parse::<f64>().ok().map(Into::into)"),
                "rust_decimal::Decimal" => {
                    format!("{raw}.trim().parse::<rust_decimal::Decimal>().ok().map(Into::into)")
                }
                "uuid::Uuid" => format!("uuid::Uuid::parse_str({raw}.trim()).ok().map(Into::into)"),
                "time::OffsetDateTime" => {
                    format!("Self::parse_datetime({raw}.trim(), false).map(Into::into)")
                }
                "Option<time::OffsetDateTime>" => {
                    format!("Self::parse_datetime({raw}.trim(), false).map(Into::into)")
                }
                _ => format!("Some(Self::parse_bind({raw}.trim()))"),
            }
        };

        let cursor_value_expr = |ty: &str, field_name: &str| -> Option<String> {
            match ty {
            "String" => Some(format!("Some(row.{field_name}.clone())")),
            "bool" | "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64" | "f32"
            | "f64" | "rust_decimal::Decimal" | "uuid::Uuid" => Some(format!("Some(row.{field_name}.to_string())")),
            "Option<String>" | "Option<bool>" | "Option<i8>" | "Option<i16>" | "Option<i32>"
            | "Option<i64>" | "Option<u8>" | "Option<u16>" | "Option<u32>" | "Option<u64>"
            | "Option<f32>" | "Option<f64>" | "Option<rust_decimal::Decimal>" | "Option<uuid::Uuid>" => {
                Some(format!("row.{field_name}.as_ref().map(|v| v.to_string())"))
            }
            "time::OffsetDateTime" => Some(format!(
                "row.{field_name}.format(&time::format_description::well_known::Rfc3339).ok()"
            )),
            "Option<time::OffsetDateTime>" => Some(format!(
                "row.{field_name}.as_ref().and_then(|v| v.format(&time::format_description::well_known::Rfc3339).ok())"
            )),
            _ => None,
        }
        };

        writeln!(
            out,
            "    fn parse_bind_for_col(name: &str, raw: &str) -> Option<BindValue> {{"
        )
        .unwrap();
        writeln!(out, "        match name {{").unwrap();
        for f in &db_fields {
            writeln!(
                out,
                "            \"{name}\" => {expr},",
                name = f.name,
                expr = parse_bind_expr(&f.ty, "raw")
            )
            .unwrap();
        }
        writeln!(out, "            _ => None,").unwrap();
        writeln!(out, "        }}").unwrap();
        writeln!(out, "    }}").unwrap();

        // -- per-target-model parse_bind helpers (deduplicated) --
        {
            let mut unique_targets: BTreeMap<String, Vec<&RelationPathSpec>> = BTreeMap::new();
            for rel_path in &relation_paths {
                unique_targets
                    .entry(rel_path.target_model.clone())
                    .or_default()
                    .push(rel_path);
            }
            for (target_model, _) in &unique_targets {
                let fn_name = format!("parse_bind_for_{}_cols", to_snake(target_model));
                let target_cfg = schema
                    .models
                    .get(target_model)
                    .unwrap_or_else(|| panic!("Target model '{}' not found", target_model));
                let target_pk = target_cfg.pk.clone().unwrap_or_else(|| "id".to_string());
                let target_fields = parse_fields(target_cfg, &target_pk);
                writeln!(
                    out,
                    "    fn {fn_name}(column: &str, raw: &str) -> Option<BindValue> {{"
                )
                .unwrap();
                writeln!(out, "        match column {{").unwrap();
                for tf in &target_fields {
                    writeln!(
                        out,
                        "            \"{col}\" => {expr},",
                        col = tf.name,
                        expr = parse_bind_expr(&tf.ty, "raw")
                    )
                    .unwrap();
                }
                writeln!(out, "            _ => None,").unwrap();
                writeln!(out, "        }}").unwrap();
                writeln!(out, "    }}").unwrap();
            }

            writeln!(
                out,
                "    fn parse_bind_for_relation(relation: &str, column: &str, raw: &str) -> Option<BindValue> {{"
            )
            .unwrap();
            writeln!(out, "        match relation {{").unwrap();
            for (target_model, rel_paths) in &unique_targets {
                let helper_fn = format!("parse_bind_for_{}_cols", to_snake(target_model));
                for rel_path in rel_paths {
                    let rel_key = rel_path.path.join("__");
                    writeln!(
                        out,
                        "            \"{rel}\" => Self::{helper_fn}(column, raw),",
                        rel = rel_key
                    )
                    .unwrap();
                }
            }
            writeln!(out, "            _ => None,").unwrap();
            writeln!(out, "        }}").unwrap();
            writeln!(out, "    }}").unwrap();
        }

        writeln!(out, "    fn parse_bind(raw: &str) -> BindValue {{").unwrap();
        writeln!(out, "        let trimmed = raw.trim();").unwrap();
        writeln!(
        out,
        "        let lower = trimmed.to_ascii_lowercase(); if lower == \"true\" {{ return true.into(); }} if lower == \"false\" {{ return false.into(); }}"
    )
    .unwrap();
        writeln!(
            out,
            "        if let Ok(v) = trimmed.parse::<i64>() {{ return v.into(); }}"
        )
        .unwrap();
        writeln!(
            out,
            "        if let Ok(v) = trimmed.parse::<rust_decimal::Decimal>() {{ return v.into(); }}"
        )
        .unwrap();
        writeln!(
            out,
            "        if let Ok(v) = trimmed.parse::<f64>() {{ return v.into(); }}"
        )
        .unwrap();
        writeln!(
            out,
            "        if let Ok(v) = uuid::Uuid::parse_str(trimmed) {{ return v.into(); }}"
        )
        .unwrap();
        writeln!(
            out,
            "        if let Some(v) = Self::parse_datetime(trimmed, false) {{ return v.into(); }}"
        )
        .unwrap();
        writeln!(out, "        trimmed.to_string().into()").unwrap();
        writeln!(out, "    }}").unwrap();

        writeln!(
            out,
            "    fn parse_datetime(raw: &str, end_of_day: bool) -> Option<time::OffsetDateTime> {{"
        )
        .unwrap();
        writeln!(out, "        let trimmed = raw.trim();").unwrap();
        writeln!(
        out,
        "        if let Ok(dt) = time::OffsetDateTime::parse(trimmed, &time::format_description::well_known::Rfc3339) {{ return Some(dt); }}"
    )
    .unwrap();
        writeln!(out, "        if trimmed.len() == 10 {{").unwrap();
        writeln!(
        out,
        "            let date = time::Date::parse(trimmed, &time::macros::format_description!(\"[year]-[month]-[day]\")).ok()?;"
    )
    .unwrap();
        writeln!(
        out,
        "            let t = if end_of_day {{ time::Time::from_hms(23, 59, 59).ok()? }} else {{ time::Time::MIDNIGHT }};"
    )
    .unwrap();
        writeln!(
            out,
            "            return Some(date.with_time(t).assume_offset(time::UtcOffset::UTC));"
        )
        .unwrap();
        writeln!(out, "        }}").unwrap();
        writeln!(out, "        None").unwrap();
        writeln!(out, "    }}").unwrap();

        // -- per-target-model filter_has / filter_has_like helpers (deduplicated) --
        {
            let mut unique_targets: BTreeMap<String, Vec<&RelationPathSpec>> = BTreeMap::new();
            for rel_path in &relation_paths {
                unique_targets
                    .entry(rel_path.target_model.clone())
                    .or_default()
                    .push(rel_path);
            }
            for (target_model, _) in &unique_targets {
                let target_title = to_title_case(target_model);
                let target_col_ident = format!("{}DbCol", target_title);
                let target_cfg = schema
                    .models
                    .get(target_model)
                    .unwrap_or_else(|| panic!("Target model '{}' not found", target_model));
                let target_pk = target_cfg.pk.clone().unwrap_or_else(|| "id".to_string());
                let target_fields = parse_fields(target_cfg, &target_pk);

                // filter_has_for_{model}_cols
                let fn_name = format!("filter_has_for_{}_cols", to_snake(target_model));
                writeln!(
                    out,
                    "    fn {fn_name}<'db>(column: &str, rq: Query<'db, {target_title}Model>, bind: BindValue) -> Query<'db, {target_title}Model> {{"
                )
                .unwrap();
                writeln!(out, "        match column {{").unwrap();
                for tf in &target_fields {
                    writeln!(
                        out,
                        "            \"{col}\" => rq.where_col({target_col_ident}::{variant}, Op::Eq, bind),",
                        col = tf.name,
                        variant = to_title_case(&tf.name)
                    )
                    .unwrap();
                }
                writeln!(out, "            _ => rq,").unwrap();
                writeln!(out, "        }}").unwrap();
                writeln!(out, "    }}").unwrap();

                // filter_has_like_for_{model}_cols — only String fields
                let string_fields: Vec<_> = target_fields
                    .iter()
                    .filter(|tf| tf.ty.contains("String"))
                    .collect();
                if !string_fields.is_empty() {
                    let fn_name_like =
                        format!("filter_has_like_for_{}_cols", to_snake(target_model));
                    writeln!(
                        out,
                        "    fn {fn_name_like}<'db>(column: &str, rq: Query<'db, {target_title}Model>, pattern: String) -> Query<'db, {target_title}Model> {{"
                    )
                    .unwrap();
                    writeln!(out, "        match column {{").unwrap();
                    for tf in &string_fields {
                        writeln!(
                            out,
                            "            \"{col}\" => rq.where_col({target_col_ident}::{variant}, Op::Like, pattern),",
                            col = tf.name,
                            variant = to_title_case(&tf.name)
                        )
                        .unwrap();
                    }
                    writeln!(out, "            _ => rq,").unwrap();
                    writeln!(out, "        }}").unwrap();
                    writeln!(out, "    }}").unwrap();
                }

                // locale_has helpers
                let target_localized_fields: Vec<String> = target_cfg
                    .localized
                    .clone()
                    .unwrap_or_default()
                    .into_iter()
                    .map(|s| to_snake(&s))
                    .collect();
                if !target_localized_fields.is_empty() {
                    let target_table = target_cfg
                        .table
                        .as_ref()
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| to_snake(target_model));
                    let target_pk_str = target_cfg
                        .pk
                        .as_ref()
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| "id".to_string());
                    let target_owner_const =
                        format!("{}_OWNER_TYPE", to_snake(target_model).to_uppercase());

                    let fn_locale_has =
                        format!("filter_locale_has_for_{}_cols", to_snake(target_model));
                    writeln!(
                        out,
                        "    fn {fn_locale_has}<'db>(column: &str, rq: Query<'db, {target_title}Model>, field: &str, locale: &str, value: String) -> Query<'db, {target_title}Model> {{"
                    )
                    .unwrap();
                    writeln!(out, "        match column {{").unwrap();
                    for tf in &target_localized_fields {
                        writeln!(
                            out,
                            "            \"{tf}\" => rq.where_exists_raw(\"EXISTS (SELECT 1 FROM localized l WHERE l.owner_type = ? AND l.owner_id = {target_table}.{target_pk_str} AND l.field = ? AND l.locale = ? AND l.value = ?)\".to_string(), vec![localized::{target_owner_const}.to_string(), field.to_string(), locale.to_string(), value]),"
                        )
                        .unwrap();
                    }
                    writeln!(out, "            _ => rq,").unwrap();
                    writeln!(out, "        }}").unwrap();
                    writeln!(out, "    }}").unwrap();

                    let fn_locale_has_like =
                        format!("filter_locale_has_like_for_{}_cols", to_snake(target_model));
                    writeln!(
                        out,
                        "    fn {fn_locale_has_like}<'db>(column: &str, rq: Query<'db, {target_title}Model>, field: &str, locale: &str, pattern: String) -> Query<'db, {target_title}Model> {{"
                    )
                    .unwrap();
                    writeln!(out, "        match column {{").unwrap();
                    for tf in &target_localized_fields {
                        writeln!(
                            out,
                            "            \"{tf}\" => rq.where_exists_raw(\"EXISTS (SELECT 1 FROM localized l WHERE l.owner_type = ? AND l.owner_id = {target_table}.{target_pk_str} AND l.field = ? AND l.locale = ? AND l.value LIKE ?)\".to_string(), vec![localized::{target_owner_const}.to_string(), field.to_string(), locale.to_string(), pattern]),"
                        )
                        .unwrap();
                    }
                    writeln!(out, "            _ => rq,").unwrap();
                    writeln!(out, "        }}").unwrap();
                    writeln!(out, "    }}").unwrap();
                }
            }
        }

        writeln!(out, "}}").unwrap();

        writeln!(
            out,
            "impl GeneratedTableAdapter for {table_adapter_ident} {{"
        )
        .unwrap();
        writeln!(out, "    type Query<'db> = Query<'db, {model_title}Model>;").unwrap();
        writeln!(out, "    type Row = {model_title}Record;").unwrap();
        writeln!(
            out,
            "    fn model_key(&self) -> &'static str {{ \"{model_title}\" }}"
        )
        .unwrap();
        writeln!(
        out,
        "    fn sortable_columns(&self) -> &'static [&'static str] {{ &[{sortable_cols_lit}] }}"
    )
        .unwrap();
        writeln!(
        out,
        "    fn timestamp_columns(&self) -> &'static [&'static str] {{ &[{timestamp_cols_lit}] }}"
    )
        .unwrap();
        writeln!(
            out,
            "    fn column_descriptors(&self) -> &'static [DataTableColumnDescriptor] {{"
        )
        .unwrap();
        writeln!(out, "        &[").unwrap();
        for f in &db_fields {
            let ops = column_filter_ops_lit(f);
            let label = crate::schema::to_label(&f.name);
            let sortable = !f.ty.contains("serde_json");
            writeln!(
            out,
            "            DataTableColumnDescriptor {{ name: \"{name}\", label: \"{label}\", data_type: \"{ty}\", sortable: {sortable}, localized: false, filter_ops: {ops} }},",
            name = f.name,
            ty = f.ty,
        )
        .unwrap();
        }
        for f in &localized_fields {
            let label = crate::schema::to_label(f);
            writeln!(
            out,
            "            DataTableColumnDescriptor {{ name: \"{name}\", label: \"{label}\", data_type: \"String\", sortable: false, localized: true, filter_ops: &[\"locale_eq\", \"locale_like\"] }},",
            name = f,
        )
        .unwrap();
        }
        writeln!(out, "        ]").unwrap();
        writeln!(out, "    }}").unwrap();
        writeln!(
        out,
        "    fn relation_column_descriptors(&self) -> &'static [DataTableRelationColumnDescriptor] {{"
    )
    .unwrap();
        writeln!(out, "        &[").unwrap();
        for rel_path in &relation_paths {
            let rel_key = rel_path.path.join("__");
            let target_cfg = schema
                .models
                .get(&rel_path.target_model)
                .unwrap_or_else(|| {
                    panic!(
                        "Relation path '{}' target model not found",
                        rel_path.target_model
                    )
                });
            let target_pk = target_cfg.pk.clone().unwrap_or_else(|| "id".to_string());
            let target_fields = parse_fields(target_cfg, &target_pk);
            for tf in &target_fields {
                let ops = if tf.ty.contains("String") {
                    "&[\"has_eq\", \"has_like\"]"
                } else {
                    "&[\"has_eq\"]"
                };
                writeln!(
                out,
                "            DataTableRelationColumnDescriptor {{ relation: \"{relation}\", column: \"{column}\", data_type: \"{ty}\", filter_ops: {ops} }},",
                relation = rel_key,
                column = tf.name,
                ty = tf.ty
            )
            .unwrap();
            }
            let target_localized_fields: Vec<String> = target_cfg
                .localized
                .clone()
                .unwrap_or_default()
                .into_iter()
                .map(|s| to_snake(&s))
                .collect();
            for tf in &target_localized_fields {
                writeln!(
                out,
                "            DataTableRelationColumnDescriptor {{ relation: \"{relation}\", column: \"{column}\", data_type: \"String\", filter_ops: &[\"locale_has_eq\", \"locale_has_like\"] }},",
                relation = rel_key,
                column = tf
            )
            .unwrap();
            }
        }
        writeln!(out, "        ]").unwrap();
        writeln!(out, "    }}").unwrap();
        writeln!(
            out,
            "    fn filter_patterns(&self) -> &'static [&'static str] {{"
        )
        .unwrap();
        writeln!(out, "        &[").unwrap();
        writeln!(out, "            \"f-<col>\",").unwrap();
        writeln!(out, "            \"f-like-<col>\",").unwrap();
        writeln!(out, "            \"f-gte-<col>\",").unwrap();
        writeln!(out, "            \"f-lte-<col>\",").unwrap();
        writeln!(out, "            \"f-date-from-<col>\",").unwrap();
        writeln!(out, "            \"f-date-to-<col>\",").unwrap();
        writeln!(out, "            \"f-like-any-<col1|col2|...>\",").unwrap();
        writeln!(out, "            \"f-any-<col1|col2|...>\",").unwrap();
        writeln!(out, "            \"f-has-<relation>-<col>\",").unwrap();
        writeln!(out, "            \"f-has-like-<relation>-<col>\",").unwrap();
        if !localized_fields.is_empty() {
            writeln!(out, "            \"f-locale-<col>\",").unwrap();
            writeln!(out, "            \"f-locale-like-<col>\",").unwrap();
        }
        let has_relation_locale = relation_paths.iter().any(|rel_path| {
            let Some(target_cfg) = schema.models.get(&rel_path.target_model) else {
                return false;
            };
            !target_cfg.localized.clone().unwrap_or_default().is_empty()
        });
        if has_relation_locale {
            writeln!(out, "            \"f-locale-has-<relation>-<col>\",").unwrap();
            writeln!(out, "            \"f-locale-has-like-<relation>-<col>\",").unwrap();
        }
        writeln!(out, "        ]").unwrap();
        writeln!(out, "    }}").unwrap();

        writeln!(
        out,
        "    fn apply_auto_filter<'db>(&self, query: Query<'db, {model_title}Model>, filter: &ParsedFilter, value: &str) -> anyhow::Result<Option<Query<'db, {model_title}Model>>> where Self: 'db {{"
    )
    .unwrap();
        writeln!(out, "        let trimmed = value.trim();").unwrap();
        writeln!(
            out,
            "        if trimmed.is_empty() {{ return Ok(Some(query)); }}"
        )
        .unwrap();
        writeln!(out, "        match filter {{").unwrap();
        writeln!(out, "            ParsedFilter::Eq {{ column }} => {{").unwrap();
        writeln!(
        out,
        "                let Some(col) = Self::parse_col(column.as_str()) else {{ return Ok(None); }};"
    )
    .unwrap();
        writeln!(
        out,
        "                let Some(bind) = Self::parse_bind_for_col(column.as_str(), trimmed) else {{ return Ok(None); }};"
    )
    .unwrap();
        writeln!(
            out,
            "                Ok(Some(query.where_col(col, Op::Eq, bind)))"
        )
        .unwrap();
        writeln!(out, "            }}").unwrap();
        writeln!(out, "            ParsedFilter::Like {{ column }} => {{").unwrap();
        writeln!(
        out,
        "                let Some(col) = Self::parse_like_col(column.as_str()) else {{ return Ok(None); }};"
    )
    .unwrap();
        writeln!(
        out,
        "                Ok(Some(query.where_col(col, Op::Like, format!(\"%{{}}%\", trimmed))))"
    )
        .unwrap();
        writeln!(out, "            }}").unwrap();
        writeln!(out, "            ParsedFilter::Gte {{ column }} => {{").unwrap();
        writeln!(
        out,
        "                let Some(col) = Self::parse_col(column.as_str()) else {{ return Ok(None); }};"
    )
    .unwrap();
        writeln!(
        out,
        "                let Some(bind) = Self::parse_bind_for_col(column.as_str(), trimmed) else {{ return Ok(None); }};"
    )
    .unwrap();
        writeln!(
            out,
            "                Ok(Some(query.where_col(col, Op::Ge, bind)))"
        )
        .unwrap();
        writeln!(out, "            }}").unwrap();
        writeln!(out, "            ParsedFilter::Lte {{ column }} => {{").unwrap();
        writeln!(
        out,
        "                let Some(col) = Self::parse_col(column.as_str()) else {{ return Ok(None); }};"
    )
    .unwrap();
        writeln!(
        out,
        "                let Some(bind) = Self::parse_bind_for_col(column.as_str(), trimmed) else {{ return Ok(None); }};"
    )
    .unwrap();
        writeln!(
            out,
            "                Ok(Some(query.where_col(col, Op::Le, bind)))"
        )
        .unwrap();
        writeln!(out, "            }}").unwrap();
        writeln!(out, "            ParsedFilter::DateFrom {{ column }} => {{").unwrap();
        writeln!(
        out,
        "                let Some(col) = Self::parse_col(column.as_str()) else {{ return Ok(None); }};"
    )
    .unwrap();
        writeln!(
        out,
        "                let Some(ts) = Self::parse_datetime(trimmed, false) else {{ return Ok(None); }};"
    )
    .unwrap();
        writeln!(
            out,
            "                Ok(Some(query.where_col(col, Op::Ge, ts)))"
        )
        .unwrap();
        writeln!(out, "            }}").unwrap();
        writeln!(out, "            ParsedFilter::DateTo {{ column }} => {{").unwrap();
        writeln!(
        out,
        "                let Some(col) = Self::parse_col(column.as_str()) else {{ return Ok(None); }};"
    )
    .unwrap();
        writeln!(
        out,
        "                let Some(ts) = Self::parse_datetime(trimmed, true) else {{ return Ok(None); }};"
    )
    .unwrap();
        writeln!(
            out,
            "                Ok(Some(query.where_col(col, Op::Le, ts)))"
        )
        .unwrap();
        writeln!(out, "            }}").unwrap();
        writeln!(out, "            ParsedFilter::LocaleEq {{ column }} => {{").unwrap();
        if localized_fields.is_empty() {
            writeln!(out, "                Ok(None)").unwrap();
        } else {
            writeln!(
            out,
            "                let Some(field) = Self::parse_locale_field(column.as_str()) else {{ return Ok(None); }};"
        )
        .unwrap();
            writeln!(
                out,
                "                let locale = core_i18n::current_locale().to_string();"
            )
            .unwrap();
            writeln!(
            out,
            "                let clause = \"EXISTS (SELECT 1 FROM localized l WHERE l.owner_type = ? AND l.owner_id = {table}.{pk} AND l.field = ? AND l.locale = ? AND l.value = ?)\".to_string();",
        )
        .unwrap();
            writeln!(
            out,
            "                Ok(Some(query.where_exists_raw(clause, vec![localized::{model_snake_upper}_OWNER_TYPE.to_string(), field.to_string(), locale, trimmed.to_string()])))",
        )
        .unwrap();
        }
        writeln!(out, "            }}").unwrap();
        writeln!(
            out,
            "            ParsedFilter::LocaleLike {{ column }} => {{"
        )
        .unwrap();
        if localized_fields.is_empty() {
            writeln!(out, "                Ok(None)").unwrap();
        } else {
            writeln!(
            out,
            "                let Some(field) = Self::parse_locale_field(column.as_str()) else {{ return Ok(None); }};"
        )
        .unwrap();
            writeln!(
                out,
                "                let locale = core_i18n::current_locale().to_string();"
            )
            .unwrap();
            writeln!(
                out,
                "                let pattern = format!(\"%{{}}%\", trimmed);"
            )
            .unwrap();
            writeln!(
            out,
            "                let clause = \"EXISTS (SELECT 1 FROM localized l WHERE l.owner_type = ? AND l.owner_id = {table}.{pk} AND l.field = ? AND l.locale = ? AND l.value LIKE ?)\".to_string();",
        )
        .unwrap();
            writeln!(
            out,
            "                Ok(Some(query.where_exists_raw(clause, vec![localized::{model_snake_upper}_OWNER_TYPE.to_string(), field.to_string(), locale, pattern])))",
        )
        .unwrap();
        }
        writeln!(out, "            }}").unwrap();
        writeln!(out, "            ParsedFilter::LikeAny {{ columns }} => {{").unwrap();
        writeln!(out, "                let mut applied = false;").unwrap();
        writeln!(
            out,
            "                let pattern = format!(\"%{{}}%\", trimmed);"
        )
        .unwrap();
        writeln!(
            out,
            "                let next = query.where_group(|group| {{"
        )
        .unwrap();
        writeln!(out, "                    let mut q = group;").unwrap();
        writeln!(out, "                    for column in columns {{").unwrap();
        writeln!(
            out,
            "                        if let Some(col) = Self::parse_like_col(column.as_str()) {{"
        )
        .unwrap();
        writeln!(
        out,
        "                            if applied {{ q = q.or_where_col(col, Op::Like, pattern.clone()); }} else {{ q = q.where_col(col, Op::Like, pattern.clone()); applied = true; }}"
    )
    .unwrap();
        writeln!(out, "                        }}").unwrap();
        writeln!(out, "                    }}").unwrap();
        writeln!(out, "                    q").unwrap();
        writeln!(out, "                }});").unwrap();
        writeln!(
            out,
            "                if applied {{ Ok(Some(next)) }} else {{ Ok(None) }}"
        )
        .unwrap();
        writeln!(out, "            }}").unwrap();
        writeln!(out, "            ParsedFilter::Any {{ columns }} => {{").unwrap();
        writeln!(out, "                let mut applied = false;").unwrap();
        writeln!(
            out,
            "                let next = query.where_group(|group| {{"
        )
        .unwrap();
        writeln!(out, "                    let mut q = group;").unwrap();
        writeln!(out, "                    for column in columns {{").unwrap();
        writeln!(
            out,
            "                        if let Some(col) = Self::parse_col(column.as_str()) {{"
        )
        .unwrap();
        writeln!(
        out,
        "                            if let Some(bind) = Self::parse_bind_for_col(column.as_str(), trimmed) {{ if applied {{ q = q.or_where_col(col, Op::Eq, bind.clone()); }} else {{ q = q.where_col(col, Op::Eq, bind.clone()); applied = true; }} }}"
    )
    .unwrap();
        writeln!(out, "                        }}").unwrap();
        writeln!(out, "                    }}").unwrap();
        writeln!(out, "                    q").unwrap();
        writeln!(out, "                }});").unwrap();
        writeln!(
            out,
            "                if applied {{ Ok(Some(next)) }} else {{ Ok(None) }}"
        )
        .unwrap();
        writeln!(out, "            }}").unwrap();
        writeln!(
            out,
            "            ParsedFilter::Has {{ relation, column }} => {{"
        )
        .unwrap();
        writeln!(out, "                match relation.as_str() {{").unwrap();
        for rel_path in &relation_paths {
            let rel_key = rel_path.path.join("__");
            let target_snake = to_snake(&rel_path.target_model);
            let helper_bind = format!("parse_bind_for_{}_cols", target_snake);
            let helper_has = format!("filter_has_for_{}_cols", target_snake);
            let leaf_expr = format!("Self::{helper_has}(column.as_str(), {{var}}, bind)");
            let has_expr =
                build_nested_where_has_expr(schema, name, &rel_path.path, &leaf_expr, "query");
            writeln!(
                out,
                "                    \"{rel_name}\" => {{ let Some(bind) = Self::{helper_bind}(column.as_str(), trimmed) else {{ return Ok(None); }}; Ok(Some({has_expr})) }},",
                rel_name = rel_key,
            )
            .unwrap();
        }
        writeln!(out, "                    _ => Ok(None),").unwrap();
        writeln!(out, "                }}").unwrap();
        writeln!(out, "            }}").unwrap();
        writeln!(
            out,
            "            ParsedFilter::HasLike {{ relation, column }} => {{"
        )
        .unwrap();
        writeln!(
            out,
            "                let pattern = format!(\"%{{}}%\", trimmed);"
        )
        .unwrap();
        writeln!(out, "                match relation.as_str() {{").unwrap();
        for rel_path in &relation_paths {
            // Only emit HasLike arm if target model has String fields
            let target_cfg_check = schema.models.get(&rel_path.target_model);
            let has_string_fields = target_cfg_check.map_or(false, |c| {
                let pk = c.pk.clone().unwrap_or_else(|| "id".to_string());
                parse_fields(c, &pk).iter().any(|f| f.ty.contains("String"))
            });
            if !has_string_fields {
                continue;
            }
            let rel_key = rel_path.path.join("__");
            let target_snake = to_snake(&rel_path.target_model);
            let helper_like = format!("filter_has_like_for_{}_cols", target_snake);
            let leaf_expr =
                format!("Self::{helper_like}(column.as_str(), {{var}}, pattern.clone())");
            let has_like_expr =
                build_nested_where_has_expr(schema, name, &rel_path.path, &leaf_expr, "query");
            writeln!(
                out,
                "                    \"{rel_name}\" => Ok(Some({has_like_expr})),",
                rel_name = rel_key,
            )
            .unwrap();
        }
        writeln!(out, "                    _ => Ok(None),").unwrap();
        writeln!(out, "                }}").unwrap();
        writeln!(out, "            }}").unwrap();
        writeln!(
            out,
            "            ParsedFilter::LocaleHas {{ relation, column }} => {{"
        )
        .unwrap();
        writeln!(
        out,
        "                let Some(field) = Self::parse_locale_field_for_relation(relation.as_str(), column.as_str()) else {{ return Ok(None); }};"
    )
    .unwrap();
        writeln!(
            out,
            "                let locale = core_i18n::current_locale().to_string();"
        )
        .unwrap();
        writeln!(out, "                match relation.as_str() {{").unwrap();
        for rel_path in &relation_paths {
            let target_cfg_check = schema.models.get(&rel_path.target_model);
            let has_locale = target_cfg_check
                .and_then(|c| c.localized.as_ref())
                .map_or(false, |l| !l.is_empty());
            if !has_locale {
                continue;
            }
            let rel_key = rel_path.path.join("__");
            let target_snake = to_snake(&rel_path.target_model);
            let helper_locale_has = format!("filter_locale_has_for_{}_cols", target_snake);
            let leaf_expr = format!(
                "Self::{helper_locale_has}(column.as_str(), {{var}}, &field, &locale, trimmed.to_string())"
            );
            let has_expr =
                build_nested_where_has_expr(schema, name, &rel_path.path, &leaf_expr, "query");
            writeln!(
                out,
                "                    \"{rel}\" => Ok(Some({has_expr})),",
                rel = rel_key,
            )
            .unwrap();
        }
        writeln!(out, "                    _ => Ok(None),").unwrap();
        writeln!(out, "                }}").unwrap();
        writeln!(out, "            }}").unwrap();
        writeln!(
            out,
            "            ParsedFilter::LocaleHasLike {{ relation, column }} => {{"
        )
        .unwrap();
        writeln!(
        out,
        "                let Some(field) = Self::parse_locale_field_for_relation(relation.as_str(), column.as_str()) else {{ return Ok(None); }};"
    )
    .unwrap();
        writeln!(
            out,
            "                let locale = core_i18n::current_locale().to_string();"
        )
        .unwrap();
        writeln!(
            out,
            "                let pattern = format!(\"%{{}}%\", trimmed);"
        )
        .unwrap();
        writeln!(out, "                match relation.as_str() {{").unwrap();
        for rel_path in &relation_paths {
            let target_cfg_check = schema.models.get(&rel_path.target_model);
            let has_locale = target_cfg_check
                .and_then(|c| c.localized.as_ref())
                .map_or(false, |l| !l.is_empty());
            if !has_locale {
                continue;
            }
            let rel_key = rel_path.path.join("__");
            let target_snake = to_snake(&rel_path.target_model);
            let helper_locale_has_like =
                format!("filter_locale_has_like_for_{}_cols", target_snake);
            let leaf_expr = format!(
                "Self::{helper_locale_has_like}(column.as_str(), {{var}}, &field, &locale, pattern.clone())"
            );
            let has_expr =
                build_nested_where_has_expr(schema, name, &rel_path.path, &leaf_expr, "query");
            writeln!(
                out,
                "                    \"{rel}\" => Ok(Some({has_expr})),",
                rel = rel_key,
            )
            .unwrap();
        }
        writeln!(out, "                    _ => Ok(None),").unwrap();
        writeln!(out, "                }}").unwrap();
        writeln!(out, "            }}").unwrap();
        writeln!(out, "        }}").unwrap();
        writeln!(out, "    }}").unwrap();

        writeln!(
        out,
        "    fn apply_sort<'db>(&self, query: Query<'db, {model_title}Model>, column: &str, dir: SortDirection) -> anyhow::Result<Query<'db, {model_title}Model>> where Self: 'db {{"
    )
    .unwrap();
        writeln!(
        out,
        "        let dir = match dir {{ SortDirection::Asc => OrderDir::Asc, SortDirection::Desc => OrderDir::Desc }};"
    )
    .unwrap();
        writeln!(out, "        let next = match column {{").unwrap();
        for f in &db_fields {
            writeln!(
                out,
                "            \"{name}\" => query.order_by({col_ident}::{variant}, dir),",
                name = f.name,
                variant = to_title_case(&f.name)
            )
            .unwrap();
        }
        writeln!(out, "            _ => query,").unwrap();
        writeln!(out, "        }};").unwrap();
        writeln!(out, "        Ok(next)").unwrap();
        writeln!(out, "    }}").unwrap();

        writeln!(
        out,
        "    fn apply_cursor<'db>(&self, query: Query<'db, {model_title}Model>, column: &str, dir: SortDirection, cursor: &str) -> anyhow::Result<Option<Query<'db, {model_title}Model>>> where Self: 'db {{"
    )
    .unwrap();
        writeln!(
            out,
            "        let Some(col) = Self::parse_col(column) else {{ return Ok(None); }};"
        )
        .unwrap();
        writeln!(
        out,
        "        let Some(bind) = Self::parse_bind_for_col(column, cursor) else {{ return Ok(None); }};"
    )
    .unwrap();
        writeln!(
        out,
        "        let op = match dir {{ SortDirection::Asc => Op::Gt, SortDirection::Desc => Op::Lt }};"
    )
    .unwrap();
        writeln!(out, "        Ok(Some(query.where_col(col, op, bind)))").unwrap();
        writeln!(out, "    }}").unwrap();

        writeln!(
            out,
            "    fn cursor_from_row(&self, row: &{model_title}Record, column: &str) -> Option<String> {{"
        )
        .unwrap();
        writeln!(out, "        match column {{").unwrap();
        for f in &db_fields {
            let Some(expr) = cursor_value_expr(&f.ty, &f.name) else {
                continue;
            };
            writeln!(
                out,
                "            \"{name}\" => {expr},",
                name = f.name,
                expr = expr
            )
            .unwrap();
        }
        writeln!(out, "            _ => None,").unwrap();
        writeln!(out, "        }}").unwrap();
        writeln!(out, "    }}").unwrap();

        writeln!(
        out,
        "    fn count<'db>(&self, query: Query<'db, {model_title}Model>) -> BoxFuture<'db, anyhow::Result<i64>> where Self: 'db {{"
    )
    .unwrap();
        writeln!(
            out,
            "        Box::pin(async move {{ query.count().await }})"
        )
        .unwrap();
        writeln!(out, "    }}").unwrap();

        writeln!(
        out,
        "    fn fetch_page<'db>(&self, query: Query<'db, {model_title}Model>, page: i64, per_page: i64) -> BoxFuture<'db, anyhow::Result<Vec<{model_title}Record>>> where Self: 'db {{"
    )
    .unwrap();
        writeln!(
            out,
            "        Box::pin(async move {{ Ok(query.paginate(page, per_page).await?.data) }})"
        )
        .unwrap();
        writeln!(out, "    }}").unwrap();
        writeln!(out, "}}").unwrap();

        let data_table_config_ident = format!("{}DataTableConfig", model_title);
        let data_table_hooks_ident = format!("{}DataTableHooks", model_title);
        let default_data_table_hooks_ident = format!("{}DefaultDataTableHooks", model_title);
        let data_table_ident = format!("{}DataTable", model_title);

        writeln!(out, "#[derive(Debug, Clone, Copy)]").unwrap();
        writeln!(out, "pub struct {data_table_config_ident} {{").unwrap();
        writeln!(out, "    pub default_sorting_column: &'static str,").unwrap();
        writeln!(out, "    pub default_sorted: SortDirection,").unwrap();
        writeln!(
            out,
            "    pub default_export_ignore_columns: &'static [&'static str],"
        )
        .unwrap();
        writeln!(
            out,
            "    pub default_timestamp_columns: &'static [&'static str],"
        )
        .unwrap();
        writeln!(out, "    pub default_unsortable: &'static [&'static str],").unwrap();
        writeln!(out, "    pub default_row_per_page: Option<i64>,").unwrap();
        writeln!(out, "}}").unwrap();

        writeln!(out, "impl Default for {data_table_config_ident} {{").unwrap();
        writeln!(out, "    fn default() -> Self {{").unwrap();
        writeln!(out, "        Self {{").unwrap();
        writeln!(out, "            default_sorting_column: \"{pk}\",").unwrap();
        writeln!(out, "            default_sorted: SortDirection::Desc,").unwrap();
        writeln!(
            out,
            "            default_export_ignore_columns: &[\"actions\", \"action\"],"
        )
        .unwrap();
        writeln!(
            out,
            "            default_timestamp_columns: &[{timestamp_cols_lit}],"
        )
        .unwrap();
        writeln!(out, "            default_unsortable: &[],").unwrap();
        writeln!(out, "            default_row_per_page: None,").unwrap();
        writeln!(out, "        }}").unwrap();
        writeln!(out, "    }}").unwrap();
        writeln!(out, "}}").unwrap();

        writeln!(
            out,
            "pub trait {data_table_hooks_ident}: Send + Sync + 'static {{"
        )
        .unwrap();
        writeln!(
        out,
        "    fn scope<'db>(&'db self, query: Query<'db, {model_title}Model>, _input: &DataTableInput, _ctx: &DataTableContext) -> Query<'db, {model_title}Model> {{ query }}"
    )
    .unwrap();
        writeln!(
        out,
        "    fn authorize(&self, _input: &DataTableInput, _ctx: &DataTableContext) -> anyhow::Result<bool> {{ Ok(true) }}"
    )
    .unwrap();
        writeln!(
        out,
        "    fn filter_query<'db>(&'db self, _query: Query<'db, {model_title}Model>, _filter_key: &str, _value: &str, _input: &DataTableInput, _ctx: &DataTableContext) -> anyhow::Result<Option<Query<'db, {model_title}Model>>> {{ Ok(None) }}"
    )
    .unwrap();
        writeln!(
        out,
        "    fn filters<'db>(&'db self, query: Query<'db, {model_title}Model>, _input: &DataTableInput, _ctx: &DataTableContext) -> anyhow::Result<Query<'db, {model_title}Model>> {{ Ok(query) }}"
    )
    .unwrap();
        writeln!(
        out,
        "    fn map_row(&self, _row: &mut {model_title}Record, _input: &DataTableInput, _ctx: &DataTableContext) -> anyhow::Result<()> {{ Ok(()) }}"
    )
    .unwrap();
        writeln!(
        out,
        "    fn default_row_to_record(&self, row: {model_title}Record) -> anyhow::Result<serde_json::Map<String, serde_json::Value>> {{"
    )
    .unwrap();
        writeln!(out, "        let value = serde_json::to_value(&row)?;").unwrap();
        writeln!(
        out,
        "        let mut record = match value {{ serde_json::Value::Object(map) => map, _ => anyhow::bail!(\"Generated row must serialize to a JSON object\"), }};"
    )
    .unwrap();
        for f in &db_fields {
            if hidden_fields.contains(&f.name) {
                writeln!(out, "        record.remove(\"{}\");", f.name).unwrap();
            }
        }
        for enum_field in &enum_explained_fields {
            if hidden_fields.contains(&enum_field.name) {
                writeln!(
                    out,
                    "        record.remove(\"{}\");",
                    enum_field.explained_name
                )
                .unwrap();
            }
        }
        for f in &localized_fields {
            if hidden_fields.contains(f) {
                writeln!(out, "        record.remove(\"{f}\");").unwrap();
                writeln!(out, "        record.remove(\"{f}_translations\");").unwrap();
            }
        }
        for a in &single_attachments {
            if hidden_fields.contains(&a.name) {
                writeln!(out, "        record.remove(\"{}\");", a.name).unwrap();
                writeln!(out, "        record.remove(\"{}_url\");", a.name).unwrap();
            }
        }
        for a in &multi_attachments {
            if hidden_fields.contains(&a.name) {
                writeln!(out, "        record.remove(\"{}\");", a.name).unwrap();
                writeln!(out, "        record.remove(\"{}_urls\");", a.name).unwrap();
            }
        }
        if has_meta && hidden_fields.contains("meta") {
            writeln!(out, "        record.remove(\"meta\");").unwrap();
        }
        if use_snowflake_id {
            writeln!(
                out,
                "        if let Some(id_value) = record.get(\"{pk}\").cloned() {{"
            )
            .unwrap();
            writeln!(out, "            let id_text = match id_value {{").unwrap();
            writeln!(
                out,
                "                serde_json::Value::Number(number) => number.to_string(),"
            )
            .unwrap();
            writeln!(
                out,
                "                serde_json::Value::String(text) => text,"
            )
            .unwrap();
            writeln!(out, "                other => other.to_string(),").unwrap();
            writeln!(out, "            }};").unwrap();
            writeln!(
            out,
            "            record.insert(\"{pk}\".to_string(), serde_json::Value::String(id_text));"
            )
            .unwrap();
            writeln!(out, "        }}").unwrap();
        }
        for computed in &computed_fields {
            writeln!(
                out,
                "        record.insert(\"{name}\".to_string(), serde_json::to_value(row.{name}())?);",
                name = computed.name
            )
            .unwrap();
        }
        writeln!(out, "        Ok(record)").unwrap();
        writeln!(out, "    }}").unwrap();
        writeln!(
        out,
        "    fn row_to_record(&self, row: {model_title}Record, _input: &DataTableInput, _ctx: &DataTableContext) -> anyhow::Result<serde_json::Map<String, serde_json::Value>> {{"
    )
    .unwrap();
        writeln!(out, "        self.default_row_to_record(row)").unwrap();
        writeln!(out, "    }}").unwrap();
        writeln!(
        out,
        "    fn summary<'db>(&'db self, _query: Query<'db, {model_title}Model>, _input: &DataTableInput, _ctx: &DataTableContext) -> BoxFuture<'db, anyhow::Result<Option<serde_json::Value>>> {{ Box::pin(async {{ Ok(None) }}) }}"
    )
    .unwrap();
        writeln!(out, "}}").unwrap();

        writeln!(out, "#[derive(Default)]").unwrap();
        writeln!(out, "pub struct {default_data_table_hooks_ident};").unwrap();
        writeln!(
            out,
            "impl {data_table_hooks_ident} for {default_data_table_hooks_ident} {{}}"
        )
        .unwrap();

        writeln!(
        out,
        "pub struct {data_table_ident}<H = {default_data_table_hooks_ident}> where H: {data_table_hooks_ident} {{"
    )
    .unwrap();
        writeln!(out, "    pub db: sqlx::PgPool,").unwrap();
        writeln!(out, "    pub hooks: H,").unwrap();
        writeln!(out, "    pub config: {data_table_config_ident},").unwrap();
        writeln!(out, "    adapter: {table_adapter_ident},").unwrap();
        writeln!(out, "}}").unwrap();

        writeln!(
            out,
            "impl {data_table_ident}<{default_data_table_hooks_ident}> {{"
        )
        .unwrap();
        writeln!(out, "    pub fn new(db: sqlx::PgPool) -> Self {{").unwrap();
        writeln!(out, "        Self {{").unwrap();
        writeln!(out, "            db,").unwrap();
        writeln!(out, "            hooks: {default_data_table_hooks_ident},").unwrap();
        writeln!(
            out,
            "            config: {data_table_config_ident}::default(),"
        )
        .unwrap();
        writeln!(out, "            adapter: {table_adapter_ident},").unwrap();
        writeln!(out, "        }}").unwrap();
        writeln!(out, "    }}").unwrap();
        writeln!(out, "}}").unwrap();

        writeln!(
            out,
            "impl<H: {data_table_hooks_ident}> {data_table_ident}<H> {{"
        )
        .unwrap();
        writeln!(
        out,
        "    pub fn with_hooks<NH: {data_table_hooks_ident}>(self, hooks: NH) -> {data_table_ident}<NH> {{"
    )
    .unwrap();
        writeln!(out, "        {data_table_ident} {{").unwrap();
        writeln!(out, "            db: self.db,").unwrap();
        writeln!(out, "            hooks,").unwrap();
        writeln!(out, "            config: self.config,").unwrap();
        writeln!(out, "            adapter: {table_adapter_ident},").unwrap();
        writeln!(out, "        }}").unwrap();
        writeln!(out, "    }}").unwrap();
        writeln!(
            out,
            "    pub fn with_config(mut self, config: {data_table_config_ident}) -> Self {{"
        )
        .unwrap();
        writeln!(out, "        self.config = config;").unwrap();
        writeln!(out, "        self").unwrap();
        writeln!(out, "    }}").unwrap();
        writeln!(out, "}}").unwrap();

        writeln!(
            out,
            "impl<H: {data_table_hooks_ident}> AutoDataTable for {data_table_ident}<H> {{"
        )
        .unwrap();
        writeln!(out, "    type Adapter = {table_adapter_ident};").unwrap();
        writeln!(
            out,
            "    fn adapter(&self) -> &Self::Adapter {{ &self.adapter }}"
        )
        .unwrap();
        writeln!(
        out,
        "    fn base_query<'db>(&'db self, input: &DataTableInput, ctx: &DataTableContext) -> Query<'db, {model_title}Model> {{"
    )
    .unwrap();
        writeln!(
            out,
            "        self.hooks.scope({model_title}Model::query(&self.db), input, ctx)"
        )
        .unwrap();
        writeln!(out, "    }}").unwrap();
        writeln!(
        out,
        "    fn authorize(&self, input: &DataTableInput, ctx: &DataTableContext) -> anyhow::Result<bool> {{ self.hooks.authorize(input, ctx) }}"
    )
    .unwrap();
        writeln!(
        out,
        "    fn filter_query<'db>(&'db self, query: Query<'db, {model_title}Model>, filter_key: &str, value: &str, input: &DataTableInput, ctx: &DataTableContext) -> anyhow::Result<Option<Query<'db, {model_title}Model>>> {{ self.hooks.filter_query(query, filter_key, value, input, ctx) }}"
    )
    .unwrap();
        writeln!(
        out,
        "    fn filters<'db>(&'db self, query: Query<'db, {model_title}Model>, input: &DataTableInput, ctx: &DataTableContext) -> anyhow::Result<Query<'db, {model_title}Model>> {{ self.hooks.filters(query, input, ctx) }}"
    )
    .unwrap();
        writeln!(
        out,
        "    fn map_row(&self, row: &mut {model_title}Record, input: &DataTableInput, ctx: &DataTableContext) -> anyhow::Result<()> {{ self.hooks.map_row(row, input, ctx) }}"
    )
    .unwrap();
        writeln!(
        out,
        "    fn row_to_record(&self, row: {model_title}Record, input: &DataTableInput, ctx: &DataTableContext) -> anyhow::Result<serde_json::Map<String, serde_json::Value>> {{ self.hooks.row_to_record(row, input, ctx) }}"
    )
    .unwrap();
        writeln!(
        out,
        "    fn summary<'db>(&'db self, query: Query<'db, {model_title}Model>, input: &DataTableInput, ctx: &DataTableContext) -> BoxFuture<'db, anyhow::Result<Option<serde_json::Value>>> where Self: 'db {{ self.hooks.summary(query, input, ctx) }}"
    )
    .unwrap();
        writeln!(
        out,
        "    fn default_sorting_column(&self) -> &'static str {{ self.config.default_sorting_column }}"
    )
    .unwrap();
        writeln!(
            out,
            "    fn default_sorted(&self) -> SortDirection {{ self.config.default_sorted }}"
        )
        .unwrap();
        writeln!(
        out,
        "    fn default_export_ignore_columns(&self) -> &'static [&'static str] {{ self.config.default_export_ignore_columns }}"
    )
    .unwrap();
        writeln!(
        out,
        "    fn default_timestamp_columns(&self) -> &'static [&'static str] {{ self.config.default_timestamp_columns }}"
    )
    .unwrap();
        writeln!(
        out,
        "    fn default_unsortable(&self) -> &'static [&'static str] {{ self.config.default_unsortable }}"
    )
    .unwrap();
        writeln!(
        out,
        "    fn default_row_per_page(&self, ctx: &DataTableContext) -> i64 {{ self.config.default_row_per_page.unwrap_or(ctx.default_per_page) }}"
    )
        .unwrap();
        writeln!(out, "}}").unwrap();
    }

    let datatable_section = out;
    let mut out = String::new();

    // Implement ActiveRecord for Record
    writeln!(out).unwrap();
    writeln!(out, "use core_db::common::active_record::ActiveRecord;").unwrap();
    writeln!(out, "#[async_trait::async_trait]").unwrap();
    writeln!(out, "impl ActiveRecord for {model_title}Record {{").unwrap();
    writeln!(out, "    type Id = {parent_pk_ty};").unwrap();
    writeln!(
        out,
        "    async fn find(db: &sqlx::PgPool, id: Self::Id) -> anyhow::Result<Option<Self>> {{"
    )
    .unwrap();
    writeln!(
        out,
        "        {model_title}Model::find(db, id).await.map_err(|e| e.into())"
    )
    .unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(out, "}}").unwrap();

    let active_record_section = out;
    let mut out = String::new();

    writeln!(out, "pub struct {model_title}Model;").unwrap();
    writeln!(out, "impl {model_title}Model {{").unwrap();
    writeln!(out, "    pub const TABLE: &'static str = \"{table}\";").unwrap();
    writeln!(
        out,
        "    pub const MODEL_KEY: &'static str = \"{model_snake}\";"
    )
    .unwrap();
    writeln!(out, "    pub const PK: &'static str = \"{pk}\";").unwrap();
    writeln!(
        out,
        "    pub fn query<'db>(db: impl Into<DbConn<'db>>) -> Query<'db, {model_title}Model> {{"
    )
    .unwrap();
    writeln!(out, "        Query::new(db)").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(
        out,
        "    pub fn query_with_base_url<'db>(db: impl Into<DbConn<'db>>, base_url: Option<String>) -> Query<'db, {model_title}Model> {{"
    )
    .unwrap();
    writeln!(out, "        Query::new_with_base_url(db, base_url)").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(
        out,
        "    pub fn create<'db>(db: impl Into<DbConn<'db>>) -> Create<'db, {model_title}Model> {{"
    )
    .unwrap();
    writeln!(out, "        Create::new(db)").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(
        out,
        "    pub fn create_with_base_url<'db>(db: impl Into<DbConn<'db>>, base_url: Option<String>) -> Create<'db, {model_title}Model> {{"
    )
    .unwrap();
    writeln!(out, "        Create::new_with_base_url(db, base_url)").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(
        out,
        "    pub fn patch<'db>(db: impl Into<DbConn<'db>>) -> Patch<'db, {model_title}Model> {{"
    )
    .unwrap();
    writeln!(out, "        Patch::new(db)").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(
        out,
        "    pub fn patch_with_base_url<'db>(db: impl Into<DbConn<'db>>, base_url: Option<String>) -> Patch<'db, {model_title}Model> {{"
    )
    .unwrap();
    writeln!(out, "        Patch::new_with_base_url(db, base_url)").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(
        out,
        "    pub async fn find<'db>(db: impl Into<DbConn<'db>>, id: {parent_pk_ty}) -> Result<Option<{model_title}Record>> {{"
    )
    .unwrap();
    writeln!(
        out,
        "        Query::<{model_title}Model>::new(db).find(id).await"
    )
    .unwrap();
    writeln!(out, "    }}").unwrap();
    // _transform_create_value — handles hashed fields for CreateModel::transform_create_value
    writeln!(out, "    fn _transform_create_value(col: &str, value: BindValue) -> anyhow::Result<BindValue> {{").unwrap();
    {
        let hashed_fields: Vec<&FieldSpec> = db_fields
            .iter()
            .filter(|f| matches!(f.special_type, Some(SpecialType::Hashed)))
            .collect();
        if hashed_fields.is_empty() {
            writeln!(out, "        let _ = col;").unwrap();
            writeln!(out, "        Ok(value)").unwrap();
        } else {
            writeln!(out, "        match col {{").unwrap();
            for field in &hashed_fields {
                let col_variant = to_title_case(&field.name);
                writeln!(
                    out,
                    "            c if c == {col_ident}::{col_variant}.as_sql() => {{"
                )
                .unwrap();
                writeln!(
                    out,
                    "                let BindValue::String(value) = value else {{"
                )
                .unwrap();
                writeln!(out, "                    anyhow::bail!(\"column '{{}}' expects String before hashing, got '{{:?}}'\", col, value);").unwrap();
                writeln!(out, "                }};").unwrap();
                writeln!(out, "                let hashed = core_db::common::auth::hash::hash_password(&value)?;").unwrap();
                writeln!(out, "                Ok(hashed.into())").unwrap();
                writeln!(out, "            }}").unwrap();
            }
            writeln!(out, "            _ => Ok(value),").unwrap();
            writeln!(out, "        }}").unwrap();
        }
    }
    writeln!(out, "    }}").unwrap();
    // _transform_patch_value — handles hashed fields for PatchModel::transform_patch_value
    writeln!(out, "    fn _transform_patch_value(col: &str, value: BindValue) -> anyhow::Result<BindValue> {{").unwrap();
    {
        let hashed_fields: Vec<&FieldSpec> = db_fields
            .iter()
            .filter(|f| matches!(f.special_type, Some(SpecialType::Hashed)))
            .collect();
        if hashed_fields.is_empty() {
            writeln!(out, "        let _ = col;").unwrap();
            writeln!(out, "        Ok(value)").unwrap();
        } else {
            writeln!(out, "        match col {{").unwrap();
            for field in &hashed_fields {
                let col_variant = to_title_case(&field.name);
                writeln!(
                    out,
                    "            c if c == {col_ident}::{col_variant}.as_sql() => {{"
                )
                .unwrap();
                writeln!(
                    out,
                    "                let BindValue::String(value) = value else {{"
                )
                .unwrap();
                writeln!(out, "                    anyhow::bail!(\"column '{{}}' expects String before hashing, got '{{:?}}'\", col, value);").unwrap();
                writeln!(out, "                }};").unwrap();
                writeln!(out, "                let hashed = core_db::common::auth::hash::hash_password(&value)?;").unwrap();
                writeln!(out, "                Ok(hashed.into())").unwrap();
                writeln!(out, "            }}").unwrap();
            }
            writeln!(out, "            _ => Ok(value),").unwrap();
            writeln!(out, "        }}").unwrap();
        }
    }
    writeln!(out, "    }}").unwrap();
    if emit_hooks {
        let pk_snake_local = to_snake(&pk);
        writeln!(out).unwrap();
        writeln!(
            out,
            "    async fn convert_delete_to_update<'tx>(db: &DbConn<'tx>, ids: &[{parent_pk_ty}], overrides: serde_json::Value) -> Result<u64> {{"
        )
        .unwrap();
        writeln!(out, "        let map = overrides.as_object()").unwrap();
        writeln!(
            out,
            "            .ok_or_else(|| anyhow::anyhow!(\"observer overrides must be a JSON object\"))?;"
        )
        .unwrap();
        writeln!(
            out,
            "        let mut set_clauses: Vec<String> = Vec::new();"
        )
        .unwrap();
        writeln!(out, "        let mut binds: Vec<BindValue> = Vec::new();").unwrap();
        writeln!(out, "        let mut idx = 1usize;").unwrap();
        writeln!(out, "        for (key, val) in map {{").unwrap();
        writeln!(out, "            match key.as_str() {{").unwrap();
        for f in &db_fields {
            let deser_ty = json_deser_type_for_field(&f.ty, &enum_specs);
            writeln!(out, "                \"{}\" => {{", f.name).unwrap();
            writeln!(
                out,
                "                    let v: {deser_ty} = serde_json::from_value(val.clone())?;"
            )
            .unwrap();
            writeln!(
                out,
                "                    set_clauses.push(format!(\"{{}} = ${{}}\", \"{}\", idx));",
                f.name
            )
            .unwrap();
            writeln!(out, "                    binds.push(v.into());").unwrap();
            writeln!(out, "                    idx += 1;").unwrap();
            writeln!(out, "                }}").unwrap();
        }
        writeln!(
            out,
            "                other => anyhow::bail!(\"unknown column '{{}}' in observer delete overrides\", other),"
        )
        .unwrap();
        writeln!(out, "            }}").unwrap();
        writeln!(out, "        }}").unwrap();
        writeln!(
            out,
            "        if set_clauses.is_empty() {{ anyhow::bail!(\"observer Modify returned empty overrides\"); }}"
        )
        .unwrap();
        writeln!(
            out,
            "        let phs: Vec<String> = ids.iter().enumerate().map(|(i, _)| format!(\"${{}}\", idx + i)).collect();"
        )
        .unwrap();
        writeln!(
            out,
            "        let sql = format!(\"UPDATE {table} SET {{}} WHERE {pk_snake_local} IN ({{}})\", set_clauses.join(\", \"), phs.join(\", \"));"
        )
        .unwrap();
        writeln!(out, "        let mut q = sqlx::query(&sql);").unwrap();
        writeln!(
            out,
            "        for b in &binds {{ q = bind_query(q, b.clone()); }}"
        )
        .unwrap();
        writeln!(out, "        for id in ids {{ q = q.bind(id); }}").unwrap();
        writeln!(out, "        let res = db.execute(q).await?;").unwrap();
        writeln!(out, "        Ok(res.rows_affected())").unwrap();
        writeln!(out, "    }}").unwrap();
    }
    writeln!(out, "}}\n").unwrap();
    if !cfg.model_impl_items.is_empty() {
        out.push_str(&render_custom_impl_block(
            &format!("{model_title}Model"),
            &cfg.model_impl_items,
        ));
    }
    writeln!(out, "impl ModelDef for {model_title}Model {{").unwrap();
    writeln!(out, "    type Pk = {parent_pk_ty};").unwrap();
    writeln!(out, "    type Record = {model_title}Record;").unwrap();
    writeln!(out, "    type Create = {model_title}Create;").unwrap();
    writeln!(out, "    type Changes = {model_title}Changes;").unwrap();
    writeln!(
        out,
        "    const TABLE: &'static str = {model_title}Model::TABLE;"
    )
    .unwrap();
    writeln!(
        out,
        "    const MODEL_KEY: &'static str = {model_title}Model::MODEL_KEY;"
    )
    .unwrap();
    writeln!(out, "    const PK_COL: &'static str = \"{pk}\";").unwrap();
    writeln!(out, "}}\n").unwrap();
    writeln!(
        out,
        "impl core_db::common::model_api::QueryModel for {model_title}Model {{"
    )
    .unwrap();
    writeln!(
        out,
        "    const DEFAULT_SELECT: &'static str = \"{base_select}\";"
    )
    .unwrap();
    writeln!(out, "    const HAS_SOFT_DELETE: bool = {has_soft_delete};").unwrap();
    {
        let sd_col = if has_soft_delete { "deleted_at" } else { "" };
        writeln!(
            out,
            "    const SOFT_DELETE_COL: &'static str = \"{sd_col}\";"
        )
        .unwrap();
    }
    writeln!(out, "    const HAS_CREATED_AT: bool = {has_created_at};").unwrap();
    writeln!(out, "    const HAS_UPDATED_AT: bool = {has_updated_at};").unwrap();
    writeln!(
        out,
        "    fn query_all<'db>(state: QueryState<'db>) -> core_db::common::model_api::BoxModelFuture<'db, Vec<Self::Record>> {{"
    )
    .unwrap();
    writeln!(out, "        Box::pin(async move {{").unwrap();
    out.push_str(&render_query_all_body(
        &model_title,
        &row_ident,
        has_soft_delete,
        &table,
        &model_snake,
        &pk,
        &parent_pk_ty,
        &relations,
        &localized_fields,
        has_meta,
        has_attachments,
        skip_profiler,
    ));
    writeln!(out, "        }})").unwrap();
    writeln!(out, "    }}").unwrap();
    // query_first: limit(1) + query_all
    writeln!(
        out,
        "    fn query_first<'db>(state: QueryState<'db>) -> core_db::common::model_api::BoxModelFuture<'db, Option<Self::Record>> {{"
    )
    .unwrap();
    writeln!(out, "        Box::pin(async move {{").unwrap();
    writeln!(
        out,
        "            let mut v = Self::query_all(state.limit(1)).await?;"
    )
    .unwrap();
    writeln!(out, "            Ok(v.pop())").unwrap();
    writeln!(out, "        }})").unwrap();
    writeln!(out, "    }}").unwrap();
    // query_find: where_col_str + query_first
    writeln!(
        out,
        "    fn query_find<'db>(state: QueryState<'db>, id: Self::Pk) -> core_db::common::model_api::BoxModelFuture<'db, Option<Self::Record>> {{"
    )
    .unwrap();
    writeln!(
        out,
        "        Box::pin(async move {{ Self::query_first(state.where_col_str(\"{pk}\", Op::Eq, id.into())).await }})"
    )
    .unwrap();
    writeln!(out, "    }}").unwrap();
    // query_count: use to_count_sql
    writeln!(
        out,
        "    fn query_count<'db>(state: QueryState<'db>) -> core_db::common::model_api::BoxModelFuture<'db, i64> {{"
    )
    .unwrap();
    writeln!(out, "        Box::pin(async move {{").unwrap();
    out.push_str(&render_query_count_body(
        has_soft_delete,
        &table,
        skip_profiler,
    ));
    writeln!(out, "        }})").unwrap();
    writeln!(out, "    }}").unwrap();
    // query_delete: inline with observer hooks and soft-delete
    writeln!(
        out,
        "    fn query_delete<'db>(state: QueryState<'db>) -> core_db::common::model_api::BoxModelFuture<'db, u64> {{"
    )
    .unwrap();
    writeln!(out, "        Box::pin(async move {{").unwrap();
    out.push_str(&render_query_delete_body(
        &table,
        &model_snake,
        &col_ident,
        has_soft_delete,
        emit_hooks,
        &row_ident,
        &to_snake(&pk),
        skip_profiler,
        &parent_pk_ty,
    ));
    writeln!(out, "        }})").unwrap();
    writeln!(out, "    }}").unwrap();
    // query_paginate: inline using to_count_sql + to_select_sql
    writeln!(
        out,
        "    fn query_paginate<'db>(state: QueryState<'db>, page: i64, per_page: i64) -> core_db::common::model_api::BoxModelFuture<'db, core_db::common::model_api::Page<Self::Record>> {{"
    )
    .unwrap();
    writeln!(out, "        Box::pin(async move {{").unwrap();
    out.push_str(&render_query_paginate_body(
        &model_title,
        &row_ident,
        has_soft_delete,
        &table,
        &model_snake,
        &pk,
        &parent_pk_ty,
        &relations,
        &localized_fields,
        has_meta,
        has_attachments,
        skip_profiler,
    ));
    writeln!(out, "        }})").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(out, "}}\n").unwrap();
    out.push_str(&render_create_model_impl(
        &model_title,
        &insert_ident,
        &query_ident,
        &pk,
        &table,
    ));
    out.push_str(&render_create_field_impl(
        &model_title,
        &col_ident,
        &db_fields,
    ));
    out.push_str(&render_create_conflict_field_impl(&model_title, &col_ident));
    out.push_str(&render_patch_model_impl(
        &model_title,
        &query_ident,
        &update_ident,
        &col_ident,
        &pk_col_variant,
        &parent_pk_ty,
        &table,
        has_soft_delete,
    ));
    out.push_str(&render_patch_assign_field_impl(
        &model_title,
        &col_ident,
        &db_fields,
    ));
    out.push_str(&render_patch_numeric_field_impl(
        &model_title,
        &col_ident,
        &db_fields,
    ));
    writeln!(
        out,
        "impl core_db::common::model_api::ColExpr for {col_ident} {{"
    )
    .unwrap();
    writeln!(
        out,
        "    fn col_sql(self) -> &'static str {{ self.as_sql() }}"
    )
    .unwrap();
    writeln!(out, "}}\n").unwrap();
    writeln!(
        out,
        "impl core_db::common::model_api::QueryField<{model_title}Model> for {col_ident} {{"
    )
    .unwrap();
    writeln!(out, "    type Value = BindValue;").unwrap();
    writeln!(
        out,
        "    fn where_col<'db>(field: Self, state: QueryState<'db>, op: Op, value: BindValue) -> QueryState<'db> {{"
    )
    .unwrap();
    writeln!(
        out,
        "        state.where_col_str(field.as_sql(), op, value)"
    )
    .unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(
        out,
        "    fn or_where_col<'db>(field: Self, state: QueryState<'db>, op: Op, value: BindValue) -> QueryState<'db> {{"
    )
    .unwrap();
    writeln!(
        out,
        "        state.or_where_col_str(field.as_sql(), op, value)"
    )
    .unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(
        out,
        "    fn where_in<'db>(field: Self, state: QueryState<'db>, values: &[BindValue]) -> QueryState<'db> {{"
    )
    .unwrap();
    writeln!(out, "        state.where_in_str(field.as_sql(), values)").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(
        out,
        "    fn order_by<'db>(field: Self, state: QueryState<'db>, dir: OrderDir) -> QueryState<'db> {{"
    )
    .unwrap();
    writeln!(out, "        state.order_by_str(field.as_sql(), dir)").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(
        out,
        "    fn where_null<'db>(field: Self, state: QueryState<'db>) -> QueryState<'db> {{"
    )
    .unwrap();
    writeln!(out, "        state.where_null_str(field.as_sql())").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(
        out,
        "    fn where_not_null<'db>(field: Self, state: QueryState<'db>) -> QueryState<'db> {{"
    )
    .unwrap();
    writeln!(out, "        state.where_not_null_str(field.as_sql())").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(out, "}}\n").unwrap();
    if !relations.is_empty() {
        let model_query_ty = "QueryState<'db>".to_string();
        for (rel_idx, rel) in relations.iter().enumerate() {
            let rel_snake = to_snake(&rel.name);
            let target_model_title = to_title_case(&rel.target_model);
            let target_record_ident = format!("{target_model_title}Record");
            let rel_ty = match rel.kind {
                RelationKind::BelongsTo => {
                    format!("OneRelation<{model_title}Model, {target_record_ident}, {rel_idx}>")
                }
                RelationKind::HasMany => {
                    format!("ManyRelation<{model_title}Model, {target_record_ident}, {rel_idx}>")
                }
            };
            writeln!(
                out,
                "impl core_db::common::model_api::IncludeRelation<{model_title}Model> for {rel_ty} {{"
            )
            .unwrap();
            writeln!(
                out,
                "    fn include<'db>(_relation: Self, state: {model_query_ty}) -> {model_query_ty} {{"
            )
            .unwrap();
            writeln!(out, "        state").unwrap();
            writeln!(out, "    }}").unwrap();
            writeln!(out, "}}\n").unwrap();

            let link_clause = match rel.kind {
                RelationKind::HasMany => format!(
                    "{}.{} = {}.{}",
                    rel.target_table, rel.foreign_key, table, rel.local_key
                ),
                RelationKind::BelongsTo => format!(
                    "{}.{} = {}.{}",
                    rel.target_table, rel.target_pk, table, rel.foreign_key
                ),
            };
            writeln!(
                out,
                "impl core_db::common::model_api::WhereHasRelation<{model_title}Model> for {rel_ty} {{"
            )
            .unwrap();
            writeln!(out, "    type Target = {target_model_title}Model;").unwrap();
            writeln!(
                out,
                "    fn where_has<'db, F>(_relation: Self, mut state: {model_query_ty}, scope: F) -> {model_query_ty}"
            )
            .unwrap();
            writeln!(out, "    where").unwrap();
            writeln!(
                out,
                "        F: FnOnce(Query<'db, Self::Target>) -> Query<'db, Self::Target>,"
            )
            .unwrap();
            writeln!(out, "    {{").unwrap();
            writeln!(out, "        let start_idx = state.binds.len() + 1;").unwrap();
            writeln!(
                out,
                "        let scoped = scope({target_model_title}Model::query_with_base_url(state.db.clone(), state.base_url.clone()));"
            )
            .unwrap();
            writeln!(
                out,
                "        let (mut sub_where, sub_binds) = scoped.into_inner().into_where_parts();"
            )
            .unwrap();
            writeln!(
                out,
                "        sub_where.insert(0, \"{link_clause}\".to_string());"
            )
            .unwrap();
            writeln!(
                out,
                "        let mut clause = String::from(\"EXISTS (SELECT 1 FROM {rel_table} WHERE \");",
                rel_table = rel.target_table
            )
            .unwrap();
            writeln!(out, "        clause.push_str(&sub_where.join(\" AND \"));").unwrap();
            writeln!(out, "        clause.push(')');").unwrap();
            writeln!(
                out,
                "        let clause = renumber_placeholders(&clause, start_idx);"
            )
            .unwrap();
            writeln!(out, "        state.where_sql.push(clause);").unwrap();
            writeln!(out, "        state.binds.extend(sub_binds);").unwrap();
            writeln!(out, "        state").unwrap();
            writeln!(out, "    }}").unwrap();
            writeln!(
                out,
                "    fn or_where_has<'db, F>(_relation: Self, mut state: {model_query_ty}, scope: F) -> {model_query_ty}"
            )
            .unwrap();
            writeln!(out, "    where").unwrap();
            writeln!(
                out,
                "        F: FnOnce(Query<'db, Self::Target>) -> Query<'db, Self::Target>,"
            )
            .unwrap();
            writeln!(out, "    {{").unwrap();
            writeln!(out, "        let start_idx = state.binds.len() + 1;").unwrap();
            writeln!(
                out,
                "        let scoped = scope({target_model_title}Model::query_with_base_url(state.db.clone(), state.base_url.clone()));"
            )
            .unwrap();
            writeln!(
                out,
                "        let (mut sub_where, sub_binds) = scoped.into_inner().into_where_parts();"
            )
            .unwrap();
            writeln!(
                out,
                "        sub_where.insert(0, \"{link_clause}\".to_string());"
            )
            .unwrap();
            writeln!(
                out,
                "        let mut clause = String::from(\"EXISTS (SELECT 1 FROM {rel_table} WHERE \");",
                rel_table = rel.target_table
            )
            .unwrap();
            writeln!(out, "        clause.push_str(&sub_where.join(\" AND \"));").unwrap();
            writeln!(out, "        clause.push(')');").unwrap();
            writeln!(
                out,
                "        let clause = renumber_placeholders(&clause, start_idx);"
            )
            .unwrap();
            writeln!(out, "        if let Some(last) = state.where_sql.pop() {{").unwrap();
            writeln!(
                out,
                "            state.where_sql.push(format!(\"({{}} OR {{}})\", last, clause));"
            )
            .unwrap();
            writeln!(out, "        }} else {{").unwrap();
            writeln!(out, "            state.where_sql.push(clause);").unwrap();
            writeln!(out, "        }}").unwrap();
            writeln!(out, "        state.binds.extend(sub_binds);").unwrap();
            writeln!(out, "        state").unwrap();
            writeln!(out, "    }}").unwrap();
            writeln!(out, "}}\n").unwrap();

            match rel.kind {
                RelationKind::BelongsTo => {
                    writeln!(
                        out,
                        "impl core_db::common::model_api::RecordOneRelation<{model_title}Model> for {rel_ty} {{"
                    )
                    .unwrap();
                    writeln!(out, "    type Target = {target_record_ident};").unwrap();
                    writeln!(
                        out,
                        "    fn get<'a>(_relation: Self, record: &'a {model_title}Record) -> Option<&'a Self::Target> {{"
                    )
                    .unwrap();
                    writeln!(out, "        record.{rel_snake}.as_deref()").unwrap();
                    writeln!(out, "    }}").unwrap();
                    writeln!(out, "}}\n").unwrap();
                }
                RelationKind::HasMany => {
                    writeln!(
                        out,
                        "impl core_db::common::model_api::RecordManyRelation<{model_title}Model> for {rel_ty} {{"
                    )
                    .unwrap();
                    writeln!(out, "    type Target = {target_record_ident};").unwrap();
                    writeln!(
                        out,
                        "    fn get<'a>(_relation: Self, record: &'a {model_title}Record) -> &'a [Self::Target] {{"
                    )
                    .unwrap();
                    writeln!(out, "        &record.{rel_snake}").unwrap();
                    writeln!(out, "    }}").unwrap();
                    writeln!(out, "}}\n").unwrap();
                }
            }
        }
    }

    let model_runtime_section = out;

    let mut context = TemplateContext::new();
    context
        .insert("imports", imports.trim_end().to_string())
        .unwrap();
    context
        .insert("constants", constants.trim_end().to_string())
        .unwrap();
    context
        .insert(
            "row_view_json_section",
            row_view_json_section.trim_start().to_string(),
        )
        .unwrap();
    context
        .insert(
            "column_model_section",
            column_model_section.trim_start().to_string(),
        )
        .unwrap();
    context
        .insert("query_section", query_section.trim_start().to_string())
        .unwrap();
    context
        .insert("insert_section", insert_section.trim_start().to_string())
        .unwrap();
    context
        .insert("update_section", update_section.trim_start().to_string())
        .unwrap();
    context
        .insert(
            "datatable_section",
            datatable_section.trim_start().to_string(),
        )
        .unwrap();
    context
        .insert(
            "active_record_section",
            active_record_section.trim_start().to_string(),
        )
        .unwrap();
    context
        .insert(
            "model_runtime_section",
            model_runtime_section.trim_start().to_string(),
        )
        .unwrap();
    render_template("models/model.rs.tpl", &context).unwrap()
}

fn generate_common() -> String {
    render_template("models/common.rs.tpl", &TemplateContext::new()).unwrap()
}

fn render_custom_impl_block(target_ident: &str, items: &[String]) -> String {
    if items.is_empty() {
        return String::new();
    }

    let mut out = String::new();
    writeln!(out, "impl {target_ident} {{").unwrap();
    for item in items {
        writeln!(out, "{}", indent_generated_block(item, 4)).unwrap();
    }
    writeln!(out, "}}\n").unwrap();
    out
}

fn indent_generated_block(block: &str, spaces: usize) -> String {
    let indent = " ".repeat(spaces);
    block
        .lines()
        .map(|line| {
            if line.trim().is_empty() {
                String::new()
            } else {
                format!("{indent}{line}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}
