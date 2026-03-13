use anyhow::{anyhow, bail, Context};
use quote::ToTokens;
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::{
    Attribute, Expr, ExprLit, Fields, GenericArgument, ImplItem, ImplItemFn, Item, ItemEnum,
    ItemImpl, ItemStruct, Lit, Meta, ReturnType, Token, Type,
};

#[derive(Debug, Default, Clone)]
pub struct Schema {
    pub models: BTreeMap<String, ModelSpec>,
    pub extra_sections: BTreeMap<String, EnumOrOther>,
}

#[derive(Debug, Clone)]
pub struct ModelSpec {
    pub table: Option<String>,
    pub pk: Option<String>,
    pub pk_type: Option<String>,
    /// Primary key generation strategy.
    /// Supported values:
    /// - "snowflake" (default for i64 pk)
    /// - "manual" (caller sets id explicitly)
    pub id_strategy: Option<String>,
    pub localized: Option<Vec<String>>,
    pub meta: Option<Vec<String>>,
    pub attachment: Option<Vec<String>>,
    pub attachments: Option<Vec<String>>,
    pub fields: Option<Vec<String>>,
    pub relations: Option<Vec<String>>,
    pub computed: Option<Vec<String>>,
    pub touch: Option<Vec<String>>,
    pub hidden: Option<Vec<String>>,
    pub soft_delete: bool,
    #[allow(dead_code)]
    pub disable_id: bool,
    #[allow(dead_code)]
    pub disable_timestamps: bool,
    /// Whether lifecycle observer hooks are generated for this model.
    /// Default: true — set `observe = false` to opt out.
    pub observe: bool,
    /// Whether SQL queries on this model are included in profiler output.
    /// Default: true — set `profile = false` to exclude (e.g. profiler tables).
    pub profile: bool,
    pub helper_items: Vec<String>,
    pub view_impl_items: Vec<String>,
    pub with_relations_impl_items: Vec<String>,
}

