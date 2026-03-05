//! Exports Rust contract types to TypeScript.
//!
//! Uses `ts-rs` to convert Rust types with `#[derive(TS)]` into TypeScript
//! definitions, then writes them to `frontend/src/` alongside framework
//! shared types (ApiResponse, DataTable*, platform primitives, enums).
//!
//! Run: `cargo run -p app --bin export-types`
//! Or:  `make gen-types`

use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use ts_rs::TS;

// ── Generated types (ts-rs) ──────────────────────────────────

/// A generated TypeScript file: imports + ts-rs type definitions.
struct TsFile {
    /// Relative path from `frontend/src/`, e.g. `admin/types/admin.ts`
    rel_path: String,
    /// TypeScript definitions produced by ts-rs (collected at runtime).
    definitions: Vec<String>,
    /// Contract-facing enums referenced by the definitions.
    enums: BTreeSet<String>,
}

#[derive(Clone, Copy)]
struct AutoTsType {
    rel_path: &'static str,
    rust_path: &'static str,
    export: fn() -> String,
}

include!(concat!(env!("OUT_DIR"), "/export_types_registry.rs"));

fn main() {
    let base = Path::new("frontend/src");

    // ── 1. Contract types via ts-rs ─────────────────────────
    let mut files = load_discovered_ts_files();
    let contract_enum_renderers = collect_contract_enum_renderers();
    let known_contract_enum_names: BTreeSet<String> =
        contract_enum_renderers.keys().cloned().collect();

    // Determine which contract-facing enums are actually referenced by DTOs.
    let known_contract_types = collect_declared_contract_types(&files);
    for ts_file in &mut files {
        ts_file.enums = detect_enum_references(
            ts_file.rel_path.as_str(),
            &ts_file.definitions,
            &known_contract_types,
            &known_contract_enum_names,
        );
    }

    // Write ts-rs generated files
    for ts_file in &files {
        let path = base.join(&ts_file.rel_path);
        write_file(&path, &assemble(ts_file));
    }

    // ── 2. Enum types (serde-derived) ────────────────────────
    let required_enums_by_portal = collect_required_enums_by_portal(&files);
    let required_enums_shared = collect_required_enums(&required_enums_by_portal);
    let shared_enums_ts = assemble_enums_file(&required_enums_shared, &contract_enum_renderers);
    write_file(&base.join("shared/types/enums.ts"), &shared_enums_ts);

    let all_portals = collect_all_portals(&files, &required_enums_by_portal);
    for portal in &all_portals {
        let enums = required_enums_by_portal
            .get(portal.as_str())
            .cloned()
            .unwrap_or_default();
        let enums_ts = assemble_enums_file(&enums, &contract_enum_renderers);
        write_file(&base.join(format!("{portal}/types/enums.ts")), &enums_ts);
    }

    // ── 3. Static framework types (not derived from Rust structs) ──
    //
    // These mirror core-web types that don't live in the app crate.
    // The scaffold also writes identical initial copies; this binary
    // overwrites them to keep everything in sync after contract changes.
    write_file(
        &base.join("shared/types/platform.ts"),
        &render_shared_platform_ts(),
    );

    for (portal, content) in assemble_portal_indexes(&all_portals, &files) {
        write_file(&base.join(format!("{portal}/types/index.ts")), &content);
    }
    write_file(&base.join("shared/types/api.ts"), SHARED_API_TS);
    write_file(&base.join("shared/types/datatable.ts"), SHARED_DATATABLE_TS);
    write_file(&base.join("shared/types/index.ts"), SHARED_INDEX_TS);

    println!("\nTypeScript types regenerated in frontend/src/");
}

// ── Helpers ──────────────────────────────────────────────────

fn load_discovered_ts_files() -> Vec<TsFile> {
    let mut grouped: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();

    for discovered in discovered_ts_types() {
        grouped
            .entry(discovered.rel_path.to_string())
            .or_default()
            .push((discovered.rust_path.to_string(), (discovered.export)()));
    }

    let mut files = Vec::new();
    for (rel_path, mut exports) in grouped {
        exports.sort_by(|a, b| a.0.cmp(&b.0));
        let definitions = exports.into_iter().map(|(_, def)| def).collect();
        files.push(TsFile {
            rel_path,
            definitions,
            enums: BTreeSet::new(),
        });
    }
    files.sort_by(|a, b| a.rel_path.cmp(&b.rel_path));
    files
}

