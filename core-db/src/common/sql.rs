#![allow(dead_code)]

use serde_json::Value;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Like,
    ILike,
}

impl Op {
    pub const fn as_sql(self) -> &'static str {
        match self {
            Op::Eq => "=",
            Op::Ne => "!=",
            Op::Lt => "<",
            Op::Le => "<=",
            Op::Gt => ">",
            Op::Ge => ">=",
            Op::Like => "LIKE",
            Op::ILike => "ILIKE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderDir {
    Asc,
    Desc,
}

impl OrderDir {
    pub const fn as_sql(self) -> &'static str {
        match self {
            OrderDir::Asc => "ASC",
            OrderDir::Desc => "DESC",
        }
    }
}

#[derive(Debug, Clone)]
pub enum BindValue {
    I16(i16),
    I32(i32),
    I64(i64),
    F64(f64),
    Bool(bool),
    String(String),
    Time(OffsetDateTime),
    TimeOpt(Option<OffsetDateTime>),
    Uuid(Uuid),
    Json(Value),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RawJoinKind {
    Inner,
    Left,
    Right,
    Full,
}

#[derive(Debug, Clone)]
pub struct RawClause {
    sql: String,
    binds: Vec<BindValue>,
}

impl RawClause {
    pub fn new<T: Into<BindValue>>(
        sql: impl Into<String>,
        binds: impl IntoIterator<Item = T>,
    ) -> anyhow::Result<Self> {
        let sql = sql.into();
        if sql.trim().is_empty() {
            anyhow::bail!("raw clause cannot be empty");
        }
        if contains_dollar_placeholder(&sql) {
            anyhow::bail!("raw clause must use '?' placeholders only (not '$n')");
        }
        let binds: Vec<BindValue> = binds.into_iter().map(Into::into).collect();
        let expected = count_question_placeholders(&sql);
        if expected != binds.len() {
            anyhow::bail!(
                "raw clause placeholder count mismatch: expected {}, got {}",
                expected,
                binds.len()
            );
        }
        Ok(Self { sql, binds })
    }