impl Default for ModelSpec {
    fn default() -> Self {
        Self {
            table: None,
            pk: None,
            pk_type: None,
            id_strategy: None,
            localized: None,
            meta: None,
            attachment: None,
            attachments: None,
            fields: None,
            relations: None,
            computed: None,
            touch: None,
            hidden: None,
            soft_delete: false,
            disable_id: false,
            disable_timestamps: false,
            observe: true,
            profile: true,
            helper_items: Vec::new(),
            view_impl_items: Vec::new(),
            with_relations_impl_items: Vec::new(),
        }
    }
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

#[derive(Debug, Clone)]
pub enum EnumOrOther {
    Enum(EnumSpec),
    Other(serde_json::Value),
}

#[derive(Debug, Clone)]
pub struct EnumSpec {
    pub type_name: String,
    pub storage: String,
    pub variants: EnumVariants,
}

#[derive(Debug, Clone)]
pub enum EnumVariants {
    Simple(Vec<String>),
    Explicit(Vec<EnumVariant>),
}

#[derive(Debug, Clone)]
pub struct EnumVariant {
    pub name: String,
    pub value: serde_json::Value,
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
    Decimal,
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

#[derive(Debug, Clone)]
enum ParsedFieldKind {
    Db,
    Localized,
    Meta(String),
    Attachment {
        kind: String,
        multiple: bool,
    },
    Relation {
        kind: RelationKind,
        target_model: String,
        foreign_key: String,
        local_key: Option<String>,
        touch: bool,
    },
}

#[derive(Debug, Clone, Default)]
struct ParsedFieldOptions {
    pk: bool,
    pk_strategy: Option<String>,
    hashed: bool,
    hidden: bool,
    kind: Option<String>,
    foreign_key: Option<String>,
    local_key: Option<String>,
    touch: bool,
}

#[derive(Debug, Clone)]
struct ParsedField {
    name: String,
    ty: String,
    kind: ParsedFieldKind,
    options: ParsedFieldOptions,
}

#[derive(Debug, Clone)]
enum CustomImplTarget {
    View { model_key: String },
    WithRelations { model_key: String },
}

#[derive(Debug, Clone)]
struct ParsedCustomImpl {
    target: CustomImplTarget,
    items: Vec<String>,
    computed: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct FrameworkModelSource {
    pub name: String,
    pub content: String,
}

pub fn load(path_str: &str) -> Result<Schema, Box<dyn Error>> {
    load_inner(path_str).map_err(Into::into)
}

fn load_inner(path_str: &str) -> anyhow::Result<Schema> {
    let path = Path::new(path_str);
    if path.is_dir() {
        let mut master = Schema::default();
        let mut entries = fs::read_dir(path)?
            .collect::<Result<Vec<_>, std::io::Error>>()
            .with_context(|| format!("failed to read {}", path.display()))?;
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let p = entry.path();
            if p.is_file() && p.extension().and_then(|ext| ext.to_str()) == Some("rs") {
                let raw = fs::read_to_string(&p)
                    .with_context(|| format!("failed to read {}", p.display()))?;
                let partial = parse_model_source(&raw, &p)?;
                master = merge_schema_layers(master, partial, &p.display().to_string());
            }
        }
        Ok(master)
    } else {
        let raw = fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        parse_model_source(&raw, path)
    }
}

pub fn framework_model_source_paths_from_core_db() -> Vec<PathBuf> {
    let core_db_src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../core-db/src");
    vec![
        core_db_src.join("platform/attachments/model.rs"),
        core_db_src.join("platform/countries/model.rs"),
        core_db_src.join("platform/localized/model.rs"),
        core_db_src.join("platform/meta/model.rs"),
        core_db_src.join("platform/sql_profiler/request_model.rs"),
        core_db_src.join("platform/sql_profiler/query_model.rs"),
        core_db_src.join("framework_models/failed_job.rs"),
        core_db_src.join("framework_models/outbox_job.rs"),
        core_db_src.join("framework_models/personal_access_token.rs"),
        core_db_src.join("framework_models/webhook_log.rs"),
        core_db_src.join("framework_models/http_client_log.rs"),
    ]
}

pub fn load_framework() -> Result<Schema, Box<dyn Error>> {
    let framework_paths = framework_model_source_paths_from_core_db();
    load_framework_from_paths(&framework_paths)
}

pub fn load_framework_from_paths(paths: &[PathBuf]) -> Result<Schema, Box<dyn Error>> {
    load_schema_layers_from_paths(paths).map_err(Into::into)
}

pub fn load_framework_from_sources(
    sources: &[FrameworkModelSource],
) -> Result<Schema, Box<dyn Error>> {
    load_schema_layers_from_sources(sources).map_err(Into::into)
}

fn load_schema_layers_from_paths(paths: &[PathBuf]) -> anyhow::Result<Schema> {
    let mut master = Schema::default();
    for path in paths {
        let raw = fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let partial = parse_model_source(&raw, path).with_context(|| {
            format!(
                "failed to parse framework model source '{}'",
                path.display()
            )
        })?;
        master = merge_schema_layers(master, partial, &path.display().to_string());
    }
    Ok(master)
}

fn load_schema_layers_from_sources(sources: &[FrameworkModelSource]) -> anyhow::Result<Schema> {
    let mut master = Schema::default();
    for source in sources {
        let partial = parse_model_source(&source.content, Path::new(&source.name))
            .with_context(|| format!("failed to parse framework model source '{}'", source.name))?;
        master = merge_schema_layers(master, partial, &source.name);
    }
    Ok(master)
}

fn merge_schema_layers(base: Schema, overlay: Schema, overlay_source: &str) -> Schema {
    let mut merged = base;

    for (model_name, model) in overlay.models {
        if merged.models.contains_key(&model_name) {
            panic!(
                "Duplicate model definition '{}' found in layer '{}'",
                model_name, overlay_source
            );
        }
        merged.models.insert(model_name, model);
    }

    for (enum_name, enum_spec) in overlay.extra_sections {
        if merged.extra_sections.contains_key(&enum_name) {
            panic!(
                "Duplicate enum definition '{}' found in layer '{}'",
                enum_name, overlay_source
            );
        }
        merged.extra_sections.insert(enum_name, enum_spec);
    }

    merged
}

pub fn load_with_framework(app_model_path: &str) -> Result<Schema, Box<dyn Error>> {
    let framework_paths = framework_model_source_paths_from_core_db();
    load_with_framework_from_paths(app_model_path, &framework_paths)
}

pub fn load_with_framework_from_paths(
    app_model_path: &str,
    framework_paths: &[PathBuf],
) -> Result<Schema, Box<dyn Error>> {
    let framework = load_schema_layers_from_paths(framework_paths)?;
    let app = load_inner(app_model_path)?;
    Ok(merge_schema_layers(framework, app, app_model_path))
}

pub fn load_with_framework_from_sources(
    app_model_path: &str,
    framework_sources: &[FrameworkModelSource],
) -> Result<Schema, Box<dyn Error>> {
    let framework = load_schema_layers_from_sources(framework_sources)?;
    let app = load_inner(app_model_path)?;
    Ok(merge_schema_layers(framework, app, app_model_path))
}

fn parse_model_source(raw: &str, source: &Path) -> anyhow::Result<Schema> {
    let file = syn::parse_file(raw)
        .with_context(|| format!("failed to parse Rust model source {}", source.display()))?;

    let mut parsed_model: Option<(String, ModelSpec)> = None;
    let mut extra_sections = BTreeMap::new();
    let mut helper_items = Vec::new();
    let mut custom_impls = Vec::new();

    for item in file.items {
        match item {
            Item::Struct(item_struct) if has_attr(&item_struct.attrs, "rf_model") => {
                if parsed_model.is_some() {
                    bail!(
                        "file '{}' contains multiple #[rf_model] structs; keep one model per file",
                        source.display()
                    );
                }
                parsed_model = Some(parse_model_struct(&item_struct, source)?);
            }
            Item::Enum(item_enum) if has_attr(&item_enum.attrs, "rf_db_enum") => {
                let (enum_name, enum_spec) = parse_db_enum(&item_enum, source)?;
                if extra_sections
                    .insert(enum_name.clone(), EnumOrOther::Enum(enum_spec))
                    .is_some()
                {
                    bail!(
                        "duplicate enum definition '{}' in {}",
                        enum_name,
                        source.display()
                    );
                }
            }
            Item::Impl(item_impl) if has_attr(&item_impl.attrs, "rf_view_impl") => {
                custom_impls.push(parse_custom_impl(item_impl, source, CustomImplKind::View)?);
            }
            Item::Impl(item_impl) if has_attr(&item_impl.attrs, "rf_with_relations_impl") => {
                custom_impls.push(parse_custom_impl(
                    item_impl,
                    source,
                    CustomImplKind::WithRelations,
                )?);
            }
            other => {
                helper_items.push(other.into_token_stream().to_string());
            }
        }
    }

    let Some((model_key, mut model)) = parsed_model else {
        bail!(
            "file '{}' does not contain a #[rf_model] struct",
            source.display()
        );
    };

    let mut computed = Vec::new();
    for parsed in custom_impls {
        match parsed.target {
            CustomImplTarget::View { model_key: target } => {
                if target != model_key {
                    bail!(
                        "file '{}' contains #[rf_view_impl] for model '{}' but the file model is '{}'",
                        source.display(),
                        target,
                        model_key
                    );
                }
                model.view_impl_items.extend(parsed.items);
                computed.extend(parsed.computed);
            }
            CustomImplTarget::WithRelations { model_key: target } => {
                if target != model_key {
                    bail!(
                        "file '{}' contains #[rf_with_relations_impl] for model '{}' but the file model is '{}'",
                        source.display(),
                        target,
                        model_key
                    );
                }
                model.with_relations_impl_items.extend(parsed.items);
                if !parsed.computed.is_empty() {
                    bail!(
                        "#[rf_computed] methods are only supported inside #[rf_view_impl] blocks (file: {})",
                        source.display()
                    );
                }
            }
        }
    }

    if !helper_items.is_empty() {
        model.helper_items = helper_items;
    }
    if !computed.is_empty() {
        model.computed = Some(computed);
    }

    let mut models = BTreeMap::new();
    models.insert(model_key, model);

    Ok(Schema {
        models,
        extra_sections,
    })
}

#[derive(Debug, Clone, Copy)]
enum CustomImplKind {
    View,
    WithRelations,
}

fn parse_model_struct(item: &ItemStruct, source: &Path) -> anyhow::Result<(String, ModelSpec)> {
    let model_name = item.ident.to_string();
    let model_key = to_snake(&model_name);
    let model_attr = parse_model_attr(&item.attrs, source, &model_name)?;
    let fields_named = match &item.fields {
        Fields::Named(fields) => &fields.named,
        _ => bail!(
            "#[rf_model] struct '{}' in '{}' must use named fields",
            model_name,
            source.display()
        ),
    };

    let mut parsed_fields = Vec::new();
    for field in fields_named {
        let Some(ident) = &field.ident else {
            bail!(
                "model '{}' in '{}' contains an unnamed field",
                model_name,
                source.display()
            );
        };
        let name = ident.to_string();
        let options = parse_field_options(&field.attrs, source, &model_name, &name)?;
        let ty = type_to_compact_string(&field.ty);
        let kind =
            classify_field_kind(&field.ty, &name, &model_key, &options, source, &model_name)?;
        parsed_fields.push(ParsedField {
            name,
            ty,
            kind,
            options,
        });
    }

    let pk_field = resolve_pk_field(&parsed_fields, source, &model_name)?;
    let pk_name = pk_field.name.clone();
    let pk_type = pk_field.ty.clone();
    let id_strategy = pk_field.options.pk_strategy.clone();

    let mut model = ModelSpec {
        table: model_attr.table.or_else(|| Some(model_key.clone())),
        pk: Some(pk_name.clone()),
        pk_type: Some(pk_type),
        id_strategy,
        soft_delete: model_attr.soft_delete,
        observe: model_attr.observe,
        profile: model_attr.profile,
        ..ModelSpec::default()
    };

    let mut db_fields = Vec::new();
    let mut localized = Vec::new();
    let mut meta = Vec::new();
    let mut attachment = Vec::new();
    let mut attachments = Vec::new();
    let mut relations = Vec::new();
    let mut hidden = Vec::new();
    let mut touch = Vec::new();

    for field in parsed_fields {
        match &field.kind {
            ParsedFieldKind::Db => {
                if field.options.hashed {
                    db_fields.push(format!("{}:hashed", field.name));
                } else {
                    db_fields.push(format!("{}:{}", field.name, field.ty));
                }
            }
            ParsedFieldKind::Localized => localized.push(field.name.clone()),
            ParsedFieldKind::Meta(inner) => meta.push(format!("{}:{}", field.name, inner)),
            ParsedFieldKind::Attachment { kind, multiple } => {
                if *multiple {
                    attachments.push(format!("{}:{}", field.name, kind));
                } else {
                    attachment.push(format!("{}:{}", field.name, kind));
                }
            }
            ParsedFieldKind::Relation {
                kind,
                target_model,
                foreign_key,
                local_key,
                touch: should_touch,
            } => {
                let relation_kind = match kind {
                    RelationKind::BelongsTo => "belongs_to",
                    RelationKind::HasMany => "has_many",
                };
                let resolved_local_key = local_key.clone().unwrap_or_else(|| {
                    if matches!(kind, RelationKind::HasMany) {
                        pk_name.clone()
                    } else {
                        "id".to_string()
                    }
                });
                relations.push(format!(
                    "{}:{}:{}:{}:{}",
                    field.name, relation_kind, target_model, foreign_key, resolved_local_key
                ));
                if *should_touch {
                    touch.push(field.name.clone());
                }
            }
        }

        if field.options.hidden {
            hidden.push(match &field.kind {
                ParsedFieldKind::Meta(_) => "meta".to_string(),
                _ => field.name.clone(),
            });
        }
    }

    if !db_fields.is_empty() {
        model.fields = Some(db_fields);
    }
    if !localized.is_empty() {
        model.localized = Some(localized);
    }
    if !meta.is_empty() {
        model.meta = Some(meta);
    }
    if !attachment.is_empty() {
        model.attachment = Some(attachment);
    }
    if !attachments.is_empty() {
        model.attachments = Some(attachments);
    }
    if !relations.is_empty() {
        model.relations = Some(relations);
    }
    if !hidden.is_empty() {
        hidden.sort();
        hidden.dedup();
        model.hidden = Some(hidden);
    }
    if !touch.is_empty() {
        touch.sort();
        touch.dedup();
        model.touch = Some(touch);
    }

    Ok((model_key, model))
}

#[derive(Debug, Clone)]
struct ParsedModelAttr {
    table: Option<String>,
    soft_delete: bool,
    observe: bool,
    profile: bool,
}

fn parse_model_attr(
    attrs: &[Attribute],
    source: &Path,
    model_name: &str,
) -> anyhow::Result<ParsedModelAttr> {
    let mut out = ParsedModelAttr {
        table: None,
        soft_delete: false,
        observe: true,
        profile: true,
    };

    for attr in attrs.iter().filter(|attr| attr.path().is_ident("rf_model")) {
        for meta in parse_attr_meta_list(attr)? {
            match meta {
                Meta::Path(path) if path.is_ident("soft_delete") => out.soft_delete = true,
                Meta::NameValue(nv) if nv.path.is_ident("table") => {
                    out.table = Some(expr_to_string(&nv.value)?);
                }
                Meta::NameValue(nv) if nv.path.is_ident("observe") => {
                    out.observe = expr_to_bool(&nv.value)?;
                }
                Meta::NameValue(nv) if nv.path.is_ident("profile") => {
                    out.profile = expr_to_bool(&nv.value)?;
                }
                other => {
                    bail!(
                        "unsupported #[rf_model(...)] option '{}' on model '{}' in '{}'",
                        meta_path_name(&other),
                        model_name,
                        source.display()
                    );
                }
            }
        }
    }

    Ok(out)
}

fn resolve_pk_field<'a>(
    fields: &'a [ParsedField],
    source: &Path,
    model_name: &str,
) -> anyhow::Result<&'a ParsedField> {
    let marked = fields
        .iter()
        .filter(|field| field.options.pk)
        .collect::<Vec<_>>();
    if marked.len() > 1 {
        bail!(
            "model '{}' in '{}' has multiple #[rf(pk)] fields",
            model_name,
            source.display()
        );
    }
    if let Some(field) = marked.into_iter().next() {
        return Ok(field);
    }
    if let Some(field) = fields.iter().find(|field| field.name == "id") {
        return Ok(field);
    }

