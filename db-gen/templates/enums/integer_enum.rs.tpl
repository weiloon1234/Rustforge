#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[repr({{rust_type}})]
pub enum {{name}} {
{{variant_decls}}
}

impl Default for {{name}} {
    fn default() -> Self {
        Self::{{default_variant}}
    }
}

impl ts_rs::TS for {{name}} {
    type WithoutGenerics = Self;

    fn name() -> String {
        "{{name}}".to_string()
    }

    fn inline() -> String {
        Self::name()
    }

    fn inline_flattened() -> String {
        panic!("{{name}} cannot be flattened")
    }

    fn decl() -> String {
        "type {{name}} = {{ts_union_literal}};".to_string()
    }

    fn decl_concrete() -> String {
        Self::decl()
    }
}

impl {{name}} {
    pub const fn as_str(self) -> &'static str {
        match self {
{{as_str_arms}}
        }
    }

    pub const fn as_label(self) -> &'static str {
        match self {
{{as_label_arms}}
        }
    }

    pub fn from_storage(raw: &str) -> Option<Self> {
        let value = raw.trim().parse::<i64>().ok()?;
        match value {
{{from_storage_arms}}
            _ => None,
        }
    }

    pub const fn i18n_key(self) -> &'static str {
        match self {
{{i18n_key_arms}}
        }
    }

    pub fn explained_label(self) -> String {
        let i18n_key = self.i18n_key();
        let translated_key = core_i18n::t(i18n_key);
        if translated_key != i18n_key {
            return translated_key;
        }
        let fallback_label = self.as_label();
        let translated_label = core_i18n::t(fallback_label);
        if translated_label != fallback_label {
            return translated_label;
        }
        fallback_label.to_string()
    }

    pub const fn variants() -> &'static [Self] {
        &[{{variant_self_list}}]
    }
{{datatable_filter_options_section}}
}

impl sqlx::Encode<'_, sqlx::Postgres> for {{name}} {
    fn encode_by_ref(&self, buf: &mut sqlx::postgres::PgArgumentBuffer) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        <{{rust_type}} as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(&(*self as {{rust_type}}), buf)
    }
}

impl sqlx::Decode<'_, sqlx::Postgres> for {{name}} {
    fn decode(value: sqlx::postgres::PgValueRef<'_>) -> Result<Self, sqlx::error::BoxDynError> {
        let num = <{{rust_type}} as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
{{decode_match}}
    }
}

impl sqlx::Type<sqlx::Postgres> for {{name}} {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <{{rust_type}} as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

impl From<{{name}}> for core_db::common::sql::BindValue {
    fn from(v: {{name}}) -> Self {
        core_db::common::sql::BindValue::I64(v as i64)
    }
}