    pub fn into_parts(self) -> (String, Vec<BindValue>) {
        (self.sql, self.binds)
    }
}

#[derive(Debug, Clone)]
pub struct RawOrderExpr(String);

impl RawOrderExpr {
    pub fn new(sql: impl Into<String>) -> anyhow::Result<Self> {
        let sql = sql.into();
        if sql.trim().is_empty() {
            anyhow::bail!("raw order expression cannot be empty");
        }
        if contains_dollar_placeholder(&sql) {
            anyhow::bail!("raw order expression must not contain '$n' placeholders");
        }
        Ok(Self(sql))
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct RawGroupExpr(String);

impl RawGroupExpr {
    pub fn new(sql: impl Into<String>) -> anyhow::Result<Self> {
        let sql = sql.into();
        if sql.trim().is_empty() {
            anyhow::bail!("raw group expression cannot be empty");
        }
        if contains_dollar_placeholder(&sql) {
            anyhow::bail!("raw group expression must not contain '$n' placeholders");
        }
        Ok(Self(sql))
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct RawSelectExpr(String);

impl RawSelectExpr {
    pub fn new(sql: impl Into<String>) -> anyhow::Result<Self> {
        let sql = sql.into();
        if sql.trim().is_empty() {
            anyhow::bail!("raw select expression cannot be empty");
        }
        if contains_dollar_placeholder(&sql) {
            anyhow::bail!("raw select expression must not contain '$n' placeholders");
        }
        Ok(Self(sql))
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct RawJoinSpec {
    kind: RawJoinKind,
    table: String,
    on: RawClause,
}

impl RawJoinSpec {
    pub fn new(kind: RawJoinKind, table: impl Into<String>, on: RawClause) -> anyhow::Result<Self> {
        let table = table.into();
        if table.trim().is_empty() {
            anyhow::bail!("raw join table cannot be empty");
        }
        if contains_dollar_placeholder(&table) {
            anyhow::bail!("raw join table must not contain '$n' placeholders");
        }
        Ok(Self { kind, table, on })
    }

    pub fn inner(table: impl Into<String>, on: RawClause) -> anyhow::Result<Self> {
        Self::new(RawJoinKind::Inner, table, on)
    }

    pub fn left(table: impl Into<String>, on: RawClause) -> anyhow::Result<Self> {
        Self::new(RawJoinKind::Left, table, on)
    }

    pub fn right(table: impl Into<String>, on: RawClause) -> anyhow::Result<Self> {
        Self::new(RawJoinKind::Right, table, on)
    }

    pub fn full(table: impl Into<String>, on: RawClause) -> anyhow::Result<Self> {
        Self::new(RawJoinKind::Full, table, on)
    }

    pub fn into_parts(self) -> (RawJoinKind, String, String, Vec<BindValue>) {
        let (on_sql, on_binds) = self.on.into_parts();
        (self.kind, self.table, on_sql, on_binds)
    }
}

pub enum DbTxnScope<'a> {
    Reused(DbConn<'a>),
    Owned(Arc<tokio::sync::Mutex<PgTransaction<'a>>>),
}

impl From<i16> for BindValue {
    fn from(v: i16) -> Self {
        BindValue::I16(v)
    }
}

impl From<i32> for BindValue {
    fn from(v: i32) -> Self {
        BindValue::I32(v)
    }
}

impl From<i64> for BindValue {
    fn from(v: i64) -> Self {
        BindValue::I64(v)
    }
}

impl From<f64> for BindValue {
    fn from(v: f64) -> Self {
        BindValue::F64(v)
    }
}

impl From<bool> for BindValue {
    fn from(v: bool) -> Self {
        BindValue::Bool(v)
    }
}

impl From<String> for BindValue {
    fn from(v: String) -> Self {
        BindValue::String(v)
    }
}

impl From<&str> for BindValue {
    fn from(v: &str) -> Self {
        BindValue::String(v.to_owned())
    }
}

impl From<OffsetDateTime> for BindValue {
    fn from(v: OffsetDateTime) -> Self {
        BindValue::Time(v)
    }
}

impl From<Option<OffsetDateTime>> for BindValue {
    fn from(v: Option<OffsetDateTime>) -> Self {
        BindValue::TimeOpt(v)
    }
}

impl From<Value> for BindValue {
    fn from(v: Value) -> Self {
        BindValue::Json(v)
    }
}

/// Renumber placeholders like `$1` in a SQL fragment starting at `start`.
pub fn renumber_placeholders(sql: &str, start: usize) -> String {
    let mut out = String::with_capacity(sql.len() + 8);
    let mut i = 0;
    let bytes = sql.as_bytes();
    while i < bytes.len() {
        if bytes[i] == b'$' {
            i += 1;
            let start_idx = i;
            while i < bytes.len() && bytes[i].is_ascii_digit() {
                i += 1;
            }
            let num: usize = sql[start_idx..i].parse().unwrap_or(0);
            let new_idx = start + num - 1;
            out.push('$');
            out.push_str(&new_idx.to_string());
        } else {
            out.push(bytes[i] as char);
            i += 1;
        }
    }
    out
}

fn contains_dollar_placeholder(sql: &str) -> bool {
    let bytes = sql.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'$' {
            i += 1;
            if i < bytes.len() && bytes[i].is_ascii_digit() {
                return true;
            }
            continue;
        }
        i += 1;
    }
    false
}

fn count_question_placeholders(sql: &str) -> usize {
    sql.as_bytes().iter().filter(|&&b| b == b'?').count()
}

const SNOWFLAKE_EPOCH_MS: u64 = 1_704_067_200_000; // 2024-01-01T00:00:00Z
const SNOWFLAKE_SEQUENCE_MASK: u64 = (1 << 12) - 1;
const SNOWFLAKE_NODE_MASK: u64 = (1 << 10) - 1;
const SNOWFLAKE_TIMESTAMP_MASK: u64 = (1 << 41) - 1;

static LAST_SNOWFLAKE_ID: AtomicU64 = AtomicU64::new(0);
static SNOWFLAKE_NODE_BITS: OnceLock<u64> = OnceLock::new();

fn snowflake_node_bits() -> u64 {
    *SNOWFLAKE_NODE_BITS.get_or_init(|| {
        let parsed = std::env::var("SNOWFLAKE_NODE_ID")
            .ok()
            .and_then(|raw| raw.parse::<u16>().ok())
            .unwrap_or_else(|| (std::process::id() as u16) & (SNOWFLAKE_NODE_MASK as u16));
        (u64::from(parsed) & SNOWFLAKE_NODE_MASK) << 12
    })
}

fn now_unix_ms() -> u64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(d) => d.as_millis() as u64,
        Err(_) => 0,
    }
}

/// Generate a positive, time-sortable i64 ID using a Snowflake-style layout.
///
/// Bit layout: 41 bits timestamp (ms since 2024-01-01 UTC), 10 bits node, 12 bits sequence.
/// Result remains monotonic per process and preserves chronological ordering by ID.
pub fn generate_snowflake_i64() -> i64 {
    loop {
        let now_ms = now_unix_ms().saturating_sub(SNOWFLAKE_EPOCH_MS) & SNOWFLAKE_TIMESTAMP_MASK;
        let node_bits = snowflake_node_bits();
        let prev = LAST_SNOWFLAKE_ID.load(Ordering::Relaxed);
        let prev_ts = (prev >> 22) & SNOWFLAKE_TIMESTAMP_MASK;

        let next = if now_ms > prev_ts {
            (now_ms << 22) | node_bits
        } else {
            let next_seq =
                (prev & SNOWFLAKE_SEQUENCE_MASK).wrapping_add(1) & SNOWFLAKE_SEQUENCE_MASK;
            if next_seq == 0 {
                (((prev_ts + 1) & SNOWFLAKE_TIMESTAMP_MASK) << 22) | node_bits
            } else {
                (prev & !SNOWFLAKE_SEQUENCE_MASK) | next_seq
            }
        };

        if LAST_SNOWFLAKE_ID
            .compare_exchange_weak(prev, next, Ordering::SeqCst, Ordering::Relaxed)
            .is_ok()
        {
            return next as i64;
        }
    }
}

impl From<Uuid> for BindValue {
    fn from(v: Uuid) -> Self {
        BindValue::Uuid(v)
    }
}

pub type PgQueryAs<'q, T> =
    sqlx::query::QueryAs<'q, sqlx::Postgres, T, sqlx::postgres::PgArguments>;
pub type PgQuery<'q> = sqlx::query::Query<'q, sqlx::Postgres, sqlx::postgres::PgArguments>;
pub type PgQueryScalar<'q, T> =
    sqlx::query::QueryScalar<'q, sqlx::Postgres, T, sqlx::postgres::PgArguments>;

/// Type alias for PostgreSQL transaction
pub type PgTransaction<'a> = sqlx::Transaction<'a, sqlx::Postgres>;

/// Database connection wrapper that can hold either a pool or transaction reference.
/// This enables running queries in both regular and transactional contexts.
#[derive(Clone)]
pub enum DbConn<'a> {
    Pool(&'a sqlx::PgPool),
    Tx(Arc<tokio::sync::Mutex<PgTransaction<'a>>>),
}

impl<'a> From<&'a sqlx::PgPool> for DbConn<'a> {
    fn from(pool: &'a sqlx::PgPool) -> Self {
        DbConn::Pool(pool)
    }
}

impl<'a> DbConn<'a> {
    /// Create a DbConn from a pool reference
    pub fn pool(pool: &'a sqlx::PgPool) -> Self {
        DbConn::Pool(pool)
    }

    /// Create a DbConn from a transaction mutex reference
    pub fn tx(tx: Arc<tokio::sync::Mutex<PgTransaction<'a>>>) -> Self {
        DbConn::Tx(tx)
    }

    pub async fn begin_scope(&'a self) -> Result<DbTxnScope<'a>, sqlx::Error> {
        match self {
            DbConn::Pool(pool) => {
                let tx = pool.begin().await?;
                Ok(DbTxnScope::Owned(Arc::new(tokio::sync::Mutex::new(tx))))
            }
            DbConn::Tx(_) => Ok(DbTxnScope::Reused(self.clone())),
        }
    }

    pub async fn execute<'q>(
        &self,
        query: PgQuery<'q>,
    ) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
        match self {
            DbConn::Pool(pool) => query.execute(*pool).await,
            DbConn::Tx(tx) => {
                let mut tx = tx.lock().await;
                query.execute(&mut **tx).await
            }
        }
    }

    pub async fn fetch_all<'q, O>(&self, query: PgQueryAs<'q, O>) -> Result<Vec<O>, sqlx::Error>
    where
        O: for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin + 'static,
    {
        match self {
            DbConn::Pool(pool) => query.fetch_all(*pool).await,
            DbConn::Tx(tx) => {
                let mut tx = tx.lock().await;
                query.fetch_all(&mut **tx).await
            }
        }
    }

    pub async fn fetch_one<'q, O>(&self, query: PgQueryAs<'q, O>) -> Result<O, sqlx::Error>
    where
        O: for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin + 'static,
    {
        match self {
            DbConn::Pool(pool) => query.fetch_one(*pool).await,
            DbConn::Tx(tx) => {
                let mut tx = tx.lock().await;
                query.fetch_one(&mut **tx).await
            }
        }
    }

    pub async fn fetch_optional<'q, O>(
        &self,
        query: PgQueryAs<'q, O>,
    ) -> Result<Option<O>, sqlx::Error>
    where
        O: for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin + 'static,
    {
        match self {
            DbConn::Pool(pool) => query.fetch_optional(*pool).await,
            DbConn::Tx(tx) => {
                let mut tx = tx.lock().await;
                query.fetch_optional(&mut **tx).await
            }
        }
    }