    bail!(
        "model '{}' in '{}' must define an 'id' field or mark a field with #[rf(pk(...))]",
        model_name,
        source.display()
    )
}

fn parse_field_options(
    attrs: &[Attribute],
    source: &Path,
    model_name: &str,
    field_name: &str,
) -> anyhow::Result<ParsedFieldOptions> {
    let mut out = ParsedFieldOptions::default();

    for attr in attrs.iter().filter(|attr| attr.path().is_ident("rf")) {
        for meta in parse_attr_meta_list(attr)? {
            match meta {
                Meta::Path(path) if path.is_ident("pk") => out.pk = true,
                Meta::Path(path) if path.is_ident("hashed") => out.hashed = true,
                Meta::Path(path) if path.is_ident("hidden") => out.hidden = true,
                Meta::Path(path) if path.is_ident("touch") => out.touch = true,
                Meta::NameValue(nv) if nv.path.is_ident("kind") => {
                    out.kind = Some(expr_to_string(&nv.value)?);
                }
                Meta::NameValue(nv) if nv.path.is_ident("foreign_key") => {
                    out.foreign_key = Some(expr_to_string(&nv.value)?);
                }
                Meta::NameValue(nv) if nv.path.is_ident("local_key") => {
                    out.local_key = Some(expr_to_string(&nv.value)?);
                }
                Meta::List(list) if list.path.is_ident("pk") => {
                    out.pk = true;
                    for nested in parse_meta_list_tokens(&list.tokens)? {
                        match nested {
                            Meta::NameValue(nv) if nv.path.is_ident("strategy") => {
                                out.pk_strategy = Some(expr_to_string(&nv.value)?);
                            }
                            other => {
                                bail!(
                                    "unsupported #[rf(pk(...))] option '{}' on field '{}.{}' in '{}'",
                                    meta_path_name(&other),
                                    model_name,
                                    field_name,
                                    source.display()
                                );
                            }
                        }
                    }
                }
                other => {
                    bail!(
                        "unsupported #[rf(...)] option '{}' on field '{}.{}' in '{}'",
                        meta_path_name(&other),
                        model_name,
                        field_name,
                        source.display()
                    );
                }
            }
        }
    }

    Ok(out)
}