fn collect_all_portals(
    files: &[TsFile],
    required_enums_by_portal: &BTreeMap<String, BTreeSet<String>>,
) -> BTreeSet<String> {
    let mut portals = BTreeSet::new();

    for portal in discovered_portals() {
        portals.insert((*portal).to_string());
    }

    for file in files {
        if let Some(portal) = portal_from_rel_path(&file.rel_path) {
            portals.insert(portal.to_string());
        }
    }

    portals.extend(required_enums_by_portal.keys().cloned());
    portals
}

fn assemble_portal_indexes(
    portals: &BTreeSet<String>,
    files: &[TsFile],
) -> BTreeMap<String, String> {
    let mut modules_by_portal: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for file in files {
        let Some(portal) = portal_from_rel_path(&file.rel_path) else {
            continue;
        };

        let Some(rel_from_types_dir) = file.rel_path.strip_prefix(&format!("{portal}/types/"))
        else {
            continue;
        };
        if rel_from_types_dir == "index.ts" || rel_from_types_dir == "enums.ts" {
            continue;
        }

        let Some(module_name) = rel_from_types_dir.strip_suffix(".ts") else {
            continue;
        };
        modules_by_portal
            .entry(portal.to_string())
            .or_default()
            .insert(module_name.to_string());
    }

    let mut indexes = BTreeMap::new();
    for portal in portals {
        let mut out = String::new();
        out.push_str(&format!("export * from \"@{portal}/types/enums\";\n"));

        if let Some(modules) = modules_by_portal.get(portal) {
            for module in modules {
                out.push_str(&format!("export * from \"@{portal}/types/{module}\";\n"));
            }
        } else {
            out.push_str("// Add portal-specific contract types here as contracts are created.\n");
        }

        indexes.insert(portal.to_string(), out);
    }
    indexes
}

fn enum_to_ts_type<T: Serialize>(name: &str, variants: &[T]) -> String {
    let parts: Vec<String> = variants
        .iter()
        .map(|v| serde_json::to_string(v).unwrap())
        .collect();
    format!("export type {} = {};", name, parts.join(" | "))
}

fn ts_const_key(raw: &str) -> String {
    let mut out = String::new();
    let mut last_was_underscore = false;

    for ch in raw.chars() {
        let normalized = if ch.is_ascii_alphanumeric() {
            ch.to_ascii_uppercase()
        } else {
            '_'
        };

        if normalized == '_' {
            if out.is_empty() || last_was_underscore {
                continue;
            }
            out.push('_');
            last_was_underscore = true;
            continue;
        }

        out.push(normalized);
        last_was_underscore = false;
    }

    while out.ends_with('_') {
        out.pop();
    }

    if out.is_empty() {
        return "_".to_string();
    }

    if out.chars().next().is_some_and(|ch| ch.is_ascii_digit()) {
        format!("_{out}")
    } else {
        out
    }
}

fn assemble(f: &TsFile) -> String {
    let header = "// Auto-generated by `cargo run -p app --bin export-types`.\n\
                  // Do not edit manually — run `make gen-types` to regenerate.\n";
    let mut out = String::from(header);
    let portal = portal_from_rel_path(&f.rel_path)
        .unwrap_or_else(|| panic!("invalid TS export path (missing portal): {}", f.rel_path));
    if let Some(import) = enum_import_line(portal, &f.enums) {
        out.push_str(&import);
        out.push('\n');
        out.push('\n');
    } else {
        out.push('\n');
    }
    for (i, def) in f.definitions.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        out.push_str(def);
        out.push('\n');
    }
    out
}

