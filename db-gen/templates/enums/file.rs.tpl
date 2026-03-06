// AUTO-GENERATED FILE — DO NOT EDIT
// Generated from TOML schema enum definitions

#[derive(Debug, Clone, Copy)]
pub struct SchemaEnumTsMeta {
    pub name: &'static str,
    pub variants: &'static [&'static str],
}

{{enum_blocks}}
pub const SCHEMA_ENUM_TS_META: &[SchemaEnumTsMeta] = &[
{{schema_enum_ts_meta_entries}}
];