fn classify_field_kind(
    ty: &Type,
    field_name: &str,
    model_key: &str,
    options: &ParsedFieldOptions,
    source: &Path,
    model_name: &str,
) -> anyhow::Result<ParsedFieldKind> {
    let Some(last_segment) = type_last_segment(ty) else {
        return Ok(ParsedFieldKind::Db);
    };

    match last_segment.ident.to_string().as_str() {
        "Localized" => Ok(ParsedFieldKind::Localized),
        "Meta" => {
            let inner = single_generic_type(last_segment, source, model_name, field_name)?;
            Ok(ParsedFieldKind::Meta(normalize_meta_type_name(&inner)))
        }
        "Attachment" => {
            let kind = options.kind.clone().ok_or_else(|| {
                anyhow!(
                    "field '{}.{}' in '{}' uses Attachment but is missing #[rf(kind = ...)]",
                    model_name,
                    field_name,
                    source.display()
                )
            })?;
            Ok(ParsedFieldKind::Attachment {
                kind,
                multiple: false,
            })
        }
        "Attachments" => {
            let kind = options.kind.clone().ok_or_else(|| {
                anyhow!(
                    "field '{}.{}' in '{}' uses Attachments but is missing #[rf(kind = ...)]",
                    model_name,
                    field_name,
                    source.display()
                )
            })?;
            Ok(ParsedFieldKind::Attachment {
                kind,
                multiple: true,
            })
        }
        "BelongsTo" => {
            let target_model = to_snake(&single_generic_type(
                last_segment,
                source,
                model_name,
                field_name,
            )?);
            Ok(ParsedFieldKind::Relation {
                kind: RelationKind::BelongsTo,
                target_model,
                foreign_key: options
                    .foreign_key
                    .clone()
                    .unwrap_or_else(|| format!("{}_id", field_name)),
                local_key: options.local_key.clone(),
                touch: options.touch,
            })
        }
        "HasMany" => {
            let target_model = to_snake(&single_generic_type(
                last_segment,
                source,
                model_name,
                field_name,
            )?);
            Ok(ParsedFieldKind::Relation {
                kind: RelationKind::HasMany,
                target_model,
                foreign_key: options
                    .foreign_key
                    .clone()
                    .unwrap_or_else(|| format!("{}_id", model_key)),
                local_key: options.local_key.clone(),
                touch: false,
            })
        }
        _ => Ok(ParsedFieldKind::Db),
    }
}

