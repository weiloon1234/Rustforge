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

    // Determine which contract-facing enums are actually referenced by DTOs.
    let known_contract_types = collect_declared_contract_types(&files);
    for ts_file in &mut files {
        ts_file.enums = detect_enum_references(
            ts_file.rel_path.as_str(),
            &ts_file.definitions,
            &known_contract_types,
        );
    }

    // Write ts-rs generated files
    for ts_file in &files {
        let path = base.join(&ts_file.rel_path);
        write_file(&path, &assemble(ts_file));
    }

    // ── 2. Enum types (serde-derived) ────────────────────────
    let required_enums_by_portal = collect_required_enums_by_portal(&files);

    let all_portals = collect_all_portals(&files, &required_enums_by_portal);
    for portal in &all_portals {
        let enums = required_enums_by_portal
            .get(portal.as_str())
            .cloned()
            .unwrap_or_default();
        let enums_ts = assemble_enums_file(&enums);
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

fn assemble_enums_file(required_enums: &BTreeSet<String>) -> String {
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
        let rendered = render_contract_enum(name).unwrap_or_else(|| {
            panic!(
                "missing enum renderer for `{}`; add it to render_contract_enum()",
                name
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

    let mut enums = BTreeSet::new();
    let mut unknown = Vec::new();
    for ident in used_identifiers {
        if is_contract_enum(&ident) {
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
export type MultiLang<TLocale extends string = LocaleCode> = Record<TLocale, string>;

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
  content_type: string;
  size: number;
  width: number | null;
  height: number | null;
  created_at: string;
}}

// field -> owner_id -> attachments
export type AttachmentMap = Record<string, Record<number, Attachment[]>>;
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

fn is_contract_enum(name: &str) -> bool {
    matches!(name, "AdminType" | "Permission" | "AuthClientType")
}

fn render_contract_enum(name: &str) -> Option<String> {
    match name {
        "AdminType" => Some(render_admin_type_enum()),
        "Permission" => Some(render_permission_enum()),
        "AuthClientType" => Some(render_auth_client_type_enum()),
        _ => None,
    }
}

fn render_admin_type_enum() -> String {
    use generated::models::AdminType;
    enum_to_ts_type("AdminType", AdminType::variants())
}

fn render_permission_enum() -> String {
    use generated::permissions::Permission;
    enum_to_ts_type("Permission", Permission::all())
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
export * from \"@shared/types/platform\";
";
