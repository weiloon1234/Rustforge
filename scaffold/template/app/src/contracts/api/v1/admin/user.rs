use crate::contracts::types::username::UsernameString;
use core_web::contracts::rustforge_contract;
use core_web::ids::SnowflakeId;
use core_web::Patch;
use generated::models::UserBanStatus;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use validator::Validate;

#[rustforge_contract]
#[derive(TS)]
#[ts(export, export_to = "admin/types/")]
pub struct CreateUserInput {
    #[rf(nested)]
    #[rf(async_unique(table = "users", column = "username"))]
    pub username: UsernameString,
    #[serde(default)]
    #[rf(email)]
    pub email: Option<String>,
    #[serde(default)]
    #[rf(length(min = 1, max = 120))]
    pub name: Option<String>,
    #[rf(length(min = 8, max = 128))]
    pub password: String,
    #[serde(default)]
    pub country_iso2: Option<String>,
    #[serde(default)]
    pub contact_number: Option<String>,
    #[serde(default)]
    pub introducer_username: Option<String>,
}

#[rustforge_contract]
#[derive(TS)]
#[ts(export, export_to = "admin/types/")]
pub struct UpdateUserInput {
    #[serde(default)]
    pub id: SnowflakeId,
    #[serde(default)]
    #[rf(nested)]
    #[rf(async_unique(table = "users", column = "username", ignore = "id"))]
    pub username: Option<UsernameString>,
    #[serde(default)]
    #[rf(email)]
    pub email: Patch<String>,
    #[serde(default)]
    #[rf(length(min = 1, max = 120))]
    pub name: Patch<String>,
    #[serde(default)]
    #[rf(length(min = 8, max = 128))]
    pub password: Option<String>,
    #[serde(default)]
    pub country_iso2: Patch<String>,
    #[serde(default)]
    pub contact_number: Patch<String>,
}

impl CreateUserInput {
    pub fn normalize(mut self) -> Self {
        self.email = self.email.map(|v| v.to_ascii_lowercase());
        self.introducer_username = self.introducer_username.map(|v| v.to_ascii_lowercase());
        self
    }
}

impl UpdateUserInput {
    pub fn normalize(mut self) -> Self {
        self.email = self.email.map_value(|v| v.to_ascii_lowercase());
        self
    }
}

#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct UserManageOutput {
    pub id: SnowflakeId,
    pub uuid: String,
    pub username: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub locale: Option<String>,
    pub country_iso2: Option<String>,
    pub contact_number: Option<String>,
    pub ban: UserBanStatus,
    #[schemars(with = "String")]
    #[ts(type = "string")]
    pub created_at: time::OffsetDateTime,
    #[schemars(with = "String")]
    #[ts(type = "string")]
    pub updated_at: time::OffsetDateTime,
}

impl From<generated::models::UserRecord> for UserManageOutput {
    fn from(value: generated::models::UserRecord) -> Self {
        Self {
            id: value.id.into(),
            uuid: value.uuid,
            username: value.username,
            name: value.name,
            email: value.email,
            locale: value.locale,
            country_iso2: value.country_iso2,
            contact_number: value.contact_number,
            ban: value.ban,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct UserBanInput {
    pub ban: UserBanStatus,
}

#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct UserBanOutput {
    pub banned: bool,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct BatchResolveInput {
    pub ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct BatchResolveOutput {
    #[ts(inline)]
    pub entries: Vec<BatchResolveEntry>,
}

#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct BatchResolveEntry {
    pub id: SnowflakeId,
    pub username: String,
    pub name: Option<String>,
}