fn parse_db_enum(item: &ItemEnum, source: &Path) -> anyhow::Result<(String, EnumSpec)> {
    let mut storage = None;
    for attr in item
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("rf_db_enum"))
    {
        for meta in parse_attr_meta_list(attr)? {
            match meta {
                Meta::NameValue(nv) if nv.path.is_ident("storage") => {
                    storage = Some(expr_to_string(&nv.value)?);
                }
                other => {
                    bail!(
                        "unsupported #[rf_db_enum(...)] option '{}' on enum '{}' in '{}'",
                        meta_path_name(&other),
                        item.ident,
                        source.display()
                    );
                }
            }
        }
    }

    let storage = storage.ok_or_else(|| {
        anyhow!(
            "enum '{}' in '{}' is missing #[rf_db_enum(storage = ...)]",
            item.ident,
            source.display()
        )
    })?;

    let enum_name = item.ident.to_string();
    let variants = match storage.as_str() {
        "string" | "text" => parse_string_enum_variants(item, source)?,
        "i16" | "i32" | "i64" => parse_integer_enum_variants(item, source)?,
        other => bail!(
            "enum '{}' in '{}' uses unsupported storage '{}'",
            item.ident,
            source.display(),
            other
        ),
    };

    Ok((
        enum_name,
        EnumSpec {
            type_name: "enum".to_string(),
            storage,
            variants,
        },
    ))
}