fn assemble_enums_file(
    required_enums: &BTreeSet<String>,
    contract_enum_renderers: &BTreeMap<String, String>,
) -> String {
    let header = "// Auto-generated by `cargo run -p app --bin export-types`.\n\
                  // Do not edit manually — run `make gen-types` to regenerate.\n";
    let mut out = String::from(header);

    if required_enums.is_empty() {
        out.push_str("// No contract-facing enums currently required.\n");
        return out;
    }

    out.push('\n');
    for (idx, name) in required_enums.iter().enumerate() {
        if idx > 0 {
            out.push('\n');
            out.push('\n');
        }
        let rendered = contract_enum_renderers.get(name).unwrap_or_else(|| {
            panic!(
                "missing enum renderer for `{name}`; add it to collect_contract_enum_renderers()"
            )
        });
        out.push_str(&rendered);
    }
    out.push('\n');
    out
}

fn collect_required_enums_by_portal(files: &[TsFile]) -> BTreeMap<String, BTreeSet<String>> {
    let mut out: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for file in files {
        if file.enums.is_empty() {
            continue;
        }
        let portal = portal_from_rel_path(&file.rel_path).unwrap_or_else(|| {
            panic!("invalid TS export path (missing portal): {}", file.rel_path)
        });
        out.entry(portal.to_string())
            .or_default()
            .extend(file.enums.iter().cloned());
    }
    out
}

fn collect_required_enums(
    required_enums_by_portal: &BTreeMap<String, BTreeSet<String>>,
) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for enums in required_enums_by_portal.values() {
        out.extend(enums.iter().cloned());
    }
    out
}

fn collect_declared_contract_types(files: &[TsFile]) -> BTreeSet<String> {
    let mut declared = BTreeSet::new();
    for file in files {
        for def in &file.definitions {
            collect_declared_types(def, &mut declared);
        }
    }
    declared
}

fn detect_enum_references(
    rel_path: &str,
    definitions: &[String],
    known_contract_types: &BTreeSet<String>,
    known_contract_enums: &BTreeSet<String>,
) -> BTreeSet<String> {
    let mut used_identifiers = BTreeSet::new();
    for def in definitions {
        collect_used_type_identifiers(def, &mut used_identifiers);
    }

    for known_type in known_contract_types {
        used_identifiers.remove(known_type);
    }
    for builtin in ts_builtins() {
        used_identifiers.remove(*builtin);
    }
    for shared in ts_shared_types() {
        used_identifiers.remove(*shared);
    }

    let mut enums = BTreeSet::new();
    let mut unknown = Vec::new();
    for ident in used_identifiers {
        if known_contract_enums.contains(&ident) {
            enums.insert(ident);
        } else {
            unknown.push(ident);
        }
    }

    if !unknown.is_empty() {
        panic!(
            "Type export `{}` references external types without enum exporters: {}",
            rel_path,
            unknown.join(", ")
        );
    }

    enums
}

