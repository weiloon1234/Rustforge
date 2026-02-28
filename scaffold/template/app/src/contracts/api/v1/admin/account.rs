use crate::contracts::types::username::UsernameString;
use core_web::contracts::rustforge_contract;
use generated::{models::AdminType, permissions::Permission};
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
    #[ts(type = "string")]
    pub username: UsernameString,
    #[serde(default)]
    #[rf(email)]
    pub email: Option<String>,
    #[rf(length(min = 1, max = 120))]
    pub name: String,
    #[rf(length(min = 8, max = 128))]
    pub password: String,
    #[serde(default)]
    #[ts(type = "Permission[]")]
    pub abilities: Vec<Permission>,
}

#[rustforge_contract]
#[derive(TS)]
#[ts(export, export_to = "admin/types/")]
pub struct UpdateAdminInput {
    #[serde(skip, default)]
    __target_id: i64,
    #[serde(default)]
    #[rf(nested)]
    #[rf(async_unique(
        table = "admin",
        column = "username",
        ignore(column = "id", field = "__target_id")
    ))]
    #[ts(type = "string | null")]
    pub username: Option<UsernameString>,
    #[serde(default)]
    #[rf(email)]
    pub email: Option<String>,
    #[serde(default)]
    #[rf(length(min = 1, max = 120))]
    pub name: Option<String>,
    #[serde(default)]
    #[ts(type = "Permission[] | null")]
    pub abilities: Option<Vec<Permission>>,
}

impl UpdateAdminInput {
    pub fn with_target_id(mut self, id: i64) -> Self {
        self.__target_id = id;
        self
    }
}

#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct AdminOutput {
    pub id: i64,
    pub username: String,
    pub email: Option<String>,
    pub name: String,
    #[ts(type = "AdminType")]
    pub admin_type: AdminType,
    #[serde(default)]
    #[ts(type = "Permission[]")]
    pub abilities: Vec<String>,
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
                    .filter_map(|item| item.as_str().map(ToString::to_string))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        Self {
            id: value.id,
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