    pub async fn fetch_scalar<'q, O>(&self, query: PgQueryScalar<'q, O>) -> Result<O, sqlx::Error>
    where
        O: Send
            + Unpin
            + 'static
            + for<'r> sqlx::Decode<'r, sqlx::Postgres>
            + sqlx::Type<sqlx::Postgres>,
    {
        match self {
            DbConn::Pool(pool) => query.fetch_one(*pool).await,
            DbConn::Tx(tx) => {
                let mut tx = tx.lock().await;
                query.fetch_one(&mut **tx).await
            }
        }
    }

    pub async fn fetch_all_scalar<'q, O>(
        &self,
        query: PgQueryScalar<'q, O>,
    ) -> Result<Vec<O>, sqlx::Error>
    where
        O: Send
            + Unpin
            + 'static
            + for<'r> sqlx::Decode<'r, sqlx::Postgres>
            + sqlx::Type<sqlx::Postgres>,
    {
        match self {
            DbConn::Pool(pool) => query.fetch_all(*pool).await,
            DbConn::Tx(tx) => {
                let mut tx = tx.lock().await;
                query.fetch_all(&mut **tx).await
            }
        }
    }
}

impl<'a> DbTxnScope<'a> {
    pub fn conn(&'a self) -> DbConn<'a> {
        match self {
            DbTxnScope::Reused(conn) => conn.clone(),
            DbTxnScope::Owned(tx) => DbConn::tx(tx.clone()),
        }
    }

    pub async fn rollback(self) -> Result<(), sqlx::Error> {
        match self {
            DbTxnScope::Reused(_) => Ok(()),
            DbTxnScope::Owned(tx) => {
                let tx = Arc::try_unwrap(tx).map_err(|_| {
                    sqlx::Error::Protocol("transaction scope still has active handles".to_string())
                })?;
                let tx = tx.into_inner();
                tx.rollback().await
            }
        }
    }

    pub async fn commit(self) -> Result<(), sqlx::Error> {
        match self {
            DbTxnScope::Reused(_) => Ok(()),
            DbTxnScope::Owned(tx) => {
                let tx = Arc::try_unwrap(tx).map_err(|_| {
                    sqlx::Error::Protocol("transaction scope still has active handles".to_string())
                })?;
                let tx = tx.into_inner();
                tx.commit().await
            }
        }
    }
}

pub fn bind<'q, T>(q: PgQueryAs<'q, T>, value: BindValue) -> PgQueryAs<'q, T> {
    match value {
        BindValue::I16(v) => q.bind(v),
        BindValue::I32(v) => q.bind(v),
        BindValue::I64(v) => q.bind(v),
        BindValue::F64(v) => q.bind(v),
        BindValue::Bool(v) => q.bind(v),
        BindValue::String(v) => q.bind(v),
        BindValue::Time(v) => q.bind(v),
        BindValue::TimeOpt(v) => q.bind(v),
        BindValue::Uuid(v) => q.bind(v),
        BindValue::Json(v) => q.bind(sqlx::types::Json(v)),
    }
}

