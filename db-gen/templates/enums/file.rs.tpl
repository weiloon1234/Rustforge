// AUTO-GENERATED FILE — DO NOT EDIT
// Generated from Rust model source enum definitions

#[derive(Debug, Clone, Copy)]
pub struct SchemaEnumVariantMeta {
    pub value: &'static str,
    pub label: &'static str,
    pub i18n_key: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub struct SchemaEnumTsMeta {
    pub name: &'static str,
    pub variants: &'static [SchemaEnumVariantMeta],
}

{{enum_blocks}}
pub const SCHEMA_ENUM_TS_META: &[SchemaEnumTsMeta] = &[
{{schema_enum_ts_meta_entries}}
];
