use crate::contracts::types::username::UsernameString;
use core_web::auth::AuthClientType;
use core_web::contracts::rustforge_contract;
use core_web::ids::SnowflakeId;
use core_web::Patch;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use validator::Validate;

#[rustforge_contract]
#[derive(TS)]
#[ts(export, export_to = "user/types/")]
pub struct UserLoginInput {
    #[rf(nested)]
    pub username: UsernameString,

    #[rf(length(min = 8, max = 128))]
    pub password: String,

    pub client_type: AuthClientType,
}

#[rustforge_contract]
#[derive(TS)]
#[ts(export, export_to = "user/types/")]
pub struct UserRegisterInput {
    #[rf(nested)]
    pub username: UsernameString,

    #[rf(length(min = 8, max = 128))]
    #[rf(must_match(other = "password_confirmation"))]
    pub password: String,

    #[rf(length(min = 8, max = 128))]
    pub password_confirmation: String,

    #[serde(default)]
    #[rf(length(min = 1, max = 120))]
    pub name: Option<String>,

    #[serde(default)]
    #[rf(email)]
    pub email: Option<String>,

    #[serde(default)]
    #[rf(length(min = 2, max = 2))]
    pub country_iso2: Option<String>,

    #[serde(default)]
    #[rf(length(min = 1, max = 32))]
    pub contact_number: Option<String>,

    #[serde(default)]
    #[rf(length(min = 1, max = 64))]
    pub referral_code: Option<String>,

    pub client_type: AuthClientType,
}

impl UserRegisterInput {
    pub fn normalize(mut self) -> Self {
        self.country_iso2 = self.country_iso2.map(|v| v.to_ascii_uppercase());
        self
    }
}

#[rustforge_contract]
#[derive(TS)]
#[ts(export, export_to = "user/types/")]
pub struct UserRefreshInput {
    pub client_type: AuthClientType,
    #[serde(default)]
    #[rf(length(min = 1, max = 256))]
    pub refresh_token: Option<String>,
}

#[rustforge_contract]
#[derive(TS)]
#[ts(export, export_to = "user/types/")]
pub struct UserLogoutInput {
    pub client_type: AuthClientType,
    #[serde(default)]
    #[rf(length(min = 1, max = 256))]
    pub refresh_token: Option<String>,
}

#[rustforge_contract]
#[derive(TS)]
#[ts(export, export_to = "user/types/")]
pub struct UserProfileUpdateInput {
    #[serde(default)]
    #[rf(length(min = 1, max = 120))]
    pub name: Patch<String>,

    #[serde(default)]
    #[rf(email)]
    pub email: Patch<String>,

    #[serde(default)]
    #[rf(length(min = 2, max = 2))]
    pub country_iso2: Patch<String>,

    #[serde(default)]
    #[rf(length(min = 1, max = 32))]
    pub contact_number: Patch<String>,
}

impl UserProfileUpdateInput {
    pub fn normalize(mut self) -> Self {
        self.country_iso2 = self.country_iso2.map_value(|v| v.to_ascii_uppercase());
        self
    }
}

#[rustforge_contract]
#[derive(TS)]
#[ts(export, export_to = "user/types/")]
pub struct UserLocaleUpdateInput {
    #[rf(length(min = 2, max = 16))]
    pub locale: String,
}

#[rustforge_contract]
#[derive(TS)]
#[ts(export, export_to = "user/types/")]
pub struct UserPasswordUpdateInput {
    #[rf(length(min = 8, max = 128))]
    pub current_password: String,
    #[rf(length(min = 8, max = 128))]
    #[rf(must_match(other = "password_confirmation"))]
    pub password: String,
    #[rf(length(min = 8, max = 128))]
    pub password_confirmation: String,
}

#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[ts(export, export_to = "user/types/")]
pub struct UserAuthOutput {
    pub token_type: String,
    pub access_token: String,
    #[schemars(with = "Option<String>")]
    #[ts(type = "string | null")]
    pub access_expires_at: Option<time::OffsetDateTime>,
    pub refresh_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[ts(export, export_to = "user/types/")]
pub struct UserMeOutput {
    pub id: SnowflakeId,
    pub uuid: String,
    pub username: String,
    pub name: String,
    pub email: Option<String>,
    pub locale: Option<String>,
    pub country_iso2: Option<String>,
    pub contact_number: Option<String>,
}

#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[ts(export, export_to = "user/types/")]
pub struct UserProfileUpdateOutput {
    pub id: SnowflakeId,
    pub username: String,
    pub name: String,
    pub email: Option<String>,
    pub locale: Option<String>,
    pub country_iso2: Option<String>,
    pub contact_number: Option<String>,
}

#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[ts(export, export_to = "user/types/")]
pub struct UserLocaleUpdateOutput {
    pub locale: String,
}

#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[ts(export, export_to = "user/types/")]
pub struct UserPasswordUpdateOutput {
    pub updated: bool,
}

#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[ts(export, export_to = "user/types/")]
pub struct UserLogoutOutput {
    pub revoked: bool,
}

#[derive(Debug, Clone, Deserialize, JsonSchema, TS)]
#[ts(export, export_to = "user/types/")]
pub struct ResolveReferralQuery {
    #[serde(default)]
    pub code: Option<String>,
}

#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[ts(export, export_to = "user/types/")]
pub struct ResolveReferralOutput {
    pub username: String,
    pub name: Option<String>,
}