fn parse_string_enum_variants(item: &ItemEnum, source: &Path) -> anyhow::Result<EnumVariants> {
    let mut explicit = Vec::new();
    let mut all_default = true;

    for variant in &item.variants {
        let explicit_value = parse_variant_value_attr(&variant.attrs)?;
        if variant.discriminant.is_some() {
            bail!(
                "string enum '{}' in '{}' cannot use numeric discriminants on variant '{}'",
                item.ident,
                source.display(),
                variant.ident
            );
        }

        let value = explicit_value.unwrap_or_else(|| variant.ident.to_string().to_lowercase());
        if value != variant.ident.to_string().to_lowercase() {
            all_default = false;
        }
        explicit.push(EnumVariant {
            name: variant.ident.to_string(),
            value: serde_json::Value::String(value),
        });
    }

    if all_default {
        Ok(EnumVariants::Simple(
            item.variants
                .iter()
                .map(|variant| variant.ident.to_string())
                .collect(),
        ))
    } else {
        Ok(EnumVariants::Explicit(explicit))
    }
}

fn parse_integer_enum_variants(item: &ItemEnum, source: &Path) -> anyhow::Result<EnumVariants> {
    let mut variants = Vec::new();
    for variant in &item.variants {
        let Some((_, expr)) = &variant.discriminant else {
            bail!(
                "integer enum '{}' in '{}' variant '{}' must use an explicit discriminant",
                item.ident,
                source.display(),
                variant.ident
            );
        };
        let value = expr_to_i64(expr)?;
        variants.push(EnumVariant {
            name: variant.ident.to_string(),
            value: serde_json::Value::Number(serde_json::Number::from(value)),
        });
    }
    Ok(EnumVariants::Explicit(variants))
}

fn parse_variant_value_attr(attrs: &[Attribute]) -> anyhow::Result<Option<String>> {
    for attr in attrs.iter().filter(|attr| attr.path().is_ident("rf")) {
        for meta in parse_attr_meta_list(attr)? {
            match meta {
                Meta::NameValue(nv) if nv.path.is_ident("value") => {
                    return Ok(Some(expr_to_string(&nv.value)?));
                }
                _ => {}
            }
        }
    }
    Ok(None)
}

