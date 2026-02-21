use std::collections::{BTreeSet, HashMap, HashSet};
use std::sync::Arc;

use async_trait::async_trait;
use axum::body::Body;
use axum::extract::{Query, State};
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::Response;
use bytes::Bytes;
use core_datatable::{
    DataTableAsyncExportManager, DataTableAsyncExportState, DataTableContext, DataTableExecution,
    DataTableExportMode, DataTableInput, DataTableRegistry,
};
use core_db::infra::storage::Storage;
use core_mailer::MailPayload;
use schemars::JsonSchema;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};
use tokio_util::io::ReaderStream;
use validator::Validate;

use crate::contracts::{ContractJson, RequestContract, ResponseContract};
use crate::error::AppError;
use crate::extract::request_headers::RequestHeaders;
use crate::openapi::{
    aide::{
        axum::routing::{get_with, post_with},
        transform::TransformOperation,
    },
    require_bearer_auth, with_route_notes, ApiRouter,
};
use crate::response::ApiResponse;

pub const DEFAULT_DATATABLE_PREFIX: &str = "/api/v1/admin/datatable";

#[derive(Debug, Clone)]
pub struct DataTableRouteOptions {
    pub require_bearer_auth: bool,
}

impl Default for DataTableRouteOptions {
    fn default() -> Self {
        Self {
            require_bearer_auth: false,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DataTablePaginationModeDto {
    Offset,
    Cursor,
}

impl DataTablePaginationModeDto {
    fn to_core(self) -> core_datatable::DataTablePaginationMode {
        match self {
            Self::Offset => core_datatable::DataTablePaginationMode::Offset,
            Self::Cursor => core_datatable::DataTablePaginationMode::Cursor,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DataTableSortDirectionDto {
    Asc,
    Desc,
}

impl DataTableSortDirectionDto {
    fn to_core(self) -> core_datatable::SortDirection {
        match self {
            Self::Asc => core_datatable::SortDirection::Asc,
            Self::Desc => core_datatable::SortDirection::Desc,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Validate, JsonSchema)]
pub struct DataTableQueryRequestBase {
    #[serde(default = "default_include_meta")]
    pub include_meta: bool,
    #[serde(default)]
    #[validate(range(min = 1))]
    #[schemars(range(min = 1))]
    pub page: Option<i64>,
    #[serde(default)]
    #[validate(range(min = 1, max = 500))]
    #[schemars(range(min = 1, max = 500))]
    pub per_page: Option<i64>,
    #[serde(default)]
    #[validate(length(min = 1, max = 256))]
    #[schemars(length(min = 1, max = 256))]
    pub cursor: Option<String>,
    #[serde(default)]
    pub pagination_mode: Option<DataTablePaginationModeDto>,
    #[serde(default)]
    #[validate(length(min = 1, max = 128))]
    #[schemars(length(min = 1, max = 128))]
    pub sorting_column: Option<String>,
    #[serde(default)]
    pub sorting: Option<DataTableSortDirectionDto>,
    #[serde(default)]
    #[validate(length(min = 1, max = 64))]
    #[schemars(length(min = 1, max = 64))]
    pub timezone: Option<String>,
    #[serde(default)]
    #[validate(length(min = 1, max = 40))]
    #[schemars(length(min = 1, max = 40))]
    pub created_at_from: Option<String>,
    #[serde(default)]
    #[validate(length(min = 1, max = 40))]
    #[schemars(length(min = 1, max = 40))]
    pub created_at_to: Option<String>,
}

impl Default for DataTableQueryRequestBase {
    fn default() -> Self {
        Self {
            include_meta: true,
            page: None,
            per_page: None,
            cursor: None,
            pagination_mode: None,
            sorting_column: None,
            sorting: None,
            timezone: None,
            created_at_from: None,
            created_at_to: None,
        }
    }
}

impl DataTableQueryRequestBase {
    pub fn to_input(&self) -> DataTableInput {
        let mut input = DataTableInput::default();
        if let Some(page) = self.page {
            input.page = page;
        }
        if let Some(per_page) = self.per_page {
            input.ipp = per_page;
        }
        input.cursor = self.cursor.clone();
        if let Some(mode) = self.pagination_mode {
            input.pagination_mode = mode.to_core();
        }
        input.sorting_column = self.sorting_column.clone();
        input.sorting = self.sorting.map(DataTableSortDirectionDto::to_core);
        input.timezone = self.timezone.clone();
        if let Some(from) = self
            .created_at_from
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            input
                .params
                .insert("f-date-from-created_at".to_string(), from.to_string());
        }
        if let Some(to) = self
            .created_at_to
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            input
                .params
                .insert("f-date-to-created_at".to_string(), to.to_string());
        }
        input
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Validate, JsonSchema)]
pub struct DataTableEmailExportRequestBase {
    #[validate(nested)]
    pub query: DataTableQueryRequestBase,
    #[validate(length(min = 1, max = 20))]
    #[schemars(length(min = 1, max = 20))]
    pub recipients: Vec<String>,
    #[serde(default)]
    #[validate(length(min = 1, max = 160))]
    #[schemars(length(min = 1, max = 160))]
    pub subject: Option<String>,
    #[serde(default)]
    #[validate(length(min = 1, max = 120))]
    #[schemars(length(min = 1, max = 120))]
    pub export_file_name: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DataTableFilterFieldType {
    Text,
    Select,
    Number,
    Date,
    Datetime,
    Boolean,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DataTableFilterOptionDto {
    pub label: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DataTableFilterFieldDto {
    pub field: String,
    pub filter_key: String,
    #[serde(rename = "type")]
    pub field_type: DataTableFilterFieldType,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<DataTableFilterOptionDto>>,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct DataTableColumnMetaDto {
    pub name: String,
    pub data_type: String,
    pub sortable: bool,
    pub localized: bool,
    pub filter_ops: Vec<String>,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct DataTableRelationColumnMetaDto {
    pub relation: String,
    pub column: String,
    pub data_type: String,
    pub filter_ops: Vec<String>,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct DataTableDefaultsDto {
    pub sorting_column: String,
    pub sorted: String,
    pub per_page: i64,
    pub export_ignore_columns: Vec<String>,
    pub timestamp_columns: Vec<String>,
    pub unsortable: Vec<String>,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct DataTableMetaDto {
    pub model_key: String,
    pub defaults: DataTableDefaultsDto,
    pub columns: Vec<DataTableColumnMetaDto>,
    pub relation_columns: Vec<DataTableRelationColumnMetaDto>,
    pub filter_rows: Vec<Vec<DataTableFilterFieldDto>>,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct DataTableDiagnosticsDto {
    pub duration_ms: u64,
    pub auto_filters_applied: usize,
    pub unknown_filters: Vec<String>,
    pub unknown_filter_mode: String,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
#[schemars(bound = "Row: JsonSchema")]
pub struct DataTableQueryResponseDto<Row>
where
    Row: Serialize + JsonSchema,
{
    pub records: Vec<Row>,
    pub per_page: i64,
    pub total_records: i64,
    pub total_pages: i64,
    pub page: i64,
    pub pagination_mode: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_more: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
    pub diagnostics: DataTableDiagnosticsDto,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<DataTableMetaDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DataTableEmailExportState {
    WaitingCsv,
    Uploading,
    Sending,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DataTableEmailExportStatusDto {
    pub state: DataTableEmailExportState,
    pub recipients: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub updated_at_unix: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sent_at_unix: Option<i64>,
}

#[derive(Default, Clone)]
pub struct DataTableEmailExportManager {
    jobs: Arc<RwLock<HashMap<String, DataTableEmailExportStatusDto>>>,
}

impl DataTableEmailExportManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn init(&self, job_id: &str, recipients: Vec<String>, subject: Option<String>) {
        let mut jobs = self.jobs.write().await;
        jobs.insert(
            job_id.to_string(),
            DataTableEmailExportStatusDto {
                state: DataTableEmailExportState::WaitingCsv,
                recipients,
                subject,
                link_url: None,
                error: None,
                updated_at_unix: now_unix(),
                sent_at_unix: None,
            },
        );
    }

    pub async fn status(&self, job_id: &str) -> Option<DataTableEmailExportStatusDto> {
        let jobs = self.jobs.read().await;
        jobs.get(job_id).cloned()
    }

    pub async fn set_state(
        &self,
        job_id: &str,
        state: DataTableEmailExportState,
        link_url: Option<String>,
        error: Option<String>,
    ) {
        let mut jobs = self.jobs.write().await;
        if let Some(job) = jobs.get_mut(job_id) {
            job.state = state;
            job.updated_at_unix = now_unix();
            if let Some(link) = link_url {
                job.link_url = Some(link);
            }
            job.error = error;
            if matches!(job.state, DataTableEmailExportState::Completed) {
                job.sent_at_unix = Some(now_unix());
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct DataTableEmailExportQueuedDto {
    pub job_id: String,
    pub csv_state: String,
    pub email_state: DataTableEmailExportState,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct DataTableExportStatusResponseDto {
    pub job_id: String,
    pub model_key: String,
    pub csv_state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub csv_error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub csv_file_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub csv_content_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub csv_total_records: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<DataTableEmailExportStatusDto>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct DataTableExportStatusQueryDto {
    #[serde(default)]
    pub job_id: Option<String>,
    #[serde(default)]
    pub id: Option<String>,
}

#[async_trait]
pub trait DataTableRouteState: Clone + Send + Sync + 'static {
    fn datatable_registry(&self) -> &Arc<DataTableRegistry>;
    fn datatable_async_exports(&self) -> &Arc<DataTableAsyncExportManager>;
    fn datatable_storage(&self) -> &Arc<dyn Storage>;
    fn datatable_mailer(&self) -> &Arc<core_mailer::Mailer>;
    fn datatable_email_exports(&self) -> &Arc<DataTableEmailExportManager>;
    fn datatable_export_link_ttl_secs(&self) -> u64;
    async fn datatable_context(&self, headers: &HeaderMap) -> DataTableContext;
}

pub trait DataTableScopedContract: Clone + Send + Sync + 'static {
    type QueryRequest: RequestContract;
    type EmailRequest: RequestContract;
    type Row: ResponseContract + DeserializeOwned + Send + Sync + 'static;

    fn scoped_key(&self) -> &'static str;

    fn query_to_input(&self, req: &Self::QueryRequest) -> DataTableInput;

    fn email_to_input(&self, req: &Self::EmailRequest) -> DataTableInput;

    fn email_recipients(&self, req: &Self::EmailRequest) -> Vec<String>;

    fn email_subject(&self, _req: &Self::EmailRequest) -> Option<String> {
        None
    }

    fn export_file_name(&self, _req: &Self::EmailRequest) -> Option<String> {
        None
    }

    fn include_meta(&self, _req: &Self::QueryRequest) -> bool {
        true
    }

    fn include_default_created_at_range(&self) -> bool {
        true
    }

    fn filter_rows(&self) -> Vec<Vec<DataTableFilterFieldDto>> {
        Vec::new()
    }
}

#[derive(Debug, Clone)]
struct ScopedDataTableState<S, C> {
    inner: S,
    contract: C,
    scoped_key: String,
}

pub fn routes_for_scoped_contract<S, C>(prefix: &str, state: S, contract: C) -> ApiRouter
where
    S: DataTableRouteState,
    C: DataTableScopedContract,
{
    routes_for_scoped_contract_with_options(
        prefix,
        state,
        contract,
        DataTableRouteOptions::default(),
    )
}

pub fn routes_for_scoped_contract_with_options<S, C>(
    prefix: &str,
    state: S,
    contract: C,
    options: DataTableRouteOptions,
) -> ApiRouter
where
    S: DataTableRouteState,
    C: DataTableScopedContract,
{
    let query_path = format!("{prefix}/query");
    let export_csv_path = format!("{prefix}/export/csv");
    let export_email_path = format!("{prefix}/export/email");
    let export_status_path = format!("{prefix}/export/status");
    let scoped_key = contract.scoped_key().trim().to_string();
    let op_scope = datatable_operation_scope(prefix, scoped_key.as_str());

    let state = ScopedDataTableState {
        inner: state,
        contract,
        scoped_key,
    };

    ApiRouter::new()
        .api_route(
            query_path.as_str(),
            post_with(query_scoped::<S, C>, |op| {
                datatable_operation(
                    op,
                    &format!("datatable_{}_query", op_scope),
                    "Datatable query",
                    "Execute datatable query and optionally include metadata.",
                    &["Use include_meta=true on first request to fetch filters + shape metadata."],
                    options.require_bearer_auth,
                )
            }),
        )
        .api_route(
            export_csv_path.as_str(),
            post_with(export_csv_scoped::<S, C>, |op| {
                datatable_operation(
                    op,
                    &format!("datatable_{}_export_csv", op_scope),
                    "Datatable CSV export",
                    "Generate and stream CSV export for scoped datatable query.",
                    &["Response content-type is text/csv."],
                    options.require_bearer_auth,
                )
            }),
        )
        .api_route(
            export_email_path.as_str(),
            post_with(export_email_scoped::<S, C>, |op| {
                datatable_operation(
                    op,
                    &format!("datatable_{}_export_email", op_scope),
                    "Datatable CSV email export",
                    "Queue CSV export email delivery (link-based).",
                    &[
                        "CSV is generated async, uploaded to storage, then email sends presigned download link.",
                    ],
                    options.require_bearer_auth,
                )
            }),
        )
        .api_route(
            export_status_path.as_str(),
            get_with(export_status_scoped::<S, C>, |op| {
                datatable_operation(
                    op,
                    &format!("datatable_{}_export_status", op_scope),
                    "Datatable export status",
                    "Get status for CSV export and email delivery by job id.",
                    &[],
                    options.require_bearer_auth,
                )
            }),
        )
        .with_state(state)
}

fn datatable_operation<'t>(
    op: TransformOperation<'t>,
    operation_id: &str,
    summary: &str,
    description: &str,
    extra_notes: &[&str],
    needs_bearer_auth: bool,
) -> TransformOperation<'t> {
    let mut notes = vec!["Framework-provided datatable route collection (core_web::datatable)."];
    notes.extend_from_slice(extra_notes);
    let op = op
        .id(operation_id)
        .summary(summary)
        .tag("datatable")
        .description(description);
    let op = if needs_bearer_auth {
        require_bearer_auth(op)
    } else {
        op
    };
    with_route_notes(op, &notes)
}

fn datatable_operation_scope(prefix: &str, scoped_key: &str) -> String {
    let raw = format!("{prefix}_{scoped_key}");
    let mut out = String::new();
    let mut last_was_sep = false;
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            last_was_sep = false;
        } else if !last_was_sep {
            out.push('_');
            last_was_sep = true;
        }
    }
    out.trim_matches('_').to_string()
}

async fn query_scoped<S, C>(
    State(state): State<ScopedDataTableState<S, C>>,
    headers: RequestHeaders,
    req: ContractJson<C::QueryRequest>,
) -> Result<ApiResponse<DataTableQueryResponseDto<C::Row>>, AppError>
where
    S: DataTableRouteState,
    C: DataTableScopedContract,
{
    let mut input = state.contract.query_to_input(&req.0);
    if !state.contract.include_default_created_at_range() {
        strip_default_created_at_filters(&mut input);
    }
    input = bind_scoped_key(input, state.scoped_key.as_str());
    input.export = DataTableExportMode::None;

    let ctx = state.inner.datatable_context(&headers).await;
    let exec = state
        .inner
        .datatable_registry()
        .execute(&input, &ctx)
        .await
        .map_err(map_datatable_error)?;

    let page = match exec {
        DataTableExecution::Page(page) => page,
        DataTableExecution::Csv(_) => {
            return Err(AppError::BadRequest(
                "Query endpoint does not support CSV export mode".to_string(),
            ));
        }
    };

    let records = decode_records::<C::Row>(page.records, state.scoped_key.as_str())?;

    let meta = if state.contract.include_meta(&req.0) {
        let describe = state
            .inner
            .datatable_registry()
            .describe(state.scoped_key.as_str(), &ctx)
            .map_err(map_datatable_error)?;
        Some(build_meta(&state.contract, describe))
    } else {
        None
    };

    let payload = DataTableQueryResponseDto {
        records,
        per_page: page.per_page,
        total_records: page.total_records,
        total_pages: page.total_pages,
        page: page.page,
        pagination_mode: pagination_mode_to_str(page.pagination_mode).to_string(),
        has_more: page.has_more,
        next_cursor: page.next_cursor,
        diagnostics: DataTableDiagnosticsDto {
            duration_ms: page.diagnostics.duration_ms,
            auto_filters_applied: page.diagnostics.auto_filters_applied,
            unknown_filters: page.diagnostics.unknown_filters,
            unknown_filter_mode: unknown_filter_mode_to_str(page.diagnostics.unknown_filter_mode)
                .to_string(),
        },
        meta,
    };

    Ok(ApiResponse::success(payload, "datatable query"))
}

async fn export_csv_scoped<S, C>(
    State(state): State<ScopedDataTableState<S, C>>,
    headers: RequestHeaders,
    req: ContractJson<C::QueryRequest>,
) -> Result<Response, AppError>
where
    S: DataTableRouteState,
    C: DataTableScopedContract,
{
    let mut input = state.contract.query_to_input(&req.0);
    if !state.contract.include_default_created_at_range() {
        strip_default_created_at_filters(&mut input);
    }
    input = bind_scoped_key(input, state.scoped_key.as_str());
    input.export = DataTableExportMode::Csv;

    let ctx = state.inner.datatable_context(&headers).await;
    let exec = state
        .inner
        .datatable_registry()
        .execute(&input, &ctx)
        .await
        .map_err(map_datatable_error)?;

    let csv = match exec {
        DataTableExecution::Csv(csv) => csv,
        DataTableExecution::Page(_) => {
            return Err(AppError::BadRequest(
                "CSV export endpoint requires export mode".to_string(),
            ));
        }
    };

    stream_csv_response(
        csv.file_path.as_str(),
        csv.file_name.as_str(),
        csv.content_type.as_str(),
    )
    .await
}

async fn export_email_scoped<S, C>(
    State(state): State<ScopedDataTableState<S, C>>,
    headers: RequestHeaders,
    req: ContractJson<C::EmailRequest>,
) -> Result<ApiResponse<DataTableEmailExportQueuedDto>, AppError>
where
    S: DataTableRouteState,
    C: DataTableScopedContract,
{
    let recipients = normalize_recipients(state.contract.email_recipients(&req.0));
    if recipients.is_empty() {
        return Err(AppError::BadRequest(
            "At least one valid recipient is required".to_string(),
        ));
    }

    let subject = state
        .contract
        .email_subject(&req.0)
        .unwrap_or_else(|| format!("Datatable export ({})", state.scoped_key));

    let mut input = state.contract.email_to_input(&req.0);
    if !state.contract.include_default_created_at_range() {
        strip_default_created_at_filters(&mut input);
    }
    input = bind_scoped_key(input, state.scoped_key.as_str());
    input.export = DataTableExportMode::Csv;
    if input.export_file_name.is_none() {
        input.export_file_name = state.contract.export_file_name(&req.0);
    }

    let ctx = state.inner.datatable_context(&headers).await;
    let ticket = state
        .inner
        .datatable_async_exports()
        .enqueue(input, ctx)
        .await
        .map_err(map_datatable_error)?;

    state
        .inner
        .datatable_email_exports()
        .init(&ticket.job_id, recipients.clone(), Some(subject.clone()))
        .await;

    spawn_email_delivery_task(
        state.inner.clone(),
        ticket.job_id.clone(),
        state.scoped_key.clone(),
        recipients,
        subject,
    );

    Ok(ApiResponse::success(
        DataTableEmailExportQueuedDto {
            job_id: ticket.job_id,
            csv_state: async_state_to_str(ticket.state).to_string(),
            email_state: DataTableEmailExportState::WaitingCsv,
        },
        "datatable email export queued",
    ))
}

async fn export_status_scoped<S, C>(
    State(state): State<ScopedDataTableState<S, C>>,
    Query(params): Query<DataTableExportStatusQueryDto>,
) -> Result<ApiResponse<DataTableExportStatusResponseDto>, AppError>
where
    S: DataTableRouteState,
    C: DataTableScopedContract,
{
    let job_id = params
        .job_id
        .or(params.id)
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| AppError::BadRequest("Missing export job id".to_string()))?;

    let csv_status = state
        .inner
        .datatable_async_exports()
        .status(&job_id)
        .await
        .ok_or_else(|| AppError::NotFound(format!("Unknown export job '{}'", job_id)))?;

    let email = state.inner.datatable_email_exports().status(&job_id).await;

    Ok(ApiResponse::success(
        DataTableExportStatusResponseDto {
            job_id,
            model_key: csv_status.model,
            csv_state: async_state_to_str(csv_status.state).to_string(),
            csv_error: csv_status.error,
            csv_file_name: csv_status.csv.as_ref().map(|csv| csv.file_name.clone()),
            csv_content_type: csv_status.csv.as_ref().map(|csv| csv.content_type.clone()),
            csv_total_records: csv_status.csv.as_ref().map(|csv| csv.total_records),
            email,
        },
        "datatable export status",
    ))
}

fn spawn_email_delivery_task<S>(
    state: S,
    job_id: String,
    scoped_key: String,
    recipients: Vec<String>,
    subject: String,
) where
    S: DataTableRouteState,
{
    tokio::spawn(async move {
        if let Err(err) = run_email_delivery_task(
            state.clone(),
            job_id.clone(),
            scoped_key,
            recipients,
            subject,
        )
        .await
        {
            state
                .datatable_email_exports()
                .set_state(
                    &job_id,
                    DataTableEmailExportState::Failed,
                    None,
                    Some(app_error_message(&err)),
                )
                .await;
        }
    });
}

async fn run_email_delivery_task<S>(
    state: S,
    job_id: String,
    scoped_key: String,
    recipients: Vec<String>,
    subject: String,
) -> Result<(), AppError>
where
    S: DataTableRouteState,
{
    loop {
        let csv_status = state.datatable_async_exports().status(&job_id).await;
        let Some(csv_status) = csv_status else {
            return Err(AppError::NotFound(format!(
                "Unknown export job '{}'",
                job_id
            )));
        };

        match csv_status.state {
            DataTableAsyncExportState::Queued | DataTableAsyncExportState::Running => {
                sleep(Duration::from_secs(1)).await;
            }
            DataTableAsyncExportState::Failed => {
                state
                    .datatable_email_exports()
                    .set_state(
                        &job_id,
                        DataTableEmailExportState::Failed,
                        None,
                        csv_status
                            .error
                            .or_else(|| Some("CSV export failed".to_string())),
                    )
                    .await;
                return Ok(());
            }
            DataTableAsyncExportState::Completed => {
                let Some(csv) = csv_status.csv else {
                    return Err(AppError::Internal(anyhow::anyhow!(
                        "CSV export completed without csv payload"
                    )));
                };

                state
                    .datatable_email_exports()
                    .set_state(&job_id, DataTableEmailExportState::Uploading, None, None)
                    .await;

                let file_data = tokio::fs::read(csv.file_path.as_str())
                    .await
                    .map_err(|err| {
                        AppError::Internal(anyhow::anyhow!(
                            "Failed to read csv export file '{}': {}",
                            csv.file_path,
                            err
                        ))
                    })?;

                let safe_name = sanitize_file_name(csv.file_name.as_str());
                let object_key = format!(
                    "datatable/exports/{}/{}/{}",
                    scoped_key.replace('.', "/"),
                    job_id,
                    safe_name
                );

                state
                    .datatable_storage()
                    .put(
                        object_key.as_str(),
                        Bytes::from(file_data),
                        csv.content_type.as_str(),
                    )
                    .await
                    .map_err(AppError::from)?;

                let link = state
                    .datatable_storage()
                    .presign_get(
                        object_key.as_str(),
                        state.datatable_export_link_ttl_secs().max(1),
                    )
                    .await
                    .map_err(AppError::from)?;

                state
                    .datatable_email_exports()
                    .set_state(
                        &job_id,
                        DataTableEmailExportState::Sending,
                        Some(link.clone()),
                        None,
                    )
                    .await;

                let body = format!(
                    "Datatable export is ready.\n\nModel: {}\nDownload: {}",
                    scoped_key, link
                );

                state
                    .datatable_mailer()
                    .queue_raw(MailPayload {
                        to: recipients,
                        subject,
                        body,
                    })
                    .await
                    .map_err(AppError::from)?;

                state
                    .datatable_email_exports()
                    .set_state(
                        &job_id,
                        DataTableEmailExportState::Completed,
                        Some(link),
                        None,
                    )
                    .await;

                return Ok(());
            }
        }
    }
}

fn bind_scoped_key(mut input: DataTableInput, scoped_key: &str) -> DataTableInput {
    input.model = Some(scoped_key.to_string());
    input
        .params
        .insert("model".to_string(), scoped_key.to_string());
    input
}

fn decode_records<Row>(records: Vec<Value>, scoped_key: &str) -> Result<Vec<Row>, AppError>
where
    Row: DeserializeOwned,
{
    let mut out = Vec::with_capacity(records.len());
    for value in records {
        let row = serde_json::from_value::<Row>(value).map_err(|err| {
            AppError::Internal(anyhow::anyhow!(
                "Datatable row shape mismatch for '{}': {}",
                scoped_key,
                err
            ))
        })?;
        out.push(row);
    }
    Ok(out)
}

fn build_meta<C>(contract: &C, describe: core_datatable::DataTableDescribe) -> DataTableMetaDto
where
    C: DataTableScopedContract,
{
    let mut filter_rows = contract.filter_rows();
    if contract.include_default_created_at_range() {
        inject_default_created_at_range(&mut filter_rows, &describe.columns);
    }

    DataTableMetaDto {
        model_key: describe.model,
        defaults: DataTableDefaultsDto {
            sorting_column: describe.defaults.sorting_column,
            sorted: sort_direction_to_str(describe.defaults.sorted).to_string(),
            per_page: describe.defaults.per_page,
            export_ignore_columns: describe.defaults.export_ignore_columns,
            timestamp_columns: describe.defaults.timestamp_columns,
            unsortable: describe.defaults.unsortable,
        },
        columns: describe
            .columns
            .into_iter()
            .map(|col| DataTableColumnMetaDto {
                name: col.name,
                data_type: col.data_type,
                sortable: col.sortable,
                localized: col.localized,
                filter_ops: col.filter_ops,
            })
            .collect(),
        relation_columns: describe
            .relation_columns
            .into_iter()
            .map(|rel| DataTableRelationColumnMetaDto {
                relation: rel.relation,
                column: rel.column,
                data_type: rel.data_type,
                filter_ops: rel.filter_ops,
            })
            .collect(),
        filter_rows,
    }
}

fn inject_default_created_at_range(
    filter_rows: &mut Vec<Vec<DataTableFilterFieldDto>>,
    columns: &[core_datatable::DataTableColumnMeta],
) {
    let has_created_at = columns.iter().any(|col| col.name == "created_at");
    if !has_created_at {
        return;
    }

    let mut existing_keys = HashSet::new();
    for row in filter_rows.iter() {
        for field in row {
            existing_keys.insert(field.filter_key.clone());
        }
    }

    let mut row = Vec::new();
    if !existing_keys.contains("f-date-from-created_at") {
        row.push(DataTableFilterFieldDto {
            field: "created_at_from".to_string(),
            filter_key: "f-date-from-created_at".to_string(),
            field_type: DataTableFilterFieldType::Datetime,
            label: "Created At From".to_string(),
            placeholder: Some("Start datetime".to_string()),
            description: None,
            options: None,
        });
    }

    if !existing_keys.contains("f-date-to-created_at") {
        row.push(DataTableFilterFieldDto {
            field: "created_at_to".to_string(),
            filter_key: "f-date-to-created_at".to_string(),
            field_type: DataTableFilterFieldType::Datetime,
            label: "Created At To".to_string(),
            placeholder: Some("End datetime".to_string()),
            description: None,
            options: None,
        });
    }

    if !row.is_empty() {
        filter_rows.push(row);
    }
}

fn strip_default_created_at_filters(input: &mut DataTableInput) {
    input.params.remove("f-date-from-created_at");
    input.params.remove("f-date-to-created_at");
}

fn normalize_recipients(recipients: Vec<String>) -> Vec<String> {
    let mut unique = BTreeSet::new();
    for recipient in recipients {
        let email = recipient.trim().to_ascii_lowercase();
        if !email.is_empty() {
            unique.insert(email);
        }
    }
    unique.into_iter().collect()
}

fn default_include_meta() -> bool {
    true
}

fn sanitize_file_name(file_name: &str) -> String {
    let fallback = "datatable-export.csv";
    let trimmed = file_name.trim();
    if trimmed.is_empty() {
        return fallback.to_string();
    }

    let mut out = String::with_capacity(trimmed.len());
    for ch in trimmed.chars() {
        let valid = ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-');
        out.push(if valid { ch } else { '_' });
    }

    let normalized = out
        .trim_matches(|c: char| c == '.' || c == '_' || c == '-')
        .to_string();

    if normalized.is_empty() {
        fallback.to_string()
    } else {
        normalized
    }
}

async fn stream_csv_response(
    file_path: &str,
    file_name: &str,
    content_type: &str,
) -> Result<Response, AppError> {
    let file = tokio::fs::File::open(file_path).await.map_err(|err| {
        AppError::Internal(anyhow::anyhow!(
            "Failed to open CSV export file '{}': {}",
            file_path,
            err
        ))
    })?;
    let stream = ReaderStream::new(file);

    let mut response = Response::new(Body::from_stream(stream));
    *response.status_mut() = StatusCode::OK;

    let content_type = HeaderValue::from_str(content_type)
        .unwrap_or_else(|_| HeaderValue::from_static("text/csv; charset=utf-8"));
    response
        .headers_mut()
        .insert(header::CONTENT_TYPE, content_type);

    if let Ok(disposition) = HeaderValue::from_str(&format!(
        "attachment; filename=\"{}\"",
        sanitize_file_name(file_name)
    )) {
        response
            .headers_mut()
            .insert(header::CONTENT_DISPOSITION, disposition);
    }

    Ok(response)
}

fn map_datatable_error(err: anyhow::Error) -> AppError {
    let msg = err.to_string();

    if msg.contains("Missing datatable model key")
        || msg.contains("Unknown datatable model")
        || msg.contains("Unknown datatable filter")
        || msg.contains("Permission denied")
        || msg.contains("Pagination mode")
    {
        return AppError::BadRequest(msg);
    }

    AppError::Internal(err)
}

fn sort_direction_to_str(dir: core_datatable::SortDirection) -> &'static str {
    match dir {
        core_datatable::SortDirection::Asc => "asc",
        core_datatable::SortDirection::Desc => "desc",
    }
}

fn pagination_mode_to_str(mode: core_datatable::DataTablePaginationMode) -> &'static str {
    match mode {
        core_datatable::DataTablePaginationMode::Offset => "offset",
        core_datatable::DataTablePaginationMode::Cursor => "cursor",
    }
}

fn unknown_filter_mode_to_str(mode: core_datatable::DataTableUnknownFilterMode) -> &'static str {
    match mode {
        core_datatable::DataTableUnknownFilterMode::Ignore => "ignore",
        core_datatable::DataTableUnknownFilterMode::Warn => "warn",
        core_datatable::DataTableUnknownFilterMode::Error => "error",
    }
}

fn async_state_to_str(state: DataTableAsyncExportState) -> &'static str {
    match state {
        DataTableAsyncExportState::Queued => "queued",
        DataTableAsyncExportState::Running => "running",
        DataTableAsyncExportState::Completed => "completed",
        DataTableAsyncExportState::Failed => "failed",
    }
}

fn now_unix() -> i64 {
    time::OffsetDateTime::now_utc().unix_timestamp()
}

fn app_error_message(err: &AppError) -> String {
    match err {
        AppError::Internal(inner) => inner.to_string(),
        AppError::NotFound(msg)
        | AppError::BadRequest(msg)
        | AppError::Unauthorized(msg)
        | AppError::Forbidden(msg)
        | AppError::TooManyRequests(msg)
        | AppError::UnprocessableEntity(msg) => msg.clone(),
        AppError::Validation { message, .. } => message.clone(),
    }
}
