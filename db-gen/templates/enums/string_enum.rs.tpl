#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub enum {{name}} {
{{variant_list}}
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
        match raw.trim() {
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
        let s = match self {
{{encode_arms}}
        };
        <&str as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(&s, buf)
    }
}

impl sqlx::Decode<'_, sqlx::Postgres> for {{name}} {
    fn decode(value: sqlx::postgres::PgValueRef<'_>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <&str as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        match s {
{{decode_arms}}
            _ => Err(format!("Invalid {{name}}: {}", s).into()),
        }
    }
}

impl sqlx::Type<sqlx::Postgres> for {{name}} {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <String as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

impl From<{{name}}> for core_db::common::sql::BindValue {
    fn from(v: {{name}}) -> Self {
        let s = match v {
{{encode_arms_qualified}}
        };
        core_db::common::sql::BindValue::String(s.to_string())
    }
}
