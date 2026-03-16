use schemars::JsonSchema;
use serde::Serialize;
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct AdminNotificationCountsOutput {
    pub deposit: i64,
    pub withdrawal: i64,
}
