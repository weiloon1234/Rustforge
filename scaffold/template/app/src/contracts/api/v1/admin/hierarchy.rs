use core_web::ids::SnowflakeId;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ResolveUserQuery {
    pub username: String,
}

#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct ResolvedUser {
    pub id: SnowflakeId,
    pub username: String,
    pub name: Option<String>,
    pub introducer_user_id: Option<SnowflakeId>,
}

#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct AdminDownlineNode {
    pub id: SnowflakeId,
    pub uuid: String,
    pub username: String,
    pub name: Option<String>,
    #[ts(type = "number")]
    pub downline_count: i64,
}

#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct AdminDownlinesOutput {
    pub parent_id: SnowflakeId,
    pub parent_username: String,
    pub parent_name: Option<String>,
    pub parent_introducer_user_id: Option<SnowflakeId>,
    #[ts(inline)]
    pub downlines: Vec<AdminDownlineNode>,
}
