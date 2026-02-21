use std::collections::BTreeMap;
use std::sync::Arc;

use async_trait::async_trait;
use axum::body::Body;
use axum::extract::{Form, Multipart, Query, State};
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::Response;
use axum::Json;
use core_datatable::{
    DataTableAsyncExportManager, DataTableContext, DataTableExecution, DataTableExportMode,
    DataTableInput, DataTableRegistry,
};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{json, Value};
use tokio_util::io::ReaderStream;
use tracing::info;

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
use crate::utils::datatable::datatable_input_from_form;

pub const DEFAULT_DATATABLE_PREFIX: &str = "/admin/dt";

#[derive(Debug, Clone)]
pub struct DataTableRouteOptions {
    pub include_multipart_endpoints: bool,
    pub require_bearer_auth: bool,
}

impl Default for DataTableRouteOptions {
    fn default() -> Self {
        Self {
            include_multipart_endpoints: true,
            require_bearer_auth: false,
        }
    }
}

#[async_trait]
pub trait DataTableRouteState: Clone + Send + Sync + 'static {
    fn datatable_registry(&self) -> &Arc<DataTableRegistry>;
    fn datatable_async_exports(&self) -> &Arc<DataTableAsyncExportManager>;
    async fn datatable_context(&self, headers: &HeaderMap) -> DataTableContext;
}

#[derive(Debug, Clone, Deserialize, JsonSchema, Default)]
pub struct DataTableRequestDto {
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default, alias = "p")]
    pub page: Option<i64>,
    #[serde(default, alias = "per_page")]
    pub ipp: Option<i64>,
    #[serde(default)]
    pub cursor: Option<String>,
    #[serde(default, alias = "paginate_mode")]
    pub pagination_mode: Option<String>,
    #[serde(default)]
    pub sorting_column: Option<String>,
    #[serde(default)]
    pub sorting: Option<String>,
    #[serde(default)]
    pub export: Option<String>,
    #[serde(default)]
    pub export_file_name: Option<String>,
    #[serde(default)]
    pub timezone: Option<String>,
    #[serde(default, flatten)]
    pub params: BTreeMap<String, String>,
}

impl DataTableRequestDto {
    fn into_input(self) -> DataTableInput {
        let mut params = self.params;
        insert_opt_string(&mut params, "model", self.model);
        insert_opt_i64(&mut params, "p", self.page);
        insert_opt_i64(&mut params, "ipp", self.ipp);
        insert_opt_string(&mut params, "cursor", self.cursor);
        insert_opt_string(&mut params, "pagination_mode", self.pagination_mode);
        insert_opt_string(&mut params, "sorting_column", self.sorting_column);
        insert_opt_string(&mut params, "sorting", self.sorting);
        insert_opt_string(&mut params, "export", self.export);
        insert_opt_string(&mut params, "export_file_name", self.export_file_name);
        insert_opt_string(&mut params, "timezone", self.timezone);
        DataTableInput::from_pairs(params)
    }
}

#[derive(Debug, Clone, Deserialize, JsonSchema, Default)]
pub struct BoundDataTableRequestDto {
    #[serde(default, alias = "p")]
    pub page: Option<i64>,
    #[serde(default, alias = "per_page")]
    pub ipp: Option<i64>,
    #[serde(default)]
    pub cursor: Option<String>,
    #[serde(default, alias = "paginate_mode")]
    pub pagination_mode: Option<String>,
    #[serde(default)]
    pub sorting_column: Option<String>,
    #[serde(default)]
    pub sorting: Option<String>,
    #[serde(default)]
    pub export: Option<String>,
    #[serde(default)]
    pub export_file_name: Option<String>,
    #[serde(default)]
    pub timezone: Option<String>,
    #[serde(default, flatten)]
    pub params: BTreeMap<String, String>,
}

impl BoundDataTableRequestDto {
    fn into_input(self, model: &str) -> DataTableInput {
        let mut params = self.params;
        insert_opt_string(&mut params, "model", Some(model.to_string()));
        insert_opt_i64(&mut params, "p", self.page);
        insert_opt_i64(&mut params, "ipp", self.ipp);
        insert_opt_string(&mut params, "cursor", self.cursor);
        insert_opt_string(&mut params, "pagination_mode", self.pagination_mode);
        insert_opt_string(&mut params, "sorting_column", self.sorting_column);
        insert_opt_string(&mut params, "sorting", self.sorting);
        insert_opt_string(&mut params, "export", self.export);
        insert_opt_string(&mut params, "export_file_name", self.export_file_name);
        insert_opt_string(&mut params, "timezone", self.timezone);
        DataTableInput::from_pairs(params)
    }
}