fn render_shared_platform_ts() -> String {
    use generated::{DEFAULT_LOCALE, SUPPORTED_LOCALES};

    if !SUPPORTED_LOCALES.contains(&DEFAULT_LOCALE) {
        panic!(
            "DEFAULT_LOCALE `{}` is not included in SUPPORTED_LOCALES",
            DEFAULT_LOCALE
        );
    }

    let locale_union = SUPPORTED_LOCALES
        .iter()
        .map(|locale| format!("\"{locale}\""))
        .collect::<Vec<_>>()
        .join(" | ");

    format!(
        "\
export type LocaleCode = {locale_union};
export const DEFAULT_LOCALE: LocaleCode = \"{default_locale}\";

// Localized text payload generated from app language settings.
export type LocalizedText<TLocale extends string = LocaleCode> = Record<TLocale, string>;

// field -> owner_id -> locale -> value
export type LocalizedMap<TLocale extends string = LocaleCode> = Record<
  string,
  Record<number, Record<TLocale, string>>
>;

export type JsonPrimitive = string | number | boolean | null;
export type JsonValue = JsonPrimitive | JsonObject | JsonValue[];
export interface JsonObject {{
  [key: string]: JsonValue;
}}

// Generic typed meta shape (compile-time typed keys/values).
export type MetaRecord<
  TShape extends Record<string, unknown> = Record<string, JsonValue>
> = Partial<TShape>;

// field -> owner_id -> value
export type MetaMap = Record<string, Record<number, JsonValue>>;

export interface AttachmentUploadDto {{
  id?: string | null;
  name?: string | null;
  path: string;
  content_type: string;
  size: number;
  width?: number | null;
  height?: number | null;
}}

// Backward-compatible alias used by generated model APIs.
export type AttachmentInput = AttachmentUploadDto;

export interface Attachment {{
  id: string;
  path: string;
  url: string;
  content_type: string;
  size: number;
  width: number | null;
  height: number | null;
  created_at: string;
}}

// field -> owner_id -> attachments
export type AttachmentMap = Record<string, Record<number, Attachment[]>>;

export type CountryStatus = \"enabled\" | \"disabled\";

export interface CountryCurrency {{
  code: string;
  name?: string | null;
  symbol?: string | null;
  minor_units?: number | null;
}}

export interface CountryRuntime {{
  iso2: string;
  iso3: string;
  iso_numeric?: string | null;
  name: string;
  official_name?: string | null;
  capital?: string | null;
  capitals: string[];
  region?: string | null;
  subregion?: string | null;
  currencies: CountryCurrency[];
  primary_currency_code?: string | null;
  calling_code?: string | null;
  calling_root?: string | null;
  calling_suffixes: string[];
  tlds: string[];
  timezones: string[];
  latitude?: number | null;
  longitude?: number | null;
  independent?: boolean | null;
  status: CountryStatus;
  assignment_status?: string | null;
  un_member?: boolean | null;
  flag_emoji?: string | null;
  created_at: string;
  updated_at: string;
}}
",
        default_locale = DEFAULT_LOCALE
    )
}

fn collect_used_type_identifiers(definition: &str, out: &mut BTreeSet<String>) {
    for line in definition.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty()
            || trimmed.starts_with("//")
            || trimmed.starts_with("/*")
            || trimmed.starts_with('*')
            || trimmed.starts_with("*/")
            || trimmed == "{"
            || trimmed == "}"
        {
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("export type ") {
            if let Some(idx) = rest.find('=') {
                collect_uppercase_identifiers(&rest[idx + 1..], out);
            }
            continue;
        }

        if trimmed.starts_with("export interface ") || trimmed.starts_with("export enum ") {
            continue;
        }

        if let Some(type_expr) = trimmed.strip_prefix('|') {
            collect_uppercase_identifiers(type_expr, out);
            continue;
        }

        if let Some(idx) = trimmed.find(':') {
            collect_uppercase_identifiers(&trimmed[idx + 1..], out);
        }
    }
}

fn collect_declared_types(definition: &str, out: &mut BTreeSet<String>) {
    for line in definition.lines() {
        let trimmed = line.trim_start();
        for prefix in ["export interface ", "export type ", "export enum "] {
            if let Some(rest) = trimmed.strip_prefix(prefix) {
                if let Some(name) = read_identifier(rest) {
                    out.insert(name.to_string());
                }
                break;
            }
        }
    }
}

fn read_identifier(input: &str) -> Option<&str> {
    let mut end = 0usize;
    for (idx, ch) in input.char_indices() {
        if idx == 0 {
            if !(ch.is_ascii_alphabetic() || ch == '_') {
                return None;
            }
            end = ch.len_utf8();
            continue;
        }
        if ch.is_ascii_alphanumeric() || ch == '_' {
            end = idx + ch.len_utf8();
        } else {
            break;
        }
    }
    if end == 0 {
        None
    } else {
        Some(&input[..end])
    }
}

fn collect_uppercase_identifiers(text: &str, out: &mut BTreeSet<String>) {
    let mut token = String::new();
    for ch in text.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            token.push(ch);
        } else {
            maybe_push_uppercase(&token, out);
            token.clear();
        }
    }
    maybe_push_uppercase(&token, out);
}

fn maybe_push_uppercase(token: &str, out: &mut BTreeSet<String>) {
    if token
        .chars()
        .next()
        .is_some_and(|ch| ch.is_ascii_uppercase())
    {
        out.insert(token.to_string());
    }
}