fn parse_custom_impl(
    mut item_impl: ItemImpl,
    source: &Path,
    kind: CustomImplKind,
) -> anyhow::Result<ParsedCustomImpl> {
    item_impl.attrs = strip_named_attrs(
        &item_impl.attrs,
        &[match kind {
            CustomImplKind::View => "rf_view_impl",
            CustomImplKind::WithRelations => "rf_with_relations_impl",
        }],
    );

    let target_ident = self_type_ident(&item_impl.self_ty).ok_or_else(|| {
        anyhow!(
            "custom impl in '{}' must target a simple type path",
            source.display()
        )
    })?;

    let target = match kind {
        CustomImplKind::View => {
            let Some(base) = target_ident.strip_suffix("View") else {
                bail!(
                    "#[rf_view_impl] in '{}' must target XxxView, found '{}'",
                    source.display(),
                    target_ident
                );
            };
            CustomImplTarget::View {
                model_key: to_snake(base),
            }
        }
        CustomImplKind::WithRelations => {
            let Some(base) = target_ident.strip_suffix("WithRelations") else {
                bail!(
                    "#[rf_with_relations_impl] in '{}' must target XxxWithRelations, found '{}'",
                    source.display(),
                    target_ident
                );
            };
            CustomImplTarget::WithRelations {
                model_key: to_snake(base),
            }
        }
    };

    let mut items = Vec::new();
    let mut computed = Vec::new();

    for impl_item in item_impl.items {
        match impl_item {
            ImplItem::Fn(mut method) => {
                let is_computed = has_attr(&method.attrs, "rf_computed");
                method.attrs = strip_named_attrs(&method.attrs, &["rf_computed"]);
                if is_computed {
                    validate_computed_method(&method, source)?;
                    let ty = match &method.sig.output {
                        ReturnType::Type(_, ty) => type_to_compact_string(ty),
                        ReturnType::Default => unreachable!(),
                    };
                    computed.push(format!("{}:{}", method.sig.ident, ty));
                }
                items.push(method.into_token_stream().to_string());
            }
            other => {
                bail!(
                    "unsupported item '{}' inside custom impl in '{}'; only methods are supported",
                    impl_item_name(&other),
                    source.display()
                );
            }
        }
    }

    Ok(ParsedCustomImpl {
        target,
        items,
        computed,
    })
}

fn validate_computed_method(method: &ImplItemFn, source: &Path) -> anyhow::Result<()> {
    if method.sig.receiver().is_none() {
        bail!(
            "#[rf_computed] method '{}' in '{}' must take &self or self",
            method.sig.ident,
            source.display()
        );
    }
    if method.sig.inputs.len() != 1 {
        bail!(
            "#[rf_computed] method '{}' in '{}' must not take extra arguments",
            method.sig.ident,
            source.display()
        );
    }
    if matches!(method.sig.output, ReturnType::Default) {
        bail!(
            "#[rf_computed] method '{}' in '{}' must declare a return type",
            method.sig.ident,
            source.display()
        );
    }
    Ok(())
}

fn parse_attr_meta_list(attr: &Attribute) -> anyhow::Result<Vec<Meta>> {
    match &attr.meta {
        Meta::Path(_) => Ok(Vec::new()),
        Meta::List(list) => parse_meta_list_tokens(&list.tokens),
        Meta::NameValue(_) => bail!(
            "attribute '{}' must use list syntax",
            attr.path()
                .segments
                .last()
                .map(|segment| segment.ident.to_string())
                .unwrap_or_else(|| "<unknown>".to_string())
        ),
    }
}

fn parse_meta_list_tokens(tokens: &proc_macro2::TokenStream) -> anyhow::Result<Vec<Meta>> {
    Ok(Punctuated::<Meta, Token![,]>::parse_terminated
        .parse2(tokens.clone())?
        .into_iter()
        .collect())
}

fn has_attr(attrs: &[Attribute], name: &str) -> bool {
    attrs.iter().any(|attr| attr.path().is_ident(name))
}

fn strip_named_attrs(attrs: &[Attribute], names: &[&str]) -> Vec<Attribute> {
    attrs
        .iter()
        .filter(|attr| !names.iter().any(|name| attr.path().is_ident(name)))
        .cloned()
        .collect()
}

fn meta_path_name(meta: &Meta) -> String {
    match meta {
        Meta::Path(path) => path
            .segments
            .last()
            .map(|segment| segment.ident.to_string())
            .unwrap_or_else(|| "<unknown>".to_string()),
        Meta::List(list) => list
            .path
            .segments
            .last()
            .map(|segment| segment.ident.to_string())
            .unwrap_or_else(|| "<unknown>".to_string()),
        Meta::NameValue(nv) => nv
            .path
            .segments
            .last()
            .map(|segment| segment.ident.to_string())
            .unwrap_or_else(|| "<unknown>".to_string()),
    }
}

fn expr_to_string(expr: &Expr) -> anyhow::Result<String> {
    match expr {
        Expr::Lit(ExprLit {
            lit: Lit::Str(value),
            ..
        }) => Ok(value.value()),
        Expr::Lit(ExprLit {
            lit: Lit::Int(value),
            ..
        }) => Ok(value.base10_digits().to_string()),
        Expr::Lit(ExprLit {
            lit: Lit::Bool(value),
            ..
        }) => Ok(value.value().to_string()),
        Expr::Path(path) => Ok(path
            .path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect::<Vec<_>>()
            .join("::")),
        other => bail!(
            "unsupported attribute value '{}'; use a string, bool, integer, or simple path",
            other.to_token_stream()
        ),
    }
}