#[derive(Debug, Clone)]
struct BoundDataTableState<S> {
    inner: S,
    model: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct DataTableDescribeQueryDto {
    pub model: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct DataTableExportStatusQueryDto {
    #[serde(default)]
    pub job_id: Option<String>,
    #[serde(default)]
    pub id: Option<String>,
}

pub fn routes<S>(state: S) -> ApiRouter
where
    S: DataTableRouteState,
{
    routes_with_prefix_and_options(
        DEFAULT_DATATABLE_PREFIX,
        state,
        DataTableRouteOptions::default(),
    )
}

pub fn routes_with_options<S>(state: S, options: DataTableRouteOptions) -> ApiRouter
where
    S: DataTableRouteState,
{
    routes_with_prefix_and_options(DEFAULT_DATATABLE_PREFIX, state, options)
}

pub fn routes_with_prefix<S>(prefix: &str, state: S) -> ApiRouter
where
    S: DataTableRouteState,
{
    routes_with_prefix_and_options(prefix, state, DataTableRouteOptions::default())
}

pub fn routes_with_prefix_and_options<S>(
    prefix: &str,
    state: S,
    options: DataTableRouteOptions,
) -> ApiRouter
where
    S: DataTableRouteState,
{
    let root = prefix.to_string();
    let describe = format!("{prefix}/describe");
    let export_stream = format!("{prefix}/export/stream");
    let export_async = format!("{prefix}/export/async");
    let export_async_form = format!("{prefix}/export/async/form");
    let export_async_json = format!("{prefix}/export/async/json");
    let export_status = format!("{prefix}/export/status");
    let form_path = format!("{prefix}/form");
    let json_path = format!("{prefix}/json");

    let root_route = {
        let route = get_with(load_datatable_query::<S>, |op| {
            datatable_operation(
                op,
                "datatable_execute_query",
                "Datatable execute (query)",
                "Execute datatable via query params.",
                &[],
                options.require_bearer_auth,
            )
        });
        if options.include_multipart_endpoints {
            route.post_with(load_datatable_multipart::<S>, |op| {
                datatable_operation(
                    op,
                    "datatable_execute_multipart",
                    "Datatable execute (multipart)",
                    "Execute datatable via multipart payload.",
                    &["Use /admin/dt/json for OpenAPI-friendly request schema docs."],
                    options.require_bearer_auth,
                )
            })
        } else {
            route
        }
    };

    let export_stream_route = {
        let route = get_with(stream_csv_query::<S>, |op| {
            datatable_operation(
                op,
                "datatable_csv_stream_query",
                "Datatable CSV stream (query)",
                "Stream CSV export directly.",
                &["Response content-type is text/csv."],
                options.require_bearer_auth,
            )
        });
        if options.include_multipart_endpoints {
            route.post_with(stream_csv_multipart::<S>, |op| {
                datatable_operation(
                    op,
                    "datatable_csv_stream_multipart",
                    "Datatable CSV stream (multipart)",
                    "Stream CSV export directly from multipart request.",
                    &[
                        "Response content-type is text/csv.",
                        "Use /admin/dt/json + export=csv for OpenAPI-friendly request schema docs.",
                    ],
                    options.require_bearer_auth,
                )
            })
        } else {
            route
        }
    };

    let export_async_route = {
        let route = get_with(queue_csv_export_query::<S>, |op| {
            datatable_operation(
                op,
                "datatable_csv_queue_query",
                "Datatable CSV queue (query)",
                "Queue async CSV export job.",
                &[],
                options.require_bearer_auth,
            )
        });
        if options.include_multipart_endpoints {
            route.post_with(queue_csv_export_multipart::<S>, |op| {
                datatable_operation(
                    op,
                    "datatable_csv_queue_multipart",
                    "Datatable CSV queue (multipart)",
                    "Queue async CSV export job from multipart payload.",
                    &["Use /admin/dt/export/async/json for OpenAPI-friendly request schema docs."],
                    options.require_bearer_auth,
                )
            })
        } else {
            route
        }
    };

    ApiRouter::new()
        .api_route(root.as_str(), root_route)
        .api_route(
            describe.as_str(),
            get_with(describe_datatable::<S>, |op| {
                datatable_operation(
                    op,
                    "datatable_describe",
                    "Datatable describe",
                    "Return datatable metadata (columns, filters, defaults).",
                    &[],
                    options.require_bearer_auth,
                )
            }),
        )
        .api_route(export_stream.as_str(), export_stream_route)
        .api_route(export_async.as_str(), export_async_route)
        .api_route(
            export_async_form.as_str(),
            post_with(queue_csv_export_form::<S>, |op| {
                datatable_operation(
                    op,
                    "datatable_csv_queue_form",
                    "Datatable CSV queue (form)",
                    "Queue async CSV export job from form payload.",
                    &[],
                    options.require_bearer_auth,
                )
            }),
        )
        .api_route(
            export_async_json.as_str(),
            post_with(queue_csv_export_json::<S>, |op| {
                datatable_operation(
                    op,
                    "datatable_csv_queue_json",
                    "Datatable CSV queue (json)",
                    "Queue async CSV export job from JSON payload.",
                    &[],
                    options.require_bearer_auth,
                )
            }),
        )
        .api_route(
            export_status.as_str(),
            get_with(async_export_status::<S>, |op| {
                datatable_operation(
                    op,
                    "datatable_csv_status",
                    "Datatable CSV status",
                    "Get async export job status.",
                    &[],
                    options.require_bearer_auth,
                )
            }),
        )
        .api_route(
            form_path.as_str(),
            post_with(load_datatable_form::<S>, |op| {
                datatable_operation(
                    op,
                    "datatable_execute_form",
                    "Datatable execute (form)",
                    "Execute datatable via form payload.",
                    &[],
                    options.require_bearer_auth,
                )
            }),
        )
        .api_route(
            json_path.as_str(),
            post_with(load_datatable_json::<S>, |op| {
                datatable_operation(
                    op,
                    "datatable_execute_json",
                    "Datatable execute (json)",
                    "Execute datatable via JSON payload.",
                    &[],
                    options.require_bearer_auth,
                )
            }),
        )
        .with_state(state)
}

pub fn routes_for_model<S>(prefix: &str, model: &str, state: S) -> ApiRouter
where
    S: DataTableRouteState,
{
    routes_for_model_with_options(prefix, model, state, DataTableRouteOptions::default())
}

pub fn routes_for_model_with_options<S>(
    prefix: &str,
    model: &str,
    state: S,
    options: DataTableRouteOptions,
) -> ApiRouter
where
    S: DataTableRouteState,
{
    let root = prefix.to_string();
    let describe = format!("{prefix}/describe");
    let export_stream = format!("{prefix}/export/stream");
    let export_async = format!("{prefix}/export/async");
    let export_async_form = format!("{prefix}/export/async/form");
    let export_async_json = format!("{prefix}/export/async/json");
    let export_status = format!("{prefix}/export/status");
    let form_path = format!("{prefix}/form");
    let json_path = format!("{prefix}/json");

    let op_scope = datatable_operation_scope(prefix, model);
    let model_name = model.trim().to_string();
    let bound_state = BoundDataTableState {
        inner: state,
        model: model_name.clone(),
    };

    let root_route = {
        let route = get_with(load_bound_datatable_query::<S>, |op| {
            datatable_operation(
                op,
                &format!("datatable_{}_execute_query", op_scope),
                &format!("{} datatable execute (query)", model_name),
                "Execute datatable via query params.",
                &[&format!("Model is bound to `{}` by route.", model_name)],
                options.require_bearer_auth,
            )
        });
        if options.include_multipart_endpoints {
            route.post_with(load_bound_datatable_multipart::<S>, |op| {
                datatable_operation(
                    op,
                    &format!("datatable_{}_execute_multipart", op_scope),
                    &format!("{} datatable execute (multipart)", model_name),
                    "Execute datatable via multipart payload.",
                    &[
                        &format!("Model is bound to `{}` by route.", model_name),
                        &format!(
                            "Use {}/json for OpenAPI-friendly request schema docs.",
                            prefix
                        ),
                    ],
                    options.require_bearer_auth,
                )
            })
        } else {
            route
        }
    };

    let export_stream_route = {
        let route = get_with(stream_bound_csv_query::<S>, |op| {
            datatable_operation(
                op,
                &format!("datatable_{}_csv_stream_query", op_scope),
                &format!("{} datatable CSV stream (query)", model_name),
                "Stream CSV export directly.",
                &[
                    "Response content-type is text/csv.",
                    &format!("Model is bound to `{}` by route.", model_name),
                ],
                options.require_bearer_auth,
            )
        });
        if options.include_multipart_endpoints {
            route.post_with(stream_bound_csv_multipart::<S>, |op| {
                datatable_operation(
                    op,
                    &format!("datatable_{}_csv_stream_multipart", op_scope),
                    &format!("{} datatable CSV stream (multipart)", model_name),
                    "Stream CSV export directly from multipart request.",
                    &[
                        "Response content-type is text/csv.",
                        &format!("Model is bound to `{}` by route.", model_name),
                        &format!(
                            "Use {}/json + export=csv for OpenAPI-friendly request schema docs.",
                            prefix
                        ),
                    ],
                    options.require_bearer_auth,
                )
            })
        } else {
            route
        }
    };

    let export_async_route = {
        let route = get_with(queue_bound_csv_export_query::<S>, |op| {
            datatable_operation(
                op,
                &format!("datatable_{}_csv_queue_query", op_scope),
                &format!("{} datatable CSV queue (query)", model_name),
                "Queue async CSV export job.",
                &[&format!("Model is bound to `{}` by route.", model_name)],
                options.require_bearer_auth,
            )
        });
        if options.include_multipart_endpoints {
            route.post_with(queue_bound_csv_export_multipart::<S>, |op| {
                datatable_operation(
                    op,
                    &format!("datatable_{}_csv_queue_multipart", op_scope),
                    &format!("{} datatable CSV queue (multipart)", model_name),
                    "Queue async CSV export job from multipart payload.",
                    &[
                        &format!("Model is bound to `{}` by route.", model_name),
                        &format!(
                            "Use {}/export/async/json for OpenAPI-friendly request schema docs.",
                            prefix
                        ),
                    ],
                    options.require_bearer_auth,
                )
            })
        } else {
            route
        }
    };

    ApiRouter::new()
        .api_route(root.as_str(), root_route)
        .api_route(
            describe.as_str(),
            get_with(describe_bound_datatable::<S>, |op| {
                datatable_operation(
                    op,
                    &format!("datatable_{}_describe", op_scope),
                    &format!("{} datatable describe", model_name),
                    "Return datatable metadata (columns, filters, defaults).",
                    &[&format!("Model is bound to `{}` by route.", model_name)],
                    options.require_bearer_auth,
                )
            }),
        )
        .api_route(export_stream.as_str(), export_stream_route)
        .api_route(export_async.as_str(), export_async_route)
        .api_route(
            export_async_form.as_str(),
            post_with(queue_bound_csv_export_form::<S>, |op| {
                datatable_operation(
                    op,
                    &format!("datatable_{}_csv_queue_form", op_scope),
                    &format!("{} datatable CSV queue (form)", model_name),
                    "Queue async CSV export job from form payload.",
                    &[&format!("Model is bound to `{}` by route.", model_name)],
                    options.require_bearer_auth,
                )
            }),
        )
        .api_route(
            export_async_json.as_str(),
            post_with(queue_bound_csv_export_json::<S>, |op| {
                datatable_operation(
                    op,
                    &format!("datatable_{}_csv_queue_json", op_scope),
                    &format!("{} datatable CSV queue (json)", model_name),
                    "Queue async CSV export job from JSON payload.",
                    &[&format!("Model is bound to `{}` by route.", model_name)],
                    options.require_bearer_auth,
                )
            }),
        )
        .api_route(
            export_status.as_str(),
            get_with(async_bound_export_status::<S>, |op| {
                datatable_operation(
                    op,
                    &format!("datatable_{}_csv_status", op_scope),
                    &format!("{} datatable CSV status", model_name),
                    "Get async export job status.",
                    &[&format!("Model is bound to `{}` by route.", model_name)],
                    options.require_bearer_auth,
                )
            }),
        )
        .api_route(
            form_path.as_str(),
            post_with(load_bound_datatable_form::<S>, |op| {
                datatable_operation(
                    op,
                    &format!("datatable_{}_execute_form", op_scope),
                    &format!("{} datatable execute (form)", model_name),
                    "Execute datatable via form payload.",
                    &[&format!("Model is bound to `{}` by route.", model_name)],
                    options.require_bearer_auth,
                )
            }),
        )
        .api_route(
            json_path.as_str(),
            post_with(load_bound_datatable_json::<S>, |op| {
                datatable_operation(
                    op,
                    &format!("datatable_{}_execute_json", op_scope),
                    &format!("{} datatable execute (json)", model_name),
                    "Execute datatable via JSON payload.",
                    &[&format!("Model is bound to `{}` by route.", model_name)],
                    options.require_bearer_auth,
                )
            }),
        )
        .with_state(bound_state)
}

fn datatable_operation<'t>(
    op: TransformOperation<'t>,
    operation_id: &str,
    summary: &str,
    description: &str,
    extra_notes: &[&str],
    needs_bearer_auth: bool,
) -> TransformOperation<'t> {
    let mut notes = vec![
        "Framework-provided datatable route collection (core_web::datatable).",
        "Apply app-level auth middleware when mounting admin datatable routes.",
    ];
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

fn datatable_operation_scope(prefix: &str, model: &str) -> String {
    let raw = format!("{prefix}_{model}");
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

fn bind_model(mut input: DataTableInput, model: &str) -> DataTableInput {
    input.model = Some(model.to_string());
    input.params.insert("model".to_string(), model.to_string());
    input
}

async fn load_bound_datatable_query<S>(
    State(state): State<BoundDataTableState<S>>,
    Query(params): Query<BoundDataTableRequestDto>,
    headers: RequestHeaders,
) -> Result<ApiResponse<Value>, AppError>
where
    S: DataTableRouteState,
{
    let input = params.into_input(state.model.as_str());
    execute_datatable(&state.inner, input, &headers).await
}

async fn load_bound_datatable_multipart<S>(
    State(state): State<BoundDataTableState<S>>,
    headers: RequestHeaders,
    multipart: Multipart,
) -> Result<ApiResponse<Value>, AppError>
where
    S: DataTableRouteState,
{
    let params = multipart_to_params(multipart).await?;
    let input = bind_model(datatable_input_from_form(&params), state.model.as_str());
    execute_datatable(&state.inner, input, &headers).await
}

async fn load_bound_datatable_form<S>(
    State(state): State<BoundDataTableState<S>>,
    headers: RequestHeaders,
    Form(params): Form<BoundDataTableRequestDto>,
) -> Result<ApiResponse<Value>, AppError>
where
    S: DataTableRouteState,
{
    let input = params.into_input(state.model.as_str());
    execute_datatable(&state.inner, input, &headers).await
}

async fn load_bound_datatable_json<S>(
    State(state): State<BoundDataTableState<S>>,
    headers: RequestHeaders,
    Json(body): Json<BoundDataTableRequestDto>,
) -> Result<ApiResponse<Value>, AppError>
where
    S: DataTableRouteState,
{
    let input = body.into_input(state.model.as_str());
    execute_datatable(&state.inner, input, &headers).await
}

async fn describe_bound_datatable<S>(
    State(state): State<BoundDataTableState<S>>,
    headers: RequestHeaders,
) -> Result<ApiResponse<Value>, AppError>
where
    S: DataTableRouteState,
{
    let ctx = state.inner.datatable_context(&headers).await;
    let describe = state
        .inner
        .datatable_registry()
        .describe(state.model.as_str(), &ctx)
        .map_err(map_datatable_error)?;
    Ok(ApiResponse::success(
        serde_json::to_value(describe)?,
        "datatable describe",
    ))
}

async fn queue_bound_csv_export_query<S>(
    State(state): State<BoundDataTableState<S>>,
    Query(params): Query<BoundDataTableRequestDto>,
    headers: RequestHeaders,
) -> Result<ApiResponse<Value>, AppError>
where
    S: DataTableRouteState,
{
    let input = params.into_input(state.model.as_str());
    queue_csv_export(&state.inner, input, &headers).await
}

async fn queue_bound_csv_export_multipart<S>(
    State(state): State<BoundDataTableState<S>>,
    headers: RequestHeaders,
    multipart: Multipart,
) -> Result<ApiResponse<Value>, AppError>
where
    S: DataTableRouteState,
{
    let params = multipart_to_params(multipart).await?;
    let input = bind_model(datatable_input_from_form(&params), state.model.as_str());
    queue_csv_export(&state.inner, input, &headers).await
}

async fn queue_bound_csv_export_form<S>(
    State(state): State<BoundDataTableState<S>>,
    headers: RequestHeaders,
    Form(params): Form<BoundDataTableRequestDto>,
) -> Result<ApiResponse<Value>, AppError>
where
    S: DataTableRouteState,
{
    let input = params.into_input(state.model.as_str());
    queue_csv_export(&state.inner, input, &headers).await
}

async fn queue_bound_csv_export_json<S>(
    State(state): State<BoundDataTableState<S>>,
    headers: RequestHeaders,
    Json(body): Json<BoundDataTableRequestDto>,
) -> Result<ApiResponse<Value>, AppError>
where
    S: DataTableRouteState,
{
    let input = body.into_input(state.model.as_str());
    queue_csv_export(&state.inner, input, &headers).await
}

async fn async_bound_export_status<S>(
    State(state): State<BoundDataTableState<S>>,
    Query(params): Query<DataTableExportStatusQueryDto>,
) -> Result<ApiResponse<Value>, AppError>
where
    S: DataTableRouteState,
{
    let job_id = params
        .job_id
        .or(params.id)
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| AppError::BadRequest("Missing export job id".to_string()))?;

    let status = state
        .inner
        .datatable_async_exports()
        .status(&job_id)
        .await
        .ok_or_else(|| AppError::NotFound(format!("Unknown export job '{}'", job_id)))?;

    Ok(ApiResponse::success(
        serde_json::to_value(status)?,
        "datatable export status",
    ))
}

async fn stream_bound_csv_query<S>(
    State(state): State<BoundDataTableState<S>>,
    Query(params): Query<BoundDataTableRequestDto>,
    headers: RequestHeaders,
) -> Result<Response, AppError>
where
    S: DataTableRouteState,
{
    let input = params.into_input(state.model.as_str());
    stream_csv_export(&state.inner, input, &headers).await
}

async fn stream_bound_csv_multipart<S>(
    State(state): State<BoundDataTableState<S>>,
    headers: RequestHeaders,
    multipart: Multipart,
) -> Result<Response, AppError>
where
    S: DataTableRouteState,
{
    let params = multipart_to_params(multipart).await?;
    let input = bind_model(datatable_input_from_form(&params), state.model.as_str());
    stream_csv_export(&state.inner, input, &headers).await
}

async fn load_datatable_query<S>(
    State(state): State<S>,
    Query(params): Query<DataTableRequestDto>,
    headers: RequestHeaders,
) -> Result<ApiResponse<Value>, AppError>
where
    S: DataTableRouteState,
{
    let input = params.into_input();
    execute_datatable(&state, input, &headers).await
}

async fn load_datatable_multipart<S>(
    State(state): State<S>,
    headers: RequestHeaders,
    multipart: Multipart,
) -> Result<ApiResponse<Value>, AppError>
where
    S: DataTableRouteState,
{
    let params = multipart_to_params(multipart).await?;
    let input = datatable_input_from_form(&params);
    execute_datatable(&state, input, &headers).await
}

async fn load_datatable_form<S>(
    State(state): State<S>,
    headers: RequestHeaders,
    Form(params): Form<DataTableRequestDto>,
) -> Result<ApiResponse<Value>, AppError>
where
    S: DataTableRouteState,
{
    let input = params.into_input();
    execute_datatable(&state, input, &headers).await
}

async fn load_datatable_json<S>(
    State(state): State<S>,
    headers: RequestHeaders,
    Json(body): Json<DataTableRequestDto>,
) -> Result<ApiResponse<Value>, AppError>
where
    S: DataTableRouteState,
{
    let input = body.into_input();
    execute_datatable(&state, input, &headers).await
}

async fn describe_datatable<S>(
    State(state): State<S>,
    Query(params): Query<DataTableDescribeQueryDto>,
    headers: RequestHeaders,
) -> Result<ApiResponse<Value>, AppError>
where
    S: DataTableRouteState,
{
    let model = params.model.trim();
    if model.is_empty() {
        return Err(AppError::BadRequest(
            "Missing datatable model key".to_string(),
        ));
    }
    let ctx = state.datatable_context(&headers).await;
    let describe = state
        .datatable_registry()
        .describe(model, &ctx)
        .map_err(map_datatable_error)?;
    Ok(ApiResponse::success(
        serde_json::to_value(describe)?,
        "datatable describe",
    ))
}

async fn queue_csv_export_query<S>(
    State(state): State<S>,
    Query(params): Query<DataTableRequestDto>,
    headers: RequestHeaders,
) -> Result<ApiResponse<Value>, AppError>
where
    S: DataTableRouteState,
{
    let input = params.into_input();
    queue_csv_export(&state, input, &headers).await
}

async fn queue_csv_export_multipart<S>(
    State(state): State<S>,
    headers: RequestHeaders,
    multipart: Multipart,
) -> Result<ApiResponse<Value>, AppError>
where
    S: DataTableRouteState,
{
    let params = multipart_to_params(multipart).await?;
    let input = datatable_input_from_form(&params);
    queue_csv_export(&state, input, &headers).await
}

async fn queue_csv_export_form<S>(
    State(state): State<S>,
    headers: RequestHeaders,
    Form(params): Form<DataTableRequestDto>,
) -> Result<ApiResponse<Value>, AppError>
where
    S: DataTableRouteState,
{
    let input = params.into_input();
    queue_csv_export(&state, input, &headers).await
}

async fn queue_csv_export_json<S>(
    State(state): State<S>,
    headers: RequestHeaders,
    Json(body): Json<DataTableRequestDto>,
) -> Result<ApiResponse<Value>, AppError>
where
    S: DataTableRouteState,
{
    let input = body.into_input();
    queue_csv_export(&state, input, &headers).await
}

async fn async_export_status<S>(
    State(state): State<S>,
    Query(params): Query<DataTableExportStatusQueryDto>,
) -> Result<ApiResponse<Value>, AppError>
where
    S: DataTableRouteState,
{
    let job_id = params
        .job_id
        .or(params.id)
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| AppError::BadRequest("Missing export job id".to_string()))?;