fn ts_builtins() -> &'static [&'static str] {
    &[
        "Array",
        "Date",
        "Map",
        "Promise",
        "ReadonlyArray",
        "Record",
        "Set",
    ]
}

fn ts_shared_types() -> &'static [&'static str] {
    &[
        "ApiResponse",
        "Attachment",
        "AttachmentInput",
        "AttachmentMap",
        "AttachmentUploadDto",
        "CountryCurrency",
        "CountryRuntime",
        "CountryStatus",
        "JsonObject",
        "JsonPrimitive",
        "JsonValue",
        "LocaleCode",
        "LocalizedMap",
        "MetaMap",
        "MetaRecord",
        "LocalizedText",
    ]
}

fn collect_contract_enum_renderers() -> BTreeMap<String, String> {
    let mut out = collect_schema_enum_renderers();
    out.insert("Permission".to_string(), render_permission_enum());
    out.insert("AuthClientType".to_string(), render_auth_client_type_enum());
    out
}

fn collect_schema_enum_renderers() -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for (name, variants) in parse_schema_enum_variants() {
        out.insert(name.clone(), render_schema_enum(&name, &variants));
    }
    out
}

fn render_schema_enum(name: &str, variants: &[String]) -> String {
    let mut out = enum_to_ts_type(name, variants);
    let const_base = ts_type_const_key(name);
    let list_const = ts_plural_const_key(&const_base);

    out.push_str(&format!(
        "\n\nexport const {const_base}: Readonly<Record<string, {name}>> = {{"
    ));
    for variant in variants {
        out.push_str(&format!(
            "\n  {}: {},",
            ts_const_key(variant),
            serde_json::to_string(variant).expect("schema enum value"),
        ));
    }
    out.push_str("\n};");

    out.push_str(&format!(
        "\n\nexport const {list_const}: ReadonlyArray<{name}> = ["
    ));
    for variant in variants {
        out.push_str(&format!(
            "\n  {},",
            serde_json::to_string(variant).expect("schema enum list value"),
        ));
    }
    out.push_str("\n];");
    out
}

fn parse_schema_enum_variants() -> BTreeMap<String, Vec<String>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../generated/src/models/enums.rs");
    let source = fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "failed to read generated schema enums at {}: {e}",
            path.display()
        )
    });

    let mut out = BTreeMap::new();
    let mut cursor = 0usize;
    while let Some(offset) = source[cursor..].find("pub enum ") {
        let start = cursor + offset + "pub enum ".len();
        let Some(name) = read_identifier(&source[start..]) else {
            break;
        };
        let enum_name = name.to_string();

        let impl_marker = format!("impl {enum_name} {{");
        let impl_start = source[start..].find(&impl_marker).unwrap_or_else(|| {
            panic!(
                "missing impl block for schema enum `{enum_name}` in {}",
                path.display()
            )
        });
        let impl_start = start + impl_start;

        let as_str_start = source[impl_start..]
            .find("pub const fn as_str")
            .unwrap_or_else(|| {
                panic!(
                    "missing as_str() function for schema enum `{enum_name}` in {}",
                    path.display()
                )
            });
        let as_str_start = impl_start + as_str_start;
        let variants_start = source[as_str_start..]
            .find("pub const fn variants")
            .unwrap_or_else(|| {
                panic!(
                    "missing variants() function for schema enum `{enum_name}` in {}",
                    path.display()
                )
            });
        let variants_start = as_str_start + variants_start;

        let as_str_block = &source[as_str_start..variants_start];
        let mut values = Vec::new();
        for line in as_str_block.lines() {
            let trimmed = line.trim();
            if !trimmed.contains("=>") {
                continue;
            }
            if let Some(value) = read_quoted_literal(trimmed) {
                values.push(value.to_string());
            }
        }

        if values.is_empty() {
            panic!(
                "failed to parse schema enum values for `{enum_name}` from {}",
                path.display()
            );
        }

        out.insert(enum_name, values);
        cursor = variants_start;
    }

    out
}

fn read_quoted_literal(input: &str) -> Option<&str> {
    let start = input.find('"')?;
    let tail = &input[start + 1..];
    let end = tail.find('"')?;
    Some(&tail[..end])
}

