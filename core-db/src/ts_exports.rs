use ts_rs::TS;

use crate::platform::attachments::types::{Attachment, AttachmentUploadDto};
use crate::platform::countries::types::{Country, CountryCurrency, CountryStatus};

#[derive(Debug, Clone)]
pub struct TsExportFile {
    pub rel_path: &'static str,
    pub rust_path: &'static str,
    pub definition: String,
}

pub fn ts_export_files() -> Vec<TsExportFile> {
    vec![TsExportFile {
        rel_path: "shared/types/platform.ts",
        rust_path: "core_db::ts_exports::platform",
        definition: render_platform_ts(),
    }]
}

fn render_platform_ts() -> String {
    let sections = vec![
        "export type JsonPrimitive = string | number | boolean | null;".to_string(),
        "export type JsonValue = JsonPrimitive | JsonObject | JsonValue[];".to_string(),
        "export interface JsonObject {\n  [key: string]: JsonValue;\n}".to_string(),
        "export type MetaRecord<TShape extends Record<string, unknown> = Record<string, JsonValue>> = Partial<TShape>;".to_string(),
        "export type MetaMap = Record<string, Record<number, JsonValue>>;".to_string(),
        export_decl(AttachmentUploadDto::decl()),
        "export type AttachmentInput = AttachmentUploadDto;".to_string(),
        export_decl(Attachment::decl()),
        "export type AttachmentMap = Record<string, Record<number, Attachment[]>>;".to_string(),
        export_decl(CountryStatus::decl()),
        export_decl(CountryCurrency::decl()),
        export_decl(Country::decl()),
    ];
    sections.join("\n\n")
}

fn export_decl(mut decl: String) -> String {
    if decl.trim_start().starts_with("export ") {
        return decl;
    }
    if decl.starts_with("interface ") || decl.starts_with("type ") {
        decl.insert_str(0, "export ");
        return decl;
    }
    format!("export {decl}")
}
