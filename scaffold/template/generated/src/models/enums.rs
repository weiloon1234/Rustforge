// AUTO-GENERATED FILE — DO NOT EDIT
// Generated from TOML schema enum definitions

#[derive(Debug, Clone, Copy)]
pub struct SchemaEnumTsMeta {
    pub name: &'static str,
    pub variants: &'static [&'static str],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub enum AdminType {
    #[serde(rename = "developer")]
    Developer,
    #[serde(rename = "superadmin")]
    SuperAdmin,
    #[serde(rename = "admin")]
    Admin
}

impl Default for AdminType {
    fn default() -> Self {
        Self::Developer
    }
}

impl ts_rs::TS for AdminType {
    type WithoutGenerics = Self;

    fn name() -> String {
        "AdminType".to_string()
    }

    fn inline() -> String {
        Self::name()
    }

    fn inline_flattened() -> String {
        panic!("AdminType cannot be flattened")
    }

    fn decl() -> String {
        "type AdminType = \"developer\" | \"superadmin\" | \"admin\";".to_string()
    }

    fn decl_concrete() -> String {
        Self::decl()
    }
}

impl AdminType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Developer => "developer",
            Self::SuperAdmin => "superadmin",
            Self::Admin => "admin",
        }
    }

    pub const fn as_label(self) -> &'static str {
        match self {
            Self::Developer => "Developer",
            Self::SuperAdmin => "SuperAdmin",
            Self::Admin => "Admin",
        }
    }

    pub fn from_storage(raw: &str) -> Option<Self> {
        match raw.trim() {
            "developer" => Some(Self::Developer),
            "superadmin" => Some(Self::SuperAdmin),
            "admin" => Some(Self::Admin),
            _ => None,
        }
    }

    pub const fn i18n_key(self) -> &'static str {
        match self {
            Self::Developer => "enum.admin_type.developer",
            Self::SuperAdmin => "enum.admin_type.super_admin",
            Self::Admin => "enum.admin_type.admin",
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
        &[Self::Developer, Self::SuperAdmin, Self::Admin]
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

impl sqlx::Encode<'_, sqlx::Postgres> for AdminType {
    fn encode_by_ref(&self, buf: &mut sqlx::postgres::PgArgumentBuffer) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        let s = match self {
            Self::Developer => "developer",
            Self::SuperAdmin => "superadmin",
            Self::Admin => "admin",
        };
        <&str as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(&s, buf)
    }
}

impl sqlx::Decode<'_, sqlx::Postgres> for AdminType {
    fn decode(value: sqlx::postgres::PgValueRef<'_>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <&str as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        match s {
            "developer" => Ok(Self::Developer),
            "superadmin" => Ok(Self::SuperAdmin),
            "admin" => Ok(Self::Admin),
            _ => Err(format!("Invalid AdminType: {}", s).into()),
        }
    }
}

impl sqlx::Type<sqlx::Postgres> for AdminType {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <String as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

impl From<AdminType> for core_db::common::sql::BindValue {
    fn from(v: AdminType) -> Self {
        let s = match v {
            AdminType::Developer => "developer",
            AdminType::SuperAdmin => "superadmin",
            AdminType::Admin => "admin",
        };
        core_db::common::sql::BindValue::String(s.to_string())
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[repr(i16)]
pub enum ContentPageSystemFlag {
    #[serde(rename = "0")]
    No = 0,
    #[serde(rename = "1")]
    Yes = 1,
}

impl Default for ContentPageSystemFlag {
    fn default() -> Self {
        Self::No
    }
}

impl ts_rs::TS for ContentPageSystemFlag {
    type WithoutGenerics = Self;

    fn name() -> String {
        "ContentPageSystemFlag".to_string()
    }

    fn inline() -> String {
        Self::name()
    }

    fn inline_flattened() -> String {
        panic!("ContentPageSystemFlag cannot be flattened")
    }

    fn decl() -> String {
        "type ContentPageSystemFlag = \"0\" | \"1\";".to_string()
    }

    fn decl_concrete() -> String {
        Self::decl()
    }
}

impl ContentPageSystemFlag {
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
            Self::No => "enum.content_page_system_flag.no",
            Self::Yes => "enum.content_page_system_flag.yes",
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

impl sqlx::Encode<'_, sqlx::Postgres> for ContentPageSystemFlag {
    fn encode_by_ref(&self, buf: &mut sqlx::postgres::PgArgumentBuffer) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        <i16 as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(&(*self as i16), buf)
    }
}

impl sqlx::Decode<'_, sqlx::Postgres> for ContentPageSystemFlag {
    fn decode(value: sqlx::postgres::PgValueRef<'_>) -> Result<Self, sqlx::error::BoxDynError> {
        let num = <i16 as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        match num {
            0 => Ok(Self::No),
            1 => Ok(Self::Yes),
            _ => Err(format!("Invalid ContentPageSystemFlag: {}", num).into()),
        }
    }
}

impl sqlx::Type<sqlx::Postgres> for ContentPageSystemFlag {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <i16 as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

impl From<ContentPageSystemFlag> for core_db::common::sql::BindValue {
    fn from(v: ContentPageSystemFlag) -> Self {
        core_db::common::sql::BindValue::I64(v as i64)
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub enum CountryStatus {
    #[serde(rename = "enabled")]
    Enabled,
    #[serde(rename = "disabled")]
    Disabled
}

impl Default for CountryStatus {
    fn default() -> Self {
        Self::Enabled
    }
}

impl ts_rs::TS for CountryStatus {
    type WithoutGenerics = Self;

    fn name() -> String {
        "CountryStatus".to_string()
    }

    fn inline() -> String {
        Self::name()
    }

    fn inline_flattened() -> String {
        panic!("CountryStatus cannot be flattened")
    }

    fn decl() -> String {
        "type CountryStatus = \"enabled\" | \"disabled\";".to_string()
    }

    fn decl_concrete() -> String {
        Self::decl()
    }
}

impl CountryStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Enabled => "enabled",
            Self::Disabled => "disabled",
        }
    }