    let status = state
        .datatable_async_exports()
        .status(&job_id)
        .await
        .ok_or_else(|| AppError::NotFound(format!("Unknown export job '{}'", job_id)))?;

    Ok(ApiResponse::success(
        serde_json::to_value(status)?,
        "datatable export status",
    ))
}

async fn stream_csv_query<S>(
    State(state): State<S>,
    Query(params): Query<DataTableRequestDto>,
    headers: RequestHeaders,
) -> Result<Response, AppError>
where
    S: DataTableRouteState,
{
    let input = params.into_input();
    stream_csv_export(&state, input, &headers).await
}

async fn stream_csv_multipart<S>(
    State(state): State<S>,
    headers: RequestHeaders,
    multipart: Multipart,
) -> Result<Response, AppError>
where
    S: DataTableRouteState,
{
    let params = multipart_to_params(multipart).await?;
    let input = datatable_input_from_form(&params);
    stream_csv_export(&state, input, &headers).await
}

async fn execute_datatable<S>(
    state: &S,
    input: DataTableInput,
    headers: &HeaderMap,
) -> Result<ApiResponse<Value>, AppError>
where
    S: DataTableRouteState,
{
    let model = input
        .model
        .clone()
        .unwrap_or_else(|| "<missing>".to_string());
    let ctx = state.datatable_context(headers).await;
    let exec = state
        .datatable_registry()
        .execute(&input, &ctx)
        .await
        .map_err(map_datatable_error)?;

    let data = match exec {
        DataTableExecution::Page(page) => json!({
            "model": model,
            "mode": "page",
            "records": page.records,
            "per_page": page.per_page,
            "total_records": page.total_records,
            "total_pages": page.total_pages,
            "page": page.page,
            "pagination_mode": page.pagination_mode,
            "has_more": page.has_more,
            "next_cursor": page.next_cursor,
            "diagnostics": page.diagnostics,
        }),
        DataTableExecution::Csv(csv) => json!({
            "model": model,
            "mode": "csv",
            "file_path": csv.file_path,
            "file_name": csv.file_name,
            "content_type": csv.content_type,
            "total_records": csv.total_records,
            "diagnostics": csv.diagnostics,
        }),
    };

    if let Some(diagnostics) = data.get("diagnostics") {
        info!(
            target: "datatable",
            model = %model,
            mode = %data.get("mode").and_then(|v| v.as_str()).unwrap_or("unknown"),
            duration_ms = diagnostics.get("duration_ms").and_then(|v| v.as_u64()).unwrap_or(0),
            auto_filters_applied = diagnostics
                .get("auto_filters_applied")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            unknown_filters = diagnostics
                .get("unknown_filters")
                .and_then(|v| v.as_array())
                .map(|a| a.len())
                .unwrap_or(0),
            "datatable execution"
        );
    }

    Ok(ApiResponse::make_response(
        StatusCode::OK,
        data,
        Some("datatable executed"),
    ))
}

