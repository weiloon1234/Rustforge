// AUTO-GENERATED FILE — DO NOT EDIT
// Generated from TOML schema enum definitions

#[derive(Debug, Clone, Copy)]
pub struct SchemaEnumTsMeta {
    pub name: &'static str,
    pub variants: &'static [&'static str],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub enum ArticleStatus {
    #[serde(rename = "draft")]
    Draft,
    #[serde(rename = "published")]
    Published
}

impl Default for ArticleStatus {
    fn default() -> Self {
        Self::Draft
    }
}

impl ts_rs::TS for ArticleStatus {
    type WithoutGenerics = Self;

    fn name() -> String {
        "ArticleStatus".to_string()
    }

    fn inline() -> String {
        Self::name()
    }

    fn inline_flattened() -> String {
        panic!("ArticleStatus cannot be flattened")
    }

    fn decl() -> String {
        "type ArticleStatus = \"draft\" | \"published\";".to_string()
    }

    fn decl_concrete() -> String {
        Self::decl()
    }
}

impl ArticleStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Published => "published",
        }
    }

    pub const fn as_label(self) -> &'static str {
        match self {
            Self::Draft => "Draft",
            Self::Published => "Published",
        }
    }

    pub fn from_storage(raw: &str) -> Option<Self> {
        match raw.trim() {
            "draft" => Some(Self::Draft),
            "published" => Some(Self::Published),
            _ => None,
        }
    }

    pub const fn i18n_key(self) -> &'static str {
        match self {
            Self::Draft => "enum.article_status.draft",
            Self::Published => "enum.article_status.published",
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
        &[Self::Draft, Self::Published]
    }

    pub fn datatable_filter_options() -> Vec<core_web::datatable::DataTableFilterOptionDto> {
        Self::variants()
            .iter()
            .map(|v| {
                let label = (*v).explained_label();
                let value = (*v).as_str();
                core_web::datatable::DataTableFilterOptionDto {
                    label,
                    value: value.to_string(),
                }
            })
            .collect()
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for ArticleStatus {
    fn encode_by_ref(&self, buf: &mut sqlx::postgres::PgArgumentBuffer) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        let s = match self {
            Self::Draft => "draft",
            Self::Published => "published",
        };
        <&str as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(&s, buf)
    }
}

impl sqlx::Decode<'_, sqlx::Postgres> for ArticleStatus {
    fn decode(value: sqlx::postgres::PgValueRef<'_>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <&str as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        match s {
            "draft" => Ok(Self::Draft),
            "published" => Ok(Self::Published),
            _ => Err(format!("Invalid ArticleStatus: {}", s).into()),
        }
    }
}

impl sqlx::Type<sqlx::Postgres> for ArticleStatus {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <String as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

impl From<ArticleStatus> for core_db::common::sql::BindValue {
    fn from(v: ArticleStatus) -> Self {
        let s = match v {
            ArticleStatus::Draft => "draft",
            ArticleStatus::Published => "published",
        };
        core_db::common::sql::BindValue::String(s.to_string())
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[repr(i16)]
pub enum ArticleSystemFlag {
    #[serde(rename = "0")]
    No = 0,
    #[serde(rename = "1")]
    Yes = 1,
}

impl Default for ArticleSystemFlag {
    fn default() -> Self {
        Self::No
    }
}

impl ts_rs::TS for ArticleSystemFlag {
    type WithoutGenerics = Self;

    fn name() -> String {
        "ArticleSystemFlag".to_string()
    }

    fn inline() -> String {
        Self::name()
    }

    fn inline_flattened() -> String {
        panic!("ArticleSystemFlag cannot be flattened")
    }

    fn decl() -> String {
        "type ArticleSystemFlag = \"0\" | \"1\";".to_string()
    }

    fn decl_concrete() -> String {
        Self::decl()
    }
}

impl ArticleSystemFlag {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::No => "0",
            Self::Yes => "1",
        }
    }

    pub const fn as_label(self) -> &'static str {
        match self {
            Self::No => "No",
            Self::Yes => "Yes",
        }
    }

    pub fn from_storage(raw: &str) -> Option<Self> {
        let value = raw.trim().parse::<i64>().ok()?;
        match value {
            0 => Some(Self::No),
            1 => Some(Self::Yes),
            _ => None,
        }
    }

    pub const fn i18n_key(self) -> &'static str {
        match self {
            Self::No => "enum.article_system_flag.no",
            Self::Yes => "enum.article_system_flag.yes",
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
        &[Self::No, Self::Yes]
    }

    pub fn datatable_filter_options() -> Vec<core_web::datatable::DataTableFilterOptionDto> {
        Self::variants()
            .iter()
            .map(|v| {
                let label = (*v).explained_label();
                let value = (*v).as_str();
                core_web::datatable::DataTableFilterOptionDto {
                    label,
                    value: value.to_string(),
                }
            })
            .collect()
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for ArticleSystemFlag {
    fn encode_by_ref(&self, buf: &mut sqlx::postgres::PgArgumentBuffer) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        <i16 as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(&(*self as i16), buf)
    }
}

impl sqlx::Decode<'_, sqlx::Postgres> for ArticleSystemFlag {
    fn decode(value: sqlx::postgres::PgValueRef<'_>) -> Result<Self, sqlx::error::BoxDynError> {
        let num = <i16 as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        match num {
            0 => Ok(Self::No),
            1 => Ok(Self::Yes),
            _ => Err(format!("Invalid ArticleSystemFlag: {}", num).into()),
        }
    }
}

impl sqlx::Type<sqlx::Postgres> for ArticleSystemFlag {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <i16 as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

impl From<ArticleSystemFlag> for core_db::common::sql::BindValue {
    fn from(v: ArticleSystemFlag) -> Self {
        core_db::common::sql::BindValue::I64(v as i64)
    }
}



pub const SCHEMA_ENUM_TS_META: &[SchemaEnumTsMeta] = &[
    SchemaEnumTsMeta { name: "ArticleStatus", variants: &["draft", "published"] },
    SchemaEnumTsMeta { name: "ArticleSystemFlag", variants: &["0", "1"] },
];
