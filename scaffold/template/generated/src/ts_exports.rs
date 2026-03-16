use core_web::auth::AuthClientType;
use serde::Serialize;

use crate::models::enums::SCHEMA_ENUM_TS_META;
use crate::permissions::{Permission, PERMISSION_META};
use crate::{DEFAULT_LOCALE, SUPPORTED_LOCALES};

#[derive(Debug, Clone)]
pub struct TsExportFile {
    pub rel_path: &'static str,
    pub rust_path: &'static str,
    pub definition: String,
}

pub fn ts_export_files() -> Vec<TsExportFile> {
    vec![
        TsExportFile {
            rel_path: "shared/types/api.ts",
            rust_path: "generated::ts_exports::api",
            definition: render_api_ts(),
        },
        TsExportFile {
            rel_path: "shared/types/datatable.ts",
            rust_path: "generated::ts_exports::datatable",
            definition: render_datatable_ts(),
        },
        TsExportFile {
            rel_path: "shared/types/platform.ts",
            rust_path: "generated::ts_exports::platform",
            definition: render_platform_ts(),
        },
    ]
}

pub fn contract_enum_renderers() -> Vec<(String, String)> {
    let mut out = Vec::new();
    for meta in SCHEMA_ENUM_TS_META {
        out.push((
            meta.name.to_string(),
            render_schema_enum(meta.name, meta.variants),
        ));
    }
    out.push(("Permission".to_string(), render_permission_enum()));
    out.push(("AuthClientType".to_string(), render_auth_client_type_enum()));
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

fn render_api_ts() -> String {
    "\
export interface ApiResponse<T> {
  data: T;
  message?: string;
}

export interface ApiErrorResponse {
  message: string;
  errors?: Record<string, string[]>;
}
"
    .to_string()
}

fn render_datatable_ts() -> String {
    "\
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
"
    .to_string()
}

fn render_platform_ts() -> String {
    let locale = render_platform_locale_ts();
    let core = "\
export type JsonPrimitive = string | number | boolean | null;
export type JsonValue = JsonPrimitive | JsonObject | JsonValue[];
export interface JsonObject {
  [key: string]: JsonValue;
}

export type MetaRecord<
  TShape extends Record<string, unknown> = Record<string, JsonValue>
> = Partial<TShape>;

export type MetaMap = Record<string, Record<number, JsonValue>>;

export interface AttachmentUploadDto {
  id?: string | null;
  name?: string | null;
  path: string;
  content_type: string;
  size: number;
  width?: number | null;
  height?: number | null;
}

export type AttachmentInput = AttachmentUploadDto;

export interface Attachment {
  id: string;
  path: string;
  url: string;
  content_type: string;
  size: number;
  width: number | null;
  height: number | null;
  created_at: string;
}

export type AttachmentMap = Record<string, Record<number, Attachment[]>>;

export type CountryStatus = \"enabled\" | \"disabled\";

export interface CountryCurrency {
  code: string;
  name?: string | null;
  symbol?: string | null;
  minor_units?: number | null;
}

export interface CountryRuntime {
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
  is_default: boolean;
  assignment_status?: string | null;
  un_member?: boolean | null;
  flag_emoji?: string | null;
  created_at: string;
  updated_at: string;
}
";

    format!("{locale}\n{core}")
}

fn render_platform_locale_ts() -> String {
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

export type LocalizedText<TLocale extends string = LocaleCode> = Record<TLocale, string>;

export type LocalizedInput<TLocale extends string = LocaleCode> = Partial<Record<TLocale, string | null>>;

// field -> owner_id -> locale -> value
export type LocalizedMap<TLocale extends string = LocaleCode> = Record<
  string,
  Record<number, Record<TLocale, string>>
>;
",
        default_locale = DEFAULT_LOCALE
    )
}

fn render_schema_enum(name: &str, variants: &[&str]) -> String {
    ensure_unique_schema_enum(name, variants);

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

fn ensure_unique_schema_enum(name: &str, variants: &[&str]) {
    use std::collections::BTreeSet;

    let mut variant_values = BTreeSet::new();
    for variant in variants {
        if !variant_values.insert(*variant) {
            panic!("duplicate enum variant value `{variant}` in `{name}`");
        }
    }

    let mut const_keys = BTreeSet::new();
    for variant in variants {
        let key = ts_const_key(variant);
        if !const_keys.insert(key.clone()) {
            panic!("duplicate enum const key `{key}` in `{name}`");
        }
    }
}

fn render_permission_enum() -> String {
    ensure_unique_permission_entries();

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

fn ensure_unique_permission_entries() {
    use std::collections::BTreeSet;

    let mut values = BTreeSet::new();
    let mut const_keys = BTreeSet::new();
    for permission in Permission::all() {
        let value = permission.as_str();
        if !values.insert(value) {
            panic!("duplicate permission value `{value}`");
        }

        let key = ts_const_key(value);
        if !const_keys.insert(key.clone()) {
            panic!("duplicate permission const key `{key}`");
        }
    }
}

fn render_auth_client_type_enum() -> String {
    enum_to_ts_type(
        "AuthClientType",
        &[AuthClientType::Web, AuthClientType::Mobile],
    )
}

fn enum_to_ts_type<T: Serialize>(name: &str, variants: &[T]) -> String {
    let parts: Vec<String> = variants
        .iter()
        .map(|v| serde_json::to_string(v).expect("enum variant serialization"))
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