async fn queue_csv_export<S>(
    state: &S,
    mut input: DataTableInput,
    headers: &HeaderMap,
) -> Result<ApiResponse<Value>, AppError>
where
    S: DataTableRouteState,
{
    input.export = DataTableExportMode::Csv;
    let ctx = state.datatable_context(headers).await;
    let model = input
        .model
        .clone()
        .unwrap_or_else(|| "<missing>".to_string());
    let ticket = state
        .datatable_async_exports()
        .enqueue(input, ctx)
        .await
        .map_err(map_datatable_error)?;

    info!(
        target: "datatable",
        model = %model,
        job_id = %ticket.job_id,
        state = ?ticket.state,
        "datatable export queued"
    );

    Ok(ApiResponse::success(
        json!({
            "job_id": ticket.job_id,
            "state": ticket.state,
        }),
        "datatable export queued",
    ))
}

async fn stream_csv_export<S>(
    state: &S,
    mut input: DataTableInput,
    headers: &HeaderMap,
) -> Result<Response, AppError>
where
    S: DataTableRouteState,
{
    input.export = DataTableExportMode::Csv;
    let model = input
        .model
        .clone()
        .unwrap_or_else(|| "<missing>".to_string());
    let ctx = state.datatable_context(headers).await;
    let exec = state
        .datatable_registry()
        .execute(&input, &ctx)
        .await
        .map_err(map_datatable_error)?;

    let csv = match exec {
        DataTableExecution::Csv(csv) => csv,
        DataTableExecution::Page(_) => {
            return Err(AppError::BadRequest(
                "CSV export stream requires export mode".to_string(),
            ))
        }
    };

    let file = tokio::fs::File::open(&csv.file_path).await.map_err(|err| {
        AppError::Internal(anyhow::anyhow!(
            "Failed to open CSV export file '{}': {}",
            csv.file_path,
            err
        ))
    })?;
    let stream = ReaderStream::new(file);

    let mut response = Response::new(Body::from_stream(stream));
    *response.status_mut() = StatusCode::OK;
    let content_type = HeaderValue::from_str(&csv.content_type)
        .unwrap_or_else(|_| HeaderValue::from_static("text/csv; charset=utf-8"));
    response
        .headers_mut()
        .insert(header::CONTENT_TYPE, content_type);

    let safe_file_name = sanitize_file_name(&csv.file_name);
    if let Ok(disposition) =
        HeaderValue::from_str(&format!("attachment; filename=\"{}\"", safe_file_name))
    {
        response
            .headers_mut()
            .insert(header::CONTENT_DISPOSITION, disposition);
    }

    info!(
        target: "datatable",
        model = %model,
        file_name = %csv.file_name,
        total_records = csv.total_records,
        "datatable csv stream ready"
    );

    Ok(response)
}