    pub const fn as_label(self) -> &'static str {
        match self {
            Self::Enabled => "Enabled",
            Self::Disabled => "Disabled",
        }
    }

    pub fn from_storage(raw: &str) -> Option<Self> {
        match raw.trim() {
            "enabled" => Some(Self::Enabled),
            "disabled" => Some(Self::Disabled),
            _ => None,
        }
    }

    pub const fn i18n_key(self) -> &'static str {
        match self {
            Self::Enabled => "enum.country_status.enabled",
            Self::Disabled => "enum.country_status.disabled",
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
        &[Self::Enabled, Self::Disabled]
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

impl sqlx::Encode<'_, sqlx::Postgres> for CountryStatus {
    fn encode_by_ref(&self, buf: &mut sqlx::postgres::PgArgumentBuffer) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        let s = match self {
            Self::Enabled => "enabled",
            Self::Disabled => "disabled",
        };
        <&str as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(&s, buf)
    }
}

impl sqlx::Decode<'_, sqlx::Postgres> for CountryStatus {
    fn decode(value: sqlx::postgres::PgValueRef<'_>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <&str as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        match s {
            "enabled" => Ok(Self::Enabled),
            "disabled" => Ok(Self::Disabled),
            _ => Err(format!("Invalid CountryStatus: {}", s).into()),
        }
    }
}

impl sqlx::Type<sqlx::Postgres> for CountryStatus {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <String as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

impl From<CountryStatus> for core_db::common::sql::BindValue {
    fn from(v: CountryStatus) -> Self {
        let s = match v {
            CountryStatus::Enabled => "enabled",
            CountryStatus::Disabled => "disabled",
        };
        core_db::common::sql::BindValue::String(s.to_string())
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub enum PersonalAccessTokenKind {
    #[serde(rename = "access")]
    Access,
    #[serde(rename = "refresh")]
    Refresh
}

impl Default for PersonalAccessTokenKind {
    fn default() -> Self {
        Self::Access
    }
}

impl ts_rs::TS for PersonalAccessTokenKind {
    type WithoutGenerics = Self;

    fn name() -> String {
        "PersonalAccessTokenKind".to_string()
    }

    fn inline() -> String {
        Self::name()
    }

    fn inline_flattened() -> String {
        panic!("PersonalAccessTokenKind cannot be flattened")
    }

    fn decl() -> String {
        "type PersonalAccessTokenKind = \"access\" | \"refresh\";".to_string()
    }

    fn decl_concrete() -> String {
        Self::decl()
    }
}

impl PersonalAccessTokenKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Access => "access",
            Self::Refresh => "refresh",
        }
    }

    pub const fn as_label(self) -> &'static str {
        match self {
            Self::Access => "Access",
            Self::Refresh => "Refresh",
        }
    }

    pub fn from_storage(raw: &str) -> Option<Self> {
        match raw.trim() {
            "access" => Some(Self::Access),
            "refresh" => Some(Self::Refresh),
            _ => None,
        }
    }

    pub const fn i18n_key(self) -> &'static str {
        match self {
            Self::Access => "enum.personal_access_token_kind.access",
            Self::Refresh => "enum.personal_access_token_kind.refresh",
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
        &[Self::Access, Self::Refresh]
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

impl sqlx::Encode<'_, sqlx::Postgres> for PersonalAccessTokenKind {
    fn encode_by_ref(&self, buf: &mut sqlx::postgres::PgArgumentBuffer) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        let s = match self {
            Self::Access => "access",
            Self::Refresh => "refresh",
        };
        <&str as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(&s, buf)
    }
}

impl sqlx::Decode<'_, sqlx::Postgres> for PersonalAccessTokenKind {
    fn decode(value: sqlx::postgres::PgValueRef<'_>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <&str as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        match s {
            "access" => Ok(Self::Access),
            "refresh" => Ok(Self::Refresh),
            _ => Err(format!("Invalid PersonalAccessTokenKind: {}", s).into()),
        }
    }
}

impl sqlx::Type<sqlx::Postgres> for PersonalAccessTokenKind {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <String as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

impl From<PersonalAccessTokenKind> for core_db::common::sql::BindValue {
    fn from(v: PersonalAccessTokenKind) -> Self {
        let s = match v {
            PersonalAccessTokenKind::Access => "access",
            PersonalAccessTokenKind::Refresh => "refresh",
        };
        core_db::common::sql::BindValue::String(s.to_string())
    }
}



pub const SCHEMA_ENUM_TS_META: &[SchemaEnumTsMeta] = &[
    SchemaEnumTsMeta { name: "AdminType", variants: &["developer", "superadmin", "admin"] },
    SchemaEnumTsMeta { name: "ContentPageSystemFlag", variants: &["0", "1"] },
    SchemaEnumTsMeta { name: "CountryStatus", variants: &["enabled", "disabled"] },
    SchemaEnumTsMeta { name: "PersonalAccessTokenKind", variants: &["access", "refresh"] },
];