fn expr_to_bool(expr: &Expr) -> anyhow::Result<bool> {
    match expr {
        Expr::Lit(ExprLit {
            lit: Lit::Bool(value),
            ..
        }) => Ok(value.value()),
        other => bail!(
            "expected boolean value, found '{}'",
            other.to_token_stream()
        ),
    }
}

fn expr_to_i64(expr: &Expr) -> anyhow::Result<i64> {
    match expr {
        Expr::Lit(ExprLit {
            lit: Lit::Int(value),
            ..
        }) => value.base10_parse::<i64>().map_err(Into::into),
        other => bail!(
            "expected integer literal, found '{}'",
            other.to_token_stream()
        ),
    }
}

fn type_last_segment(ty: &Type) -> Option<&syn::PathSegment> {
    match ty {
        Type::Path(type_path) if type_path.qself.is_none() => type_path.path.segments.last(),
        _ => None,
    }
}

fn single_generic_type(
    segment: &syn::PathSegment,
    source: &Path,
    model_name: &str,
    field_name: &str,
) -> anyhow::Result<String> {
    let syn::PathArguments::AngleBracketed(args) = &segment.arguments else {
        bail!(
            "field '{}.{}' in '{}' must provide a generic type argument",
            model_name,
            field_name,
            source.display()
        );
    };
    let mut type_args = args
        .args
        .iter()
        .filter_map(|arg| match arg {
            GenericArgument::Type(ty) => Some(type_to_compact_string(ty)),
            _ => None,
        })
        .collect::<Vec<_>>();
    if type_args.len() != 1 {
        bail!(
            "field '{}.{}' in '{}' must provide exactly one generic type argument",
            model_name,
            field_name,
            source.display()
        );
    }
    Ok(type_args.remove(0))
}

fn self_type_ident(ty: &Type) -> Option<String> {
    match ty {
        Type::Path(type_path) if type_path.qself.is_none() => type_path
            .path
            .segments
            .last()
            .map(|segment| segment.ident.to_string()),
        _ => None,
    }
}

fn impl_item_name(item: &ImplItem) -> &'static str {
    match item {
        ImplItem::Const(_) => "const",
        ImplItem::Fn(_) => "fn",
        ImplItem::Type(_) => "type",
        ImplItem::Macro(_) => "macro",
        ImplItem::Verbatim(_) => "verbatim",
        _ => "item",
    }
}

fn normalize_meta_type_name(raw: &str) -> String {
    match raw {
        "String" => "String".to_string(),
        "bool" => "bool".to_string(),
        "i32" => "i32".to_string(),
        "i64" => "i64".to_string(),
        "f64" => "f64".to_string(),
        "rust_decimal::Decimal" => "decimal".to_string(),
        "serde_json::Value" => "json".to_string(),
        "time::OffsetDateTime" => "datetime".to_string(),
        other => other.to_string(),
    }
}

fn type_to_compact_string(ty: &Type) -> String {
    compact_token_string(&ty.to_token_stream().to_string())
}

fn compact_token_string(raw: &str) -> String {
    raw.chars().filter(|ch| !ch.is_whitespace()).collect()
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
            "decimal" => MetaType::Decimal,
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

    let compact = compact_token_string(ty_raw);
    let (ty, serde_attr, special_type) = match compact.as_str() {
        "datetime" | "time::OffsetDateTime" => (
            "time::OffsetDateTime".to_string(),
            Some("#[serde(with = \"time::serde::rfc3339\")]"),
            None,
        ),
        "Option<datetime>" | "Option<time::OffsetDateTime>" => (
            "Option<time::OffsetDateTime>".to_string(),
            Some("#[serde(with = \"time::serde::rfc3339::option\")]"),
            None,
        ),
        "uuid" | "Uuid" | "uuid::Uuid" => ("uuid::Uuid".to_string(), None, None),
        "hashed" => ("String".to_string(), None, Some(SpecialType::Hashed)),
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
    let trimmed = compact_token_string(raw);

    if let Some(inner) = trimmed
        .strip_prefix("Option<")
        .and_then(|s| s.strip_suffix('>'))
    {
        return format!("Option<{}>", normalize_type(inner));
    }

    match trimmed.as_str() {
        "datetime" => "time::OffsetDateTime".to_string(),
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
