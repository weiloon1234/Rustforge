use crate::registry::DataTableRegistry;
use crate::types::{
    DataTableContext, DataTableCsvExport, DataTableExecution, DataTableExportMode, DataTableInput,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use time::OffsetDateTime;
use tokio::sync::RwLock;
use tracing::{info, warn};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataTableAsyncExportState {
    Queued,
    Running,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataTableAsyncExportTicket {
    pub job_id: String,
    pub state: DataTableAsyncExportState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataTableAsyncExportStatus {
    pub job_id: String,
    pub model: String,
    pub state: DataTableAsyncExportState,
    pub created_at_unix: i64,
    pub started_at_unix: Option<i64>,
    pub finished_at_unix: Option<i64>,
    pub error: Option<String>,
    pub csv: Option<DataTableCsvExport>,
}

#[derive(Debug, Clone)]
struct DataTableAsyncExportJob {
    model: String,
    state: DataTableAsyncExportState,
    created_at: OffsetDateTime,
    started_at: Option<OffsetDateTime>,
    finished_at: Option<OffsetDateTime>,
    error: Option<String>,
    csv: Option<DataTableCsvExport>,
}

impl DataTableAsyncExportJob {
    fn to_status(&self, job_id: &str) -> DataTableAsyncExportStatus {
        DataTableAsyncExportStatus {
            job_id: job_id.to_string(),
            model: self.model.clone(),
            state: self.state,
            created_at_unix: self.created_at.unix_timestamp(),
            started_at_unix: self.started_at.map(|x| x.unix_timestamp()),
            finished_at_unix: self.finished_at.map(|x| x.unix_timestamp()),
            error: self.error.clone(),
            csv: self.csv.clone(),
        }
    }
}

#[derive(Clone)]
pub struct DataTableAsyncExportManager {
    registry: Arc<DataTableRegistry>,
    jobs: Arc<RwLock<HashMap<String, DataTableAsyncExportJob>>>,
}

impl DataTableAsyncExportManager {
    pub fn new(registry: Arc<DataTableRegistry>) -> Self {
        Self {
            registry,
            jobs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn enqueue(
        &self,
        mut input: DataTableInput,
        ctx: DataTableContext,
    ) -> Result<DataTableAsyncExportTicket> {
        let model = input
            .model
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| anyhow::anyhow!("Missing datatable model key"))?
            .to_string();

        input.export = DataTableExportMode::Csv;
        let job_id = Uuid::new_v4().to_string();
        let now = OffsetDateTime::now_utc();
        let model_for_log = model.clone();

        {
            let mut jobs = self.jobs.write().await;
            jobs.insert(
                job_id.clone(),
                DataTableAsyncExportJob {
                    model,
                    state: DataTableAsyncExportState::Queued,
                    created_at: now,
                    started_at: None,
                    finished_at: None,
                    error: None,
                    csv: None,
                },
            );
        }

        let manager = self.clone();
        let run_job_id = job_id.clone();
        tokio::spawn(async move {
            manager.run(run_job_id, input, ctx).await;
        });

        info!(
            target: "datatable",
            model = %model_for_log,
            job_id = %job_id,
            "async datatable export queued"
        );

        Ok(DataTableAsyncExportTicket {
            job_id,
            state: DataTableAsyncExportState::Queued,
        })
    }

    pub async fn status(&self, job_id: &str) -> Option<DataTableAsyncExportStatus> {
        let jobs = self.jobs.read().await;
        jobs.get(job_id).map(|job| job.to_status(job_id))
    }

    async fn run(&self, job_id: String, input: DataTableInput, ctx: DataTableContext) {
        {
            let mut jobs = self.jobs.write().await;
            if let Some(job) = jobs.get_mut(&job_id) {
                job.state = DataTableAsyncExportState::Running;
                job.started_at = Some(OffsetDateTime::now_utc());
                info!(
                    target: "datatable",
                    model = %job.model,
                    job_id = %job_id,
                    "async datatable export running"
                );
            } else {
                return;
            }
        }

        let outcome = self.registry.execute(&input, &ctx).await;
        match outcome {
            Ok(DataTableExecution::Csv(csv)) => {
                let mut jobs = self.jobs.write().await;
                if let Some(job) = jobs.get_mut(&job_id) {
                    job.state = DataTableAsyncExportState::Completed;
                    job.finished_at = Some(OffsetDateTime::now_utc());
                    let total_records = csv.total_records;
                    job.csv = Some(csv);
                    info!(
                        target: "datatable",
                        model = %job.model,
                        job_id = %job_id,
                        total_records = total_records,
                        "async datatable export completed"
                    );
                }
            }
            Ok(DataTableExecution::Page(_)) => {
                let mut jobs = self.jobs.write().await;
                if let Some(job) = jobs.get_mut(&job_id) {
                    job.state = DataTableAsyncExportState::Failed;
                    job.finished_at = Some(OffsetDateTime::now_utc());
                    job.error =
                        Some("Async export expected CSV mode but received page result".to_string());
                    warn!(
                        target: "datatable",
                        model = %job.model,
                        job_id = %job_id,
                        "async datatable export failed: non-csv execution"
                    );
                }
            }
            Err(err) => {
                let mut jobs = self.jobs.write().await;
                if let Some(job) = jobs.get_mut(&job_id) {
                    job.state = DataTableAsyncExportState::Failed;
                    job.finished_at = Some(OffsetDateTime::now_utc());
                    job.error = Some(err.to_string());
                    warn!(
                        target: "datatable",
                        model = %job.model,
                        job_id = %job_id,
                        error = %job.error.as_deref().unwrap_or("unknown"),
                        "async datatable export failed"
                    );
                }
            }
        }
    }
}
