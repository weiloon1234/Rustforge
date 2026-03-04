use schemars::JsonSchema;
use serde::Serialize;
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct AdminTiptapImageUploadOutput {
    pub folder: String,
    pub path: String,
    pub url: String,
    pub content_type: String,
    #[ts(type = "number")]
    pub size: i64,
    pub width: Option<i32>,
    pub height: Option<i32>,
}
