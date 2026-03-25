use serde::Serialize;

pub use core_web::ts_exports::TsExportFile;

use crate::models::enums::{SchemaEnumVariantMeta, SCHEMA_ENUM_TS_META};
use crate::permissions::{Permission, PERMISSION_META};
use crate::{DEFAULT_LOCALE, SUPPORTED_LOCALES};

pub fn ts_export_files() -> Vec<TsExportFile> {
    let mut files = core_web::ts_exports::ts_export_files();
    files.push(TsExportFile {
        rel_path: "shared/types/platform.ts",
        rust_path: "generated::ts_exports::platform",
        definition: render_platform_ts(),
    });
    files
}

pub fn contract_enum_renderers() -> Vec<(String, String)> {
    let mut out = core_web::ts_exports::contract_enum_renderers();
    for meta in SCHEMA_ENUM_TS_META {
        out.push((
            meta.name.to_string(),
            render_schema_enum_rich(meta.name, meta.variants),
        ));
    }
    out.push(("Permission".to_string(), render_permission_enum()));
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
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
  conversion_rate: string;
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

fn render_schema_enum_rich(name: &str, variants: &[SchemaEnumVariantMeta]) -> String {
    ensure_unique_schema_enum_rich(name, variants);

    let values: Vec<&str> = variants.iter().map(|v| v.value).collect();
    let mut out = enum_to_ts_type(name, &values);
    let const_base = ts_type_const_key(name);
    let list_const = ts_plural_const_key(&const_base);

    // Named const object (e.g. WITHDRAWAL_STATUS.PENDING = "1")
    out.push_str(&format!(
        "\n\nexport const {const_base}: Readonly<Record<string, {name}>> = {{"
    ));
    for variant in variants {
        out.push_str(&format!(
            "\n  {}: {},",
            ts_const_key(variant.label),
            serde_json::to_string(variant.value).expect("schema enum value"),
        ));
    }
    out.push_str("\n};");

    // Array of all values
    out.push_str(&format!(
        "\n\nexport const {list_const}: ReadonlyArray<{name}> = ["
    ));
    for variant in variants {
        out.push_str(&format!(
            "\n  {},",
            serde_json::to_string(variant.value).expect("schema enum list value"),
        ));
    }
    out.push_str("\n];");

    // I18N mapping (e.g. WITHDRAWAL_STATUS_I18N["1"] = "enum.withdrawal_status.pending")
    out.push_str(&format!(
        "\n\nexport const {const_base}_I18N: Readonly<Record<{name}, string>> = {{"
    ));
    for variant in variants {
        out.push_str(&format!(
            "\n  {}: {},",
            serde_json::to_string(variant.value).expect("i18n key value"),
            serde_json::to_string(variant.i18n_key).expect("i18n key string"),
        ));
    }
    out.push_str("\n};");

    out
}

fn ensure_unique_schema_enum_rich(name: &str, variants: &[SchemaEnumVariantMeta]) {
    use std::collections::BTreeSet;

    let mut variant_values = BTreeSet::new();
    for variant in variants {
        if !variant_values.insert(variant.value) {
            panic!("duplicate enum variant value `{}` in `{name}`", variant.value);
        }
    }

    let mut const_keys = BTreeSet::new();
    for variant in variants {
        let key = ts_const_key(variant.label);
        if !const_keys.insert(key.clone()) {
            panic!("duplicate enum const key `{key}` (from label `{}`) in `{name}`", variant.label);
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