fn ts_type_const_key(name: &str) -> String {
    let mut out = String::new();
    let mut previous_was_lower_or_digit = false;

    for ch in name.chars() {
        if ch == '_' {
            if !out.ends_with('_') && !out.is_empty() {
                out.push('_');
            }
            previous_was_lower_or_digit = false;
            continue;
        }

        if ch.is_ascii_uppercase() && previous_was_lower_or_digit && !out.ends_with('_') {
            out.push('_');
        }
        out.push(ch.to_ascii_uppercase());
        previous_was_lower_or_digit = ch.is_ascii_lowercase() || ch.is_ascii_digit();
    }

    out
}

fn ts_plural_const_key(base: &str) -> String {
    if let Some(stem) = base.strip_suffix('Y') {
        format!("{stem}IES")
    } else if base.ends_with('S') {
        format!("{base}ES")
    } else {
        format!("{base}S")
    }
}

fn render_permission_enum() -> String {
    use generated::permissions::{Permission, PERMISSION_META};

    let mut out = enum_to_ts_type("Permission", Permission::all());
    out.push_str(
        "\n\nexport interface PermissionMeta {\n  key: Permission;\n  guard: string;\n  label: string;\n  group: string;\n  description: string;\n}",
    );

    out.push_str("\n\nexport const PERMISSION_META: ReadonlyArray<PermissionMeta> = [");
    for meta in PERMISSION_META {
        out.push_str(&format!(
            "\n  {{ key: {}, guard: {}, label: {}, group: {}, description: {} }},",
            serde_json::to_string(&meta.key).expect("permission key"),
            serde_json::to_string(&meta.guard).expect("permission guard"),
            serde_json::to_string(&meta.label).expect("permission label"),
            serde_json::to_string(&meta.group).expect("permission group"),
            serde_json::to_string(&meta.description).expect("permission description"),
        ));
    }
    out.push_str("\n];");

    out.push_str("\n\nexport const PERMISSIONS: ReadonlyArray<Permission> = [");
    for permission in Permission::all() {
        out.push_str(&format!(
            "\n  {},",
            serde_json::to_string(&permission.as_str()).expect("permission value"),
        ));
    }
    out.push_str("\n];");

    out.push_str("\n\nexport const PERMISSION: Readonly<Record<string, Permission>> = {");
    for permission in Permission::all() {
        let value = permission.as_str();
        out.push_str(&format!(
            "\n  {}: {},",
            ts_const_key(value),
            serde_json::to_string(&value).expect("permission const value"),
        ));
    }
    out.push_str("\n};");

    out.push_str(
        "\n\nexport const PERMISSION_META_BY_KEY: Readonly<Record<Permission, PermissionMeta>> = {",
    );
    for meta in PERMISSION_META {
        out.push_str(&format!(
            "\n  {}: {{ key: {}, guard: {}, label: {}, group: {}, description: {} }},",
            serde_json::to_string(&meta.key).expect("permission key in by_key"),
            serde_json::to_string(&meta.key).expect("permission meta key field"),
            serde_json::to_string(&meta.guard).expect("permission meta guard field"),
            serde_json::to_string(&meta.label).expect("permission meta label field"),
            serde_json::to_string(&meta.group).expect("permission meta group field"),
            serde_json::to_string(&meta.description).expect("permission meta description field"),
        ));
    }
    out.push_str("\n};");

    out
}

fn render_auth_client_type_enum() -> String {
    use core_web::auth::AuthClientType;
    enum_to_ts_type(
        "AuthClientType",
        &[AuthClientType::Web, AuthClientType::Mobile],
    )
}

fn portal_from_rel_path(rel_path: &str) -> Option<&str> {
    rel_path.split('/').next()
}

fn enum_import_line(portal: &str, enums: &BTreeSet<String>) -> Option<String> {
    if enums.is_empty() {
        return None;
    }
    let joined = enums
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .join(", ");
    Some(format!(
        r#"import type {{ {} }} from "@{}/types/enums";"#,
        joined, portal
    ))
}

