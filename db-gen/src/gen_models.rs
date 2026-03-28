use crate::config::ConfigsFile;
use crate::schema::{
    parse_attachments, parse_computed, parse_fields, parse_meta, parse_relations, to_snake,
    to_title_case, AttachmentFieldSpec, EnumOrOther, EnumSpec, FieldSpec, MetaType,
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

fn render_create_model_impl(
    model_title: &str,
    pk: &str,
    _table: &str,
    db_fields: &[FieldSpec],
    enum_specs: &BTreeMap<String, EnumSpec>,
    use_snowflake_id: bool,
) -> String {
    let mut out = String::new();
    writeln!(
        out,
        "impl core_db::common::model_api::CreateModel for {model_title}Model {{"
    )
    .unwrap();
    writeln!(
        out,
        "    const USE_SNOWFLAKE_ID: bool = {use_snowflake_id};"
    )
    .unwrap();
    writeln!(
        out,
        "    fn build_create_input(state: &CreateState<'_>) -> anyhow::Result<Self::Create> {{"
    )
    .unwrap();
    writeln!(
        out,
        "        let mut input = {model_title}Create::default();"
    )
    .unwrap();
    writeln!(out, "        for assignment in &state.assignments {{").unwrap();
    writeln!(out, "            let bind = &assignment.value;").unwrap();
    writeln!(out, "            match assignment.col_sql {{").unwrap();
    for f in db_fields {
        let decode_expr = render_bind_decode_expr(&f.ty, "bind", enum_specs);
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
    writeln!(
        out,
        "    fn apply_create_overrides(mut state: CreateState<'_>, overrides: serde_json::Value) -> anyhow::Result<CreateState<'_>> {{"
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
    for f in db_fields {
        let deser_ty = json_deser_type_for_field(&f.ty, enum_specs);
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
    writeln!(
        out,
        "    fn created_row_key(row: &<Self as core_db::common::model_api::RuntimeModel>::Row) -> String {{"
    )
    .unwrap();
    writeln!(out, "        row.{pk}.to_string()").unwrap();
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
    model_snake: &str,
    db_fields: &[FieldSpec],
    enum_specs: &BTreeMap<String, EnumSpec>,
    localized_fields: &[String],
    has_meta: bool,
    has_attachments: bool,
    touch_targets: &[(String, String, String, String, String)],
    table: &str,
    pk: &str,
) -> String {
    let mut out = String::new();
    let pk_snake = to_snake(pk);
    let model_snake_upper = model_snake.to_uppercase();
    writeln!(
        out,
        "impl core_db::common::model_api::PatchModel for {model_title}Model {{"
    )
    .unwrap();
    writeln!(
        out,
        "    fn build_patch_changes(state: &PatchState<'_>) -> anyhow::Result<Self::Changes> {{"
    )
    .unwrap();
    writeln!(out, "        let mut changes = Self::Changes::default();").unwrap();
    writeln!(out, "        for assignment in &state.assignments {{").unwrap();
    writeln!(out, "            let bind = &assignment.value;").unwrap();
    writeln!(out, "            let mode = &assignment.mode;").unwrap();
    writeln!(out, "            match assignment.col_sql {{").unwrap();
    for f in db_fields {
        let decode_expr = render_bind_decode_expr(&f.ty, "bind", enum_specs);
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
    writeln!(
        out,
        "    fn apply_patch_overrides<'db>(mut state: PatchState<'db>, overrides: serde_json::Value) -> anyhow::Result<PatchState<'db>> {{"
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
    for f in db_fields {
        let deser_ty = json_deser_type_for_field(&f.ty, enum_specs);
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
    writeln!(
        out,
        "    fn row_pk(row: &<Self as core_db::common::model_api::RuntimeModel>::Row) -> Self::Pk {{"
    )
    .unwrap();
    writeln!(out, "        row.{pk_snake}.clone()").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(
        out,
        "    fn row_pk_text(row: &<Self as core_db::common::model_api::RuntimeModel>::Row) -> String {{"
    )
    .unwrap();
    writeln!(out, "        row.{pk_snake}.to_string()").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(
        out,
        "    fn persist_patch_state<'db>(db: DbConn<'db>, target_ids: Vec<Self::Pk>, state: PatchState<'db>) -> core_db::common::model_api::BoxModelFuture<'db, ()> {{"
    )
    .unwrap();
    writeln!(out, "        Box::pin(async move {{").unwrap();
    writeln!(out, "            if target_ids.is_empty() {{").unwrap();
    writeln!(out, "                return Ok(());").unwrap();
    writeln!(out, "            }}").unwrap();
    if !touch_targets.is_empty() {
        writeln!(out, "            fn to_params(len: usize) -> String {{").unwrap();
        writeln!(out, "                (1..=len).map(|i| format!(\"${{}}\", i)).collect::<Vec<_>>().join(\", \")").unwrap();
        writeln!(out, "            }}").unwrap();
        for (fk, target_snake, target_title, target_pk, target_pk_ty) in touch_targets {
            writeln!(out, "            let placeholders = to_params(target_ids.len());").unwrap();
            writeln!(
                out,
                "            let sql = format!(\"SELECT DISTINCT {} FROM {} WHERE {} IN ({{}})\", placeholders);",
                fk, table, pk
            )
            .unwrap();
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
                "                crate::generated::models::{}::{}::query()",
                target_snake, target_title
            )
            .unwrap();
            writeln!(
                out,
                "                    .where_col(crate::generated::models::{}::{}Col::{}, Op::Eq, pid)",
                target_snake,
                target_title,
                to_snake(target_pk).to_uppercase()
            )
            .unwrap();
            writeln!(out, "                    .patch()").unwrap();
            writeln!(
                out,
                "                    .assign(crate::generated::models::{}::{}Col::UPDATED_AT, time::OffsetDateTime::now_utc())?",
                target_snake, target_title
            )
            .unwrap();
            writeln!(out, "                    .save(db.clone()).await?;").unwrap();
            writeln!(out, "            }}").unwrap();
        }
    }
    if !localized_fields.is_empty() {
        writeln!(out, "            if !state.translations.is_empty() {{").unwrap();
        writeln!(out, "                let supported = localized::SUPPORTED_LOCALES;").unwrap();
        for f in localized_fields {
            writeln!(out, "                if let Some(map) = state.translations.get(\"{f}\") {{").unwrap();
            writeln!(out, "                    let mut filtered = HashMap::new();").unwrap();
            writeln!(out, "                    for (loc, val) in map {{").unwrap();
            writeln!(out, "                        if supported.contains(&loc.as_str()) {{ filtered.insert(loc.clone(), val.clone()); }}").unwrap();
            writeln!(out, "                    }}").unwrap();
            writeln!(out, "                    if !filtered.is_empty() {{").unwrap();
            writeln!(out, "                        for id in &target_ids {{").unwrap();
            writeln!(
                out,
                "                            localized::upsert_localized_many(db.clone(), localized::{}_OWNER_TYPE, id.clone(), \"{f}\", &filtered).await?;",
                model_snake_upper
            )
            .unwrap();
            writeln!(out, "                        }}").unwrap();
            writeln!(out, "                    }}").unwrap();
            writeln!(out, "                }}").unwrap();
        }
        writeln!(out, "            }}").unwrap();
    }
    if has_meta {
        writeln!(out, "            if !state.meta.is_empty() {{").unwrap();
        writeln!(out, "                for id in &target_ids {{").unwrap();
        writeln!(
            out,
            "                    localized::upsert_meta_many(db.clone(), localized::{}_OWNER_TYPE, id.clone(), &state.meta).await?;",
            model_snake_upper
        )
        .unwrap();
        writeln!(out, "                }}").unwrap();
        writeln!(out, "            }}").unwrap();
    }
    if has_attachments {
        writeln!(out, "            for id in &target_ids {{").unwrap();
        writeln!(out, "                for field in &state.attachments_clear_single {{").unwrap();
        writeln!(
            out,
            "                    localized::clear_attachment_field(db.clone(), localized::{}_OWNER_TYPE, id.clone(), field).await?;",
            model_snake_upper
        )
        .unwrap();
        writeln!(out, "                }}").unwrap();
        writeln!(out, "                for (field, att) in &state.attachments_single {{").unwrap();
        writeln!(
            out,
            "                    localized::replace_single_attachment(db.clone(), localized::{}_OWNER_TYPE, id.clone(), field, att).await?;",
            model_snake_upper
        )
        .unwrap();
        writeln!(out, "                }}").unwrap();
        writeln!(out, "                for (field, list) in &state.attachments_multi {{").unwrap();
        writeln!(
            out,
            "                    localized::add_attachments(db.clone(), localized::{}_OWNER_TYPE, id.clone(), field, list).await?;",
            model_snake_upper
        )
        .unwrap();
        writeln!(out, "                }}").unwrap();
        writeln!(out, "                for (field, ids) in &state.attachments_delete_multi {{").unwrap();
        writeln!(
            out,
            "                    localized::delete_attachment_ids(db.clone(), localized::{}_OWNER_TYPE, id.clone(), field, ids).await?;",
            model_snake_upper
        )
        .unwrap();
        writeln!(out, "                }}").unwrap();
        writeln!(out, "            }}").unwrap();
    }
    writeln!(out, "            Ok(())").unwrap();
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

fn render_feature_persistence_model_impl(
    model_title: &str,
    model_snake: &str,
    parent_pk_ty: &str,
    localized_fields: &[String],
    has_meta: bool,
    has_attachments: bool,
    touch_targets: &[(String, String, String, String, String)],
    table: &str,
    pk: &str,
) -> String {
    let mut out = String::new();
    let model_snake_upper = model_snake.to_uppercase();
    let pk_snake = to_snake(pk);
    let has_feature_owner = !localized_fields.is_empty() || has_meta || has_attachments;

    writeln!(
        out,
        "impl core_db::common::model_api::FeaturePersistenceModel for {model_title}Model {{"
    )
    .unwrap();

    if has_feature_owner && parent_pk_ty == "i64" {
        writeln!(
            out,
            "    fn create_owner_id(row: &<Self as core_db::common::model_api::RuntimeModel>::Row) -> Option<i64> {{"
        )
        .unwrap();
        writeln!(out, "        Some(row.{pk_snake}.clone())").unwrap();
        writeln!(out, "    }}").unwrap();
        writeln!(
            out,
            "    fn patch_owner_id(pk: &Self::Pk) -> Option<i64> {{"
        )
        .unwrap();
        writeln!(out, "        Some(pk.clone())").unwrap();
        writeln!(out, "    }}").unwrap();
    }

    if !localized_fields.is_empty() {
        writeln!(
            out,
            "    fn supported_locales() -> &'static [&'static str] {{"
        )
        .unwrap();
        writeln!(out, "        localized::SUPPORTED_LOCALES").unwrap();
        writeln!(out, "    }}").unwrap();
        writeln!(
            out,
            "    fn localized_owner_type() -> Option<&'static str> {{"
        )
        .unwrap();
        writeln!(
            out,
            "        Some(localized::{model_snake_upper}_OWNER_TYPE)"
        )
        .unwrap();
        writeln!(out, "    }}").unwrap();
        writeln!(
            out,
            "    fn upsert_localized_many<'db>("
        )
        .unwrap();
        writeln!(out, "        db: DbConn<'db>,").unwrap();
        writeln!(out, "        owner_type: &'static str,").unwrap();
        writeln!(out, "        owner_id: i64,").unwrap();
        writeln!(out, "        field: &'static str,").unwrap();
        writeln!(out, "        values: HashMap<String, String>,").unwrap();
        writeln!(
            out,
            "    ) -> core_db::common::model_api::BoxModelFuture<'db, ()> {{"
        )
        .unwrap();
        writeln!(
            out,
            "        Box::pin(async move {{ localized::upsert_localized_many(db, owner_type, owner_id, field, &values).await }})"
        )
        .unwrap();
        writeln!(out, "    }}").unwrap();
    }

    if has_meta {
        writeln!(
            out,
            "    fn meta_owner_type() -> Option<&'static str> {{"
        )
        .unwrap();
        writeln!(
            out,
            "        Some(localized::{model_snake_upper}_OWNER_TYPE)"
        )
        .unwrap();
        writeln!(out, "    }}").unwrap();
        writeln!(
            out,
            "    fn upsert_meta_many<'db>("
        )
        .unwrap();
        writeln!(out, "        db: DbConn<'db>,").unwrap();
        writeln!(out, "        owner_type: &'static str,").unwrap();
        writeln!(out, "        owner_id: i64,").unwrap();
        writeln!(out, "        values: HashMap<String, serde_json::Value>,").unwrap();
        writeln!(
            out,
            "    ) -> core_db::common::model_api::BoxModelFuture<'db, ()> {{"
        )
        .unwrap();
        writeln!(
            out,
            "        Box::pin(async move {{ localized::upsert_meta_many(db, owner_type, owner_id, &values).await }})"
        )
        .unwrap();
        writeln!(out, "    }}").unwrap();
    }

    if has_attachments {
        writeln!(
            out,
            "    fn attachment_owner_type() -> Option<&'static str> {{"
        )
        .unwrap();
        writeln!(
            out,
            "        Some(localized::{model_snake_upper}_OWNER_TYPE)"
        )
        .unwrap();
        writeln!(out, "    }}").unwrap();
        writeln!(
            out,
            "    fn clear_attachment_field<'db>("
        )
        .unwrap();
        writeln!(out, "        db: DbConn<'db>,").unwrap();
        writeln!(out, "        owner_type: &'static str,").unwrap();
        writeln!(out, "        owner_id: i64,").unwrap();
        writeln!(out, "        field: &'static str,").unwrap();
        writeln!(
            out,
            "    ) -> core_db::common::model_api::BoxModelFuture<'db, ()> {{"
        )
        .unwrap();
        writeln!(
            out,
            "        Box::pin(async move {{ localized::clear_attachment_field(db, owner_type, owner_id, field).await }})"
        )
        .unwrap();
        writeln!(out, "    }}").unwrap();
        writeln!(
            out,
            "    fn replace_single_attachment<'db>("
        )
        .unwrap();
        writeln!(out, "        db: DbConn<'db>,").unwrap();
        writeln!(out, "        owner_type: &'static str,").unwrap();
        writeln!(out, "        owner_id: i64,").unwrap();
        writeln!(out, "        field: &'static str,").unwrap();
        writeln!(out, "        value: AttachmentInput,").unwrap();
        writeln!(
            out,
            "    ) -> core_db::common::model_api::BoxModelFuture<'db, ()> {{"
        )
        .unwrap();
        writeln!(
            out,
            "        Box::pin(async move {{ localized::replace_single_attachment(db, owner_type, owner_id, field, &value).await }})"
        )
        .unwrap();
        writeln!(out, "    }}").unwrap();
        writeln!(
            out,
            "    fn add_attachments<'db>("
        )
        .unwrap();
        writeln!(out, "        db: DbConn<'db>,").unwrap();
        writeln!(out, "        owner_type: &'static str,").unwrap();
        writeln!(out, "        owner_id: i64,").unwrap();
        writeln!(out, "        field: &'static str,").unwrap();
        writeln!(out, "        values: Vec<AttachmentInput>,").unwrap();
        writeln!(
            out,
            "    ) -> core_db::common::model_api::BoxModelFuture<'db, ()> {{"
        )
        .unwrap();
        writeln!(
            out,
            "        Box::pin(async move {{ localized::add_attachments(db, owner_type, owner_id, field, &values).await }})"
        )
        .unwrap();
        writeln!(out, "    }}").unwrap();
        writeln!(
            out,
            "    fn delete_attachment_ids<'db>("
        )
        .unwrap();
        writeln!(out, "        db: DbConn<'db>,").unwrap();
        writeln!(out, "        owner_type: &'static str,").unwrap();
        writeln!(out, "        owner_id: i64,").unwrap();
        writeln!(out, "        field: &'static str,").unwrap();
        writeln!(out, "        ids: Vec<uuid::Uuid>,").unwrap();
        writeln!(
            out,
            "    ) -> core_db::common::model_api::BoxModelFuture<'db, ()> {{"
        )
        .unwrap();
        writeln!(
            out,
            "        Box::pin(async move {{ localized::delete_attachment_ids(db, owner_type, owner_id, field, &ids).await }})"
        )
        .unwrap();
        writeln!(out, "    }}").unwrap();
    }

    if !touch_targets.is_empty() {
        writeln!(
            out,
            "    fn persist_create_related<'db>("
        )
        .unwrap();
        writeln!(out, "        db: DbConn<'db>,").unwrap();
        writeln!(
            out,
            "        row: <Self as core_db::common::model_api::RuntimeModel>::Row,"
        )
        .unwrap();
        writeln!(
            out,
            "    ) -> core_db::common::model_api::BoxModelFuture<'db, ()> {{"
        )
        .unwrap();
        writeln!(out, "        Box::pin(async move {{").unwrap();
        for (fk, target_snake, target_title, target_pk, _target_pk_ty) in touch_targets {
            writeln!(out, "            if let Some(parent_id) = row.{fk} {{").unwrap();
            writeln!(
                out,
                "                crate::generated::models::{}::{}::query()",
                target_snake, target_title
            )
            .unwrap();
            writeln!(
                out,
                "                    .where_col(crate::generated::models::{}::{}Col::{}, Op::Eq, parent_id)",
                target_snake,
                target_title,
                to_snake(target_pk).to_uppercase()
            )
            .unwrap();
            writeln!(out, "                    .patch()").unwrap();
            writeln!(
                out,
                "                    .assign(crate::generated::models::{}::{}Col::UPDATED_AT, time::OffsetDateTime::now_utc())?",
                target_snake, target_title
            )
            .unwrap();
            writeln!(out, "                    .save(db.clone()).await?;").unwrap();
            writeln!(out, "            }}").unwrap();
        }
        writeln!(out, "            Ok(())").unwrap();
        writeln!(out, "        }})").unwrap();
        writeln!(out, "    }}").unwrap();

        writeln!(
            out,
            "    fn persist_patch_related<'db>("
        )
        .unwrap();
        writeln!(out, "        db: DbConn<'db>,").unwrap();
        writeln!(out, "        target_ids: Vec<Self::Pk>,").unwrap();
        writeln!(
            out,
            "    ) -> core_db::common::model_api::BoxModelFuture<'db, ()> {{"
        )
        .unwrap();
        writeln!(out, "        Box::pin(async move {{").unwrap();
        writeln!(out, "            if target_ids.is_empty() {{").unwrap();
        writeln!(out, "                return Ok(());").unwrap();
        writeln!(out, "            }}").unwrap();
        writeln!(
            out,
            "            fn to_params(len: usize) -> String {{ (1..=len).map(|i| format!(\"${{}}\", i)).collect::<Vec<_>>().join(\", \") }}"
        )
        .unwrap();
        for (fk, target_snake, target_title, target_pk, target_pk_ty) in touch_targets {
            writeln!(out, "            let placeholders = to_params(target_ids.len());").unwrap();
            writeln!(
                out,
                "            let sql = format!(\"SELECT DISTINCT {} FROM {} WHERE {} IN ({{}})\", placeholders);",
                fk, table, pk
            )
            .unwrap();
            writeln!(
                out,
                "            let mut q = sqlx::query_scalar::<_, {target_pk_ty}>(&sql);"
            )
            .unwrap();
            writeln!(
                out,
                "            for id in target_ids {{ q = bind_scalar(q, id.clone()); }}"
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
                "                crate::generated::models::{}::{}::query()",
                target_snake, target_title
            )
            .unwrap();
            writeln!(
                out,
                "                    .where_col(crate::generated::models::{}::{}Col::{}, Op::Eq, pid)",
                target_snake,
                target_title,
                to_snake(target_pk).to_uppercase()
            )
            .unwrap();
            writeln!(out, "                    .patch()").unwrap();
            writeln!(
                out,
                "                    .assign(crate::generated::models::{}::{}Col::UPDATED_AT, time::OffsetDateTime::now_utc())?",
                target_snake, target_title
            )
            .unwrap();
            writeln!(out, "                    .save(db.clone()).await?;").unwrap();
            writeln!(out, "            }}").unwrap();
        }
        writeln!(out, "            Ok(())").unwrap();
        writeln!(out, "        }})").unwrap();
        writeln!(out, "    }}").unwrap();
    }

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
    let model_snake_upper = model_snake.to_uppercase();

    let table = cfg.table.as_deref().unwrap_or(&model_snake).to_string();
    let pk = cfg.pk.clone().unwrap_or_else(|| "id".to_string());

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
    writeln!(imports, "use std::collections::HashMap;").unwrap();
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
                    target_mod,
                    target_title,
                    target_title,
                    target_title,
                    target_title,
                    target_rel_import
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
                    target_mod,
                    target_title,
                    target_title,
                    target_title,
                    target_title,
                    target_rel_import
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
            RelationKind::BelongsTo | RelationKind::HasOne => {
                record_fields.push(format!(
                    "    pub {rel_field}: Option<Box<{target_record}>>,",
                ));
            }
        }
    }
    record_fields.push("    #[serde(skip)]".to_string());
    record_fields.push("    #[schemars(skip)]".to_string());
    record_fields
        .push("    pub __relation_counts: std::collections::HashMap<String, i64>,".to_string());
    record_fields.push("    #[serde(skip)]".to_string());
    record_fields.push("    #[schemars(skip)]".to_string());
    record_fields
        .push("    pub __relation_aggregates: std::collections::HashMap<String, f64>,".to_string());
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
        "    pub fn update<'db>(&self) -> Patch<'db, {model_title}Model> {{"
    )
    .unwrap();
    writeln!(
        out,
        "        {model_title}Model::query().where_col({col_ident}::{pk_variant}, Op::Eq, self.{pk}.clone()).patch()",
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
            RelationKind::BelongsTo | RelationKind::HasOne => {
                writeln!(out, "        {rel_field}: None,").unwrap();
            }
        }
    }
    writeln!(out, "        __relation_counts: HashMap::new(),").unwrap();
    writeln!(out, "        __relation_aggregates: HashMap::new(),").unwrap();
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

    writeln!(
        out,
        "impl core_db::common::model_api::RelationMetricRecord for {record_ident} {{"
    )
    .unwrap();
    writeln!(
        out,
        "    fn relation_counts(&self) -> &std::collections::HashMap<String, i64> {{"
    )
    .unwrap();
    writeln!(out, "        &self.__relation_counts").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(
        out,
        "    fn relation_aggregates(&self) -> &std::collections::HashMap<String, f64> {{"
    )
    .unwrap();
    writeln!(out, "        &self.__relation_aggregates").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(
        out,
        "    fn relation_counts_mut(&mut self) -> &mut std::collections::HashMap<String, i64> {{"
    )
    .unwrap();
    writeln!(out, "        &mut self.__relation_counts").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(out, "    fn relation_aggregates_mut(&mut self) -> &mut std::collections::HashMap<String, f64> {{").unwrap();
    writeln!(out, "        &mut self.__relation_aggregates").unwrap();
    writeln!(out, "    }}").unwrap();
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
        for (fn_name, kind) in [
            ("sum", "Sum"),
            ("avg", "Avg"),
            ("min", "Min"),
            ("max", "Max"),
        ] {
            writeln!(
                out,
                "    pub fn {fn_name}<R, A>(&self, relation: R, target: A) -> Option<f64>"
            )
            .unwrap();
            writeln!(out, "    where").unwrap();
            writeln!(
                out,
                "        R: core_db::common::model_api::CountRelation<{model_title}Model>,"
            )
            .unwrap();
            writeln!(
                out,
                "        A: Into<core_db::common::model_api::AggregateTarget<R::TargetModel>>,",
            )
            .unwrap();
            writeln!(out, "    {{").unwrap();
            writeln!(
                out,
                "        let key = core_db::common::model_api::relation_aggregate_lookup_key("
            )
            .unwrap();
            writeln!(out, "            R::name(relation),").unwrap();
            writeln!(
                out,
                "            core_db::common::model_api::RelationAggregateKind::{kind},"
            )
            .unwrap();
            writeln!(out, "            &target.into().into_spec(),").unwrap();
            writeln!(out, "        );").unwrap();
            writeln!(out, "        self.__relation_aggregates.get(&key).copied()").unwrap();
            writeln!(out, "    }}").unwrap();
        }
        writeln!(
            out,
            "    pub fn aggregate(&self, key: &str) -> Option<f64> {{"
        )
        .unwrap();
        writeln!(out, "        self.__relation_aggregates.get(key).copied()").unwrap();
        writeln!(out, "    }}").unwrap();
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
                RelationKind::BelongsTo | RelationKind::HasOne => {
                    format!("OneRelation<{model_title}Model, {target_record_ident}, {rel_idx}>")
                }
                RelationKind::HasMany => {
                    format!("ManyRelation<{model_title}Model, {target_record_ident}, {rel_idx}>")
                }
            };
            let rel_value = match rel.kind {
                RelationKind::BelongsTo | RelationKind::HasOne => format!(
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
        let rel_name = to_snake(&rel.name);
        let rel_upper = rel_name.to_uppercase();
        let runtime_ident = format!("REL_RUNTIME_{rel_upper}");
        let target_title = to_title_case(&rel.target_model);
        match rel.kind {
            RelationKind::HasMany => {
                let parent_key_fn = format!("rel_{rel_name}_parent_key");
                let child_key_fn = format!("rel_{rel_name}_child_key");
                let assign_fn = format!("rel_{rel_name}_assign");
                let target_fk_expr = if relation_target_field_is_optional(schema, rel) {
                    format!("record.{}.clone()", rel.foreign_key)
                } else {
                    format!("Some(record.{}.clone())", rel.foreign_key)
                };
                writeln!(out, "fn {parent_key_fn}(record: &{record_ident}) -> Option<{parent_pk_ty}> {{ Some(record.{pk}.clone()) }}").unwrap();
                writeln!(out, "fn {child_key_fn}(record: &{target_title}Record) -> Option<{parent_pk_ty}> {{ {target_fk_expr} }}").unwrap();
                writeln!(out, "fn {assign_fn}(record: &mut {record_ident}, children: Vec<{target_title}Record>) {{ record.{rel_name} = children; }}").unwrap();
                writeln!(out, "static {runtime_ident}: core_db::common::model_api::HasManyRuntime<{model_title}Model, {target_title}Model, {parent_pk_ty}> = core_db::common::model_api::HasManyRuntime {{").unwrap();
                writeln!(out, "    name: \"{rel_name}\",").unwrap();
                writeln!(out, "    target_table: \"{}\",", rel.target_table).unwrap();
                writeln!(out, "    target_pk: \"{}\",", rel.target_pk).unwrap();
                writeln!(out, "    foreign_key: \"{}\",", rel.foreign_key).unwrap();
                writeln!(out, "    parent_key: {parent_key_fn},").unwrap();
                writeln!(out, "    child_foreign_key: {child_key_fn},").unwrap();
                writeln!(out, "    assign: {assign_fn},").unwrap();
                writeln!(out, "}};\n").unwrap();
            }
            RelationKind::HasOne => {
                let parent_key_fn = format!("rel_{rel_name}_parent_key");
                let child_key_fn = format!("rel_{rel_name}_child_key");
                let assign_fn = format!("rel_{rel_name}_assign");
                let target_fk_expr = if relation_target_field_is_optional(schema, rel) {
                    format!("record.{}.clone()", rel.foreign_key)
                } else {
                    format!("Some(record.{}.clone())", rel.foreign_key)
                };
                writeln!(out, "fn {parent_key_fn}(record: &{record_ident}) -> Option<{parent_pk_ty}> {{ Some(record.{pk}.clone()) }}").unwrap();
                writeln!(out, "fn {child_key_fn}(record: &{target_title}Record) -> Option<{parent_pk_ty}> {{ {target_fk_expr} }}").unwrap();
                writeln!(out, "fn {assign_fn}(record: &mut {record_ident}, child: Option<{target_title}Record>) {{ record.{rel_name} = child.map(Box::new); }}").unwrap();
                writeln!(out, "static {runtime_ident}: core_db::common::model_api::HasOneRuntime<{model_title}Model, {target_title}Model, {parent_pk_ty}> = core_db::common::model_api::HasOneRuntime {{").unwrap();
                writeln!(out, "    name: \"{rel_name}\",").unwrap();
                writeln!(out, "    target_table: \"{}\",", rel.target_table).unwrap();
                writeln!(out, "    target_pk: \"{}\",", rel.target_pk).unwrap();
                writeln!(out, "    foreign_key: \"{}\",", rel.foreign_key).unwrap();
                writeln!(out, "    parent_key: {parent_key_fn},").unwrap();
                writeln!(out, "    child_foreign_key: {child_key_fn},").unwrap();
                writeln!(out, "    assign: {assign_fn},").unwrap();
                writeln!(out, "}};\n").unwrap();
            }
            RelationKind::BelongsTo => {
                let parent_key_fn = format!("rel_{rel_name}_parent_key");
                let target_key_fn = format!("rel_{rel_name}_target_key");
                let assign_fn = format!("rel_{rel_name}_assign");
                let parent_fk_expr = if fields
                    .iter()
                    .any(|f| f.name == rel.foreign_key && f.ty.starts_with("Option<"))
                {
                    format!("record.{}.clone()", rel.foreign_key)
                } else {
                    format!("Some(record.{}.clone())", rel.foreign_key)
                };
                writeln!(out, "fn {parent_key_fn}(record: &{record_ident}) -> Option<{}> {{ {parent_fk_expr} }}", rel.target_pk_ty).unwrap();
                writeln!(out, "fn {target_key_fn}(record: &{target_title}Record) -> {} {{ record.{}.clone() }}", rel.target_pk_ty, rel.target_pk).unwrap();
                writeln!(out, "fn {assign_fn}(record: &mut {record_ident}, child: Option<{target_title}Record>) {{ record.{rel_name} = child.map(Box::new); }}").unwrap();
                writeln!(out, "static {runtime_ident}: core_db::common::model_api::BelongsToRuntime<{model_title}Model, {target_title}Model, {}> = core_db::common::model_api::BelongsToRuntime {{", rel.target_pk_ty).unwrap();
                writeln!(out, "    name: \"{rel_name}\",").unwrap();
                writeln!(out, "    target_table: \"{}\",", rel.target_table).unwrap();
                writeln!(out, "    target_key_sql: \"{}\",", rel.target_pk).unwrap();
                writeln!(out, "    parent_foreign_key: {parent_key_fn},").unwrap();
                writeln!(out, "    target_key: {target_key_fn},").unwrap();
                writeln!(out, "    assign: {assign_fn},").unwrap();
                writeln!(out, "}};\n").unwrap();
            }
        }
    }

    writeln!(
        out,
        "impl core_db::common::model_api::RuntimeModel for {model_title}Model {{"
    )
    .unwrap();
    writeln!(out, "    type Row = {row_ident};").unwrap();
    writeln!(out, "    fn hydrate_records<'db>(db: DbConn<'db>, rows: Vec<Self::Row>, base_url: Option<String>) -> core_db::common::model_api::BoxModelFuture<'db, Vec<Self::Record>> {{").unwrap();
    writeln!(out, "        Box::pin(async move {{ crate::generated::models::{model_snake}::hydrate_records(db, &rows, base_url.as_deref()).await }})").unwrap();
    writeln!(out, "    }}").unwrap();
    if parent_pk_ty == "i64" {
        writeln!(
            out,
            "    fn record_pk_i64(record: &Self::Record) -> Option<i64> {{"
        )
        .unwrap();
        writeln!(out, "        Some(record.{pk}.clone())").unwrap();
        writeln!(out, "    }}").unwrap();
    }
    if relations.is_empty() {
        writeln!(out, "    fn relation_runtimes() -> &'static [&'static dyn core_db::common::model_api::ErasedRelationRuntime<Self>] {{").unwrap();
        writeln!(out, "        &[]").unwrap();
        writeln!(out, "    }}").unwrap();
    } else {
        writeln!(out, "    fn relation_runtimes() -> &'static [&'static dyn core_db::common::model_api::ErasedRelationRuntime<Self>] {{").unwrap();
        writeln!(out, "        static RELATIONS: [&'static dyn core_db::common::model_api::ErasedRelationRuntime<{model_title}Model>; {}] = [", relations.len()).unwrap();
        for rel in &relations {
            let runtime_ident = format!("REL_RUNTIME_{}", to_snake(&rel.name).to_uppercase());
            writeln!(out, "            &{runtime_ident},").unwrap();
        }
        writeln!(out, "        ];").unwrap();
        writeln!(out, "        &RELATIONS").unwrap();
        writeln!(out, "    }}").unwrap();
    }
    writeln!(out, "}}\n").unwrap();

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
    let insert_struct_section = String::new();
    let insert_builder_methods_section = String::new();
    let insert_save_methods_section = String::new();
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
    let update_struct_section = String::new();
    let update_builder_methods_section = String::new();
    let update_save_methods_section = String::new();
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

        writeln!(out, "pub struct {table_adapter_ident} {{").unwrap();
        writeln!(out, "    db: sqlx::PgPool,").unwrap();
        writeln!(out, "}}").unwrap();
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
            "        let db = self.db.clone(); Box::pin(async move {{ query.count(&db).await }})"
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
            "        let db = self.db.clone(); Box::pin(async move {{ Ok(query.paginate(&db, page, per_page).await?.data) }})"
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
        writeln!(out, "            db: db.clone(),").unwrap();
        writeln!(out, "            hooks: {default_data_table_hooks_ident},").unwrap();
        writeln!(
            out,
            "            config: {data_table_config_ident}::default(),"
        )
        .unwrap();
        writeln!(out, "            adapter: {table_adapter_ident} {{ db }},").unwrap();
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
        writeln!(out, "            db: self.db.clone(),").unwrap();
        writeln!(out, "            hooks,").unwrap();
        writeln!(out, "            config: self.config,").unwrap();
        writeln!(
            out,
            "            adapter: {table_adapter_ident} {{ db: self.db }},"
        )
        .unwrap();
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
            "        self.hooks.scope({model_title}Model::query(), input, ctx)"
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
        "    pub fn query<'db>() -> Query<'db, {model_title}Model> {{"
    )
    .unwrap();
    writeln!(out, "        Query::new()").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(
        out,
        "    pub fn query_with_base_url<'db>(base_url: Option<String>) -> Query<'db, {model_title}Model> {{"
    )
    .unwrap();
    writeln!(out, "        Query::new_with_base_url(base_url)").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(
        out,
        "    pub fn create<'db>() -> Create<'db, {model_title}Model> {{"
    )
    .unwrap();
    writeln!(out, "        Create::new()").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(
        out,
        "    pub fn create_with_base_url<'db>(base_url: Option<String>) -> Create<'db, {model_title}Model> {{"
    )
    .unwrap();
    writeln!(out, "        Create::new_with_base_url(base_url)").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(
        out,
        "    pub fn patch<'db>() -> Patch<'db, {model_title}Model> {{"
    )
    .unwrap();
    writeln!(out, "        Patch::new()").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(
        out,
        "    pub fn patch_with_base_url<'db>(base_url: Option<String>) -> Patch<'db, {model_title}Model> {{"
    )
    .unwrap();
    writeln!(out, "        Patch::new_with_base_url(base_url)").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(
        out,
        "    pub async fn find<'db>(db: impl Into<DbConn<'db>>, id: {parent_pk_ty}) -> Result<Option<{model_title}Record>> {{"
    )
    .unwrap();
    writeln!(
        out,
        "        Query::<{model_title}Model>::new().find(db, id).await"
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
    writeln!(out, "    const PROFILE_QUERIES: bool = {};", !skip_profiler).unwrap();
    writeln!(out, "    const OBSERVE_HOOKS: bool = {emit_hooks};").unwrap();
    writeln!(out, "}}\n").unwrap();
    writeln!(
        out,
        "impl core_db::common::model_api::ChunkModel for {model_title}Model {{"
    )
    .unwrap();
    writeln!(
        out,
        "    fn record_pk(record: &Self::Record) -> Self::Pk {{"
    )
    .unwrap();
    writeln!(out, "        record.{pk}.clone()").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(out, "}}\n").unwrap();
    writeln!(
        out,
        "impl core_db::common::model_api::DeleteModel for {model_title}Model {{"
    )
    .unwrap();
    writeln!(out, "    fn row_pk(row: &Self::Row) -> Self::Pk {{").unwrap();
    writeln!(out, "        row.{pk}.clone()").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(out, "    fn row_pk_text(row: &Self::Row) -> String {{").unwrap();
    writeln!(out, "        row.{pk}.to_string()").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(
        out,
        "    fn delete_override_update<'db>(db: DbConn<'db>, ids: Vec<Self::Pk>, overrides: serde_json::Value) -> core_db::common::model_api::BoxModelFuture<'db, u64> {{"
    )
    .unwrap();
    if emit_hooks {
        writeln!(
            out,
            "        Box::pin(async move {{ {model_title}Model::convert_delete_to_update(&db, &ids, overrides).await }})"
        )
        .unwrap();
    } else {
        writeln!(
            out,
            "        Box::pin(async move {{ let _ = (db, ids, overrides); anyhow::bail!(\"delete observer overrides are not enabled for {{}}\", <Self as core_db::common::model_api::ModelDef>::TABLE) }})"
        )
        .unwrap();
    }
    writeln!(out, "    }}").unwrap();
    writeln!(out, "}}\n").unwrap();
    out.push_str(&render_create_model_impl(
        &model_title,
        &pk,
        &table,
        &db_fields,
        &enum_specs,
        use_snowflake_id,
    ));
    out.push_str(&render_feature_persistence_model_impl(
        &model_title,
        &model_snake,
        &parent_pk_ty,
        &localized_fields,
        has_meta,
        has_attachments,
        &touch_targets,
        &table,
        &pk,
    ));
    out.push_str(&render_create_field_impl(
        &model_title,
        &col_ident,
        &db_fields,
    ));
    out.push_str(&render_create_conflict_field_impl(&model_title, &col_ident));
    out.push_str(&render_patch_model_impl(
        &model_title,
        &model_snake,
        &db_fields,
        &enum_specs,
        &localized_fields,
        has_meta,
        has_attachments,
        &touch_targets,
        &table,
        &pk,
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
            let rel_name = to_snake(&rel.name);
            let target_table = rel.target_table.clone();
            let target_pk = rel.target_pk.clone();
            let foreign_key = rel.foreign_key.clone();
            let target_model_title = to_title_case(&rel.target_model);
            let target_record_ident = format!("{target_model_title}Record");
            let target_soft_delete = schema
                .models
                .get(&rel.target_model)
                .map(|m| m.soft_delete)
                .unwrap_or(false);
            let local_key = rel.local_key.clone();
            let scope_query_expr = if let Some(scope_name) = rel.scope.as_ref() {
                format!(
                    "{target_model_title}Model::{scope_name}({target_model_title}Model::query_with_base_url(state.base_url.clone()))"
                )
            } else {
                format!("{target_model_title}Model::query_with_base_url(state.base_url.clone())")
            };
            let rel_ty = match rel.kind {
                RelationKind::BelongsTo | RelationKind::HasOne => {
                    format!("OneRelation<{model_title}Model, {target_record_ident}, {rel_idx}>")
                }
                RelationKind::HasMany => {
                    format!("ManyRelation<{model_title}Model, {target_record_ident}, {rel_idx}>")
                }
            };
            let rel_kind = match rel.kind {
                RelationKind::BelongsTo => "belongs_to",
                RelationKind::HasMany => "has_many",
                RelationKind::HasOne => "has_one",
            };
            writeln!(
                out,
                "impl core_db::common::model_api::IncludeRelation<{model_title}Model> for {rel_ty} {{"
            )
            .unwrap();
            if let Some(scope_name) = rel.scope.as_ref() {
                writeln!(
                    out,
                    "    fn load_spec<'db>(_relation: Self, base_url: Option<String>) -> core_db::common::model_api::WithRelationSpec {{"
                )
                .unwrap();
                writeln!(
                    out,
                    "        let scoped = {target_model_title}Model::{scope_name}({target_model_title}Model::query_with_base_url(base_url));"
                )
                .unwrap();
                writeln!(out, "        let inner = scoped.into_inner();").unwrap();
                writeln!(
                    out,
                    "        let selects = if inner.selects == core_db::common::model_api::parse_select_list(<{target_model_title}Model as core_db::common::model_api::QueryModel>::DEFAULT_SELECT) {{ Vec::new() }} else {{ inner.selects.clone() }};"
                )
                .unwrap();
                writeln!(
                    out,
                    "        core_db::common::model_api::WithRelationSpec {{ name: \"{rel_name}\", kind: \"{rel_kind}\", target_table: \"{target_table}\", target_pk: \"{target_pk}\", foreign_key: \"{foreign_key}\", local_key: \"{local_key}\", has_soft_delete: {target_soft_delete}, selects, filters: inner.filters, orders: inner.orders, limit: inner.limit, offset: inner.offset, with_deleted: inner.with_deleted, only_deleted: inner.only_deleted, nested: vec![], counts: inner.count_relations, aggregates: inner.aggregate_relations }}"
                )
                .unwrap();
                writeln!(out, "    }}").unwrap();
            } else {
                writeln!(
                    out,
                    "    fn load_spec<'db>(_relation: Self, _base_url: Option<String>) -> core_db::common::model_api::WithRelationSpec {{"
                )
                .unwrap();
                writeln!(
                    out,
                    "        core_db::common::model_api::WithRelationSpec {{ name: \"{rel_name}\", kind: \"{rel_kind}\", target_table: \"{target_table}\", target_pk: \"{target_pk}\", foreign_key: \"{foreign_key}\", local_key: \"{local_key}\", has_soft_delete: {target_soft_delete}, selects: vec![], filters: vec![], orders: vec![], limit: None, offset: None, with_deleted: false, only_deleted: false, nested: vec![], counts: vec![], aggregates: vec![] }}"
                )
                .unwrap();
                writeln!(out, "    }}").unwrap();
            }
            writeln!(out, "}}\n").unwrap();

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
            writeln!(out, "        let scoped = scope({scope_query_expr});").unwrap();
            writeln!(
                out,
                "        let relation_spec = <Self as core_db::common::model_api::IncludeRelation<{model_title}Model>>::load_spec(_relation, state.base_url.clone());"
            )
            .unwrap();
            writeln!(
                out,
                "        match core_db::common::model_api::relation_exists_from_query_state(relation_spec, scoped.into_inner(), <{target_model_title}Model as core_db::common::model_api::QueryModel>::DEFAULT_SELECT) {{"
            )
            .unwrap();
            writeln!(
                out,
                "            Ok(node) => state.push_relation_exists(core_db::common::model_api::ExistenceBoolean::And, node),"
            )
            .unwrap();
            writeln!(
                out,
                "            Err(err) => state.defer_error(err.to_string()),"
            )
            .unwrap();
            writeln!(out, "        }}").unwrap();
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
            writeln!(out, "        let scoped = scope({scope_query_expr});").unwrap();
            writeln!(
                out,
                "        let relation_spec = <Self as core_db::common::model_api::IncludeRelation<{model_title}Model>>::load_spec(_relation, state.base_url.clone());"
            )
            .unwrap();
            writeln!(
                out,
                "        match core_db::common::model_api::relation_exists_from_query_state(relation_spec, scoped.into_inner(), <{target_model_title}Model as core_db::common::model_api::QueryModel>::DEFAULT_SELECT) {{"
            )
            .unwrap();
            writeln!(
                out,
                "            Ok(node) => state.push_relation_exists(core_db::common::model_api::ExistenceBoolean::Or, node),"
            )
            .unwrap();
            writeln!(
                out,
                "            Err(err) => state.defer_error(err.to_string()),"
            )
            .unwrap();
            writeln!(out, "        }}").unwrap();
            writeln!(out, "    }}").unwrap();
            writeln!(out, "}}\n").unwrap();

            match rel.kind {
                RelationKind::BelongsTo | RelationKind::HasOne => {
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

                    if let Some(scope_name) = rel.scope.as_ref() {
                        writeln!(
                            out,
                            "impl core_db::common::model_api::CountRelation<{model_title}Model> for {rel_ty} {{"
                        )
                        .unwrap();
                        writeln!(out, "    type TargetModel = {target_model_title}Model;").unwrap();
                        writeln!(
                            out,
                            "    fn name(relation: Self) -> &'static str {{ relation.name() }}"
                        )
                        .unwrap();
                        writeln!(
                            out,
                            "    fn spec<'db>(_relation: Self, base_url: Option<String>) -> core_db::common::model_api::CountRelationSpec {{ let scoped = {target_model_title}Model::{scope_name}({target_model_title}Model::query_with_base_url(base_url)); let inner = scoped.into_inner(); core_db::common::model_api::CountRelationSpec {{ name: \"{rel_name}\", target_table: \"{target_table}\", target_pk: \"{target_pk}\", foreign_key: \"{foreign_key}\", has_soft_delete: {target_soft_delete}, filters: inner.filters, with_deleted: inner.with_deleted, only_deleted: inner.only_deleted }} }}"
                        )
                        .unwrap();
                        writeln!(out, "}}\n").unwrap();
                    } else {
                        writeln!(
                            out,
                            "impl core_db::common::model_api::CountRelation<{model_title}Model> for {rel_ty} {{"
                        )
                        .unwrap();
                        writeln!(out, "    type TargetModel = {target_model_title}Model;").unwrap();
                        writeln!(
                            out,
                            "    fn name(relation: Self) -> &'static str {{ relation.name() }}"
                        )
                        .unwrap();
                        writeln!(
                            out,
                            "    fn spec<'db>(_relation: Self, _base_url: Option<String>) -> core_db::common::model_api::CountRelationSpec {{ core_db::common::model_api::CountRelationSpec {{ name: \"{rel_name}\", target_table: \"{target_table}\", target_pk: \"{target_pk}\", foreign_key: \"{foreign_key}\", has_soft_delete: {target_soft_delete}, filters: vec![], with_deleted: false, only_deleted: false }} }}"
                        )
                        .unwrap();
                        writeln!(out, "}}\n").unwrap();
                    }
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
