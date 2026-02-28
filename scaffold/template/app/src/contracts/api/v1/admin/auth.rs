use crate::contracts::types::username::UsernameString;
use core_web::auth::AuthClientType;
use core_web::contracts::rustforge_contract;
use generated::models::AdminType;
use schemars::JsonSchema;
use serde::Serialize;
use ts_rs::TS;
use validator::Validate;

#[rustforge_contract]
#[derive(TS)]
#[ts(export, export_to = "admin/types/")]
pub struct AdminLoginInput {
    #[rf(nested)]
    #[ts(type = "string")]
    pub username: UsernameString,

    #[rf(length(min = 8, max = 128))]
    pub password: String,

    #[ts(type = "AuthClientType")]
    pub client_type: AuthClientType,
}

#[rustforge_contract]
#[derive(TS)]
#[ts(export, export_to = "admin/types/")]
pub struct AdminRefreshInput {
    #[ts(type = "AuthClientType")]
    pub client_type: AuthClientType,
    #[serde(default)]
    #[rf(length(min = 1, max = 256))]
    pub refresh_token: Option<String>,
}

#[rustforge_contract]
#[derive(TS)]
#[ts(export, export_to = "admin/types/")]
pub struct AdminLogoutInput {
    #[ts(type = "AuthClientType")]
    pub client_type: AuthClientType,
    #[serde(default)]
    #[rf(length(min = 1, max = 256))]
    pub refresh_token: Option<String>,
}

#[rustforge_contract]
#[derive(TS)]
#[ts(export, export_to = "admin/types/")]
pub struct AdminProfileUpdateInput {
    #[rf(length(min = 1, max = 120))]
    pub name: String,
    #[serde(default)]
    #[rf(email)]
    pub email: Option<String>,
}

#[rustforge_contract]
#[derive(TS)]
#[ts(export, export_to = "admin/types/")]
pub struct AdminPasswordUpdateInput {
    #[rf(length(min = 8, max = 128))]
    pub current_password: String,
    #[rf(length(min = 8, max = 128))]
    #[rf(must_match(other = "password_confirmation"))]
    pub password: String,
    #[rf(length(min = 8, max = 128))]
    pub password_confirmation: String,
}

#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct AdminAuthOutput {
    pub token_type: String,
    pub access_token: String,
    #[schemars(with = "Option<String>")]
    #[ts(type = "string | null")]
    pub access_expires_at: Option<time::OffsetDateTime>,
    pub refresh_token: Option<String>,
    #[serde(default)]
    pub scopes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct AdminMeOutput {
    pub id: i64,
    pub username: String,
    pub email: Option<String>,
    pub name: String,
    #[ts(type = "AdminType")]
    pub admin_type: AdminType,
    #[serde(default)]
    pub scopes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct AdminProfileUpdateOutput {
    pub id: i64,
    pub username: String,
    pub email: Option<String>,
    pub name: String,
    #[ts(type = "AdminType")]
    pub admin_type: AdminType,
}

#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct AdminPasswordUpdateOutput {
    pub updated: bool,
}

#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct AdminLogoutOutput {
    pub revoked: bool,
}