fn write_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("failed to create directory");
    }
    fs::write(path, content).unwrap_or_else(|e| {
        panic!("failed to write {}: {e}", path.display());
    });
    println!("  wrote {}", path.display());
}

// ── Static TypeScript content ────────────────────────────────
// Framework types from core-web that can't derive TS directly.

const SHARED_API_TS: &str = "\
export interface ApiResponse<T> {
  data: T;
  message?: string;
}

export interface ApiErrorResponse {
  message: string;
  errors?: Record<string, string[]>;
}
";

const SHARED_DATATABLE_TS: &str = "\
export type DataTablePaginationMode = \"offset\" | \"cursor\";

export type DataTableSortDirection = \"asc\" | \"desc\";

export interface DataTableQueryRequestBase {
  include_meta?: boolean;
  page?: number | null;
  per_page?: number | null;
  cursor?: string | null;
  pagination_mode?: DataTablePaginationMode | null;
  sorting_column?: string | null;
  sorting?: DataTableSortDirection | null;
  timezone?: string | null;
  created_at_from?: string | null;
  created_at_to?: string | null;
}

export interface DataTableEmailExportRequestBase {
  query: DataTableQueryRequestBase;
  recipients: string[];
  subject?: string | null;
  export_file_name?: string | null;
}

export type DataTableFilterFieldType =
  | \"text\"
  | \"select\"
  | \"number\"
  | \"date\"
  | \"datetime\"
  | \"time\"
  | \"boolean\";

export interface DataTableFilterOptionDto {
  label: string;
  value: string;
}

export interface DataTableFilterFieldDto {
  field: string;
  filter_key: string;
  type: DataTableFilterFieldType;
  label: string;
  placeholder?: string;
  description?: string;
  options?: DataTableFilterOptionDto[];
}

export interface DataTableColumnMetaDto {
  name: string;
  label: string;
  data_type: string;
  sortable: boolean;
  localized: boolean;
  filter_ops: string[];
}

export interface DataTableRelationColumnMetaDto {
  relation: string;
  column: string;
  data_type: string;
  filter_ops: string[];
}

export interface DataTableDefaultsDto {
  sorting_column: string;
  sorted: string;
  per_page: number;
  export_ignore_columns: string[];
  timestamp_columns: string[];
  unsortable: string[];
}

export interface DataTableDiagnosticsDto {
  duration_ms: number;
  auto_filters_applied: number;
  unknown_filters: string[];
  unknown_filter_mode: string;
}

export interface DataTableMetaDto {
  model_key: string;
  defaults: DataTableDefaultsDto;
  columns: DataTableColumnMetaDto[];
  relation_columns: DataTableRelationColumnMetaDto[];
  filter_rows: (DataTableFilterFieldDto | DataTableFilterFieldDto[])[];
}

export interface DataTableQueryResponse<T> {
  records: T[];
  per_page: number;
  total_records: number;
  total_pages: number;
  page: number;
  pagination_mode: string;
  has_more?: boolean;
  next_cursor?: string;
  summary?: unknown;
  diagnostics: DataTableDiagnosticsDto;
  meta?: DataTableMetaDto;
}

export type DataTableEmailExportState =
  | \"waiting_csv\"
  | \"uploading\"
  | \"sending\"
  | \"completed\"
  | \"failed\";

export interface DataTableEmailExportStatusDto {
  state: DataTableEmailExportState;
  recipients: string[];
  subject?: string;
  link_url?: string;
  error?: string;
  updated_at_unix: number;
  sent_at_unix?: number;
}

export interface DataTableEmailExportQueuedDto {
  job_id: string;
  csv_state: string;
  email_state: DataTableEmailExportState;
}

export interface DataTableExportStatusResponseDto {
  job_id: string;
  model_key: string;
  csv_state: string;
  csv_error?: string;
  csv_file_name?: string;
  csv_content_type?: string;
  csv_total_records?: number;
  email?: DataTableEmailExportStatusDto;
}
";

const SHARED_INDEX_TS: &str = "\
export * from \"@shared/types/api\";
export * from \"@shared/types/datatable\";
export * from \"@shared/types/enums\";
export * from \"@shared/types/platform\";
";
