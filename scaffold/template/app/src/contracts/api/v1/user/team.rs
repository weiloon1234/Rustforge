use core_web::ids::SnowflakeId;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Deserialize, JsonSchema, TS)]
#[ts(export, export_to = "user/types/")]
pub struct DownlinesQuery {
    #[serde(default)]
    pub parent_user_id: Option<SnowflakeId>,
}

#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[ts(export, export_to = "user/types/")]
pub struct DownlineNode {
    pub id: SnowflakeId,
    pub uuid: String,
    pub username: String,
    pub name: Option<String>,
    #[ts(type = "number")]
    pub downline_count: i64,
}

#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[ts(export, export_to = "user/types/")]
pub struct DownlinesOutput {
    pub parent_username: String,
    pub parent_name: Option<String>,
    #[ts(inline)]
    pub downlines: Vec<DownlineNode>,
}