pub fn bind_query<'q>(q: PgQuery<'q>, value: BindValue) -> PgQuery<'q> {
    match value {
        BindValue::I16(v) => q.bind(v),
        BindValue::I32(v) => q.bind(v),
        BindValue::I64(v) => q.bind(v),
        BindValue::F64(v) => q.bind(v),
        BindValue::Bool(v) => q.bind(v),
        BindValue::String(v) => q.bind(v),
        BindValue::Time(v) => q.bind(v),
        BindValue::TimeOpt(v) => q.bind(v),
        BindValue::Uuid(v) => q.bind(v),
        BindValue::Json(v) => q.bind(sqlx::types::Json(v)),
    }
}

pub fn bind_scalar<'q, T>(q: PgQueryScalar<'q, T>, value: BindValue) -> PgQueryScalar<'q, T> {
    match value {
        BindValue::I16(v) => q.bind(v),
        BindValue::I32(v) => q.bind(v),
        BindValue::I64(v) => q.bind(v),
        BindValue::F64(v) => q.bind(v),
        BindValue::Bool(v) => q.bind(v),
        BindValue::String(v) => q.bind(v),
        BindValue::Time(v) => q.bind(v),
        BindValue::TimeOpt(v) => q.bind(v),
        BindValue::Uuid(v) => q.bind(v),
        BindValue::Json(v) => q.bind(sqlx::types::Json(v)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_clause_rejects_empty() {
        let err =
            RawClause::new("", Vec::<i32>::new()).expect_err("expected empty raw clause error");
        assert!(err.to_string().contains("cannot be empty"));
    }

    #[test]
    fn raw_clause_rejects_dollar_placeholders() {
        let err = RawClause::new("price > $1", [100]).expect_err("expected '$n' placeholder error");
        assert!(err.to_string().contains("'?' placeholders only"));
    }

    #[test]
    fn raw_clause_rejects_placeholder_mismatch() {
        let err = RawClause::new("price > ? AND stock > ?", [100])
            .expect_err("expected placeholder mismatch");
        assert!(err.to_string().contains("placeholder count mismatch"));
    }

    #[test]
    fn raw_clause_accepts_valid_clause() {
        let clause = RawClause::new("price > ? AND stock > ?", [100, 0]).expect("valid clause");
        let (sql, binds) = clause.into_parts();
        assert_eq!(sql, "price > ? AND stock > ?");
        assert_eq!(binds.len(), 2);
    }

    #[test]
    fn raw_select_expr_rejects_empty() {
        let err = RawSelectExpr::new("   ").expect_err("expected empty raw select expression");
        assert!(err.to_string().contains("cannot be empty"));
    }

    #[test]
    fn raw_join_spec_rejects_empty_table() {
        let on = RawClause::new("u.id = p.user_id", Vec::<i32>::new()).expect("valid on clause");
        let err = RawJoinSpec::new(RawJoinKind::Left, "   ", on)
            .expect_err("expected empty join table to fail");
        assert!(err.to_string().contains("table cannot be empty"));
    }

    #[test]
    fn snowflake_ids_are_positive_and_monotonic() {
        let mut prev = generate_snowflake_i64();
        assert!(prev > 0);
        for _ in 0..2_048 {
            let id = generate_snowflake_i64();
            assert!(id > 0);
            assert!(id > prev);
            prev = id;
        }
    }
}
