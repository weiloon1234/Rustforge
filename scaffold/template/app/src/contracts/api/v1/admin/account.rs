use crate::contracts::types::username::UsernameString;
use core_web::contracts::rustforge_contract;
use core_web::ids::SnowflakeId;
use core_web::Patch;
use generated::{
    extensions::admin::types::AdminViewComputedExt, models::AdminType, permissions::Permission,
};
use schemars::JsonSchema;
use serde::Serialize;
use ts_rs::TS;
use validator::Validate;

#[rustforge_contract]
#[derive(TS)]
#[ts(export, export_to = "admin/types/")]
pub struct CreateAdminInput {
    #[rf(nested)]
    #[rf(async_unique(table = "admin", column = "username"))]
    pub username: UsernameString,
    #[serde(default)]
    #[rf(email)]
    pub email: Option<String>,
    #[rf(length(min = 1, max = 120))]
    pub name: String,
    #[rf(length(min = 8, max = 128))]
    pub password: String,
    #[serde(default)]
    pub abilities: Vec<Permission>,
}

#[rustforge_contract]
#[derive(TS)]
#[ts(export, export_to = "admin/types/")]
pub struct UpdateAdminInput {
    #[serde(default)]
    pub id: SnowflakeId,
    #[serde(default)]
    #[rf(nested)]
    #[rf(async_unique(table = "admin", column = "username", ignore = "id"))]
    pub username: Option<UsernameString>,
    #[serde(default)]
    #[rf(email)]
    pub email: Patch<String>,
    #[serde(default)]
    #[rf(length(min = 1, max = 120))]
    pub name: Option<String>,
    #[serde(default)]
    #[rf(length(min = 8, max = 128))]
    pub password: Option<String>,
    #[serde(default)]
    pub abilities: Option<Vec<Permission>>,
}

#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct AdminOutput {
    pub id: SnowflakeId,
    pub identity: String,
    pub username: String,
    pub email: Option<String>,
    pub name: String,
    pub admin_type: AdminType,
    #[serde(default)]
    pub abilities: Vec<Permission>,
    #[schemars(with = "String")]
    #[ts(type = "string")]
    pub created_at: time::OffsetDateTime,
    #[schemars(with = "String")]
    #[ts(type = "string")]
    pub updated_at: time::OffsetDateTime,
}

impl From<generated::models::AdminView> for AdminOutput {
    fn from(value: generated::models::AdminView) -> Self {
        let abilities = value
            .abilities
            .as_array()
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.as_str())
                    .filter_map(Permission::from_str)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        Self {
            id: value.id.into(),
            identity: value.identity(),
            username: value.username,
            email: value.email,
            name: value.name,
            admin_type: value.admin_type,
            abilities,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct AdminDeleteOutput {
    pub deleted: bool,
}

#[derive(Debug, Clone, serde::Deserialize, JsonSchema)]
pub struct AdminBatchResolveInput {
    pub ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct AdminBatchResolveOutput {
    #[ts(inline)]
    pub entries: Vec<AdminBatchResolveEntry>,
}

#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct AdminBatchResolveEntry {
    pub id: SnowflakeId,
    pub username: String,
    pub name: String,
}
