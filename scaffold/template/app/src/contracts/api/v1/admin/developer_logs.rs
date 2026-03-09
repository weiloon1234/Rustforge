use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct LogFileEntry {
    pub filename: String,
    pub size_bytes: u64,
    pub modified_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct LogFileListOutput {
    pub files: Vec<LogFileEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct LogFileDeleteOutput {
    pub deleted: bool,
}
