use std::collections::BTreeMap;

use ts_rs::TS;

use crate::auth::AuthClientType;
use crate::datatable::{
    DataTableColumnMetaDto, DataTableDefaultsDto, DataTableDiagnosticsDto,
    DataTableEmailExportQueuedDto, DataTableEmailExportRequestBase, DataTableEmailExportState,
    DataTableEmailExportStatusDto, DataTableExportStatusResponseDto, DataTableFilterFieldDto,
    DataTableFilterFieldType, DataTableFilterOptionDto, DataTableMetaDto,
    DataTablePaginationModeDto, DataTableQueryRequestBase, DataTableRelationColumnMetaDto,
    DataTableSortDirectionDto,
};

#[derive(Debug, Clone)]
pub struct TsExportFile {
    pub rel_path: &'static str,
    pub rust_path: &'static str,
    pub definition: String,
}

#[derive(TS)]
#[ts(rename = "ApiErrorResponse")]
#[allow(dead_code)]
struct ApiErrorResponseTs {
    pub message: String,
    #[ts(optional)]
    pub errors: Option<BTreeMap<String, Vec<String>>>,
}

pub fn ts_export_files() -> Vec<TsExportFile> {
    vec![
        TsExportFile {
            rel_path: "shared/types/api.ts",
            rust_path: "core_web::ts_exports::api",
            definition: render_api_ts(),
        },
        TsExportFile {
            rel_path: "shared/types/datatable.ts",
            rust_path: "core_web::ts_exports::datatable",
            definition: render_datatable_ts(),
        },
    ]
}

pub fn contract_enum_renderers() -> Vec<(String, String)> {
    vec![(
        "AuthClientType".to_string(),
        export_decl(AuthClientType::decl()),
    )]
}

fn render_api_ts() -> String {
    let mut out = String::new();
    out.push_str("export interface ApiResponse<T> {\n");
    out.push_str("  data: T;\n");
    out.push_str("  message?: string;\n");
    out.push_str("}\n\n");
    out.push_str(&export_decl(ApiErrorResponseTs::decl()));
    out
}

fn render_datatable_ts() -> String {
    [
        export_decl(DataTablePaginationModeDto::decl()),
        export_decl(DataTableSortDirectionDto::decl()),
        export_decl(DataTableQueryRequestBase::decl()),
        export_decl(DataTableEmailExportRequestBase::decl()),
        export_decl(DataTableFilterFieldType::decl()),
        export_decl(DataTableFilterOptionDto::decl()),
        export_decl(DataTableFilterFieldDto::decl()),
        export_decl(DataTableColumnMetaDto::decl()),
        export_decl(DataTableRelationColumnMetaDto::decl()),
        export_decl(DataTableDefaultsDto::decl()),
        export_decl(DataTableMetaDto::decl()),
        export_decl(DataTableDiagnosticsDto::decl()),
        "export interface DataTableQueryResponse<T> {\n  records: T[];\n  per_page: number;\n  total_records: number;\n  total_pages: number;\n  page: number;\n  pagination_mode: string;\n  has_more?: boolean;\n  next_cursor?: string;\n  summary?: unknown;\n  diagnostics: DataTableDiagnosticsDto;\n  meta?: DataTableMetaDto;\n}".to_string(),
        export_decl(DataTableEmailExportState::decl()),
        export_decl(DataTableEmailExportStatusDto::decl()),
        export_decl(DataTableEmailExportQueuedDto::decl()),
        export_decl(DataTableExportStatusResponseDto::decl()),
    ]
    .join("\n\n")
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
