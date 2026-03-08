use core_web::ids::SnowflakeId;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Deserialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct ChangeIntroducerInput {
    pub user_username: String,
    pub new_introducer_username: String,
    #[serde(default)]
    pub remark: Option<String>,
}

#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct ChangeIntroducerOutput {
    pub id: SnowflakeId,
    pub user_id: SnowflakeId,
    pub from_user_id: Option<SnowflakeId>,
    pub to_user_id: SnowflakeId,
    pub admin_id: SnowflakeId,
    pub remark: Option<String>,
    #[schemars(with = "String")]
    #[ts(type = "string")]
    pub created_at: time::OffsetDateTime,
}
