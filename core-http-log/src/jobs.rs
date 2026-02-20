use crate::cleanup_logs;
use core_jobs::Job;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Default)]
pub struct CleanupHttpLogs;

#[async_trait::async_trait]
impl Job for CleanupHttpLogs {
    const NAME: &'static str = "core:cleanup_http_logs";
    const QUEUE: &'static str = "default"; // Optional, defaults to "default"

    async fn handle(&self, ctx: &core_jobs::JobContext) -> anyhow::Result<()> {
        let retention = ctx.settings.http_log.retention_days;
        cleanup_logs(&ctx.db, retention).await?;
        Ok(())
    }
}
