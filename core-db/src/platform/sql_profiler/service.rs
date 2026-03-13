use crate::common::sql::SqlProfilerCollector;
use anyhow::Result;
use sqlx::PgPool;

/// Flush collected profiler data to the database.
pub async fn flush_profiler(
    pool: &PgPool,
    request_method: &str,
    request_path: &str,
    collector: SqlProfilerCollector,
) -> Result<()> {
    let (request_id, queries) = collector.finish();
    if queries.is_empty() {
        return Ok(());
    }

    let total_queries = queries.len() as i32;
    let total_duration_ms: f64 = queries
        .iter()
        .map(|q| q.duration.as_secs_f64() * 1000.0)
        .sum();

    // Insert request row
    sqlx::query(
        "INSERT INTO sql_profiler_requests (id, request_method, request_path, total_queries, total_duration_ms, created_at) VALUES ($1, $2, $3, $4, $5, NOW())"
    )
        .bind(request_id)
        .bind(request_method)
        .bind(request_path)
        .bind(total_queries)
        .bind(total_duration_ms)
        .execute(pool)
        .await?;

    // Insert query rows
    for q in queries {
        let id = crate::common::sql::generate_snowflake_i64();
        sqlx::query(
            "INSERT INTO sql_profiler_queries (id, request_id, table_name, operation, sql, binds, duration_us, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, NOW())"
        )
            .bind(id)
            .bind(request_id)
            .bind(&q.table_name)
            .bind(&q.operation)
            .bind(&q.sql)
            .bind(&q.binds)
            .bind(q.duration.as_micros() as i64)
            .execute(pool)
            .await?;
    }

    Ok(())
}

/// Delete profiler logs older than `retention_days`.
/// CASCADE will delete associated query rows.
pub async fn cleanup_profiler_logs(pool: &PgPool, retention_days: u64) -> Result<()> {
    if retention_days == 0 {
        return Ok(());
    }
    let cutoff = time::OffsetDateTime::now_utc() - time::Duration::days(retention_days as i64);
    sqlx::query("DELETE FROM sql_profiler_requests WHERE created_at < $1")
        .bind(cutoff)
        .execute(pool)
        .await?;
    Ok(())
}