async fn multipart_to_params(multipart: Multipart) -> Result<BTreeMap<String, String>, AppError> {
    let mut params = BTreeMap::new();
    let mut multipart = multipart;
    while let Some(field) = multipart.next_field().await? {
        let Some(name) = field.name().map(ToString::to_string) else {
            continue;
        };
        let value = String::from_utf8_lossy(&field.bytes().await?).to_string();
        params.insert(name, value);
    }
    Ok(params)
}

fn sanitize_file_name(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return "datatable.csv".to_string();
    }
    trimmed
        .chars()
        .map(|ch| match ch {
            '"' | '\\' | '/' | '\n' | '\r' => '_',
            _ => ch,
        })
        .collect()
}

fn map_datatable_error(err: anyhow::Error) -> AppError {
    let msg = err.to_string();
    if msg.contains("Missing datatable model key")
        || msg.contains("Unknown datatable model")
        || msg.contains("Unknown datatable filter")
        || msg.contains("Cursor mode is not supported")
        || msg.contains("Pagination mode")
    {
        return AppError::BadRequest(msg);
    }
    AppError::Internal(err)
}

fn insert_opt_string(params: &mut BTreeMap<String, String>, key: &str, value: Option<String>) {
    if let Some(v) = value {
        let trimmed = v.trim();
        if !trimmed.is_empty() {
            params.insert(key.to_string(), trimmed.to_string());
        }
    }
}

fn insert_opt_i64(params: &mut BTreeMap<String, String>, key: &str, value: Option<i64>) {
    if let Some(v) = value {
        params.insert(key.to_string(), v.to_string());
    }
}
