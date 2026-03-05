// AUTO-GENERATED FILE — DO NOT EDIT
// Generated from TOML schema enum definitions

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

    pub const fn variants() -> &'static [Self] {
        &[Self::Developer, Self::SuperAdmin, Self::Admin]
    }

    pub fn datatable_filter_options() -> Vec<core_web::datatable::DataTableFilterOptionDto> {
        Self::variants()
            .iter()
            .map(|v| {
                let s = (*v).as_str();
                core_web::datatable::DataTableFilterOptionDto {
                    label: s.to_string(),
                    value: s.to_string(),
                }
            })
            .collect()
    }
}

// sqlx support for TEXT storage
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

// For ActiveRecord BindValue
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

    pub const fn variants() -> &'static [Self] {
        &[Self::No, Self::Yes]
    }

    pub fn datatable_filter_options() -> Vec<core_web::datatable::DataTableFilterOptionDto> {
        Self::variants()
            .iter()
            .map(|v| {
                let s = (*v).as_str();
                core_web::datatable::DataTableFilterOptionDto {
                    label: s.to_string(),
                    value: s.to_string(),
                }
            })
            .collect()
    }
}

// sqlx support for integer storage
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

// For ActiveRecord BindValue
impl From<ContentPageSystemFlag> for core_db::common::sql::BindValue {
    fn from(v: ContentPageSystemFlag) -> Self {
        core_db::common::sql::BindValue::I64(v as i64)
    }
}


