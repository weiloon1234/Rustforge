// AUTO-GENERATED FILE — DO NOT EDIT
// Generated from TOML schema enum definitions

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum AdminType {
    Developer,
    SuperAdmin,
    Admin
}

impl Default for AdminType {
    fn default() -> Self {
        Self::Developer
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


