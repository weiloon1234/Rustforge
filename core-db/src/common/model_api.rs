use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::hash::Hash;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::{OnceLock, RwLock};

use crate::common::sql::{BindValue, DbConn, Op, OrderDir, SetMode};
use crate::platform::attachments::types::AttachmentInput;
use uuid::Uuid;

pub type BoxModelFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T>> + Send + 'a>>;

#[derive(Debug, Clone)]
pub struct CreateAssignment {
    pub col_sql: &'static str,
    pub value: BindValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreateConflictAction {
    DoNothing,
    Update,
}

#[derive(Debug, Clone)]
pub struct CreateConflictSpec {
    pub action: CreateConflictAction,
    pub cols: Vec<&'static str>,
}

#[derive(Debug, Clone)]
pub struct PatchAssignment {
    pub col_sql: &'static str,
    pub value: BindValue,
    pub mode: SetMode,
}

static DEFAULT_ATTACHMENT_BASE_URL: OnceLock<RwLock<Option<String>>> = OnceLock::new();

fn attachment_base_url_store() -> &'static RwLock<Option<String>> {
    DEFAULT_ATTACHMENT_BASE_URL.get_or_init(|| RwLock::new(None))
}

pub fn set_default_attachment_base_url(base_url: Option<String>) {
    *attachment_base_url_store()
        .write()
        .expect("attachment base url store poisoned") = base_url
        .filter(|value| !value.trim().is_empty())
        .map(|value| value.trim().trim_end_matches('/').to_string());
}

pub fn default_attachment_base_url() -> Option<String> {
    attachment_base_url_store()
        .read()
        .expect("attachment base url store poisoned")
        .clone()
}

fn resolve_attachment_base_url(base_url: Option<String>) -> Option<String> {
    base_url
        .filter(|value| !value.trim().is_empty())
        .map(|value| value.trim().trim_end_matches('/').to_string())
        .or_else(default_attachment_base_url)
}

fn check_deferred(err: &Option<String>) -> Result<()> {
    if let Some(msg) = err {
        anyhow::bail!("{msg}");
    }
    Ok(())
}

fn relation_aggregate_sql(kind: RelationAggregateKind, target_sql: &str) -> String {
    match kind {
        RelationAggregateKind::Sum => format!("SUM({target_sql}::DOUBLE PRECISION)"),
        RelationAggregateKind::Avg => format!("AVG({target_sql}::DOUBLE PRECISION)"),
        RelationAggregateKind::Min => format!("MIN({target_sql}::DOUBLE PRECISION)"),
        RelationAggregateKind::Max => format!("MAX({target_sql}::DOUBLE PRECISION)"),
    }
}

pub fn parse_select_list(select_sql: &str) -> Vec<SelectExpr> {
    select_sql
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(|part| SelectExpr::Column(part.to_string()))
        .collect()
}

fn render_selects(selects: &[SelectExpr]) -> String {
    if selects.is_empty() {
        "*".to_string()
    } else {
        selects
            .iter()
            .map(|expr| expr.sql().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

fn render_order_expr(expr: &OrderExpr) -> String {
    match expr {
        OrderExpr::Column { sql, dir, nulls } => match nulls {
            None => format!("{sql} {}", dir.as_sql()),
            Some(NullsOrder::First) => format!("{sql} {} NULLS FIRST", dir.as_sql()),
            Some(NullsOrder::Last) => format!("{sql} {} NULLS LAST", dir.as_sql()),
        },
        OrderExpr::Raw(sql) => sql.clone(),
    }
}

fn render_lock_clause(lock: LockClause) -> &'static str {
    match (lock.mode, lock.modifier) {
        (LockMode::Update, None) => "FOR UPDATE",
        (LockMode::Update, Some(LockModifier::SkipLocked)) => "FOR UPDATE SKIP LOCKED",
        (LockMode::Update, Some(LockModifier::NoWait)) => "FOR UPDATE NOWAIT",
        (LockMode::NoKeyUpdate, None) => "FOR NO KEY UPDATE",
        (LockMode::NoKeyUpdate, Some(LockModifier::SkipLocked)) => "FOR NO KEY UPDATE SKIP LOCKED",
        (LockMode::NoKeyUpdate, Some(LockModifier::NoWait)) => "FOR NO KEY UPDATE NOWAIT",
        (LockMode::Share, None) => "FOR SHARE",
        (LockMode::Share, Some(LockModifier::SkipLocked)) => "FOR SHARE SKIP LOCKED",
        (LockMode::Share, Some(LockModifier::NoWait)) => "FOR SHARE NOWAIT",
        (LockMode::KeyShare, None) => "FOR KEY SHARE",
        (LockMode::KeyShare, Some(LockModifier::SkipLocked)) => "FOR KEY SHARE SKIP LOCKED",
        (LockMode::KeyShare, Some(LockModifier::NoWait)) => "FOR KEY SHARE NOWAIT",
    }
}

fn raw_join_kind_sql(kind: crate::common::sql::RawJoinKind) -> &'static str {
    match kind {
        crate::common::sql::RawJoinKind::Inner => "INNER JOIN",
        crate::common::sql::RawJoinKind::Left => "LEFT JOIN",
        crate::common::sql::RawJoinKind::Right => "RIGHT JOIN",
        crate::common::sql::RawJoinKind::Full => "FULL JOIN",
    }
}

fn compile_filter_expr(
    expr: &FilterExpr,
    bind_idx: &mut usize,
    binds: &mut Vec<BindValue>,
) -> String {
    match expr {
        FilterExpr::Comparison { col_sql, op, value } => {
            let current = *bind_idx;
            *bind_idx += 1;
            binds.push(value.clone());
            format!("{col_sql} {} ${current}", op.as_sql())
        }
        FilterExpr::ColumnCmp {
            left_sql,
            op,
            right_sql,
        } => format!("{left_sql} {} {right_sql}", op.as_sql()),
        FilterExpr::ExprCmp { col_sql, op, expr } => {
            let expr_sql = match expr {
                Expr::Now => "NOW()".to_string(),
                Expr::NowPlusSeconds(seconds) => {
                    let current = *bind_idx;
                    *bind_idx += 1;
                    binds.push(BindValue::F64(*seconds));
                    format!("NOW() + (${current}::double precision * INTERVAL '1 second')")
                }
                Expr::NowMinusSeconds(seconds) => {
                    let current = *bind_idx;
                    *bind_idx += 1;
                    binds.push(BindValue::F64(*seconds));
                    format!("NOW() - (${current}::double precision * INTERVAL '1 second')")
                }
            };
            format!("{col_sql} {} {expr_sql}", op.as_sql())
        }
        FilterExpr::In {
            col_sql,
            values,
            negated,
        } => {
            let placeholders = values
                .iter()
                .map(|value| {
                    let current = *bind_idx;
                    *bind_idx += 1;
                    binds.push(value.clone());
                    format!("${current}")
                })
                .collect::<Vec<_>>();
            let op = if *negated { "NOT IN" } else { "IN" };
            format!("{col_sql} {op} ({})", placeholders.join(", "))
        }
        FilterExpr::Between { col_sql, low, high } => {
            let low_idx = *bind_idx;
            *bind_idx += 1;
            binds.push(low.clone());
            let high_idx = *bind_idx;
            *bind_idx += 1;
            binds.push(high.clone());
            format!("{col_sql} BETWEEN ${low_idx} AND ${high_idx}")
        }
        FilterExpr::Null { col_sql, negated } => {
            if *negated {
                format!("{col_sql} IS NOT NULL")
            } else {
                format!("{col_sql} IS NULL")
            }
        }
        FilterExpr::Raw { clause, binds: raw } => {
            let sql = crate::common::sql::renumber_placeholders(clause, *bind_idx);
            *bind_idx += raw.len();
            binds.extend(raw.clone());
            sql
        }
        FilterExpr::ExistsRaw { clause, binds: raw } => {
            let sql = crate::common::sql::renumber_placeholders(clause, *bind_idx);
            *bind_idx += raw.len();
            binds.extend(raw.clone());
            format!("EXISTS ({sql})")
        }
        FilterExpr::Group(children) => {
            let compiled = children
                .iter()
                .map(|child| compile_filter_expr(child, bind_idx, binds))
                .collect::<Vec<_>>();
            format!("({})", compiled.join(" AND "))
        }
        FilterExpr::Or(left, right) => {
            let left = compile_filter_expr(left, bind_idx, binds);
            let right = compile_filter_expr(right, bind_idx, binds);
            format!("({left} OR {right})")
        }
    }
}

fn compile_filters(filters: &[FilterExpr], bind_start: usize) -> (Vec<String>, Vec<BindValue>) {
    let mut bind_idx = bind_start;
    let mut binds = Vec::new();
    let clauses = filters
        .iter()
        .map(|expr| compile_filter_expr(expr, &mut bind_idx, &mut binds))
        .collect::<Vec<_>>();
    (clauses, binds)
}

fn compile_joins(joins: &[JoinExpr], bind_start: usize) -> (Vec<String>, Vec<BindValue>) {
    let mut bind_idx = bind_start;
    let mut binds = Vec::new();
    let sql = joins
        .iter()
        .map(|join| {
            let on_clause = crate::common::sql::renumber_placeholders(&join.on_clause, bind_idx);
            bind_idx += join.binds.len();
            binds.extend(join.binds.clone());
            format!("{} {} ON {}", join.kind, join.table, on_clause)
        })
        .collect::<Vec<_>>();
    (sql, binds)
}

fn compile_havings(havings: &[HavingExpr], bind_start: usize) -> (Vec<String>, Vec<BindValue>) {
    let mut bind_idx = bind_start;
    let mut binds = Vec::new();
    let clauses = havings
        .iter()
        .map(|having| {
            let clause = crate::common::sql::renumber_placeholders(&having.clause, bind_idx);
            bind_idx += having.binds.len();
            binds.extend(having.binds.clone());
            clause
        })
        .collect::<Vec<_>>();
    (clauses, binds)
}

fn append_soft_delete_filter(
    clauses: &mut Vec<String>,
    has_soft_delete: bool,
    soft_delete_col: &str,
    with_deleted: bool,
    only_deleted: bool,
) {
    if has_soft_delete {
        if only_deleted {
            clauses.push(format!("{soft_delete_col} IS NOT NULL"));
        } else if !with_deleted {
            clauses.push(format!("{soft_delete_col} IS NULL"));
        }
    }
}

fn compile_predicates(
    table: &str,
    filters: &[FilterExpr],
    existence_relations: &[RootExistenceNode],
    has_soft_delete: bool,
    soft_delete_col: &str,
    with_deleted: bool,
    only_deleted: bool,
    bind_start: usize,
) -> (Vec<String>, Vec<BindValue>) {
    let (mut clauses, mut binds) = compile_filters(filters, bind_start);
    append_soft_delete_filter(
        &mut clauses,
        has_soft_delete,
        soft_delete_col,
        with_deleted,
        only_deleted,
    );
    compile_existence_predicates(table, &mut clauses, &mut binds, existence_relations);
    (clauses, binds)
}

pub fn relation_aggregate_kind_key(kind: RelationAggregateKind) -> &'static str {
    match kind {
        RelationAggregateKind::Sum => "sum",
        RelationAggregateKind::Avg => "avg",
        RelationAggregateKind::Min => "min",
        RelationAggregateKind::Max => "max",
    }
}

pub fn relation_aggregate_key(spec: &RelationAggregateSpec) -> String {
    format!(
        "{}:{}:{}",
        relation_aggregate_kind_key(spec.kind),
        spec.relation_name,
        spec.target.key_fragment()
    )
}

pub fn prefix_nested_count_key(prefix: &str, key: &str) -> String {
    format!("{prefix}.{key}")
}

pub fn prefix_nested_aggregate_key(prefix: &str, key: &str) -> String {
    let mut parts = key.splitn(3, ':');
    let kind = parts.next().unwrap_or_default();
    let path = parts.next().unwrap_or_default();
    let column = parts.next().unwrap_or_default();
    if path.is_empty() {
        format!("{kind}:{prefix}:{column}")
    } else {
        format!("{kind}:{prefix}.{path}:{column}")
    }
}

/// Execute direct count queries for requested relations, returning relation-name → (fk_value → count).
pub async fn execute_relation_counts(
    db: &DbConn<'_>,
    parent_ids: &[BindValue],
    specs: &[CountRelationSpec],
) -> Result<HashMap<String, HashMap<i64, i64>>> {
    let mut result: HashMap<String, HashMap<i64, i64>> = HashMap::new();
    if parent_ids.is_empty() || specs.is_empty() {
        return Ok(result);
    }

    for spec in specs {
        let mut bind_idx: usize = 1;
        let placeholders: Vec<String> = parent_ids
            .iter()
            .map(|_| {
                let p = format!("${}", bind_idx);
                bind_idx += 1;
                p
            })
            .collect();

        let soft_delete_clause = if spec.only_deleted {
            " AND deleted_at IS NOT NULL"
        } else if spec.has_soft_delete && !spec.with_deleted {
            " AND deleted_at IS NULL"
        } else {
            ""
        };
        let (filter_clauses, filter_binds) = compile_filters(&spec.filters, bind_idx);
        let extra_clause = if filter_clauses.is_empty() {
            String::new()
        } else {
            format!(" AND {}", filter_clauses.join(" AND "))
        };
        let sql = format!(
            "SELECT {fk}, COUNT(*) as cnt FROM {table} WHERE {fk} IN ({placeholders}){soft_delete}{extra} GROUP BY {fk}",
            fk = spec.foreign_key,
            table = spec.target_table,
            placeholders = placeholders.join(", "),
            soft_delete = soft_delete_clause,
            extra = extra_clause,
        );

        let mut q = sqlx::query_as::<_, (i64, i64)>(&sql);
        for id in parent_ids {
            q = crate::common::sql::bind(q, id.clone());
        }
        for bind in &filter_binds {
            q = crate::common::sql::bind(q, bind.clone());
        }

        let rows: Vec<(i64, i64)> = db.fetch_all(q).await?;

        let mut counts: HashMap<i64, i64> = HashMap::new();
        for (fk_val, cnt) in rows {
            counts.insert(fk_val, cnt);
        }

        result.insert(spec.name.to_string(), counts);
    }

    Ok(result)
}

/// Execute direct aggregate queries for requested relations, returning aggregate-key -> (fk_value -> value).
pub async fn execute_relation_aggregates(
    db: &DbConn<'_>,
    parent_ids: &[BindValue],
    specs: &[RelationAggregateSpec],
) -> Result<HashMap<String, HashMap<i64, f64>>> {
    let mut result: HashMap<String, HashMap<i64, f64>> = HashMap::new();
    if parent_ids.is_empty() || specs.is_empty() {
        return Ok(result);
    }

    for spec in specs {
        let mut bind_idx: usize = 1;
        let placeholders: Vec<String> = parent_ids
            .iter()
            .map(|_| {
                let p = format!("${}", bind_idx);
                bind_idx += 1;
                p
            })
            .collect();

        let soft_delete_clause = if spec.only_deleted {
            " AND deleted_at IS NOT NULL"
        } else if spec.has_soft_delete && !spec.with_deleted {
            " AND deleted_at IS NULL"
        } else {
            ""
        };
        let (filter_clauses, filter_binds) = compile_filters(&spec.filters, bind_idx);
        let extra_clause = if filter_clauses.is_empty() {
            String::new()
        } else {
            format!(" AND {}", filter_clauses.join(" AND "))
        };
        let sql = format!(
            "SELECT {fk}, {agg} as value FROM {table} WHERE {fk} IN ({placeholders}){soft_delete}{extra} GROUP BY {fk}",
            fk = spec.foreign_key,
            agg = relation_aggregate_sql(spec.kind, spec.target.sql()),
            table = spec.target_table,
            placeholders = placeholders.join(", "),
            soft_delete = soft_delete_clause,
            extra = extra_clause,
        );

        let mut q = sqlx::query_as::<_, (i64, Option<f64>)>(&sql);
        for id in parent_ids {
            q = crate::common::sql::bind(q, id.clone());
        }
        for bind in &filter_binds {
            q = crate::common::sql::bind(q, bind.clone());
        }

        let rows: Vec<(i64, Option<f64>)> = db.fetch_all(q).await?;
        let mut values: HashMap<i64, f64> = HashMap::new();
        for (fk_val, value) in rows {
            if let Some(value) = value {
                values.insert(fk_val, value);
            }
        }

        result.insert(relation_aggregate_key(spec), values);
    }

    Ok(result)
}

pub trait ModelDef: Sized + 'static {
    type Pk: Clone + Send + Sync + Into<BindValue> + 'static;
    type Record: Clone + Send + Sync + 'static;
    type Create: Send + 'static;
    type Changes: Send + 'static;

    const TABLE: &'static str;
    const MODEL_KEY: &'static str;
    const PK_COL: &'static str;
}

pub trait QueryModel: ModelDef {
    const DEFAULT_SELECT: &'static str;
    const HAS_SOFT_DELETE: bool;
    const SOFT_DELETE_COL: &'static str;
    const HAS_CREATED_AT: bool;
    const HAS_UPDATED_AT: bool;
    const PROFILE_QUERIES: bool = true;
    const OBSERVE_HOOKS: bool = true;

    fn query_all<'db>(
        db: DbConn<'db>,
        state: QueryState<'db>,
    ) -> BoxModelFuture<'db, Vec<Self::Record>>
    where
        Self: RuntimeModel,
        Self::Record: RelationMetricRecord,
    {
        query_all_runtime::<Self>(db, state)
    }

    fn query_first<'db>(
        db: DbConn<'db>,
        state: QueryState<'db>,
    ) -> BoxModelFuture<'db, Option<Self::Record>>
    where
        Self: RuntimeModel,
        Self::Record: RelationMetricRecord,
    {
        Box::pin(async move {
            let mut rows = Self::query_all(db, state.limit(1)).await?;
            Ok(rows.pop())
        })
    }

    fn query_find<'db>(
        db: DbConn<'db>,
        state: QueryState<'db>,
        id: Self::Pk,
    ) -> BoxModelFuture<'db, Option<Self::Record>>
    where
        Self: RuntimeModel,
        Self::Record: RelationMetricRecord,
    {
        Box::pin(async move {
            Self::query_first(db, state.where_col_str(Self::PK_COL, Op::Eq, id.into())).await
        })
    }

    fn query_count<'db>(db: DbConn<'db>, state: QueryState<'db>) -> BoxModelFuture<'db, i64> {
        query_count_runtime::<Self>(db, state)
    }

    fn query_delete<'db>(db: DbConn<'db>, state: QueryState<'db>) -> BoxModelFuture<'db, u64>
    where
        Self: DeleteModel,
        Self::Row: serde::Serialize,
    {
        query_delete_runtime::<Self>(db, state)
    }

    fn query_paginate<'db>(
        db: DbConn<'db>,
        state: QueryState<'db>,
        page: i64,
        per_page: i64,
    ) -> BoxModelFuture<'db, Page<Self::Record>>
    where
        Self: RuntimeModel,
        Self::Record: RelationMetricRecord,
    {
        query_paginate_runtime::<Self>(db, state, page, per_page)
    }
}

pub trait ChunkModel: QueryModel + RuntimeModel {
    fn record_pk(record: &Self::Record) -> Self::Pk;
}

pub trait DeleteModel: RuntimeModel {
    fn row_pk(row: &Self::Row) -> Self::Pk;
    fn row_pk_text(row: &Self::Row) -> String;

    fn delete_override_update<'db>(
        db: DbConn<'db>,
        ids: Vec<Self::Pk>,
        overrides: serde_json::Value,
    ) -> BoxModelFuture<'db, u64>;
}

pub trait FeaturePersistenceModel: ModelDef {
    fn create_owner_id(_row: &<Self as RuntimeModel>::Row) -> Option<i64>
    where
        Self: RuntimeModel,
    {
        None
    }

    fn patch_owner_id(_pk: &Self::Pk) -> Option<i64> {
        None
    }

    fn supported_locales() -> &'static [&'static str] {
        &[]
    }

    fn localized_owner_type() -> Option<&'static str> {
        None
    }

    fn meta_owner_type() -> Option<&'static str> {
        None
    }

    fn attachment_owner_type() -> Option<&'static str> {
        None
    }

    fn upsert_localized_many<'db>(
        db: DbConn<'db>,
        owner_type: &'static str,
        owner_id: i64,
        field: &'static str,
        values: HashMap<String, String>,
    ) -> BoxModelFuture<'db, ()> {
        let _ = (db, owner_type, owner_id, field, values);
        Box::pin(async move { Ok(()) })
    }

    fn upsert_meta_many<'db>(
        db: DbConn<'db>,
        owner_type: &'static str,
        owner_id: i64,
        values: HashMap<String, serde_json::Value>,
    ) -> BoxModelFuture<'db, ()> {
        let _ = (db, owner_type, owner_id, values);
        Box::pin(async move { Ok(()) })
    }

    fn clear_attachment_field<'db>(
        db: DbConn<'db>,
        owner_type: &'static str,
        owner_id: i64,
        field: &'static str,
    ) -> BoxModelFuture<'db, ()> {
        let _ = (db, owner_type, owner_id, field);
        Box::pin(async move { Ok(()) })
    }

    fn replace_single_attachment<'db>(
        db: DbConn<'db>,
        owner_type: &'static str,
        owner_id: i64,
        field: &'static str,
        value: AttachmentInput,
    ) -> BoxModelFuture<'db, ()> {
        let _ = (db, owner_type, owner_id, field, value);
        Box::pin(async move { Ok(()) })
    }

    fn add_attachments<'db>(
        db: DbConn<'db>,
        owner_type: &'static str,
        owner_id: i64,
        field: &'static str,
        values: Vec<AttachmentInput>,
    ) -> BoxModelFuture<'db, ()> {
        let _ = (db, owner_type, owner_id, field, values);
        Box::pin(async move { Ok(()) })
    }

    fn delete_attachment_ids<'db>(
        db: DbConn<'db>,
        owner_type: &'static str,
        owner_id: i64,
        field: &'static str,
        ids: Vec<Uuid>,
    ) -> BoxModelFuture<'db, ()> {
        let _ = (db, owner_type, owner_id, field, ids);
        Box::pin(async move { Ok(()) })
    }

    fn persist_create_related<'db>(
        db: DbConn<'db>,
        row: <Self as RuntimeModel>::Row,
    ) -> BoxModelFuture<'db, ()>
    where
        Self: RuntimeModel,
    {
        let _ = (db, row);
        Box::pin(async move { Ok(()) })
    }

    fn persist_patch_related<'db>(
        db: DbConn<'db>,
        target_ids: Vec<Self::Pk>,
    ) -> BoxModelFuture<'db, ()> {
        let _ = (db, target_ids);
        Box::pin(async move { Ok(()) })
    }
}

pub trait QueryField<M: QueryModel>: Copy {
    type Value: Clone + Into<BindValue>;

    fn where_col<'db>(
        field: Self,
        state: QueryState<'db>,
        op: Op,
        value: Self::Value,
    ) -> QueryState<'db>;
    fn or_where_col<'db>(
        field: Self,
        state: QueryState<'db>,
        op: Op,
        value: Self::Value,
    ) -> QueryState<'db>;
    fn where_in<'db>(
        field: Self,
        state: QueryState<'db>,
        values: &[Self::Value],
    ) -> QueryState<'db>;
    fn order_by<'db>(field: Self, state: QueryState<'db>, dir: OrderDir) -> QueryState<'db>;
    fn where_null<'db>(field: Self, state: QueryState<'db>) -> QueryState<'db>;
    fn where_not_null<'db>(field: Self, state: QueryState<'db>) -> QueryState<'db>;
}

pub trait IncludeRelation<M: QueryModel>: Copy {
    fn load_spec<'db>(relation: Self, base_url: Option<String>) -> WithRelationSpec;
}

pub trait WhereHasRelation<M: QueryModel>: Copy {
    type Target: QueryModel;

    fn where_has<'db, F>(relation: Self, state: QueryState<'db>, scope: F) -> QueryState<'db>
    where
        F: FnOnce(Query<'db, Self::Target>) -> Query<'db, Self::Target>;

    fn or_where_has<'db, F>(relation: Self, state: QueryState<'db>, scope: F) -> QueryState<'db>
    where
        F: FnOnce(Query<'db, Self::Target>) -> Query<'db, Self::Target>;
}

pub trait RecordOneRelation<M: ModelDef>: Copy {
    type Target;

    fn get<'a>(relation: Self, record: &'a M::Record) -> Option<&'a Self::Target>;
}

pub trait RecordManyRelation<M: ModelDef>: Copy {
    type Target;

    fn get<'a>(relation: Self, record: &'a M::Record) -> &'a [Self::Target];
}

pub trait RelationMetricRecord {
    fn relation_counts(&self) -> &HashMap<String, i64>;
    fn relation_aggregates(&self) -> &HashMap<String, f64>;
    fn relation_counts_mut(&mut self) -> &mut HashMap<String, i64>;
    fn relation_aggregates_mut(&mut self) -> &mut HashMap<String, f64>;
}

pub trait CountRelation<M: ModelDef>: Copy {
    type TargetModel: QueryModel;

    fn name(relation: Self) -> &'static str;
    fn spec<'db>(relation: Self, base_url: Option<String>) -> CountRelationSpec;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectExpr {
    Column(String),
    Raw(String),
}

impl SelectExpr {
    fn sql(&self) -> &str {
        match self {
            SelectExpr::Column(sql) | SelectExpr::Raw(sql) => sql.as_str(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NullsOrder {
    First,
    Last,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrderExpr {
    Column {
        sql: String,
        dir: OrderDir,
        nulls: Option<NullsOrder>,
    },
    Raw(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Now,
    NowPlusSeconds(f64),
    NowMinusSeconds(f64),
}

impl Expr {
    pub fn now() -> Self {
        Self::Now
    }

    pub fn now_plus(duration: time::Duration) -> Self {
        Self::NowPlusSeconds(duration.as_seconds_f64())
    }

    pub fn now_minus(duration: time::Duration) -> Self {
        Self::NowMinusSeconds(duration.as_seconds_f64())
    }
}

#[derive(Debug, Clone)]
pub enum FilterExpr {
    Comparison {
        col_sql: String,
        op: Op,
        value: BindValue,
    },
    ColumnCmp {
        left_sql: String,
        op: Op,
        right_sql: String,
    },
    ExprCmp {
        col_sql: String,
        op: Op,
        expr: Expr,
    },
    In {
        col_sql: String,
        values: Vec<BindValue>,
        negated: bool,
    },
    Between {
        col_sql: String,
        low: BindValue,
        high: BindValue,
    },
    Null {
        col_sql: String,
        negated: bool,
    },
    Raw {
        clause: String,
        binds: Vec<BindValue>,
    },
    ExistsRaw {
        clause: String,
        binds: Vec<BindValue>,
    },
    Group(Vec<FilterExpr>),
    Or(Box<FilterExpr>, Box<FilterExpr>),
}

#[derive(Debug, Clone)]
pub struct JoinExpr {
    pub kind: String,
    pub table: String,
    pub on_clause: String,
    pub binds: Vec<BindValue>,
}

#[derive(Debug, Clone)]
pub struct HavingExpr {
    pub clause: String,
    pub binds: Vec<BindValue>,
}

/// Specification for a relation count to be loaded alongside the main query.
#[derive(Debug, Clone)]
pub struct CountRelationSpec {
    pub name: &'static str,
    pub target_table: &'static str,
    pub target_pk: &'static str,
    pub foreign_key: &'static str,
    pub has_soft_delete: bool,
    pub filters: Vec<FilterExpr>,
    pub with_deleted: bool,
    pub only_deleted: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationAggregateKind {
    Sum,
    Avg,
    Min,
    Max,
}

#[derive(Debug, Clone)]
pub struct RelationAggregateSpec {
    pub relation_name: &'static str,
    pub target_table: &'static str,
    pub target_pk: &'static str,
    pub foreign_key: &'static str,
    pub has_soft_delete: bool,
    pub target: AggregateTargetSpec,
    pub kind: RelationAggregateKind,
    pub filters: Vec<FilterExpr>,
    pub with_deleted: bool,
    pub only_deleted: bool,
}

#[derive(Debug, Clone)]
pub enum AggregateTargetSpec {
    Column(&'static str),
    Expr(String),
}

impl AggregateTargetSpec {
    pub fn sql(&self) -> &str {
        match self {
            AggregateTargetSpec::Column(sql) => sql,
            AggregateTargetSpec::Expr(sql) => sql.as_str(),
        }
    }

    pub fn key_fragment(&self) -> String {
        match self {
            AggregateTargetSpec::Column(sql) => (*sql).to_string(),
            AggregateTargetSpec::Expr(sql) => sql.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AggregateTarget<M> {
    spec: AggregateTargetSpec,
    _marker: PhantomData<fn() -> M>,
}

impl<M> AggregateTarget<M> {
    pub fn expr(sql: impl Into<String>) -> Self {
        Self {
            spec: AggregateTargetSpec::Expr(sql.into()),
            _marker: PhantomData,
        }
    }

    pub fn into_spec(self) -> AggregateTargetSpec {
        self.spec
    }
}

impl<M, T> From<Column<M, T>> for AggregateTarget<M> {
    fn from(value: Column<M, T>) -> Self {
        Self {
            spec: AggregateTargetSpec::Column(value.as_sql()),
            _marker: PhantomData,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ReturnCol<M> {
    sql: &'static str,
    _marker: PhantomData<fn() -> M>,
}

impl<M> ReturnCol<M> {
    pub const fn sql(self) -> &'static str {
        self.sql
    }
}

impl<M, T> From<Column<M, T>> for ReturnCol<M> {
    fn from(value: Column<M, T>) -> Self {
        Self {
            sql: value.as_sql(),
            _marker: PhantomData,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ReturnExpr {
    Column(&'static str),
    Raw(String),
}

#[derive(Debug, Clone)]
pub struct JsonReturnField {
    pub key: String,
    pub expr: ReturnExpr,
}

#[derive(Debug, Clone)]
pub enum ReturningSpec {
    Scalar(ReturnExpr),
    JsonObject(Vec<JsonReturnField>),
    JsonExpr(ReturnExpr),
    All,
}

fn render_return_expr(expr: &ReturnExpr) -> String {
    match expr {
        ReturnExpr::Column(sql) => (*sql).to_string(),
        ReturnExpr::Raw(sql) => sql.clone(),
    }
}

fn render_returning_spec(spec: &ReturningSpec) -> String {
    match spec {
        ReturningSpec::Scalar(expr) => render_return_expr(expr),
        ReturningSpec::JsonObject(fields) => {
            let pairs = fields
                .iter()
                .map(|field| format!("'{}', {}", field.key, render_return_expr(&field.expr)))
                .collect::<Vec<_>>()
                .join(", ");
            format!("json_build_object({pairs})")
        }
        ReturningSpec::JsonExpr(expr) => format!("to_jsonb({})", render_return_expr(expr)),
        ReturningSpec::All => "*".to_string(),
    }
}

pub fn relation_aggregate_lookup_key(
    relation_name: &str,
    kind: RelationAggregateKind,
    target: &AggregateTargetSpec,
) -> String {
    format!(
        "{}:{}:{}",
        relation_aggregate_kind_key(kind),
        relation_name,
        target.key_fragment()
    )
}

/// Trait for any relation constant that can provide its name.
/// Used by `Query::with()` to specify which relations to eager-load.
pub trait RelationName: Copy {
    fn relation_name(self) -> &'static str;
}

impl<M, T, const K: usize> RelationName for OneRelation<M, T, K> {
    fn relation_name(self) -> &'static str {
        self.name()
    }
}

impl<M, T, const K: usize> RelationName for ManyRelation<M, T, K> {
    fn relation_name(self) -> &'static str {
        self.name()
    }
}

/// Specification for a relation to eager-load, with optional WHERE conditions.
#[derive(Debug, Clone)]
pub struct WithRelationSpec {
    pub name: &'static str,
    pub kind: &'static str,
    pub target_table: &'static str,
    pub target_pk: &'static str,
    pub foreign_key: &'static str,
    pub local_key: &'static str,
    pub has_soft_delete: bool,
    pub selects: Vec<SelectExpr>,
    pub filters: Vec<FilterExpr>,
    pub orders: Vec<OrderExpr>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub with_deleted: bool,
    pub only_deleted: bool,
    pub nested: Vec<WithRelationSpec>,
    pub counts: Vec<CountRelationSpec>,
    pub aggregates: Vec<RelationAggregateSpec>,
}

pub type RelationLoadNode = WithRelationSpec;
pub type AggregateExpr = RelationAggregateSpec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockMode {
    Update,
    NoKeyUpdate,
    Share,
    KeyShare,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockModifier {
    SkipLocked,
    NoWait,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LockClause {
    pub mode: LockMode,
    pub modifier: Option<LockModifier>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExistenceBoolean {
    And,
    Or,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExistenceOperator {
    Exists,
}

#[derive(Debug, Clone)]
pub struct RelationExistenceNode {
    pub name: &'static str,
    pub kind: &'static str,
    pub target_table: &'static str,
    pub target_pk: &'static str,
    pub foreign_key: &'static str,
    pub local_key: &'static str,
    pub has_soft_delete: bool,
    pub filters: Vec<FilterExpr>,
    pub with_deleted: bool,
    pub only_deleted: bool,
    pub operator: ExistenceOperator,
    pub count: Option<i64>,
    pub children: Vec<RelationExistenceNode>,
}

#[derive(Debug, Clone)]
pub struct RootExistenceNode {
    pub boolean: ExistenceBoolean,
    pub node: RelationExistenceNode,
}

pub trait ErasedRelationRuntime<M: RuntimeModel>: Send + Sync {
    fn name(&self) -> &'static str;
    fn apply<'db>(
        &self,
        db: DbConn<'db>,
        parents: Vec<M::Record>,
        base_url: Option<String>,
        spec: WithRelationSpec,
    ) -> BoxModelFuture<'db, Vec<M::Record>>;
}

pub trait RuntimeModel: QueryModel {
    type Row: Clone
        + Send
        + Sync
        + Unpin
        + for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow>
        + 'static;

    fn hydrate_records<'db>(
        db: DbConn<'db>,
        rows: Vec<Self::Row>,
        base_url: Option<String>,
    ) -> BoxModelFuture<'db, Vec<Self::Record>>;

    fn record_pk_i64(_record: &Self::Record) -> Option<i64> {
        None
    }

    fn relation_runtimes() -> &'static [&'static dyn ErasedRelationRuntime<Self>] {
        &[]
    }
}

fn profiler_binds_string(binds: &[BindValue]) -> String {
    binds
        .iter()
        .map(|b| format!("{b}"))
        .collect::<Vec<_>>()
        .join(", ")
}

pub fn query_all_runtime<'db, M>(
    db: DbConn<'db>,
    state: QueryState<'db>,
) -> BoxModelFuture<'db, Vec<M::Record>>
where
    M: RuntimeModel,
    M::Record: RelationMetricRecord,
{
    Box::pin(async move {
        let base_url = state.base_url.clone();
        let relation_base_url = state.base_url.clone();
        let with_relations = state.with_relations.clone();
        let count_relations = state.count_relations.clone();
        let aggregate_relations = state.aggregate_relations.clone();

        let (sql, binds) = state.to_select_sql(M::TABLE, M::HAS_SOFT_DELETE, M::SOFT_DELETE_COL);
        let profiler_binds = if M::PROFILE_QUERIES && crate::common::sql::is_sql_profiler_enabled()
        {
            profiler_binds_string(&binds)
        } else {
            String::new()
        };
        let profiler_start = if M::PROFILE_QUERIES {
            Some(std::time::Instant::now())
        } else {
            None
        };

        let mut q = sqlx::query_as::<_, M::Row>(&sql);
        for bind in binds {
            q = crate::common::sql::bind(q, bind);
        }
        let rows = db.fetch_all(q).await?;

        if let Some(start) = profiler_start {
            crate::common::sql::record_profiled_query(
                M::TABLE,
                "SELECT",
                &sql,
                &profiler_binds,
                start.elapsed(),
            );
        }

        let mut records = M::hydrate_records(db.clone(), rows, base_url).await?;
        if let Some(with_relations) = with_relations.as_ref() {
            apply_loaded_relations::<M>(
                db.clone(),
                &mut records,
                relation_base_url,
                with_relations,
            )
            .await?;
        }
        apply_loaded_metrics::<M>(
            db.clone(),
            &mut records,
            &count_relations,
            &aggregate_relations,
        )
        .await?;
        Ok(records)
    })
}

pub fn query_count_runtime<'db, M>(
    db: DbConn<'db>,
    state: QueryState<'db>,
) -> BoxModelFuture<'db, i64>
where
    M: QueryModel,
{
    Box::pin(async move {
        let (sql, binds) = state.to_count_sql(M::TABLE, M::HAS_SOFT_DELETE, M::SOFT_DELETE_COL);
        let profiler_binds = if M::PROFILE_QUERIES && crate::common::sql::is_sql_profiler_enabled()
        {
            profiler_binds_string(&binds)
        } else {
            String::new()
        };
        let profiler_start = if M::PROFILE_QUERIES {
            Some(std::time::Instant::now())
        } else {
            None
        };
        let mut q = sqlx::query_scalar::<_, i64>(&sql);
        for bind in binds {
            q = crate::common::sql::bind_scalar(q, bind);
        }
        let count = db.fetch_scalar(q).await?;
        if let Some(start) = profiler_start {
            crate::common::sql::record_profiled_query(
                M::TABLE,
                "COUNT",
                &sql,
                &profiler_binds,
                start.elapsed(),
            );
        }
        Ok(count)
    })
}

pub fn query_paginate_runtime<'db, M>(
    db: DbConn<'db>,
    state: QueryState<'db>,
    page: i64,
    per_page: i64,
) -> BoxModelFuture<'db, Page<M::Record>>
where
    M: RuntimeModel,
    M::Record: RelationMetricRecord,
{
    Box::pin(async move {
        let page = if page < 1 { 1 } else { page };
        let per_page = crate::common::pagination::resolve_per_page(per_page);

        let count = M::query_count(db.clone(), state.clone()).await?;
        let last_page = ((count + per_page - 1) / per_page).max(1);
        let current_page = page.min(last_page);
        let offset_val = (current_page - 1) * per_page;

        let mut page_state = state;
        page_state.offset = Some(offset_val);
        page_state.limit = Some(per_page);
        let data = M::query_all(db, page_state).await?;

        Ok(Page {
            data,
            total: count,
            per_page,
            current_page,
            last_page,
        })
    })
}

pub fn query_delete_runtime<'db, M>(
    db: DbConn<'db>,
    state: QueryState<'db>,
) -> BoxModelFuture<'db, u64>
where
    M: DeleteModel,
    M::Row: serde::Serialize,
{
    Box::pin(async move {
        use crate::common::model_observer::{
            log_observer_error, try_get_observer, ModelEvent, ObserverAction,
        };
        use crate::common::sql::{bind, bind_query};

        if state.limit.is_some() {
            anyhow::bail!("delete() does not support limit; add where clauses");
        }

        let (where_sql, binds) =
            state.predicate_parts(M::TABLE, M::HAS_SOFT_DELETE, M::SOFT_DELETE_COL);
        if where_sql.is_empty() {
            anyhow::bail!("delete(): no conditions set");
        }

        let profiler_binds = if M::PROFILE_QUERIES && crate::common::sql::is_sql_profiler_enabled()
        {
            profiler_binds_string(&binds)
        } else {
            String::new()
        };

        let old_rows: Vec<M::Row> = if M::OBSERVE_HOOKS && try_get_observer().is_some() {
            let select_sql = format!(
                "SELECT * FROM {} WHERE {}",
                M::TABLE,
                where_sql.join(" AND ")
            );
            let mut query = sqlx::query_as::<_, M::Row>(&select_sql);
            for bind_value in &binds {
                query = bind(query, bind_value.clone());
            }
            db.fetch_all(query).await.unwrap_or_default()
        } else {
            Vec::new()
        };

        if !old_rows.is_empty() {
            if let Some(observer) = try_get_observer() {
                let old_data = serde_json::to_value(&old_rows)?;
                let event = ModelEvent {
                    model: M::MODEL_KEY,
                    table: M::TABLE,
                    record_key: None,
                };
                match observer.on_deleting(&event, &old_data).await? {
                    ObserverAction::Prevent(err) => return Err(err),
                    ObserverAction::Modify(overrides) => {
                        let ids = old_rows.iter().map(M::row_pk).collect::<Vec<_>>();
                        return M::delete_override_update(db.clone(), ids, overrides).await;
                    }
                    ObserverAction::Continue => {}
                }
            }
        }

        if M::HAS_SOFT_DELETE {
            let idx = binds.len() + 1;
            let mut sql = format!("UPDATE {} SET {} = ${idx}", M::TABLE, M::SOFT_DELETE_COL);
            sql.push_str(" WHERE ");
            sql.push_str(&where_sql.join(" AND "));
            let profiler_start = if M::PROFILE_QUERIES {
                Some(std::time::Instant::now())
            } else {
                None
            };
            let mut query = sqlx::query(&sql);
            for bind_value in binds {
                query = bind_query(query, bind_value);
            }
            query = bind_query(query, time::OffsetDateTime::now_utc().into());
            let result = db.execute(query).await?;
            if let Some(start) = profiler_start {
                crate::common::sql::record_profiled_query(
                    M::TABLE,
                    "UPDATE",
                    &sql,
                    &profiler_binds,
                    start.elapsed(),
                );
            }
            if !old_rows.is_empty() && result.rows_affected() > 0 {
                if let Some(observer) = try_get_observer() {
                    for old_row in &old_rows {
                        let event = ModelEvent {
                            model: M::MODEL_KEY,
                            table: M::TABLE,
                            record_key: Some(M::row_pk_text(old_row)),
                        };
                        match serde_json::to_value(old_row) {
                            Ok(old_data) => {
                                if let Err(err) = observer.on_deleted(&event, &old_data).await {
                                    log_observer_error("deleted", M::MODEL_KEY, &err);
                                }
                            }
                            Err(err) => log_observer_error("deleted", M::MODEL_KEY, &err),
                        }
                    }
                }
            }
            Ok(result.rows_affected())
        } else {
            let mut sql = format!("DELETE FROM {}", M::TABLE);
            sql.push_str(" WHERE ");
            sql.push_str(&where_sql.join(" AND "));
            let profiler_start = if M::PROFILE_QUERIES {
                Some(std::time::Instant::now())
            } else {
                None
            };
            let mut query = sqlx::query(&sql);
            for bind_value in binds {
                query = bind_query(query, bind_value);
            }
            let result = db.execute(query).await?;
            if let Some(start) = profiler_start {
                crate::common::sql::record_profiled_query(
                    M::TABLE,
                    "DELETE",
                    &sql,
                    &profiler_binds,
                    start.elapsed(),
                );
            }
            if !old_rows.is_empty() && result.rows_affected() > 0 {
                if let Some(observer) = try_get_observer() {
                    for old_row in &old_rows {
                        let event = ModelEvent {
                            model: M::MODEL_KEY,
                            table: M::TABLE,
                            record_key: Some(M::row_pk_text(old_row)),
                        };
                        match serde_json::to_value(old_row) {
                            Ok(old_data) => {
                                if let Err(err) = observer.on_deleted(&event, &old_data).await {
                                    log_observer_error("deleted", M::MODEL_KEY, &err);
                                }
                            }
                            Err(err) => log_observer_error("deleted", M::MODEL_KEY, &err),
                        }
                    }
                }
            }
            Ok(result.rows_affected())
        }
    })
}

pub fn create_save_runtime<'db, M>(
    db: DbConn<'db>,
    mut state: CreateState<'db>,
) -> BoxModelFuture<'db, M::Record>
where
    M: CreateModel + RuntimeModel,
    M::Create: Serialize,
    M::Row: Serialize,
{
    Box::pin(async move {
        use crate::common::model_observer::{log_observer_error, try_get_observer, ModelEvent, ObserverAction};

        let create_input = if try_get_observer().is_some() {
            Some(M::build_create_input(&state)?)
        } else {
            None
        };

        if let Some(observer) = try_get_observer() {
            if let Some(create_input) = create_input.as_ref() {
                let event = ModelEvent {
                    model: M::MODEL_KEY,
                    table: M::TABLE,
                    record_key: None,
                };
                let data = serde_json::to_value(create_input)?;
                match observer.on_creating(&event, &data).await? {
                    ObserverAction::Prevent(err) => return Err(err),
                    ObserverAction::Modify(overrides) => {
                        state = M::apply_create_overrides(state, overrides)?;
                    }
                    ObserverAction::Continue => {}
                }
            }
        }

        match db {
            DbConn::Pool(pool) => {
                let tx = pool.begin().await?;
                let tx_lock = std::sync::Arc::new(tokio::sync::Mutex::new(tx));
                let (record, row) = {
                    let db = DbConn::tx(tx_lock.clone());
                    create_save_with_db_runtime::<M>(db, state).await?
                };
                let tx = std::sync::Arc::try_unwrap(tx_lock)
                    .map_err(|_| anyhow::anyhow!("transaction scope still has active handles"))?
                    .into_inner();
                tx.commit().await?;

                if let Some(observer) = try_get_observer() {
                    let event = ModelEvent {
                        model: M::MODEL_KEY,
                        table: M::TABLE,
                        record_key: Some(M::created_row_key(&row)),
                    };
                    match serde_json::to_value(&row) {
                        Ok(data) => {
                            if let Err(err) = observer.on_created(&event, &data).await {
                                log_observer_error("created", M::MODEL_KEY, &err);
                            }
                        }
                        Err(err) => log_observer_error("created", M::MODEL_KEY, &err),
                    }
                }

                Ok(record)
            }
            DbConn::Tx(_) => {
                let (record, row) = create_save_with_db_runtime::<M>(db, state).await?;

                if let Some(observer) = try_get_observer() {
                    let event = ModelEvent {
                        model: M::MODEL_KEY,
                        table: M::TABLE,
                        record_key: Some(M::created_row_key(&row)),
                    };
                    match serde_json::to_value(&row) {
                        Ok(data) => {
                            if let Err(err) = observer.on_created(&event, &data).await {
                                log_observer_error("created", M::MODEL_KEY, &err);
                            }
                        }
                        Err(err) => log_observer_error("created", M::MODEL_KEY, &err),
                    }
                }

                Ok(record)
            }
        }
    })
}

pub async fn create_save_with_db_runtime<'db, M>(
    db: DbConn<'db>,
    mut state: CreateState<'db>,
) -> Result<(M::Record, M::Row)>
where
    M: CreateModel + RuntimeModel + FeaturePersistenceModel,
    M::Row: Serialize,
{
    if M::USE_SNOWFLAKE_ID && !state.has_col(M::PK_COL) {
        state = state.set_col(M::PK_COL, crate::common::sql::generate_snowflake_i64().into());
    }
    if M::HAS_CREATED_AT && !state.has_col("created_at") {
        state = state.set_col("created_at", time::OffsetDateTime::now_utc().into());
    }
    if M::HAS_UPDATED_AT && !state.has_col("updated_at") {
        state = state.set_col("updated_at", time::OffsetDateTime::now_utc().into());
    }
    if state.assignments.is_empty() {
        anyhow::bail!("insert: no columns set");
    }

    let base_url = state.base_url.clone();
    let (sql, binds) = state.build_insert_sql();
    let profiler_binds = if M::PROFILE_QUERIES && crate::common::sql::is_sql_profiler_enabled() {
        profiler_binds_string(&binds)
    } else {
        String::new()
    };
    let profiler_start = if M::PROFILE_QUERIES {
        Some(std::time::Instant::now())
    } else {
        None
    };

    let mut q = sqlx::query_as::<_, M::Row>(&sql);
    for bind in binds {
        q = crate::common::sql::bind(q, bind);
    }
    let row = db.fetch_one(q).await?;

    if let Some(start) = profiler_start {
        crate::common::sql::record_profiled_query(
            M::TABLE,
            "INSERT",
            &sql,
            &profiler_binds,
            start.elapsed(),
        );
    }

    M::persist_create_state(db.clone(), row.clone(), state.clone()).await?;
    let mut records = M::hydrate_records(db, vec![row.clone()], base_url).await?;
    let record = records
        .pop()
        .ok_or_else(|| anyhow::anyhow!("{}: created record not found", M::TABLE))?;
    Ok((record, row))
}

fn filter_supported_localized_values<M: FeaturePersistenceModel>(
    values: &HashMap<String, String>,
) -> HashMap<String, String> {
    let supported = M::supported_locales();
    if supported.is_empty() {
        return values.clone();
    }
    values
        .iter()
        .filter(|(locale, _)| supported.contains(&locale.as_str()))
        .map(|(locale, value)| (locale.clone(), value.clone()))
        .collect()
}

async fn persist_feature_state_for_owner<'db, M>(
    db: DbConn<'db>,
    owner_id: i64,
    translations: &HashMap<&'static str, HashMap<String, String>>,
    meta: &HashMap<String, serde_json::Value>,
    attachments_single: &HashMap<&'static str, AttachmentInput>,
    attachments_multi: &HashMap<&'static str, Vec<AttachmentInput>>,
    attachments_clear_single: &[&'static str],
    attachments_delete_multi: &HashMap<&'static str, Vec<Uuid>>,
) -> Result<()>
where
    M: FeaturePersistenceModel,
{
    if let Some(owner_type) = M::localized_owner_type() {
        for (field, values) in translations {
            let filtered = filter_supported_localized_values::<M>(values);
            if !filtered.is_empty() {
                M::upsert_localized_many(db.clone(), owner_type, owner_id, field, filtered).await?;
            }
        }
    }

    if let Some(owner_type) = M::meta_owner_type() {
        if !meta.is_empty() {
            M::upsert_meta_many(db.clone(), owner_type, owner_id, meta.clone()).await?;
        }
    }

    if let Some(owner_type) = M::attachment_owner_type() {
        for field in attachments_clear_single {
            M::clear_attachment_field(db.clone(), owner_type, owner_id, field).await?;
        }
        for (field, value) in attachments_single {
            M::replace_single_attachment(db.clone(), owner_type, owner_id, field, value.clone()).await?;
        }
        for (field, values) in attachments_multi {
            if !values.is_empty() {
                M::add_attachments(db.clone(), owner_type, owner_id, field, values.clone()).await?;
            }
        }
        for (field, ids) in attachments_delete_multi {
            if !ids.is_empty() {
                M::delete_attachment_ids(db.clone(), owner_type, owner_id, field, ids.clone()).await?;
            }
        }
    }

    Ok(())
}

async fn patch_select_target_ids<'db, M>(db: DbConn<'db>, state: &PatchState<'db>) -> Result<Vec<M::Pk>>
where
    M: PatchModel,
    M::Pk: Send
        + Unpin
        + for<'r> sqlx::Decode<'r, sqlx::Postgres>
        + sqlx::Type<sqlx::Postgres>,
{
    let (select_sql, select_binds) = state.build_target_ids_select_sql(M::PK_COL);
    let mut select_q = sqlx::query_scalar::<_, M::Pk>(&select_sql);
    for bind in select_binds {
        select_q = crate::common::sql::bind_scalar(select_q, bind);
    }
    Ok(db.fetch_all_scalar(select_q).await?)
}

async fn patch_fetch_rows_by_ids<'db, M>(db: DbConn<'db>, ids: &[M::Pk]) -> Result<Vec<M::Row>>
where
    M: PatchModel + RuntimeModel,
{
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let phs: Vec<String> = (1..=ids.len()).map(|i| format!("${i}")).collect();
    let sql = format!(
        "SELECT * FROM {} WHERE {} IN ({})",
        M::TABLE,
        M::PK_COL,
        phs.join(", ")
    );
    let mut query = sqlx::query_as::<_, M::Row>(&sql);
    for id in ids {
        query = crate::common::sql::bind(query, id.clone().into());
    }
    Ok(db.fetch_all(query).await?)
}

async fn patch_fetch_row_by_pk<'db, M>(db: DbConn<'db>, id: M::Pk) -> Result<Option<M::Row>>
where
    M: PatchModel + RuntimeModel,
{
    let sql = format!("SELECT * FROM {} WHERE {} = $1", M::TABLE, M::PK_COL);
    let query = crate::common::sql::bind(sqlx::query_as::<_, M::Row>(&sql), id.into());
    Ok(db.fetch_optional(query).await?)
}

async fn patch_prepare_runtime<'db, M>(
    db: DbConn<'db>,
    mut state: PatchState<'db>,
) -> Result<(PatchState<'db>, Vec<M::Pk>, Vec<M::Row>)>
where
    M: PatchModel + RuntimeModel,
    M::Changes: Serialize + Default,
    M::Pk: Clone
        + Send
        + Unpin
        + for<'r> sqlx::Decode<'r, sqlx::Postgres>
        + sqlx::Type<sqlx::Postgres>,
    M::Row: Serialize,
{
    use crate::common::model_observer::{try_get_observer, ModelEvent, ObserverAction};

    check_deferred(&state.deferred_error)?;
    if state.assignments.is_empty() {
        anyhow::bail!("update: no columns set");
    }
    if !state.has_conditions() {
        anyhow::bail!("update: no conditions set");
    }
    if M::HAS_UPDATED_AT && !state.has_assignment("updated_at") {
        state = state.assign_col("updated_at", time::OffsetDateTime::now_utc().into());
    }

    let target_ids = patch_select_target_ids::<M>(db.clone(), &state).await?;
    if !M::OBSERVE_HOOKS || target_ids.is_empty() {
        return Ok((state, target_ids, Vec::new()));
    }

    let Some(observer) = try_get_observer() else {
        return Ok((state, target_ids, Vec::new()));
    };

    let old_rows = patch_fetch_rows_by_ids::<M>(db, &target_ids)
        .await
        .unwrap_or_default();
    if old_rows.is_empty() {
        return Ok((state, target_ids, old_rows));
    }

    let event = ModelEvent {
        model: M::MODEL_KEY,
        table: M::TABLE,
        record_key: None,
    };
    let changes = M::build_patch_changes(&state)?;
    let old_data = serde_json::to_value(&old_rows)?;
    let changes_data = serde_json::to_value(&changes)?;

    match observer.on_updating(&event, &old_data, &changes_data).await? {
        ObserverAction::Prevent(err) => Err(err),
        ObserverAction::Modify(overrides) => {
            Ok((M::apply_patch_overrides(state, overrides)?, target_ids, old_rows))
        }
        ObserverAction::Continue => Ok((state, target_ids, old_rows)),
    }
}

async fn patch_execute_update<'db, M>(db: DbConn<'db>, state: &PatchState<'db>) -> Result<u64>
where
    M: PatchModel,
{
    let (sql, binds) = state.build_update_sql();
    let profiler_binds = if M::PROFILE_QUERIES && crate::common::sql::is_sql_profiler_enabled() {
        profiler_binds_string(&binds)
    } else {
        String::new()
    };
    let profiler_start = if M::PROFILE_QUERIES {
        Some(std::time::Instant::now())
    } else {
        None
    };

    let mut q = sqlx::query(&sql);
    for bind in &binds {
        q = crate::common::sql::bind_query(q, bind.clone());
    }
    let res = db.execute(q).await?;

    if let Some(start) = profiler_start {
        crate::common::sql::record_profiled_query(
            M::TABLE,
            "UPDATE",
            &sql,
            &profiler_binds,
            start.elapsed(),
        );
    }

    Ok(res.rows_affected())
}

async fn patch_execute_update_returning_rows<'db, M>(
    db: DbConn<'db>,
    state: &PatchState<'db>,
) -> Result<Vec<M::Row>>
where
    M: PatchModel + RuntimeModel,
{
    let (sql, binds) = state.build_update_sql_returning(&ReturningSpec::All);
    let profiler_binds = if M::PROFILE_QUERIES && crate::common::sql::is_sql_profiler_enabled() {
        profiler_binds_string(&binds)
    } else {
        String::new()
    };
    let profiler_start = if M::PROFILE_QUERIES {
        Some(std::time::Instant::now())
    } else {
        None
    };

    let mut q = sqlx::query_as::<_, M::Row>(&sql);
    for bind in &binds {
        q = crate::common::sql::bind(q, bind.clone());
    }
    let rows = db.fetch_all(q).await?;

    if let Some(start) = profiler_start {
        crate::common::sql::record_profiled_query(
            M::TABLE,
            "UPDATE",
            &sql,
            &profiler_binds,
            start.elapsed(),
        );
    }

    Ok(rows)
}

async fn patch_finalize_runtime<'db, M>(
    db: DbConn<'db>,
    state: PatchState<'db>,
    target_ids: Vec<M::Pk>,
    old_rows: Vec<M::Row>,
    affected: u64,
) -> Result<()>
where
    M: PatchModel + RuntimeModel,
    M::Pk: Clone,
    M::Row: Serialize,
{
    use crate::common::model_observer::{log_observer_error, try_get_observer, ModelEvent};

    if affected == 0 {
        return Ok(());
    }

    M::persist_patch_state(db.clone(), target_ids, state).await?;

    if !M::OBSERVE_HOOKS || old_rows.is_empty() {
        return Ok(());
    }

    let Some(observer) = try_get_observer() else {
        return Ok(());
    };

    for old_row in &old_rows {
        match patch_fetch_row_by_pk::<M>(db.clone(), M::row_pk(old_row)).await {
            Ok(Some(new_row)) => match (serde_json::to_value(old_row), serde_json::to_value(&new_row)) {
                (Ok(old_data), Ok(new_data)) => {
                    let event = ModelEvent {
                        model: M::MODEL_KEY,
                        table: M::TABLE,
                        record_key: Some(M::row_pk_text(old_row)),
                    };
                    if let Err(err) = observer.on_updated(&event, &old_data, &new_data).await {
                        log_observer_error("updated", M::MODEL_KEY, &err);
                    }
                }
                (Err(err), _) | (_, Err(err)) => {
                    log_observer_error("updated", M::MODEL_KEY, &err);
                }
            },
            Ok(None) => {}
            Err(err) => log_observer_error("updated", M::MODEL_KEY, &err),
        }
    }

    Ok(())
}

pub fn patch_save_runtime<'db, M>(db: DbConn<'db>, state: PatchState<'db>) -> BoxModelFuture<'db, u64>
where
    M: PatchModel + RuntimeModel,
    M::Changes: Serialize + Default,
    M::Pk: Clone
        + Send
        + Unpin
        + for<'r> sqlx::Decode<'r, sqlx::Postgres>
        + sqlx::Type<sqlx::Postgres>,
    M::Row: Serialize,
{
    Box::pin(async move {
        match db {
            DbConn::Pool(pool) => {
                let tx = pool.begin().await?;
                let tx_lock = std::sync::Arc::new(tokio::sync::Mutex::new(tx));
                let affected = {
                    let db = DbConn::tx(tx_lock.clone());
                    let (state, target_ids, old_rows) = patch_prepare_runtime::<M>(db.clone(), state).await?;
                    let affected = patch_execute_update::<M>(db.clone(), &state).await?;
                    patch_finalize_runtime::<M>(db, state, target_ids, old_rows, affected).await?;
                    affected
                };
                let tx = std::sync::Arc::try_unwrap(tx_lock)
                    .map_err(|_| anyhow::anyhow!("transaction scope still has active handles"))?
                    .into_inner();
                tx.commit().await?;
                Ok(affected)
            }
            DbConn::Tx(_) => {
                let (state, target_ids, old_rows) = patch_prepare_runtime::<M>(db.clone(), state).await?;
                let affected = patch_execute_update::<M>(db.clone(), &state).await?;
                patch_finalize_runtime::<M>(db, state, target_ids, old_rows, affected).await?;
                Ok(affected)
            }
        }
    })
}

pub fn patch_fetch_runtime<'db, M>(
    db: DbConn<'db>,
    state: PatchState<'db>,
) -> BoxModelFuture<'db, Vec<M::Record>>
where
    M: PatchModel + RuntimeModel,
    M::Changes: Serialize + Default,
    M::Pk: Clone
        + Send
        + Unpin
        + for<'r> sqlx::Decode<'r, sqlx::Postgres>
        + sqlx::Type<sqlx::Postgres>,
    M::Row: Serialize,
    M::Record: RelationMetricRecord,
{
    Box::pin(async move {
        match db {
            DbConn::Pool(pool) => {
                let tx = pool.begin().await?;
                let tx_lock = std::sync::Arc::new(tokio::sync::Mutex::new(tx));
                let records = {
                    let db = DbConn::tx(tx_lock.clone());
                    let base_url = state.base_url.clone();
                    let (state, target_ids, old_rows) = patch_prepare_runtime::<M>(db.clone(), state).await?;
                    let affected = patch_execute_update::<M>(db.clone(), &state).await?;
                    patch_finalize_runtime::<M>(db.clone(), state, target_ids.clone(), old_rows, affected).await?;
                    if target_ids.is_empty() {
                        Vec::new()
                    } else {
                        let binds: Vec<BindValue> = target_ids.into_iter().map(Into::into).collect();
                        let mut query_state = Query::<M>::new_with_base_url(base_url).into_inner();
                        if M::HAS_SOFT_DELETE {
                            query_state = query_state.with_deleted();
                        }
                        query_state = query_state.where_in_str(M::PK_COL, &binds);
                        M::query_all(db, query_state).await?
                    }
                };
                let tx = std::sync::Arc::try_unwrap(tx_lock)
                    .map_err(|_| anyhow::anyhow!("transaction scope still has active handles"))?
                    .into_inner();
                tx.commit().await?;
                Ok(records)
            }
            DbConn::Tx(_) => {
                let base_url = state.base_url.clone();
                let (state, target_ids, old_rows) = patch_prepare_runtime::<M>(db.clone(), state).await?;
                let affected = patch_execute_update::<M>(db.clone(), &state).await?;
                patch_finalize_runtime::<M>(db.clone(), state, target_ids.clone(), old_rows, affected).await?;
                if target_ids.is_empty() {
                    Ok(Vec::new())
                } else {
                    let binds: Vec<BindValue> = target_ids.into_iter().map(Into::into).collect();
                    let mut query_state = Query::<M>::new_with_base_url(base_url).into_inner();
                    if M::HAS_SOFT_DELETE {
                        query_state = query_state.with_deleted();
                    }
                    query_state = query_state.where_in_str(M::PK_COL, &binds);
                    M::query_all(db, query_state).await
                }
            }
        }
    })
}

pub fn patch_fetch_returning_all_runtime<'db, M>(
    db: DbConn<'db>,
    state: PatchState<'db>,
) -> BoxModelFuture<'db, Vec<M::Record>>
where
    M: PatchModel + RuntimeModel,
    M::Changes: Serialize + Default,
    M::Pk: Clone
        + Send
        + Unpin
        + for<'r> sqlx::Decode<'r, sqlx::Postgres>
        + sqlx::Type<sqlx::Postgres>,
    M::Row: Serialize,
{
    Box::pin(async move {
        match db {
            DbConn::Pool(pool) => {
                let tx = pool.begin().await?;
                let tx_lock = std::sync::Arc::new(tokio::sync::Mutex::new(tx));
                let records = {
                    let db = DbConn::tx(tx_lock.clone());
                    let base_url = state.base_url.clone();
                    let (state, target_ids, old_rows) = patch_prepare_runtime::<M>(db.clone(), state).await?;
                    let rows = patch_execute_update_returning_rows::<M>(db.clone(), &state).await?;
                    patch_finalize_runtime::<M>(db.clone(), state, target_ids, old_rows, rows.len() as u64).await?;
                    M::hydrate_records(db, rows, base_url).await?
                };
                let tx = std::sync::Arc::try_unwrap(tx_lock)
                    .map_err(|_| anyhow::anyhow!("transaction scope still has active handles"))?
                    .into_inner();
                tx.commit().await?;
                Ok(records)
            }
            DbConn::Tx(_) => {
                let base_url = state.base_url.clone();
                let (state, target_ids, old_rows) = patch_prepare_runtime::<M>(db.clone(), state).await?;
                let rows = patch_execute_update_returning_rows::<M>(db.clone(), &state).await?;
                patch_finalize_runtime::<M>(db.clone(), state, target_ids, old_rows, rows.len() as u64).await?;
                M::hydrate_records(db, rows, base_url).await
            }
        }
    })
}

fn relation_soft_delete_clause(
    has_soft_delete: bool,
    with_deleted: bool,
    only_deleted: bool,
) -> &'static str {
    if only_deleted {
        if has_soft_delete {
            " AND deleted_at IS NOT NULL"
        } else {
            ""
        }
    } else if has_soft_delete && !with_deleted {
        " AND deleted_at IS NULL"
    } else {
        ""
    }
}

fn relation_exists_from_load_spec(spec: WithRelationSpec) -> Result<RelationExistenceNode> {
    if !spec.counts.is_empty() {
        anyhow::bail!(
            "relation '{}' cannot use relation counts inside where_has(); use nested where_has instead",
            spec.name
        );
    }
    if !spec.aggregates.is_empty() {
        anyhow::bail!(
            "relation '{}' cannot use relation aggregates inside where_has(); use nested where_has instead",
            spec.name
        );
    }
    if spec.limit.is_some() || spec.offset.is_some() {
        anyhow::bail!(
            "relation '{}' cannot use scoped limit/offset inside where_has()",
            spec.name
        );
    }
    if !spec.selects.is_empty() {
        anyhow::bail!(
            "relation '{}' cannot use scoped select projections inside where_has()",
            spec.name
        );
    }
    if !spec.orders.is_empty() {
        anyhow::bail!(
            "relation '{}' cannot use ordering inside where_has()",
            spec.name
        );
    }

    let children = spec
        .nested
        .into_iter()
        .map(relation_exists_from_load_spec)
        .collect::<Result<Vec<_>>>()?;

    Ok(RelationExistenceNode {
        name: spec.name,
        kind: spec.kind,
        target_table: spec.target_table,
        target_pk: spec.target_pk,
        foreign_key: spec.foreign_key,
        local_key: spec.local_key,
        has_soft_delete: spec.has_soft_delete,
        filters: spec.filters,
        with_deleted: spec.with_deleted,
        only_deleted: spec.only_deleted,
        operator: ExistenceOperator::Exists,
        count: None,
        children,
    })
}

pub fn relation_exists_from_query_state<'db>(
    mut spec: WithRelationSpec,
    inner: QueryState<'db>,
    default_select: &str,
) -> Result<RelationExistenceNode> {
    spec.filters.extend(inner.filters);
    spec.orders = inner.orders;
    spec.limit = inner.limit;
    spec.offset = inner.offset;
    spec.selects = if inner.selects == parse_select_list(default_select) {
        Vec::new()
    } else {
        inner.selects
    };
    spec.with_deleted = spec.with_deleted || inner.with_deleted;
    spec.only_deleted = spec.only_deleted || inner.only_deleted;
    spec.nested = inner.with_relations.unwrap_or_default();
    spec.counts.extend(inner.count_relations);
    spec.aggregates.extend(inner.aggregate_relations);

    relation_exists_from_load_spec(spec)
}

fn compile_relation_exists_node(
    node: &RelationExistenceNode,
    parent_table: &str,
    alias_seed: &mut usize,
    bind_start: usize,
) -> (String, Vec<BindValue>) {
    let alias = format!("__rf_rel_{}", *alias_seed);
    *alias_seed += 1;

    let mut binds = Vec::new();
    let mut bind_idx = bind_start;
    let mut clauses = Vec::new();

    let link_clause = match node.kind {
        "belongs_to" => format!(
            "{alias}.{} = {parent_table}.{}",
            node.local_key, node.foreign_key
        ),
        "has_many" | "has_one" => {
            format!(
                "{alias}.{} = {parent_table}.{}",
                node.foreign_key, node.local_key
            )
        }
        _ => format!(
            "{alias}.{} = {parent_table}.{}",
            node.foreign_key, node.local_key
        ),
    };
    clauses.push(link_clause);

    let soft_delete_clause =
        relation_soft_delete_clause(node.has_soft_delete, node.with_deleted, node.only_deleted);
    if !soft_delete_clause.is_empty() {
        clauses.push(soft_delete_clause.trim_start_matches(" AND ").to_string());
    }

    let (filter_clauses, filter_binds) = compile_filters(&node.filters, bind_idx);
    if !filter_clauses.is_empty() {
        bind_idx += filter_binds.len();
        binds.extend(filter_binds);
        clauses.extend(filter_clauses);
    }

    for child in &node.children {
        let (child_clause, child_binds) =
            compile_relation_exists_node(child, &alias, alias_seed, bind_idx);
        bind_idx += child_binds.len();
        binds.extend(child_binds);
        clauses.push(child_clause);
    }

    let select_expr = match node.operator {
        ExistenceOperator::Exists => "1".to_string(),
    };
    (
        format!(
            "EXISTS (SELECT {select_expr} FROM {} AS {alias} WHERE {})",
            node.target_table,
            clauses.join(" AND "),
        ),
        binds,
    )
}

fn compile_existence_predicates(
    root_table: &str,
    base_where: &mut Vec<String>,
    base_binds: &mut Vec<BindValue>,
    predicates: &[RootExistenceNode],
) {
    let mut alias_seed = 0usize;
    for predicate in predicates {
        let (clause, binds) = compile_relation_exists_node(
            &predicate.node,
            root_table,
            &mut alias_seed,
            base_binds.len() + 1,
        );
        match predicate.boolean {
            ExistenceBoolean::And => base_where.push(clause),
            ExistenceBoolean::Or => {
                if let Some(last) = base_where.pop() {
                    base_where.push(format!("({last} OR {clause})"));
                } else {
                    base_where.push(clause);
                }
            }
        }
        base_binds.extend(binds);
    }
}

pub async fn apply_loaded_metrics<'db, M>(
    db: DbConn<'db>,
    records: &mut [M::Record],
    count_specs: &[CountRelationSpec],
    aggregate_specs: &[RelationAggregateSpec],
) -> Result<()>
where
    M: RuntimeModel,
    M::Record: RelationMetricRecord,
{
    if records.is_empty() {
        return Ok(());
    }

    let parent_ids: Vec<BindValue> = records
        .iter()
        .filter_map(M::record_pk_i64)
        .map(BindValue::from)
        .collect();
    if parent_ids.len() != records.len() {
        return Ok(());
    }

    let mut metric_count_specs = count_specs.to_vec();
    for aggregate_spec in aggregate_specs {
        if aggregate_spec.kind != RelationAggregateKind::Avg {
            continue;
        }
        if metric_count_specs
            .iter()
            .any(|spec| spec.name == aggregate_spec.relation_name)
        {
            continue;
        }
        metric_count_specs.push(CountRelationSpec {
            name: aggregate_spec.relation_name,
            target_table: aggregate_spec.target_table,
            target_pk: aggregate_spec.target_pk,
            foreign_key: aggregate_spec.foreign_key,
            has_soft_delete: aggregate_spec.has_soft_delete,
            filters: aggregate_spec.filters.clone(),
            with_deleted: aggregate_spec.with_deleted,
            only_deleted: aggregate_spec.only_deleted,
        });
    }

    if !metric_count_specs.is_empty() {
        let counts = execute_relation_counts(&db, &parent_ids, &metric_count_specs).await?;
        for record in records.iter_mut() {
            let Some(pk) = M::record_pk_i64(record) else {
                continue;
            };
            for (key, by_fk) in &counts {
                if let Some(&count) = by_fk.get(&pk) {
                    record.relation_counts_mut().insert(key.clone(), count);
                }
            }
        }
    }

    if !aggregate_specs.is_empty() {
        let aggregates = execute_relation_aggregates(&db, &parent_ids, aggregate_specs).await?;
        for record in records.iter_mut() {
            let Some(pk) = M::record_pk_i64(record) else {
                continue;
            };
            for (key, by_fk) in &aggregates {
                if let Some(&value) = by_fk.get(&pk) {
                    record.relation_aggregates_mut().insert(key.clone(), value);
                }
            }
        }
    }

    Ok(())
}

pub fn merge_nested_metrics_from_one<P, C>(parent: &mut P, relation_name: &str, child: &C)
where
    P: RelationMetricRecord,
    C: RelationMetricRecord,
{
    for (key, count) in child.relation_counts() {
        let prefixed = prefix_nested_count_key(relation_name, key);
        parent.relation_counts_mut().insert(prefixed, *count);
    }
    for (key, value) in child.relation_aggregates() {
        let prefixed = prefix_nested_aggregate_key(relation_name, key);
        parent.relation_aggregates_mut().insert(prefixed, *value);
    }
}

pub fn merge_nested_metrics_from_many<P, C>(parent: &mut P, relation_name: &str, children: &[C])
where
    P: RelationMetricRecord,
    C: RelationMetricRecord,
{
    let mut avg_totals: HashMap<String, (f64, i64)> = HashMap::new();
    for child in children {
        for (key, count) in child.relation_counts() {
            let prefixed = prefix_nested_count_key(relation_name, key);
            let entry = parent.relation_counts_mut().entry(prefixed).or_insert(0);
            *entry += *count;
        }
        for (key, value) in child.relation_aggregates() {
            let prefixed = prefix_nested_aggregate_key(relation_name, key);
            let mut parts = key.splitn(3, ':');
            match parts.next().unwrap_or_default() {
                "sum" => {
                    let entry = parent
                        .relation_aggregates_mut()
                        .entry(prefixed)
                        .or_insert(0.0);
                    *entry += *value;
                }
                "min" => {
                    let entry = parent
                        .relation_aggregates_mut()
                        .entry(prefixed)
                        .or_insert(*value);
                    if *value < *entry {
                        *entry = *value;
                    }
                }
                "max" => {
                    let entry = parent
                        .relation_aggregates_mut()
                        .entry(prefixed)
                        .or_insert(*value);
                    if *value > *entry {
                        *entry = *value;
                    }
                }
                "avg" => {
                    let path = parts.next().unwrap_or_default();
                    let count = child.relation_counts().get(path).copied().unwrap_or(1);
                    let entry = avg_totals.entry(prefixed).or_insert((0.0, 0));
                    entry.0 += *value * count as f64;
                    entry.1 += count;
                }
                _ => {}
            }
        }
    }
    for (key, (sum, count)) in avg_totals {
        if count > 0 {
            parent
                .relation_aggregates_mut()
                .insert(key, sum / count as f64);
        }
    }
}

pub async fn apply_loaded_relations<'db, M>(
    db: DbConn<'db>,
    records: &mut Vec<M::Record>,
    base_url: Option<String>,
    with_relations: &[WithRelationSpec],
) -> Result<()>
where
    M: RuntimeModel,
{
    if records.is_empty() || with_relations.is_empty() {
        return Ok(());
    }

    for spec in with_relations {
        if let Some(relation) = M::relation_runtimes()
            .iter()
            .find(|candidate| candidate.name() == spec.name)
        {
            let current = std::mem::take(records);
            *records = relation
                .apply(db.clone(), current, base_url.clone(), spec.clone())
                .await?;
        }
    }

    Ok(())
}

pub struct HasManyRuntime<P: RuntimeModel, T: RuntimeModel, K> {
    pub name: &'static str,
    pub target_table: &'static str,
    pub target_pk: &'static str,
    pub foreign_key: &'static str,
    pub parent_key: fn(&P::Record) -> Option<K>,
    pub child_foreign_key: fn(&T::Record) -> Option<K>,
    pub assign: fn(&mut P::Record, Vec<T::Record>),
}

pub struct HasOneRuntime<P: RuntimeModel, T: RuntimeModel, K> {
    pub name: &'static str,
    pub target_table: &'static str,
    pub target_pk: &'static str,
    pub foreign_key: &'static str,
    pub parent_key: fn(&P::Record) -> Option<K>,
    pub child_foreign_key: fn(&T::Record) -> Option<K>,
    pub assign: fn(&mut P::Record, Option<T::Record>),
}

pub async fn apply_has_many_relation<'db, P, T, K>(
    db: DbConn<'db>,
    mut parents: Vec<P::Record>,
    base_url: Option<String>,
    spec: WithRelationSpec,
    runtime: HasManyRuntime<P, T, K>,
) -> Result<Vec<P::Record>>
where
    P: RuntimeModel,
    T: RuntimeModel,
    P::Record: RelationMetricRecord,
    T::Record: RelationMetricRecord,
    K: Clone + Eq + Hash + Into<BindValue> + Send + Sync + 'static,
{
    if parents.is_empty() {
        return Ok(parents);
    }

    let mut unique_keys: Vec<K> = Vec::new();
    let mut seen = HashSet::new();
    for parent in parents.iter() {
        let Some(key) = (runtime.parent_key)(parent) else {
            continue;
        };
        if seen.insert(key.clone()) {
            unique_keys.push(key);
        }
    }

    let mut loaded_by_parent: HashMap<K, Vec<T::Record>> = HashMap::new();
    if !unique_keys.is_empty() {
        let mut bind_idx: usize = 1;
        let placeholders: Vec<String> = unique_keys
            .iter()
            .map(|_| {
                let p = format!("${}", bind_idx);
                bind_idx += 1;
                p
            })
            .collect();
        let soft_delete_clause =
            relation_soft_delete_clause(T::HAS_SOFT_DELETE, spec.with_deleted, spec.only_deleted);
        let (filter_clauses, filter_binds) = compile_filters(&spec.filters, bind_idx);
        let extra_clause = if filter_clauses.is_empty() {
            String::new()
        } else {
            format!(" AND {}", filter_clauses.join(" AND "))
        };
        let relation_select = if spec.selects.is_empty() {
            T::DEFAULT_SELECT.to_string()
        } else {
            render_selects(&spec.selects)
        };
        let scoped_order = if spec.orders.is_empty() {
            format!("{} ASC", runtime.target_pk)
        } else {
            spec.orders
                .iter()
                .map(render_order_expr)
                .collect::<Vec<_>>()
                .join(", ")
        };
        let sql = if spec.limit.is_some() || spec.offset.is_some() {
            let offset = spec.offset.unwrap_or(0);
            let limit = spec.limit.unwrap_or(i64::MAX);
            format!(
                "SELECT * FROM (SELECT {select}, ROW_NUMBER() OVER (PARTITION BY {foreign_key} ORDER BY {order_by}) AS __rf_row_num FROM {table} WHERE {foreign_key} IN ({placeholders}){soft_delete}{extra}) AS __rf_rel WHERE __rf_row_num > {offset} AND __rf_row_num <= {offset_plus_limit} ORDER BY {foreign_key}, __rf_row_num",
                select = relation_select,
                foreign_key = runtime.foreign_key,
                order_by = scoped_order,
                table = runtime.target_table,
                placeholders = placeholders.join(", "),
                soft_delete = soft_delete_clause,
                extra = extra_clause,
                offset = offset,
                offset_plus_limit = offset.saturating_add(limit),
            )
        } else {
            format!(
                "SELECT {select} FROM {table} WHERE {foreign_key} IN ({placeholders}){soft_delete}{extra} ORDER BY {foreign_key}, {order_by}",
                select = relation_select,
                table = runtime.target_table,
                foreign_key = runtime.foreign_key,
                placeholders = placeholders.join(", "),
                soft_delete = soft_delete_clause,
                extra = extra_clause,
                order_by = scoped_order,
            )
        };

        let mut query = sqlx::query_as::<_, T::Row>(&sql);
        for key in &unique_keys {
            query = crate::common::sql::bind(query, key.clone().into());
        }
        for bind in &filter_binds {
            query = crate::common::sql::bind(query, bind.clone());
        }

        let rows = db.fetch_all(query).await?;
        let mut records = T::hydrate_records(db.clone(), rows, base_url.clone()).await?;
        if !spec.nested.is_empty() {
            apply_loaded_relations::<T>(db.clone(), &mut records, base_url.clone(), &spec.nested)
                .await?;
        }
        apply_loaded_metrics::<T>(
            db.clone(),
            records.as_mut_slice(),
            &spec.counts,
            &spec.aggregates,
        )
        .await?;

        for record in records {
            if let Some(key) = (runtime.child_foreign_key)(&record) {
                loaded_by_parent.entry(key).or_default().push(record);
            }
        }
    }

    for parent in parents.iter_mut() {
        let children = (runtime.parent_key)(parent)
            .and_then(|key| loaded_by_parent.get(&key).cloned())
            .unwrap_or_default();
        merge_nested_metrics_from_many(parent, spec.name, &children);
        (runtime.assign)(parent, children);
    }

    Ok(parents)
}

impl<P, T, K> ErasedRelationRuntime<P> for HasManyRuntime<P, T, K>
where
    P: RuntimeModel,
    T: RuntimeModel,
    P::Record: RelationMetricRecord,
    T::Record: RelationMetricRecord,
    K: Clone + Eq + Hash + Into<BindValue> + Send + Sync + 'static,
{
    fn name(&self) -> &'static str {
        self.name
    }

    fn apply<'db>(
        &self,
        db: DbConn<'db>,
        parents: Vec<P::Record>,
        base_url: Option<String>,
        spec: WithRelationSpec,
    ) -> BoxModelFuture<'db, Vec<P::Record>> {
        Box::pin(apply_has_many_relation(
            db,
            parents,
            base_url,
            spec,
            HasManyRuntime::<P, T, K> {
                name: self.name,
                target_table: self.target_table,
                target_pk: self.target_pk,
                foreign_key: self.foreign_key,
                parent_key: self.parent_key,
                child_foreign_key: self.child_foreign_key,
                assign: self.assign,
            },
        ))
    }
}

pub async fn apply_has_one_relation<'db, P, T, K>(
    db: DbConn<'db>,
    mut parents: Vec<P::Record>,
    base_url: Option<String>,
    spec: WithRelationSpec,
    runtime: HasOneRuntime<P, T, K>,
) -> Result<Vec<P::Record>>
where
    P: RuntimeModel,
    T: RuntimeModel,
    P::Record: RelationMetricRecord,
    T::Record: RelationMetricRecord,
    K: Clone + Eq + Hash + Into<BindValue> + Send + Sync + 'static,
{
    if parents.is_empty() {
        return Ok(parents);
    }

    let mut unique_keys: Vec<K> = Vec::new();
    let mut seen = HashSet::new();
    for parent in parents.iter() {
        let Some(key) = (runtime.parent_key)(parent) else {
            continue;
        };
        if seen.insert(key.clone()) {
            unique_keys.push(key);
        }
    }

    let mut loaded_by_parent: HashMap<K, T::Record> = HashMap::new();
    if !unique_keys.is_empty() {
        let mut bind_idx: usize = 1;
        let placeholders: Vec<String> = unique_keys
            .iter()
            .map(|_| {
                let p = format!("${}", bind_idx);
                bind_idx += 1;
                p
            })
            .collect();
        let soft_delete_clause =
            relation_soft_delete_clause(T::HAS_SOFT_DELETE, spec.with_deleted, spec.only_deleted);
        let (filter_clauses, filter_binds) = compile_filters(&spec.filters, bind_idx);
        let extra_clause = if filter_clauses.is_empty() {
            String::new()
        } else {
            format!(" AND {}", filter_clauses.join(" AND "))
        };
        let relation_select = if spec.selects.is_empty() {
            T::DEFAULT_SELECT.to_string()
        } else {
            render_selects(&spec.selects)
        };
        let scoped_order = if spec.orders.is_empty() {
            format!("{} ASC", runtime.target_pk)
        } else {
            spec.orders
                .iter()
                .map(render_order_expr)
                .collect::<Vec<_>>()
                .join(", ")
        };
        let offset = spec.offset.unwrap_or(0);
        let limit = spec.limit.unwrap_or(1);
        let sql = format!(
            "SELECT * FROM (SELECT {select}, ROW_NUMBER() OVER (PARTITION BY {foreign_key} ORDER BY {order_by}) AS __rf_row_num FROM {table} WHERE {foreign_key} IN ({placeholders}){soft_delete}{extra}) AS __rf_rel WHERE __rf_row_num > {offset} AND __rf_row_num <= {offset_plus_limit} ORDER BY {foreign_key}, __rf_row_num",
            select = relation_select,
            foreign_key = runtime.foreign_key,
            order_by = scoped_order,
            table = runtime.target_table,
            placeholders = placeholders.join(", "),
            soft_delete = soft_delete_clause,
            extra = extra_clause,
            offset = offset,
            offset_plus_limit = offset.saturating_add(limit),
        );

        let mut query = sqlx::query_as::<_, T::Row>(&sql);
        for key in &unique_keys {
            query = crate::common::sql::bind(query, key.clone().into());
        }
        for bind in &filter_binds {
            query = crate::common::sql::bind(query, bind.clone());
        }

        let rows = db.fetch_all(query).await?;
        let mut records = T::hydrate_records(db.clone(), rows, base_url.clone()).await?;
        if !spec.nested.is_empty() {
            apply_loaded_relations::<T>(db.clone(), &mut records, base_url.clone(), &spec.nested)
                .await?;
        }
        apply_loaded_metrics::<T>(
            db.clone(),
            records.as_mut_slice(),
            &spec.counts,
            &spec.aggregates,
        )
        .await?;

        for record in records {
            if let Some(key) = (runtime.child_foreign_key)(&record) {
                loaded_by_parent.entry(key).or_insert(record);
            }
        }
    }

    for parent in parents.iter_mut() {
        let child =
            (runtime.parent_key)(parent).and_then(|key| loaded_by_parent.get(&key).cloned());
        if let Some(child_ref) = child.as_ref() {
            merge_nested_metrics_from_one(parent, spec.name, child_ref);
        }
        (runtime.assign)(parent, child);
    }

    Ok(parents)
}

impl<P, T, K> ErasedRelationRuntime<P> for HasOneRuntime<P, T, K>
where
    P: RuntimeModel,
    T: RuntimeModel,
    P::Record: RelationMetricRecord,
    T::Record: RelationMetricRecord,
    K: Clone + Eq + Hash + Into<BindValue> + Send + Sync + 'static,
{
    fn name(&self) -> &'static str {
        self.name
    }

    fn apply<'db>(
        &self,
        db: DbConn<'db>,
        parents: Vec<P::Record>,
        base_url: Option<String>,
        spec: WithRelationSpec,
    ) -> BoxModelFuture<'db, Vec<P::Record>> {
        Box::pin(apply_has_one_relation(
            db,
            parents,
            base_url,
            spec,
            HasOneRuntime::<P, T, K> {
                name: self.name,
                target_table: self.target_table,
                target_pk: self.target_pk,
                foreign_key: self.foreign_key,
                parent_key: self.parent_key,
                child_foreign_key: self.child_foreign_key,
                assign: self.assign,
            },
        ))
    }
}

pub struct BelongsToRuntime<P: RuntimeModel, T: RuntimeModel, K> {
    pub name: &'static str,
    pub target_table: &'static str,
    pub target_key_sql: &'static str,
    pub parent_foreign_key: fn(&P::Record) -> Option<K>,
    pub target_key: fn(&T::Record) -> K,
    pub assign: fn(&mut P::Record, Option<T::Record>),
}

pub async fn apply_belongs_to_relation<'db, P, T, K>(
    db: DbConn<'db>,
    mut parents: Vec<P::Record>,
    base_url: Option<String>,
    spec: WithRelationSpec,
    runtime: BelongsToRuntime<P, T, K>,
) -> Result<Vec<P::Record>>
where
    P: RuntimeModel,
    T: RuntimeModel,
    P::Record: RelationMetricRecord,
    T::Record: RelationMetricRecord,
    K: Clone + Eq + Hash + Into<BindValue> + Send + Sync + 'static,
{
    if parents.is_empty() {
        return Ok(parents);
    }

    if spec.limit.is_some() || spec.offset.is_some() {
        anyhow::bail!(
            "relation '{}' does not support scoped limit/offset for singular eager loads",
            spec.name
        );
    }

    let mut unique_keys: Vec<K> = Vec::new();
    let mut seen = HashSet::new();
    for parent in parents.iter() {
        let Some(key) = (runtime.parent_foreign_key)(parent) else {
            continue;
        };
        if seen.insert(key.clone()) {
            unique_keys.push(key);
        }
    }

    let mut by_target_key: HashMap<K, T::Record> = HashMap::new();
    if !unique_keys.is_empty() {
        let mut bind_idx: usize = 1;
        let placeholders: Vec<String> = unique_keys
            .iter()
            .map(|_| {
                let p = format!("${}", bind_idx);
                bind_idx += 1;
                p
            })
            .collect();
        let soft_delete_clause =
            relation_soft_delete_clause(T::HAS_SOFT_DELETE, spec.with_deleted, spec.only_deleted);
        let (filter_clauses, filter_binds) = compile_filters(&spec.filters, bind_idx);
        let extra_clause = if filter_clauses.is_empty() {
            String::new()
        } else {
            format!(" AND {}", filter_clauses.join(" AND "))
        };
        let relation_select = if spec.selects.is_empty() {
            T::DEFAULT_SELECT.to_string()
        } else {
            render_selects(&spec.selects)
        };
        let order_clause = if spec.orders.is_empty() {
            String::new()
        } else {
            format!(
                " ORDER BY {}",
                spec.orders
                    .iter()
                    .map(render_order_expr)
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };
        let sql = format!(
            "SELECT {select} FROM {table} WHERE {target_key} IN ({placeholders}){soft_delete}{extra}{order_clause}",
            select = relation_select,
            table = runtime.target_table,
            target_key = runtime.target_key_sql,
            placeholders = placeholders.join(", "),
            soft_delete = soft_delete_clause,
            extra = extra_clause,
            order_clause = order_clause,
        );

        let mut query = sqlx::query_as::<_, T::Row>(&sql);
        for key in &unique_keys {
            query = crate::common::sql::bind(query, key.clone().into());
        }
        for bind in &filter_binds {
            query = crate::common::sql::bind(query, bind.clone());
        }

        let rows = db.fetch_all(query).await?;
        let mut records = T::hydrate_records(db.clone(), rows, base_url.clone()).await?;
        if !spec.nested.is_empty() {
            apply_loaded_relations::<T>(db.clone(), &mut records, base_url.clone(), &spec.nested)
                .await?;
        }
        apply_loaded_metrics::<T>(
            db.clone(),
            records.as_mut_slice(),
            &spec.counts,
            &spec.aggregates,
        )
        .await?;

        for record in records {
            by_target_key.insert((runtime.target_key)(&record), record);
        }
    }

    for parent in parents.iter_mut() {
        let child =
            (runtime.parent_foreign_key)(parent).and_then(|key| by_target_key.get(&key).cloned());
        if let Some(child_ref) = child.as_ref() {
            merge_nested_metrics_from_one(parent, spec.name, child_ref);
        }
        (runtime.assign)(parent, child);
    }

    Ok(parents)
}

impl<P, T, K> ErasedRelationRuntime<P> for BelongsToRuntime<P, T, K>
where
    P: RuntimeModel,
    T: RuntimeModel,
    P::Record: RelationMetricRecord,
    T::Record: RelationMetricRecord,
    K: Clone + Eq + Hash + Into<BindValue> + Send + Sync + 'static,
{
    fn name(&self) -> &'static str {
        self.name
    }

    fn apply<'db>(
        &self,
        db: DbConn<'db>,
        parents: Vec<P::Record>,
        base_url: Option<String>,
        spec: WithRelationSpec,
    ) -> BoxModelFuture<'db, Vec<P::Record>> {
        Box::pin(apply_belongs_to_relation(
            db,
            parents,
            base_url,
            spec,
            BelongsToRuntime::<P, T, K> {
                name: self.name,
                target_table: self.target_table,
                target_key_sql: self.target_key_sql,
                parent_foreign_key: self.parent_foreign_key,
                target_key: self.target_key,
                assign: self.assign,
            },
        ))
    }
}

/// Check if a relation should be loaded based on the `with_relations` list.
/// `None` = no relations loaded. `Some(list)` = only listed relations loaded.
pub fn should_load_relation(name: &str, with_relations: &Option<Vec<WithRelationSpec>>) -> bool {
    match with_relations {
        None => false,
        Some(list) => list.iter().any(|s| s.name == name),
    }
}

/// Get the nested with specs for a relation, if any.
pub fn get_relation_nested<'a>(
    name: &str,
    with_relations: &'a Option<Vec<WithRelationSpec>>,
) -> &'a [WithRelationSpec] {
    with_relations
        .as_ref()
        .and_then(|list| list.iter().find(|s| s.name == name))
        .map(|s| s.nested.as_slice())
        .unwrap_or(&[])
}

pub fn get_relation_counts<'a>(
    name: &str,
    with_relations: &'a Option<Vec<WithRelationSpec>>,
) -> &'a [CountRelationSpec] {
    with_relations
        .as_ref()
        .and_then(|list| list.iter().find(|s| s.name == name))
        .map(|s| s.counts.as_slice())
        .unwrap_or(&[])
}

pub fn get_relation_aggregates<'a>(
    name: &str,
    with_relations: &'a Option<Vec<WithRelationSpec>>,
) -> &'a [RelationAggregateSpec] {
    with_relations
        .as_ref()
        .and_then(|list| list.iter().find(|s| s.name == name))
        .map(|s| s.aggregates.as_slice())
        .unwrap_or(&[])
}

/// Get the extra WHERE clause for a relation, if any.
/// Trait for types that represent a SQL column expression.
/// Implemented by `Column<M, T>` (typed) and per-model `DbCol` enums (untyped).
pub trait ColExpr: Copy {
    fn col_sql(self) -> &'static str;
}

impl<M, T> ColExpr for Column<M, T> {
    fn col_sql(self) -> &'static str {
        self.as_sql()
    }
}

pub trait CreateModel: QueryModel + FeaturePersistenceModel {
    const USE_SNOWFLAKE_ID: bool = false;

    fn create_save<'db>(
        db: DbConn<'db>,
        state: CreateState<'db>,
    ) -> BoxModelFuture<'db, Self::Record>
    where
        Self: RuntimeModel,
        Self::Create: Serialize,
        <Self as RuntimeModel>::Row: Serialize,
    {
        create_save_runtime::<Self>(db, state)
    }

    fn build_create_input(_state: &CreateState<'_>) -> Result<Self::Create> {
        anyhow::bail!("build_create_input() is not implemented for {}", Self::TABLE)
    }
    fn apply_create_overrides(
        _state: CreateState<'_>,
        _overrides: serde_json::Value,
    ) -> Result<CreateState<'_>> {
        anyhow::bail!(
            "apply_create_overrides() is not implemented for {}",
            Self::TABLE
        )
    }
    fn created_row_key(row: &<Self as RuntimeModel>::Row) -> String
    where
        Self: RuntimeModel,
    {
        let _ = row;
        String::new()
    }
    fn persist_create_state<'db>(
        db: DbConn<'db>,
        row: <Self as RuntimeModel>::Row,
        state: CreateState<'db>,
    ) -> BoxModelFuture<'db, ()>
    where
        Self: RuntimeModel,
    {
        Box::pin(async move {
            if let Some(owner_id) = Self::create_owner_id(&row) {
                let empty_delete_multi: HashMap<&'static str, Vec<Uuid>> = HashMap::new();
                persist_feature_state_for_owner::<Self>(
                    db.clone(),
                    owner_id,
                    &state.translations,
                    &state.meta,
                    &state.attachments_single,
                    &state.attachments_multi,
                    &[],
                    &empty_delete_multi,
                )
                .await?;
            }
            Self::persist_create_related(db, row).await
        })
    }

    fn transform_create_value(col: &str, value: BindValue) -> Result<BindValue>;
}

pub trait CreateField<M: CreateModel>: Copy {
    type Value;

    fn set<'db>(
        field: Self,
        state: CreateState<'db>,
        value: Self::Value,
    ) -> Result<CreateState<'db>>;
}

pub trait CreateConflictField<M: CreateModel>: Copy {
    fn on_conflict_do_nothing<'db>(state: CreateState<'db>, fields: &[Self]) -> CreateState<'db>;

    fn on_conflict_update<'db>(state: CreateState<'db>, fields: &[Self]) -> CreateState<'db>;
}

pub trait PatchModel: QueryModel + FeaturePersistenceModel {
    fn patch_from_query<'db>(state: QueryState<'db>) -> PatchState<'db> {
        PatchState::from_selected_query(
            state,
            Self::TABLE,
            Self::HAS_SOFT_DELETE,
            Self::SOFT_DELETE_COL,
            Self::PK_COL,
            false,
        )
    }

    fn build_patch_changes(state: &PatchState<'_>) -> Result<Self::Changes>
    where
        Self::Changes: Default,
    {
        let _ = state;
        Ok(Default::default())
    }

    fn apply_patch_overrides<'db>(
        state: PatchState<'db>,
        overrides: serde_json::Value,
    ) -> Result<PatchState<'db>> {
        let _ = overrides;
        Ok(state)
    }

    fn row_pk(_row: &<Self as RuntimeModel>::Row) -> Self::Pk
    where
        Self: RuntimeModel,
    {
        unreachable!("row_pk() must be implemented for patch observers")
    }

    fn row_pk_text(_row: &<Self as RuntimeModel>::Row) -> String
    where
        Self: RuntimeModel,
    {
        unreachable!("row_pk_text() must be implemented for patch observers")
    }

    fn persist_patch_state<'db>(
        db: DbConn<'db>,
        target_ids: Vec<Self::Pk>,
        state: PatchState<'db>,
    ) -> BoxModelFuture<'db, ()> {
        Box::pin(async move {
            for pk in &target_ids {
                if let Some(owner_id) = Self::patch_owner_id(pk) {
                    persist_feature_state_for_owner::<Self>(
                        db.clone(),
                        owner_id,
                        &state.translations,
                        &state.meta,
                        &state.attachments_single,
                        &state.attachments_multi,
                        &state.attachments_clear_single,
                        &state.attachments_delete_multi,
                    )
                    .await?;
                }
            }
            Self::persist_patch_related(db, target_ids).await
        })
    }

    fn patch_save<'db>(db: DbConn<'db>, state: PatchState<'db>) -> BoxModelFuture<'db, u64>
    where
        Self: RuntimeModel,
        Self::Changes: Serialize + Default,
        Self::Pk: Clone
            + Send
            + Unpin
            + for<'r> sqlx::Decode<'r, sqlx::Postgres>
            + sqlx::Type<sqlx::Postgres>,
        Self::Row: Serialize,
    {
        patch_save_runtime::<Self>(db, state)
    }

    fn patch_fetch<'db>(
        db: DbConn<'db>,
        state: PatchState<'db>,
    ) -> BoxModelFuture<'db, Vec<Self::Record>>
    where
        Self: RuntimeModel,
        Self::Changes: Serialize + Default,
        Self::Pk: Clone
            + Send
            + Unpin
            + for<'r> sqlx::Decode<'r, sqlx::Postgres>
            + sqlx::Type<sqlx::Postgres>,
        Self::Row: Serialize,
        Self::Record: RelationMetricRecord,
    {
        patch_fetch_runtime::<Self>(db, state)
    }

    fn patch_fetch_returning_all<'db>(
        db: DbConn<'db>,
        state: PatchState<'db>,
    ) -> BoxModelFuture<'db, Vec<Self::Record>>
    where
        Self: RuntimeModel,
        Self::Changes: Serialize + Default,
        Self::Pk: Clone
            + Send
            + Unpin
            + for<'r> sqlx::Decode<'r, sqlx::Postgres>
            + sqlx::Type<sqlx::Postgres>,
        Self::Row: Serialize,
    {
        patch_fetch_returning_all_runtime::<Self>(db, state)
    }

    fn transform_patch_value(col: &str, value: BindValue) -> Result<BindValue>;
}

pub trait PatchAssignField<M: PatchModel>: Copy {
    type Value;

    fn assign<'db>(
        field: Self,
        state: PatchState<'db>,
        value: Self::Value,
    ) -> Result<PatchState<'db>>;
}

pub trait PatchNumericField<M: PatchModel>: PatchAssignField<M> {
    fn increment<'db>(
        field: Self,
        state: PatchState<'db>,
        value: Self::Value,
    ) -> Result<PatchState<'db>>;
    fn decrement<'db>(
        field: Self,
        state: PatchState<'db>,
        value: Self::Value,
    ) -> Result<PatchState<'db>>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page<T> {
    pub data: Vec<T>,
    pub total: i64,
    pub per_page: i64,
    pub current_page: i64,
    pub last_page: i64,
}

#[derive(Debug)]
pub struct Column<M, T> {
    sql: &'static str,
    _marker: PhantomData<fn() -> (M, T)>,
}

impl<M, T> Copy for Column<M, T> {}

impl<M, T> Clone for Column<M, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<M, T> Column<M, T> {
    pub const fn new(sql: &'static str) -> Self {
        Self {
            sql,
            _marker: PhantomData,
        }
    }

    pub const fn as_sql(self) -> &'static str {
        self.sql
    }
}

#[derive(Debug)]
pub struct OneRelation<M, T, const KEY: usize> {
    name: &'static str,
    _marker: PhantomData<fn() -> (M, T)>,
}

impl<M, T, const KEY: usize> Copy for OneRelation<M, T, KEY> {}

impl<M, T, const KEY: usize> Clone for OneRelation<M, T, KEY> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<M, T, const KEY: usize> OneRelation<M, T, KEY> {
    pub const fn new(name: &'static str) -> Self {
        Self {
            name,
            _marker: PhantomData,
        }
    }

    pub const fn name(self) -> &'static str {
        self.name
    }
}

#[derive(Debug)]
pub struct ManyRelation<M, T, const KEY: usize> {
    name: &'static str,
    target_table: &'static str,
    target_pk: &'static str,
    foreign_key: &'static str,
    soft_delete: bool,
    _marker: PhantomData<fn() -> (M, T)>,
}

impl<M, T, const KEY: usize> Copy for ManyRelation<M, T, KEY> {}

impl<M, T, const KEY: usize> Clone for ManyRelation<M, T, KEY> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<M, T, const KEY: usize> ManyRelation<M, T, KEY> {
    pub const fn new(
        name: &'static str,
        target_table: &'static str,
        target_pk: &'static str,
        foreign_key: &'static str,
    ) -> Self {
        Self {
            name,
            target_table,
            target_pk,
            foreign_key,
            soft_delete: false,
            _marker: PhantomData,
        }
    }

    pub const fn new_with_soft_delete(
        name: &'static str,
        target_table: &'static str,
        target_pk: &'static str,
        foreign_key: &'static str,
    ) -> Self {
        Self {
            name,
            target_table,
            target_pk,
            foreign_key,
            soft_delete: true,
            _marker: PhantomData,
        }
    }

    pub const fn name(self) -> &'static str {
        self.name
    }

    pub const fn target_table(self) -> &'static str {
        self.target_table
    }

    pub const fn target_pk(self) -> &'static str {
        self.target_pk
    }

    pub const fn foreign_key(self) -> &'static str {
        self.foreign_key
    }

    pub const fn has_soft_delete(self) -> bool {
        self.soft_delete
    }
}

// ---------------------------------------------------------------------------
// Query wrapper
// ---------------------------------------------------------------------------

pub struct Query<'db, M: QueryModel> {
    state: QueryState<'db>,
    _marker: PhantomData<M>,
}

impl<'db, M: QueryModel> Clone for Query<'db, M> {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            _marker: PhantomData,
        }
    }
}

impl<'db, M: QueryModel> Query<'db, M> {
    pub fn new() -> Self {
        Self {
            state: QueryState::new(resolve_attachment_base_url(None), M::DEFAULT_SELECT),
            _marker: PhantomData,
        }
    }

    pub fn new_with_base_url(base_url: Option<String>) -> Self {
        Self {
            state: QueryState::new(resolve_attachment_base_url(base_url), M::DEFAULT_SELECT),
            _marker: PhantomData,
        }
    }

    pub fn from_inner(state: QueryState<'db>) -> Self {
        Self {
            state,
            _marker: PhantomData,
        }
    }

    pub fn into_inner(self) -> QueryState<'db> {
        self.state
    }

    pub fn where_col<F, V>(self, field: F, op: Op, value: V) -> Self
    where
        F: QueryField<M>,
        V: Into<F::Value>,
    {
        Self {
            state: F::where_col(field, self.state, op, value.into()),
            _marker: PhantomData,
        }
    }

    pub fn or_where_col<F, V>(self, field: F, op: Op, value: V) -> Self
    where
        F: QueryField<M>,
        V: Into<F::Value>,
    {
        Self {
            state: F::or_where_col(field, self.state, op, value.into()),
            _marker: PhantomData,
        }
    }

    pub fn where_in<F, I, V>(self, field: F, values: I) -> Self
    where
        F: QueryField<M>,
        I: IntoIterator<Item = V>,
        V: Into<F::Value>,
    {
        let values: Vec<F::Value> = values.into_iter().map(Into::into).collect();
        Self {
            state: F::where_in(field, self.state, &values),
            _marker: PhantomData,
        }
    }

    pub fn where_null<F>(self, field: F) -> Self
    where
        F: QueryField<M>,
    {
        Self {
            state: F::where_null(field, self.state),
            _marker: PhantomData,
        }
    }

    pub fn where_not_null<F>(self, field: F) -> Self
    where
        F: QueryField<M>,
    {
        Self {
            state: F::where_not_null(field, self.state),
            _marker: PhantomData,
        }
    }

    pub fn where_col_cmp<L, R>(self, left: L, op: Op, right: R) -> Self
    where
        L: QueryField<M> + ColExpr,
        R: QueryField<M> + ColExpr,
    {
        Self {
            state: self
                .state
                .where_col_cmp_str(left.col_sql(), op, right.col_sql()),
            _marker: PhantomData,
        }
    }

    pub fn or_where_col_cmp<L, R>(self, left: L, op: Op, right: R) -> Self
    where
        L: QueryField<M> + ColExpr,
        R: QueryField<M> + ColExpr,
    {
        Self {
            state: self
                .state
                .or_where_col_cmp_str(left.col_sql(), op, right.col_sql()),
            _marker: PhantomData,
        }
    }

    pub fn where_expr_cmp<F>(self, field: F, op: Op, expr: Expr) -> Self
    where
        F: QueryField<M> + ColExpr,
    {
        Self {
            state: self.state.where_expr_cmp_str(field.col_sql(), op, expr),
            _marker: PhantomData,
        }
    }

    pub fn or_where_expr_cmp<F>(self, field: F, op: Op, expr: Expr) -> Self
    where
        F: QueryField<M> + ColExpr,
    {
        Self {
            state: self.state.or_where_expr_cmp_str(field.col_sql(), op, expr),
            _marker: PhantomData,
        }
    }

    pub fn limit(self, limit: i64) -> Self {
        Self {
            state: self.state.limit(limit),
            _marker: PhantomData,
        }
    }

    pub fn offset(self, offset: i64) -> Self {
        Self {
            state: self.state.offset(offset),
            _marker: PhantomData,
        }
    }

    pub fn for_update(self) -> Self {
        Self {
            state: self.state.for_update(),
            _marker: PhantomData,
        }
    }

    pub fn for_update_skip_locked(self) -> Self {
        Self {
            state: self.state.for_update_skip_locked(),
            _marker: PhantomData,
        }
    }

    pub fn skip_locked(self) -> Self {
        Self {
            state: self.state.skip_locked(),
            _marker: PhantomData,
        }
    }

    pub fn no_wait(self) -> Self {
        Self {
            state: self.state.no_wait(),
            _marker: PhantomData,
        }
    }

    pub fn for_no_key_update(self) -> Self {
        Self {
            state: self.state.for_no_key_update(),
            _marker: PhantomData,
        }
    }

    pub fn for_key_share(self) -> Self {
        Self {
            state: self.state.for_key_share(),
            _marker: PhantomData,
        }
    }

    pub fn where_group<F>(self, scope: F) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        let state = self
            .state
            .where_group(|group_state| scope(Self::from_inner(group_state)).state);
        Self {
            state,
            _marker: PhantomData,
        }
    }

    pub fn or_where_group<F>(self, scope: F) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        let state = self
            .state
            .or_where_group(|group_state| scope(Self::from_inner(group_state)).state);
        Self {
            state,
            _marker: PhantomData,
        }
    }

    pub fn where_raw(self, clause: crate::common::sql::RawClause) -> Self {
        let (sql, binds) = clause.into_parts();
        Self {
            state: self.state.where_raw(sql, binds),
            _marker: PhantomData,
        }
    }

    pub fn or_where_raw(self, clause: crate::common::sql::RawClause) -> Self {
        let (sql, binds) = clause.into_parts();
        Self {
            state: self.state.or_where_raw(sql, binds),
            _marker: PhantomData,
        }
    }

    pub fn order_by<F>(self, field: F, dir: OrderDir) -> Self
    where
        F: QueryField<M>,
    {
        Self {
            state: F::order_by(field, self.state, dir),
            _marker: PhantomData,
        }
    }

    pub fn order_by_raw(self, expr: crate::common::sql::RawOrderExpr) -> Self {
        Self {
            state: self.state.order_raw(expr.into_inner()),
            _marker: PhantomData,
        }
    }

    pub fn select_raw(self, expr: crate::common::sql::RawSelectExpr) -> Self {
        Self {
            state: self.state.select_raw(expr.into_inner()),
            _marker: PhantomData,
        }
    }

    pub fn add_select_raw(self, expr: crate::common::sql::RawSelectExpr) -> Self {
        Self {
            state: self.state.add_select_raw(expr.into_inner()),
            _marker: PhantomData,
        }
    }

    pub fn join_raw(self, join: crate::common::sql::RawJoinSpec) -> Self {
        let (kind, table, on_sql, binds) = join.into_parts();
        Self {
            state: self
                .state
                .join_raw(raw_join_kind_sql(kind), table, on_sql, binds),
            _marker: PhantomData,
        }
    }

    pub fn having_raw(self, clause: crate::common::sql::RawClause) -> Self {
        let (sql, binds) = clause.into_parts();
        Self {
            state: self.state.having_raw(sql, binds),
            _marker: PhantomData,
        }
    }

    pub fn group_by_raw(self, expr: crate::common::sql::RawGroupExpr) -> Self {
        Self {
            state: self.state.group_by_raw(expr.into_inner()),
            _marker: PhantomData,
        }
    }

    pub fn with<R>(self, relation: R) -> Self
    where
        R: IncludeRelation<M>,
    {
        let mut state = self.state;
        let spec = R::load_spec(relation, state.base_url.clone());
        let list = state.with_relations.get_or_insert_with(Vec::new);
        list.retain(|existing| existing.name != spec.name);
        list.push(spec);
        Self {
            state,
            _marker: PhantomData,
        }
    }

    /// Eager-load a relation with additional conditions and nested relation/metric trees.
    pub fn with_scope<R, T, F>(mut self, relation: R, scope: F) -> Self
    where
        R: IncludeRelation<M> + RelationName,
        T: QueryModel,
        F: FnOnce(Query<T>) -> Query<T>,
    {
        let name = relation.relation_name();
        let dummy_state = QueryState::new(self.state.base_url.clone(), T::DEFAULT_SELECT);
        let scoped = scope(Query::<T>::from_inner(dummy_state));
        let inner = scoped.into_inner();

        let mut spec = R::load_spec(relation, self.state.base_url.clone());
        spec.filters.extend(inner.filters);
        spec.orders = inner.orders;
        spec.limit = inner.limit;
        spec.offset = inner.offset;
        spec.selects = if inner.selects == parse_select_list(T::DEFAULT_SELECT) {
            Vec::new()
        } else {
            inner.selects
        };
        spec.with_deleted = spec.with_deleted || inner.with_deleted;
        spec.only_deleted = spec.only_deleted || inner.only_deleted;
        spec.nested = inner.with_relations.unwrap_or_default();
        spec.counts.extend(inner.count_relations);
        spec.aggregates.extend(inner.aggregate_relations);
        let list = self.state.with_relations.get_or_insert_with(Vec::new);
        list.retain(|s| s.name != name);
        list.push(spec);
        self
    }

    pub fn where_has<R, F>(self, relation: R, scope: F) -> Self
    where
        R: WhereHasRelation<M>,
        F: FnOnce(Query<R::Target>) -> Query<R::Target>,
    {
        Self {
            state: R::where_has(relation, self.state, scope),
            _marker: PhantomData,
        }
    }

    pub fn or_where_has<R, F>(self, relation: R, scope: F) -> Self
    where
        R: WhereHasRelation<M>,
        F: FnOnce(Query<R::Target>) -> Query<R::Target>,
    {
        Self {
            state: R::or_where_has(relation, self.state, scope),
            _marker: PhantomData,
        }
    }

    /// Request a count for a HasMany relation instead of loading full records.
    /// The count is populated on `record.__relation_counts` and accessible via `record.count(Rel::NAME)`.
    pub fn with_count<R>(mut self, relation: R) -> Self
    where
        R: CountRelation<M>,
    {
        self.state
            .count_relations
            .push(R::spec(relation, self.state.base_url.clone()));
        self
    }

    /// Request a conditional count for a HasMany relation.
    pub fn with_count_scope<R, T, F>(mut self, relation: R, scope: F) -> Self
    where
        R: CountRelation<M>,
        T: QueryModel,
        F: FnOnce(Query<T>) -> Query<T>,
    {
        let dummy_state = QueryState::new(self.state.base_url.clone(), T::DEFAULT_SELECT);
        let scoped = scope(Query::<T>::from_inner(dummy_state));
        let inner = scoped.into_inner();

        let mut spec = R::spec(relation, self.state.base_url.clone());
        spec.filters.extend(inner.filters);
        spec.with_deleted = spec.with_deleted || inner.with_deleted;
        spec.only_deleted = spec.only_deleted || inner.only_deleted;
        self.state.count_relations.push(spec);
        self
    }

    pub fn with_sum<R, C>(self, relation: R, column: C) -> Self
    where
        R: CountRelation<M>,
        C: Into<AggregateTarget<R::TargetModel>>,
    {
        self.with_sum_target(relation, column.into())
    }

    pub fn with_sum_expr<R>(self, relation: R, target: AggregateTarget<R::TargetModel>) -> Self
    where
        R: CountRelation<M>,
    {
        self.with_sum_target(relation, target)
    }

    fn with_sum_target<R>(mut self, relation: R, target: AggregateTarget<R::TargetModel>) -> Self
    where
        R: CountRelation<M>,
    {
        let spec = R::spec(relation, self.state.base_url.clone());
        self.state.aggregate_relations.push(RelationAggregateSpec {
            relation_name: spec.name,
            target_table: spec.target_table,
            target_pk: spec.target_pk,
            foreign_key: spec.foreign_key,
            has_soft_delete: spec.has_soft_delete,
            target: target.into_spec(),
            kind: RelationAggregateKind::Sum,
            filters: spec.filters,
            with_deleted: spec.with_deleted,
            only_deleted: spec.only_deleted,
        });
        self
    }

    pub fn with_sum_scope<R, T, F, C>(self, relation: R, column: C, scope: F) -> Self
    where
        R: CountRelation<M>,
        T: QueryModel,
        F: FnOnce(Query<T>) -> Query<T>,
        C: Into<AggregateTarget<R::TargetModel>>,
    {
        self.with_sum_scope_target(relation, column.into(), scope)
    }

    pub fn with_sum_expr_scope<R, T, F>(
        self,
        relation: R,
        target: AggregateTarget<R::TargetModel>,
        scope: F,
    ) -> Self
    where
        R: CountRelation<M>,
        T: QueryModel,
        F: FnOnce(Query<T>) -> Query<T>,
    {
        self.with_sum_scope_target(relation, target, scope)
    }

    fn with_sum_scope_target<R, T, F>(
        mut self,
        relation: R,
        target: AggregateTarget<R::TargetModel>,
        scope: F,
    ) -> Self
    where
        R: CountRelation<M>,
        T: QueryModel,
        F: FnOnce(Query<T>) -> Query<T>,
    {
        let dummy_state = QueryState::new(self.state.base_url.clone(), T::DEFAULT_SELECT);
        let scoped = scope(Query::<T>::from_inner(dummy_state));
        let inner = scoped.into_inner();

        let mut spec = R::spec(relation, self.state.base_url.clone());
        spec.filters.extend(inner.filters);
        spec.with_deleted = spec.with_deleted || inner.with_deleted;
        spec.only_deleted = spec.only_deleted || inner.only_deleted;
        self.state.aggregate_relations.push(RelationAggregateSpec {
            relation_name: spec.name,
            target_table: spec.target_table,
            target_pk: spec.target_pk,
            foreign_key: spec.foreign_key,
            has_soft_delete: spec.has_soft_delete,
            target: target.into_spec(),
            kind: RelationAggregateKind::Sum,
            filters: spec.filters,
            with_deleted: spec.with_deleted,
            only_deleted: spec.only_deleted,
        });
        self
    }

    pub fn with_avg<R, C>(self, relation: R, column: C) -> Self
    where
        R: CountRelation<M>,
        C: Into<AggregateTarget<R::TargetModel>>,
    {
        self.with_avg_target(relation, column.into())
    }

    pub fn with_avg_expr<R>(self, relation: R, target: AggregateTarget<R::TargetModel>) -> Self
    where
        R: CountRelation<M>,
    {
        self.with_avg_target(relation, target)
    }

    fn with_avg_target<R>(mut self, relation: R, target: AggregateTarget<R::TargetModel>) -> Self
    where
        R: CountRelation<M>,
    {
        let spec = R::spec(relation, self.state.base_url.clone());
        self.state.aggregate_relations.push(RelationAggregateSpec {
            relation_name: spec.name,
            target_table: spec.target_table,
            target_pk: spec.target_pk,
            foreign_key: spec.foreign_key,
            has_soft_delete: spec.has_soft_delete,
            target: target.into_spec(),
            kind: RelationAggregateKind::Avg,
            filters: spec.filters,
            with_deleted: spec.with_deleted,
            only_deleted: spec.only_deleted,
        });
        self
    }

    pub fn with_avg_scope<R, T, F, C>(self, relation: R, column: C, scope: F) -> Self
    where
        R: CountRelation<M>,
        T: QueryModel,
        F: FnOnce(Query<T>) -> Query<T>,
        C: Into<AggregateTarget<R::TargetModel>>,
    {
        self.with_avg_scope_target(relation, column.into(), scope)
    }

    pub fn with_avg_expr_scope<R, T, F>(
        self,
        relation: R,
        target: AggregateTarget<R::TargetModel>,
        scope: F,
    ) -> Self
    where
        R: CountRelation<M>,
        T: QueryModel,
        F: FnOnce(Query<T>) -> Query<T>,
    {
        self.with_avg_scope_target(relation, target, scope)
    }

    fn with_avg_scope_target<R, T, F>(
        mut self,
        relation: R,
        target: AggregateTarget<R::TargetModel>,
        scope: F,
    ) -> Self
    where
        R: CountRelation<M>,
        T: QueryModel,
        F: FnOnce(Query<T>) -> Query<T>,
    {
        let dummy_state = QueryState::new(self.state.base_url.clone(), T::DEFAULT_SELECT);
        let scoped = scope(Query::<T>::from_inner(dummy_state));
        let inner = scoped.into_inner();
        let mut spec = R::spec(relation, self.state.base_url.clone());
        spec.filters.extend(inner.filters);
        spec.with_deleted = spec.with_deleted || inner.with_deleted;
        spec.only_deleted = spec.only_deleted || inner.only_deleted;
        self.state.aggregate_relations.push(RelationAggregateSpec {
            relation_name: spec.name,
            target_table: spec.target_table,
            target_pk: spec.target_pk,
            foreign_key: spec.foreign_key,
            has_soft_delete: spec.has_soft_delete,
            target: target.into_spec(),
            kind: RelationAggregateKind::Avg,
            filters: spec.filters,
            with_deleted: spec.with_deleted,
            only_deleted: spec.only_deleted,
        });
        self
    }

    pub fn with_min<R, C>(self, relation: R, column: C) -> Self
    where
        R: CountRelation<M>,
        C: Into<AggregateTarget<R::TargetModel>>,
    {
        self.with_min_target(relation, column.into())
    }

    pub fn with_min_expr<R>(self, relation: R, target: AggregateTarget<R::TargetModel>) -> Self
    where
        R: CountRelation<M>,
    {
        self.with_min_target(relation, target)
    }

    fn with_min_target<R>(mut self, relation: R, target: AggregateTarget<R::TargetModel>) -> Self
    where
        R: CountRelation<M>,
    {
        let spec = R::spec(relation, self.state.base_url.clone());
        self.state.aggregate_relations.push(RelationAggregateSpec {
            relation_name: spec.name,
            target_table: spec.target_table,
            target_pk: spec.target_pk,
            foreign_key: spec.foreign_key,
            has_soft_delete: spec.has_soft_delete,
            target: target.into_spec(),
            kind: RelationAggregateKind::Min,
            filters: spec.filters,
            with_deleted: spec.with_deleted,
            only_deleted: spec.only_deleted,
        });
        self
    }

    pub fn with_min_scope<R, T, F, C>(self, relation: R, column: C, scope: F) -> Self
    where
        R: CountRelation<M>,
        T: QueryModel,
        F: FnOnce(Query<T>) -> Query<T>,
        C: Into<AggregateTarget<R::TargetModel>>,
    {
        self.with_min_scope_target(relation, column.into(), scope)
    }

    pub fn with_min_expr_scope<R, T, F>(
        self,
        relation: R,
        target: AggregateTarget<R::TargetModel>,
        scope: F,
    ) -> Self
    where
        R: CountRelation<M>,
        T: QueryModel,
        F: FnOnce(Query<T>) -> Query<T>,
    {
        self.with_min_scope_target(relation, target, scope)
    }

    fn with_min_scope_target<R, T, F>(
        mut self,
        relation: R,
        target: AggregateTarget<R::TargetModel>,
        scope: F,
    ) -> Self
    where
        R: CountRelation<M>,
        T: QueryModel,
        F: FnOnce(Query<T>) -> Query<T>,
    {
        let dummy_state = QueryState::new(self.state.base_url.clone(), T::DEFAULT_SELECT);
        let scoped = scope(Query::<T>::from_inner(dummy_state));
        let inner = scoped.into_inner();
        let mut spec = R::spec(relation, self.state.base_url.clone());
        spec.filters.extend(inner.filters);
        spec.with_deleted = spec.with_deleted || inner.with_deleted;
        spec.only_deleted = spec.only_deleted || inner.only_deleted;
        self.state.aggregate_relations.push(RelationAggregateSpec {
            relation_name: spec.name,
            target_table: spec.target_table,
            target_pk: spec.target_pk,
            foreign_key: spec.foreign_key,
            has_soft_delete: spec.has_soft_delete,
            target: target.into_spec(),
            kind: RelationAggregateKind::Min,
            filters: spec.filters,
            with_deleted: spec.with_deleted,
            only_deleted: spec.only_deleted,
        });
        self
    }

    pub fn with_max<R, C>(self, relation: R, column: C) -> Self
    where
        R: CountRelation<M>,
        C: Into<AggregateTarget<R::TargetModel>>,
    {
        self.with_max_target(relation, column.into())
    }

    pub fn with_max_expr<R>(self, relation: R, target: AggregateTarget<R::TargetModel>) -> Self
    where
        R: CountRelation<M>,
    {
        self.with_max_target(relation, target)
    }

    fn with_max_target<R>(mut self, relation: R, target: AggregateTarget<R::TargetModel>) -> Self
    where
        R: CountRelation<M>,
    {
        let spec = R::spec(relation, self.state.base_url.clone());
        self.state.aggregate_relations.push(RelationAggregateSpec {
            relation_name: spec.name,
            target_table: spec.target_table,
            target_pk: spec.target_pk,
            foreign_key: spec.foreign_key,
            has_soft_delete: spec.has_soft_delete,
            target: target.into_spec(),
            kind: RelationAggregateKind::Max,
            filters: spec.filters,
            with_deleted: spec.with_deleted,
            only_deleted: spec.only_deleted,
        });
        self
    }

    pub fn with_max_scope<R, T, F, C>(self, relation: R, column: C, scope: F) -> Self
    where
        R: CountRelation<M>,
        T: QueryModel,
        F: FnOnce(Query<T>) -> Query<T>,
        C: Into<AggregateTarget<R::TargetModel>>,
    {
        self.with_max_scope_target(relation, column.into(), scope)
    }

    pub fn with_max_expr_scope<R, T, F>(
        self,
        relation: R,
        target: AggregateTarget<R::TargetModel>,
        scope: F,
    ) -> Self
    where
        R: CountRelation<M>,
        T: QueryModel,
        F: FnOnce(Query<T>) -> Query<T>,
    {
        self.with_max_scope_target(relation, target, scope)
    }

    fn with_max_scope_target<R, T, F>(
        mut self,
        relation: R,
        target: AggregateTarget<R::TargetModel>,
        scope: F,
    ) -> Self
    where
        R: CountRelation<M>,
        T: QueryModel,
        F: FnOnce(Query<T>) -> Query<T>,
    {
        let dummy_state = QueryState::new(self.state.base_url.clone(), T::DEFAULT_SELECT);
        let scoped = scope(Query::<T>::from_inner(dummy_state));
        let inner = scoped.into_inner();
        let mut spec = R::spec(relation, self.state.base_url.clone());
        spec.filters.extend(inner.filters);
        spec.with_deleted = spec.with_deleted || inner.with_deleted;
        spec.only_deleted = spec.only_deleted || inner.only_deleted;
        self.state.aggregate_relations.push(RelationAggregateSpec {
            relation_name: spec.name,
            target_table: spec.target_table,
            target_pk: spec.target_pk,
            foreign_key: spec.foreign_key,
            has_soft_delete: spec.has_soft_delete,
            target: target.into_spec(),
            kind: RelationAggregateKind::Max,
            filters: spec.filters,
            with_deleted: spec.with_deleted,
            only_deleted: spec.only_deleted,
        });
        self
    }

    pub fn unsafe_sql(self) -> UnsafeQuery<'db, M> {
        UnsafeQuery { inner: self }
    }

    pub fn where_exists_raw<T>(
        self,
        clause: impl Into<String>,
        binds: impl IntoIterator<Item = T>,
    ) -> Self
    where
        T: Into<BindValue>,
    {
        Self {
            state: self
                .state
                .where_exists_raw(clause.into(), binds.into_iter().map(Into::into).collect()),
            _marker: PhantomData,
        }
    }

    pub async fn all(self, db: impl Into<DbConn<'db>>) -> Result<Vec<M::Record>>
    where
        M: RuntimeModel,
        M::Record: RelationMetricRecord,
    {
        check_deferred(&self.state.deferred_error)?;
        M::query_all(db.into(), self.state).await
    }

    pub async fn first(self, db: impl Into<DbConn<'db>>) -> Result<Option<M::Record>>
    where
        M: RuntimeModel,
        M::Record: RelationMetricRecord,
    {
        check_deferred(&self.state.deferred_error)?;
        M::query_first(db.into(), self.state).await
    }

    pub async fn find(self, db: impl Into<DbConn<'db>>, id: M::Pk) -> Result<Option<M::Record>>
    where
        M: RuntimeModel,
        M::Record: RelationMetricRecord,
    {
        check_deferred(&self.state.deferred_error)?;
        M::query_find(db.into(), self.state, id).await
    }

    pub async fn count(self, db: impl Into<DbConn<'db>>) -> Result<i64> {
        check_deferred(&self.state.deferred_error)?;
        M::query_count(db.into(), self.state).await
    }

    pub async fn delete(self, db: impl Into<DbConn<'db>>) -> Result<u64>
    where
        M: DeleteModel,
        M::Row: serde::Serialize,
    {
        check_deferred(&self.state.deferred_error)?;
        M::query_delete(db.into(), self.state).await
    }

    pub async fn paginate(
        self,
        db: impl Into<DbConn<'db>>,
        page: i64,
        per_page: i64,
    ) -> Result<Page<M::Record>>
    where
        M: RuntimeModel,
        M::Record: RelationMetricRecord,
    {
        check_deferred(&self.state.deferred_error)?;
        M::query_paginate(db.into(), self.state, page, per_page).await
    }

    pub fn patch(self) -> Patch<'db, M>
    where
        M: PatchModel,
    {
        Patch {
            state: M::patch_from_query(self.state),
            _marker: PhantomData,
        }
    }

    pub fn patch_selected(self) -> Patch<'db, M>
    where
        M: PatchModel,
    {
        Patch {
            state: PatchState::from_selected_query(
                self.state,
                M::TABLE,
                M::HAS_SOFT_DELETE,
                M::SOFT_DELETE_COL,
                M::PK_COL,
                false,
            ),
            _marker: PhantomData,
        }
    }

    pub fn claim(self) -> Patch<'db, M>
    where
        M: PatchModel,
    {
        Patch {
            state: PatchState::from_selected_query(
                self.state,
                M::TABLE,
                M::HAS_SOFT_DELETE,
                M::SOFT_DELETE_COL,
                M::PK_COL,
                true,
            ),
            _marker: PhantomData,
        }
    }

    // ── Aggregate terminal methods ────────────────────────────────────

    pub async fn sum(self, db: impl Into<DbConn<'db>>, col: impl ColExpr) -> Result<Option<f64>> {
        check_deferred(&self.state.deferred_error)?;
        self.state
            .aggregate_scalar(
                db.into(),
                &format!("SUM({}::DOUBLE PRECISION)", col.col_sql()),
                M::TABLE,
                M::HAS_SOFT_DELETE,
                M::SOFT_DELETE_COL,
            )
            .await
    }

    pub async fn avg(self, db: impl Into<DbConn<'db>>, col: impl ColExpr) -> Result<Option<f64>> {
        check_deferred(&self.state.deferred_error)?;
        self.state
            .aggregate_scalar(
                db.into(),
                &format!("AVG({}::DOUBLE PRECISION)", col.col_sql()),
                M::TABLE,
                M::HAS_SOFT_DELETE,
                M::SOFT_DELETE_COL,
            )
            .await
    }

    pub async fn min_val(
        self,
        db: impl Into<DbConn<'db>>,
        col: impl ColExpr,
    ) -> Result<Option<f64>> {
        check_deferred(&self.state.deferred_error)?;
        self.state
            .aggregate_scalar(
                db.into(),
                &format!("MIN({}::DOUBLE PRECISION)", col.col_sql()),
                M::TABLE,
                M::HAS_SOFT_DELETE,
                M::SOFT_DELETE_COL,
            )
            .await
    }

    pub async fn max_val(
        self,
        db: impl Into<DbConn<'db>>,
        col: impl ColExpr,
    ) -> Result<Option<f64>> {
        check_deferred(&self.state.deferred_error)?;
        self.state
            .aggregate_scalar(
                db.into(),
                &format!("MAX({}::DOUBLE PRECISION)", col.col_sql()),
                M::TABLE,
                M::HAS_SOFT_DELETE,
                M::SOFT_DELETE_COL,
            )
            .await
    }

    pub async fn exists(self, db: impl Into<DbConn<'db>>) -> Result<bool> {
        check_deferred(&self.state.deferred_error)?;
        Ok(self.count(db).await? > 0)
    }

    // ── Increment / Decrement ─────────────────────────────────────────

    pub async fn increment(
        self,
        db: impl Into<DbConn<'db>>,
        col: impl ColExpr,
        amount: i64,
    ) -> Result<u64> {
        check_deferred(&self.state.deferred_error)?;
        self.state
            .execute_increment(
                db.into(),
                col.col_sql(),
                BindValue::I64(amount),
                M::TABLE,
                M::HAS_SOFT_DELETE,
                M::SOFT_DELETE_COL,
                M::HAS_UPDATED_AT,
            )
            .await
    }

    pub async fn decrement(
        self,
        db: impl Into<DbConn<'db>>,
        col: impl ColExpr,
        amount: i64,
    ) -> Result<u64> {
        self.increment(db, col, -amount).await
    }

    // ── Fail-fast terminal methods ────────────────────────────────────

    pub async fn first_or_fail(self, db: impl Into<DbConn<'db>>) -> Result<M::Record>
    where
        M: RuntimeModel,
        M::Record: RelationMetricRecord,
    {
        self.first(db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("{}: record not found", M::TABLE))
    }

    pub async fn find_or_fail(self, db: impl Into<DbConn<'db>>, id: M::Pk) -> Result<M::Record>
    where
        M: RuntimeModel,
        M::Record: RelationMetricRecord,
    {
        self.find(db, id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("{}: record not found", M::TABLE))
    }

    pub async fn sole(self, db: impl Into<DbConn<'db>>) -> Result<M::Record>
    where
        M: RuntimeModel,
        M::Record: RelationMetricRecord,
    {
        let mut rows = self.limit(2).all(db).await?;
        match rows.len() {
            0 => anyhow::bail!("{}: no record found", M::TABLE),
            1 => Ok(rows.remove(0)),
            _ => anyhow::bail!("{}: multiple records found", M::TABLE),
        }
    }

    // ── Chunk iteration ───────────────────────────────────────────────

    pub async fn chunk<F, Fut>(
        self,
        db: impl Into<DbConn<'db>>,
        size: i64,
        mut callback: F,
    ) -> Result<()>
    where
        M: RuntimeModel,
        M::Record: RelationMetricRecord,
        F: FnMut(Vec<M::Record>) -> Fut,
        Fut: std::future::Future<Output = Result<bool>>,
    {
        check_deferred(&self.state.deferred_error)?;
        if size < 1 {
            anyhow::bail!("chunk() size must be greater than 0");
        }
        let db = db.into();
        if self.state.lock_clause.is_some() {
            anyhow::bail!(
                "chunk() does not support row locks; use chunk_by_id() inside a transaction"
            );
        }
        let mut page = 0i64;
        loop {
            let page_state = {
                let mut s = self.state.clone();
                s.offset = Some(page * size);
                s.limit = Some(size);
                s
            };
            let rows = M::query_all(db.clone(), page_state).await?;
            if rows.is_empty() {
                break;
            }
            let should_continue = callback(rows).await?;
            if !should_continue {
                break;
            }
            page += 1;
        }
        Ok(())
    }

    pub async fn chunk_by_id<F, Fut>(
        self,
        db: impl Into<DbConn<'db>>,
        size: i64,
        mut callback: F,
    ) -> Result<()>
    where
        M: ChunkModel,
        M::Record: RelationMetricRecord,
        F: FnMut(Vec<M::Record>) -> Fut,
        Fut: std::future::Future<Output = Result<bool>>,
    {
        check_deferred(&self.state.deferred_error)?;
        if size < 1 {
            anyhow::bail!("chunk_by_id() size must be greater than 0");
        }
        if self.state.offset.is_some() || self.state.limit.is_some() {
            anyhow::bail!("chunk_by_id() does not support pre-set limit/offset");
        }
        if !self.state.orders.is_empty() {
            anyhow::bail!("chunk_by_id() does not support custom ordering; it always orders by the primary key");
        }

        let db = db.into();
        if self.state.lock_clause.is_some() && matches!(db, DbConn::Pool(_)) {
            anyhow::bail!(
                "chunk_by_id() with row locks requires a transaction DbConn::Tx; pool-backed statements release locks before the callback runs"
            );
        }

        let mut last_seen: Option<M::Pk> = None;
        loop {
            let mut page_state = self.state.clone();
            page_state = page_state
                .order_by_str(M::PK_COL, OrderDir::Asc)
                .limit(size);
            if let Some(last_seen) = last_seen.clone() {
                page_state = page_state.where_col_str(M::PK_COL, Op::Gt, last_seen.into());
            }
            let rows = M::query_all(db.clone(), page_state).await?;
            if rows.is_empty() {
                break;
            }
            last_seen = rows.last().map(M::record_pk);
            let should_continue = callback(rows).await?;
            if !should_continue {
                break;
            }
        }
        Ok(())
    }

    // ── Convenience builder methods ───────────────────────────────────

    pub fn take(self, n: i64) -> Self {
        self.limit(n)
    }

    pub fn skip(self, n: i64) -> Self {
        self.offset(n)
    }

    pub fn latest(self) -> Self {
        if M::HAS_CREATED_AT {
            Self {
                state: self.state.order_by_str("created_at", OrderDir::Desc),
                _marker: PhantomData,
            }
        } else {
            Self {
                state: self.state.order_by_str(M::PK_COL, OrderDir::Desc),
                _marker: PhantomData,
            }
        }
    }

    pub fn oldest(self) -> Self {
        if M::HAS_CREATED_AT {
            Self {
                state: self.state.order_by_str("created_at", OrderDir::Asc),
                _marker: PhantomData,
            }
        } else {
            Self {
                state: self.state.order_by_str(M::PK_COL, OrderDir::Asc),
                _marker: PhantomData,
            }
        }
    }

    pub fn in_random_order(self) -> Self {
        Self {
            state: self.state.order_raw("RANDOM()".to_string()),
            _marker: PhantomData,
        }
    }

    pub fn distinct(self) -> Self {
        Self {
            state: self.state.distinct(),
            _marker: PhantomData,
        }
    }

    pub fn for_share(self) -> Self {
        Self {
            state: self.state.for_share(),
            _marker: PhantomData,
        }
    }

    // ── Additional WHERE methods ──────────────────────────────────────

    pub fn where_between<F, V>(self, field: F, low: V, high: V) -> Self
    where
        F: ColExpr,
        V: Into<BindValue>,
    {
        Self {
            state: self
                .state
                .where_between_str(field.col_sql(), low.into(), high.into()),
            _marker: PhantomData,
        }
    }

    pub fn where_not_in<F, I, V>(self, field: F, values: I) -> Self
    where
        F: ColExpr,
        I: IntoIterator<Item = V>,
        V: Into<BindValue>,
    {
        let bind_values: Vec<BindValue> = values.into_iter().map(Into::into).collect();
        Self {
            state: self.state.where_not_in_str(field.col_sql(), &bind_values),
            _marker: PhantomData,
        }
    }

    /// Conditionally apply a scope. Laravel's `when()`.
    pub fn when<F>(self, condition: bool, scope: F) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        if condition {
            scope(self)
        } else {
            self
        }
    }

    /// Conditionally apply a scope, with an else branch.
    pub fn when_else<F, G>(self, condition: bool, if_true: F, if_false: G) -> Self
    where
        F: FnOnce(Self) -> Self,
        G: FnOnce(Self) -> Self,
    {
        if condition {
            if_true(self)
        } else {
            if_false(self)
        }
    }

    // ── Soft delete: restore ──────────────────────────────────────────

    pub async fn restore(self, db: impl Into<DbConn<'db>>) -> Result<u64> {
        check_deferred(&self.state.deferred_error)?;
        if !M::HAS_SOFT_DELETE {
            anyhow::bail!("{}: restore() not supported (no soft delete)", M::TABLE);
        }
        self.state
            .execute_restore(db.into(), M::TABLE, M::SOFT_DELETE_COL, M::HAS_UPDATED_AT)
            .await
    }

    /// Include soft-deleted records (Laravel's withTrashed).
    pub fn with_deleted(self) -> Self {
        Self {
            state: self.state.with_deleted(),
            _marker: PhantomData,
        }
    }

    /// Only return soft-deleted records (Laravel's onlyTrashed).
    pub fn only_deleted(self) -> Self {
        Self {
            state: self.state.only_deleted(),
            _marker: PhantomData,
        }
    }

    // ── Pluck helpers ─────────────────────────────────────────────────

    pub async fn pluck<K, E>(self, db: impl Into<DbConn<'db>>, extract: E) -> Result<Vec<K>>
    where
        M: RuntimeModel,
        M::Record: RelationMetricRecord,
        E: Fn(&M::Record) -> K,
    {
        let rows = self.all(db).await?;
        Ok(rows.iter().map(|r| extract(r)).collect())
    }

    pub async fn pluck_map<K, V, E>(
        self,
        db: impl Into<DbConn<'db>>,
        extract: E,
    ) -> Result<std::collections::HashMap<K, V>>
    where
        M: RuntimeModel,
        M::Record: RelationMetricRecord,
        K: Eq + std::hash::Hash,
        E: Fn(&M::Record) -> (K, V),
    {
        let rows = self.all(db).await?;
        Ok(rows.iter().map(|r| extract(r)).collect())
    }
}

// ---------------------------------------------------------------------------
// UnsafeQuery wrapper — uses QueryState methods directly (no trait needed)
// ---------------------------------------------------------------------------

pub struct UnsafeQuery<'db, M: QueryModel> {
    inner: Query<'db, M>,
}

impl<'db, M: QueryModel> UnsafeQuery<'db, M> {
    pub fn where_raw<T>(self, clause: impl Into<String>, binds: impl IntoIterator<Item = T>) -> Self
    where
        T: Into<BindValue>,
    {
        Self {
            inner: Query {
                state: self
                    .inner
                    .state
                    .where_raw(clause.into(), binds.into_iter().map(Into::into).collect()),
                _marker: PhantomData,
            },
        }
    }

    pub fn order_raw(self, expr: impl Into<String>) -> Self {
        Self {
            inner: Query {
                state: self.inner.state.order_raw(expr.into()),
                _marker: PhantomData,
            },
        }
    }

    pub fn select_raw(self, expr: impl Into<String>) -> Self {
        Self {
            inner: Query {
                state: self.inner.state.select_raw(expr.into()),
                _marker: PhantomData,
            },
        }
    }

    pub fn join_raw<T>(
        self,
        table: impl Into<String>,
        on_clause: impl Into<String>,
        binds: impl IntoIterator<Item = T>,
    ) -> Self
    where
        T: Into<BindValue>,
    {
        Self {
            inner: Query {
                state: self.inner.state.join_raw(
                    "INNER JOIN",
                    table.into(),
                    on_clause.into(),
                    binds.into_iter().map(Into::into).collect(),
                ),
                _marker: PhantomData,
            },
        }
    }

    pub fn done(self) -> Query<'db, M> {
        self.inner
    }
}

// ---------------------------------------------------------------------------
// Create wrapper
// ---------------------------------------------------------------------------

pub struct Create<'db, M: CreateModel> {
    state: CreateState<'db>,
    _marker: PhantomData<M>,
}

impl<'db, M: CreateModel> Create<'db, M> {
    pub fn new() -> Self {
        Self {
            state: CreateState::new(resolve_attachment_base_url(None), M::TABLE),
            _marker: PhantomData,
        }
    }

    pub fn new_with_base_url(base_url: Option<String>) -> Self {
        Self {
            state: CreateState::new(resolve_attachment_base_url(base_url), M::TABLE),
            _marker: PhantomData,
        }
    }

    pub fn from_inner(state: CreateState<'db>) -> Self {
        Self {
            state,
            _marker: PhantomData,
        }
    }

    pub fn into_inner(self) -> CreateState<'db> {
        self.state
    }

    pub fn set<F, V>(self, field: F, value: V) -> Result<Self>
    where
        F: CreateField<M>,
        V: Into<F::Value>,
    {
        Ok(Self {
            state: F::set(field, self.state, value.into())?,
            _marker: PhantomData,
        })
    }

    pub fn on_conflict_do_nothing<F>(self, fields: &[F]) -> Self
    where
        F: CreateConflictField<M>,
    {
        Self {
            state: F::on_conflict_do_nothing(self.state, fields),
            _marker: PhantomData,
        }
    }

    pub fn on_conflict_update<F>(self, fields: &[F]) -> Self
    where
        F: CreateConflictField<M>,
    {
        Self {
            state: F::on_conflict_update(self.state, fields),
            _marker: PhantomData,
        }
    }

    pub fn set_translation(
        self,
        field: &'static str,
        locale: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        Self {
            state: self
                .state
                .set_translation(field, locale.into(), value.into()),
            _marker: PhantomData,
        }
    }

    pub fn insert_meta_value(self, key: impl Into<String>, value: serde_json::Value) -> Self {
        Self {
            state: self.state.insert_meta_value(key, value),
            _marker: PhantomData,
        }
    }

    pub fn set_attachment_single(self, field: &'static str, input: AttachmentInput) -> Self {
        Self {
            state: self.state.set_attachment_single(field, input),
            _marker: PhantomData,
        }
    }

    pub fn add_attachment_multi(self, field: &'static str, input: AttachmentInput) -> Self {
        Self {
            state: self.state.add_attachment_multi(field, input),
            _marker: PhantomData,
        }
    }

    pub async fn save(self, db: impl Into<DbConn<'db>>) -> Result<M::Record>
    where
        M: RuntimeModel,
        M::Create: Serialize,
        M::Row: Serialize,
    {
        M::create_save(db.into(), self.state).await
    }
}

// ---------------------------------------------------------------------------
// Patch wrapper
// ---------------------------------------------------------------------------

pub struct Patch<'db, M: PatchModel> {
    state: PatchState<'db>,
    _marker: PhantomData<M>,
}

pub struct PatchReturningScalar<'db, M: PatchModel, T> {
    state: PatchState<'db>,
    returning: ReturningSpec,
    _marker: PhantomData<(M, T)>,
}

pub struct PatchReturningJson<'db, M: PatchModel> {
    state: PatchState<'db>,
    returning: ReturningSpec,
    _marker: PhantomData<M>,
}

pub struct PatchReturningAll<'db, M: PatchModel> {
    state: PatchState<'db>,
    _marker: PhantomData<M>,
}

impl<'db, M: PatchModel> Patch<'db, M> {
    pub fn new() -> Self {
        Self {
            state: PatchState::new(
                resolve_attachment_base_url(None),
                M::TABLE,
                M::HAS_SOFT_DELETE,
                M::SOFT_DELETE_COL,
            ),
            _marker: PhantomData,
        }
    }

    pub fn new_with_base_url(base_url: Option<String>) -> Self {
        Self {
            state: PatchState::new(
                resolve_attachment_base_url(base_url),
                M::TABLE,
                M::HAS_SOFT_DELETE,
                M::SOFT_DELETE_COL,
            ),
            _marker: PhantomData,
        }
    }

    pub fn from_inner(state: PatchState<'db>) -> Self {
        Self {
            state,
            _marker: PhantomData,
        }
    }

    pub fn into_inner(self) -> PatchState<'db> {
        self.state
    }

    pub fn assign<F, V>(self, field: F, value: V) -> Result<Self>
    where
        F: PatchAssignField<M>,
        V: Into<F::Value>,
    {
        Ok(Self {
            state: F::assign(field, self.state, value.into())?,
            _marker: PhantomData,
        })
    }

    pub fn increment<F, V>(self, field: F, value: V) -> Result<Self>
    where
        F: PatchNumericField<M>,
        V: Into<F::Value>,
    {
        Ok(Self {
            state: F::increment(field, self.state, value.into())?,
            _marker: PhantomData,
        })
    }

    pub fn decrement<F, V>(self, field: F, value: V) -> Result<Self>
    where
        F: PatchNumericField<M>,
        V: Into<F::Value>,
    {
        Ok(Self {
            state: F::decrement(field, self.state, value.into())?,
            _marker: PhantomData,
        })
    }

    pub fn where_raw<I, V>(self, clause: impl Into<String>, binds: I) -> Self
    where
        I: IntoIterator<Item = V>,
        V: Into<BindValue>,
    {
        Self {
            state: self
                .state
                .where_raw(clause.into(), binds.into_iter().map(Into::into).collect()),
            _marker: PhantomData,
        }
    }

    pub fn with_deleted(self) -> Self {
        Self {
            state: self.state.with_deleted(),
            _marker: PhantomData,
        }
    }

    pub fn only_deleted(self) -> Self {
        Self {
            state: self.state.only_deleted(),
            _marker: PhantomData,
        }
    }

    pub fn set_translation(
        self,
        field: &'static str,
        locale: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        Self {
            state: self
                .state
                .set_translation(field, locale.into(), value.into()),
            _marker: PhantomData,
        }
    }

    pub fn insert_meta_value(self, key: impl Into<String>, value: serde_json::Value) -> Self {
        Self {
            state: self.state.insert_meta_value(key, value),
            _marker: PhantomData,
        }
    }

    pub fn set_attachment_single(self, field: &'static str, input: AttachmentInput) -> Self {
        Self {
            state: self.state.set_attachment_single(field, input),
            _marker: PhantomData,
        }
    }

    pub fn add_attachment_multi(self, field: &'static str, input: AttachmentInput) -> Self {
        Self {
            state: self.state.add_attachment_multi(field, input),
            _marker: PhantomData,
        }
    }

    pub fn clear_attachment_single(self, field: &'static str) -> Self {
        Self {
            state: self.state.clear_attachment_single(field),
            _marker: PhantomData,
        }
    }

    pub fn delete_attachment_multi_ids(
        self,
        field: &'static str,
        ids: impl IntoIterator<Item = Uuid>,
    ) -> Self {
        Self {
            state: self.state.delete_attachment_multi_ids(field, ids),
            _marker: PhantomData,
        }
    }

    pub fn returning<T>(self, col: Column<M, T>) -> PatchReturningScalar<'db, M, T> {
        PatchReturningScalar {
            state: self.state,
            returning: ReturningSpec::Scalar(ReturnExpr::Column(col.col_sql())),
            _marker: PhantomData,
        }
    }

    pub fn returning_many<I>(self, cols: I) -> PatchReturningJson<'db, M>
    where
        I: IntoIterator<Item = ReturnCol<M>>,
    {
        let fields = cols
            .into_iter()
            .map(|col| {
                let sql = col.sql();
                JsonReturnField {
                    key: sql.to_string(),
                    expr: ReturnExpr::Column(sql),
                }
            })
            .collect::<Vec<_>>();
        PatchReturningJson {
            state: self.state,
            returning: ReturningSpec::JsonObject(fields),
            _marker: PhantomData,
        }
    }

    pub fn returning_raw(self, expr: impl Into<String>) -> PatchReturningJson<'db, M> {
        PatchReturningJson {
            state: self.state,
            returning: ReturningSpec::JsonExpr(ReturnExpr::Raw(expr.into())),
            _marker: PhantomData,
        }
    }

    pub fn returning_all(self) -> PatchReturningAll<'db, M> {
        PatchReturningAll {
            state: self.state,
            _marker: PhantomData,
        }
    }

    pub async fn save(self, db: impl Into<DbConn<'db>>) -> Result<u64>
    where
        M: RuntimeModel,
        M::Changes: Serialize + Default,
        M::Pk: Clone
            + Send
            + Unpin
            + for<'r> sqlx::Decode<'r, sqlx::Postgres>
            + sqlx::Type<sqlx::Postgres>,
        M::Row: Serialize,
    {
        check_deferred(&self.state.deferred_error)?;
        if self.state.assignments.is_empty() {
            anyhow::bail!("update: no columns set");
        }
        if !self.state.has_conditions() {
            anyhow::bail!("update: no conditions set");
        }
        M::patch_save(db.into(), self.state).await
    }

    pub async fn fetch(self, db: impl Into<DbConn<'db>>) -> Result<Vec<M::Record>>
    where
        M: RuntimeModel,
        M::Changes: Serialize + Default,
        M::Pk: Clone
            + Send
            + Unpin
            + for<'r> sqlx::Decode<'r, sqlx::Postgres>
            + sqlx::Type<sqlx::Postgres>,
        M::Row: Serialize,
        M::Record: RelationMetricRecord,
    {
        check_deferred(&self.state.deferred_error)?;
        if self.state.assignments.is_empty() {
            anyhow::bail!("update: no columns set");
        }
        if !self.state.has_conditions() {
            anyhow::bail!("update: no conditions set");
        }
        M::patch_fetch(db.into(), self.state).await
    }
}

impl<'db, M: PatchModel, T: 'static> PatchReturningScalar<'db, M, T> {
    pub async fn fetch_scalars(self, db: impl Into<DbConn<'db>>) -> Result<Vec<T>>
    where
        T: Send + Unpin + for<'r> sqlx::Decode<'r, sqlx::Postgres> + sqlx::Type<sqlx::Postgres>,
    {
        check_deferred(&self.state.deferred_error)?;
        if self.state.assignments.is_empty() {
            anyhow::bail!("update: no columns set");
        }
        if !self.state.has_conditions() {
            anyhow::bail!("update: no conditions set");
        }
        self.state
            .execute_returning_scalar(db.into(), &self.returning)
            .await
    }
}

impl<'db, M: PatchModel> PatchReturningJson<'db, M> {
    pub async fn fetch_json(self, db: impl Into<DbConn<'db>>) -> Result<Vec<serde_json::Value>> {
        check_deferred(&self.state.deferred_error)?;
        if self.state.assignments.is_empty() {
            anyhow::bail!("update: no columns set");
        }
        if !self.state.has_conditions() {
            anyhow::bail!("update: no conditions set");
        }
        self.state
            .execute_returning_json(db.into(), &self.returning)
            .await
    }
}

impl<'db, M: PatchModel> PatchReturningAll<'db, M> {
    pub async fn fetch(self, db: impl Into<DbConn<'db>>) -> Result<Vec<M::Record>>
    where
        M: RuntimeModel,
        M::Changes: Serialize + Default,
        M::Pk: Clone
            + Send
            + Unpin
            + for<'r> sqlx::Decode<'r, sqlx::Postgres>
            + sqlx::Type<sqlx::Postgres>,
        M::Row: Serialize,
    {
        check_deferred(&self.state.deferred_error)?;
        if self.state.assignments.is_empty() {
            anyhow::bail!("update: no columns set");
        }
        if !self.state.has_conditions() {
            anyhow::bail!("update: no conditions set");
        }
        M::patch_fetch_returning_all(db.into(), self.state).await
    }
}

// ---------------------------------------------------------------------------
// QueryState — shared SQL builder state (replaces per-model XxxQueryInner)
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct QueryState<'db> {
    pub base_url: Option<String>,
    pub selects: Vec<SelectExpr>,
    pub from_sql: Option<String>,
    pub count_sql: Option<String>,
    pub distinct: bool,
    pub distinct_on: Option<String>,
    pub deferred_error: Option<String>,
    pub lock_clause: Option<LockClause>,
    pub joins: Vec<JoinExpr>,
    pub filters: Vec<FilterExpr>,
    pub orders: Vec<OrderExpr>,
    pub group_by: Vec<String>,
    pub havings: Vec<HavingExpr>,
    pub offset: Option<i64>,
    pub limit: Option<i64>,
    pub with_deleted: bool,
    pub only_deleted: bool,
    pub count_relations: Vec<CountRelationSpec>,
    pub aggregate_relations: Vec<RelationAggregateSpec>,
    pub existence_relations: Vec<RootExistenceNode>,
    /// Which relations to eager-load. `None` = none. `Some(vec)` = only listed.
    pub with_relations: Option<Vec<WithRelationSpec>>,
    _scope: PhantomData<&'db ()>,
}

impl<'db> QueryState<'db> {
    pub fn new(base_url: Option<String>, default_select: &str) -> Self {
        Self {
            base_url,
            selects: parse_select_list(default_select),
            from_sql: None,
            count_sql: None,
            distinct: false,
            distinct_on: None,
            deferred_error: None,
            lock_clause: None,
            joins: vec![],
            filters: vec![],
            orders: vec![],
            group_by: vec![],
            havings: vec![],
            offset: None,
            limit: None,
            with_deleted: false,
            only_deleted: false,
            count_relations: vec![],
            aggregate_relations: vec![],
            existence_relations: vec![],
            with_relations: None,
            _scope: PhantomData,
        }
    }

    // ── WHERE helpers ──────────────────────────────────────────────────

    pub fn where_col_str(mut self, col_sql: &str, op: Op, val: BindValue) -> Self {
        if val.is_null() {
            return match op {
                Op::Eq => self.where_null_str(col_sql),
                Op::Ne => self.where_not_null_str(col_sql),
                _ => self,
            };
        }
        self.filters.push(FilterExpr::Comparison {
            col_sql: col_sql.to_string(),
            op,
            value: val,
        });
        self
    }

    pub fn or_where_col_str(mut self, col_sql: &str, op: Op, val: BindValue) -> Self {
        if val.is_null() {
            let expr = match op {
                Op::Eq => FilterExpr::Null {
                    col_sql: col_sql.to_string(),
                    negated: false,
                },
                Op::Ne => FilterExpr::Null {
                    col_sql: col_sql.to_string(),
                    negated: true,
                },
                _ => return self,
            };
            self.push_or_filter(expr);
            return self;
        }
        self.push_or_filter(FilterExpr::Comparison {
            col_sql: col_sql.to_string(),
            op,
            value: val,
        });
        self
    }

    pub fn where_in_str(mut self, col_sql: &str, vals: &[BindValue]) -> Self {
        if vals.is_empty() {
            self.filters.push(FilterExpr::Raw {
                clause: "1=0".to_string(),
                binds: vec![],
            });
            return self;
        }
        self.filters.push(FilterExpr::In {
            col_sql: col_sql.to_string(),
            values: vals.to_vec(),
            negated: false,
        });
        self
    }

    pub fn where_not_in_str(mut self, col_sql: &str, vals: &[BindValue]) -> Self {
        if vals.is_empty() {
            return self;
        }
        self.filters.push(FilterExpr::In {
            col_sql: col_sql.to_string(),
            values: vals.to_vec(),
            negated: true,
        });
        self
    }

    pub fn where_between_str(mut self, col_sql: &str, low: BindValue, high: BindValue) -> Self {
        self.filters.push(FilterExpr::Between {
            col_sql: col_sql.to_string(),
            low,
            high,
        });
        self
    }

    pub fn where_null_str(mut self, col_sql: &str) -> Self {
        self.filters.push(FilterExpr::Null {
            col_sql: col_sql.to_string(),
            negated: false,
        });
        self
    }

    pub fn where_not_null_str(mut self, col_sql: &str) -> Self {
        self.filters.push(FilterExpr::Null {
            col_sql: col_sql.to_string(),
            negated: true,
        });
        self
    }

    pub fn where_col_cmp_str(mut self, left_sql: &str, op: Op, right_sql: &str) -> Self {
        self.filters.push(FilterExpr::ColumnCmp {
            left_sql: left_sql.to_string(),
            op,
            right_sql: right_sql.to_string(),
        });
        self
    }

    pub fn or_where_col_cmp_str(mut self, left_sql: &str, op: Op, right_sql: &str) -> Self {
        self.push_or_filter(FilterExpr::ColumnCmp {
            left_sql: left_sql.to_string(),
            op,
            right_sql: right_sql.to_string(),
        });
        self
    }

    pub fn where_expr_cmp_str(mut self, col_sql: &str, op: Op, expr: Expr) -> Self {
        self.filters.push(FilterExpr::ExprCmp {
            col_sql: col_sql.to_string(),
            op,
            expr,
        });
        self
    }

    pub fn or_where_expr_cmp_str(mut self, col_sql: &str, op: Op, expr: Expr) -> Self {
        self.push_or_filter(FilterExpr::ExprCmp {
            col_sql: col_sql.to_string(),
            op,
            expr,
        });
        self
    }

    pub fn where_raw(mut self, clause: String, raw_binds: Vec<BindValue>) -> Self {
        let mut clause = clause;
        let mut idx = 1usize;
        while let Some(pos) = clause.find('?') {
            let ph = format!("${idx}");
            clause.replace_range(pos..pos + 1, &ph);
            idx += 1;
        }
        self.filters.push(FilterExpr::Raw {
            clause,
            binds: raw_binds,
        });
        self
    }

    pub fn or_where_raw(mut self, clause: String, raw_binds: Vec<BindValue>) -> Self {
        self.push_or_filter(FilterExpr::Raw {
            clause,
            binds: raw_binds,
        });
        self
    }

    pub fn where_exists_raw(mut self, clause: String, raw_binds: Vec<BindValue>) -> Self {
        self.filters.push(FilterExpr::ExistsRaw {
            clause,
            binds: raw_binds,
        });
        self
    }

    pub fn push_relation_exists(
        mut self,
        boolean: ExistenceBoolean,
        node: RelationExistenceNode,
    ) -> Self {
        self.existence_relations
            .push(RootExistenceNode { boolean, node });
        self
    }

    pub fn defer_error(mut self, message: impl Into<String>) -> Self {
        self.deferred_error = Some(message.into());
        self
    }

    fn push_or_filter(&mut self, expr: FilterExpr) {
        if let Some(last) = self.filters.pop() {
            self.filters
                .push(FilterExpr::Or(Box::new(last), Box::new(expr)));
        } else {
            self.filters.push(expr);
        }
    }

    pub fn where_group(self, f: impl FnOnce(Self) -> Self) -> Self {
        let start_where = self.filters.len();
        let mut result = f(self);
        if result.filters.len() > start_where {
            let group_filters: Vec<FilterExpr> = result.filters.drain(start_where..).collect();
            result.filters.push(FilterExpr::Group(group_filters));
        }
        result
    }

    pub fn or_where_group(self, f: impl FnOnce(Self) -> Self) -> Self {
        let start_where = self.filters.len();
        let mut result = f(self);
        if result.filters.len() > start_where {
            let group_filters: Vec<FilterExpr> = result.filters.drain(start_where..).collect();
            result.push_or_filter(FilterExpr::Group(group_filters));
        }
        result
    }

    // ── ORDER BY ───────────────────────────────────────────────────────

    pub fn order_by_str(mut self, col_sql: &str, dir: OrderDir) -> Self {
        self.orders.push(OrderExpr::Column {
            sql: col_sql.to_string(),
            dir,
            nulls: None,
        });
        self
    }

    pub fn order_by_nulls_first_str(mut self, col_sql: &str, dir: OrderDir) -> Self {
        self.orders.push(OrderExpr::Column {
            sql: col_sql.to_string(),
            dir,
            nulls: Some(NullsOrder::First),
        });
        self
    }

    pub fn order_by_nulls_last_str(mut self, col_sql: &str, dir: OrderDir) -> Self {
        self.orders.push(OrderExpr::Column {
            sql: col_sql.to_string(),
            dir,
            nulls: Some(NullsOrder::Last),
        });
        self
    }

    pub fn order_raw(mut self, expr: String) -> Self {
        self.orders.push(OrderExpr::Raw(expr));
        self
    }

    // ── LIMIT / OFFSET ────────────────────────────────────────────────

    pub fn limit(mut self, n: i64) -> Self {
        self.limit = Some(n);
        self
    }

    pub fn offset(mut self, n: i64) -> Self {
        self.offset = Some(n);
        self
    }

    // ── DISTINCT ───────────────────────────────────────────────────────

    pub fn distinct(mut self) -> Self {
        self.distinct = true;
        self
    }

    pub fn distinct_on_str(mut self, cols: &str) -> Self {
        self.distinct_on = Some(cols.to_string());
        self
    }

    // ── LOCK ───────────────────────────────────────────────────────────

    pub fn for_update(mut self) -> Self {
        self.lock_clause = Some(LockClause {
            mode: LockMode::Update,
            modifier: None,
        });
        self
    }

    pub fn for_update_skip_locked(mut self) -> Self {
        self.lock_clause = Some(LockClause {
            mode: LockMode::Update,
            modifier: Some(LockModifier::SkipLocked),
        });
        self
    }

    pub fn for_no_key_update(mut self) -> Self {
        self.lock_clause = Some(LockClause {
            mode: LockMode::NoKeyUpdate,
            modifier: None,
        });
        self
    }

    pub fn for_share(mut self) -> Self {
        self.lock_clause = Some(LockClause {
            mode: LockMode::Share,
            modifier: None,
        });
        self
    }

    pub fn for_key_share(mut self) -> Self {
        self.lock_clause = Some(LockClause {
            mode: LockMode::KeyShare,
            modifier: None,
        });
        self
    }

    pub fn skip_locked(mut self) -> Self {
        match self.lock_clause.as_mut() {
            Some(lock) => {
                if lock.modifier.is_some() {
                    self.deferred_error =
                        Some("skip_locked()/no_wait() lock modifier already set".to_string());
                } else {
                    lock.modifier = Some(LockModifier::SkipLocked);
                }
            }
            None => {
                self.deferred_error =
                    Some("skip_locked() requires a lock mode such as for_update()".to_string());
            }
        }
        self
    }

    pub fn no_wait(mut self) -> Self {
        match self.lock_clause.as_mut() {
            Some(lock) => {
                if lock.modifier.is_some() {
                    self.deferred_error =
                        Some("skip_locked()/no_wait() lock modifier already set".to_string());
                } else {
                    lock.modifier = Some(LockModifier::NoWait);
                }
            }
            None => {
                self.deferred_error =
                    Some("no_wait() requires a lock mode such as for_update()".to_string());
            }
        }
        self
    }

    // ── JOIN ───────────────────────────────────────────────────────────

    pub fn join_raw(
        mut self,
        kind: &str,
        table: String,
        on_clause: String,
        raw_binds: Vec<BindValue>,
    ) -> Self {
        self.joins.push(JoinExpr {
            kind: kind.to_string(),
            table,
            on_clause,
            binds: raw_binds,
        });
        self
    }

    // ── SELECT ─────────────────────────────────────────────────────────

    pub fn select_raw(mut self, expr: String) -> Self {
        if expr.is_empty() {
            return self;
        }
        self.selects.push(SelectExpr::Raw(expr));
        self
    }

    pub fn add_select_raw(mut self, expr: String) -> Self {
        if expr.is_empty() {
            return self;
        }
        self.selects.push(SelectExpr::Raw(expr));
        self
    }

    pub fn select_only_str(mut self, col_sql: &str) -> Self {
        self.selects = vec![SelectExpr::Column(col_sql.to_string())];
        self
    }

    // ── GROUP BY / HAVING ──────────────────────────────────────────────

    pub fn group_by_str(mut self, cols: &[&str]) -> Self {
        for c in cols {
            self.group_by.push(c.to_string());
        }
        self
    }

    pub fn group_by_raw(mut self, expr: String) -> Self {
        if !expr.trim().is_empty() {
            self.group_by.push(expr);
        }
        self
    }

    pub fn having_raw(mut self, clause: String, raw_binds: Vec<BindValue>) -> Self {
        self.havings.push(HavingExpr {
            clause,
            binds: raw_binds,
        });
        self
    }

    // ── SOFT DELETE ────────────────────────────────────────────────────

    pub fn with_deleted(mut self) -> Self {
        self.with_deleted = true;
        self
    }

    pub fn only_deleted(mut self) -> Self {
        self.only_deleted = true;
        self
    }

    // ── FROM ───────────────────────────────────────────────────────────

    pub fn from_raw(mut self, sql: &str) -> Self {
        self.from_sql = Some(sql.to_string());
        self
    }

    pub fn count_sql(mut self, sql: &str) -> Self {
        self.count_sql = Some(sql.to_string());
        self
    }

    // ── SQL assembly ───────────────────────────────────────────────────

    fn build_select_clause(
        distinct: bool,
        distinct_on: Option<&str>,
        selects: &[SelectExpr],
    ) -> String {
        let cols = render_selects(selects);
        match (distinct, distinct_on) {
            (false, None) => cols,
            (true, None) => format!("DISTINCT {}", cols),
            (_, Some(on)) => format!("DISTINCT ON ({}) {}", on, cols),
        }
    }

    /// Assemble a full SELECT statement.
    pub fn to_select_sql(
        &self,
        table: &str,
        has_soft_delete: bool,
        soft_delete_col: &str,
    ) -> (String, Vec<BindValue>) {
        let table_name = self.from_sql.as_deref().unwrap_or(table);
        let (where_sql, base_binds) = compile_predicates(
            table_name,
            &self.filters,
            &self.existence_relations,
            has_soft_delete,
            soft_delete_col,
            self.with_deleted,
            self.only_deleted,
            1,
        );

        let select_clause =
            Self::build_select_clause(self.distinct, self.distinct_on.as_deref(), &self.selects);
        let mut sql = format!("SELECT {} FROM {}", select_clause, table_name);
        let (join_sql, join_binds) = compile_joins(&self.joins, base_binds.len() + 1);
        if !join_sql.is_empty() {
            sql.push(' ');
            sql.push_str(&join_sql.join(" "));
        }
        if !where_sql.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&where_sql.join(" AND "));
        }
        if !self.group_by.is_empty() {
            sql.push_str(" GROUP BY ");
            sql.push_str(&self.group_by.join(", "));
        }
        let (having_sql, having_binds) =
            compile_havings(&self.havings, base_binds.len() + join_binds.len() + 1);
        if !having_sql.is_empty() {
            sql.push_str(" HAVING ");
            sql.push_str(&having_sql.join(" AND "));
        }
        if !self.orders.is_empty() {
            sql.push_str(" ORDER BY ");
            sql.push_str(
                &self
                    .orders
                    .iter()
                    .map(render_order_expr)
                    .collect::<Vec<_>>()
                    .join(", "),
            );
        }
        if let Some(off) = self.offset {
            sql.push_str(" OFFSET ");
            sql.push_str(&off.to_string());
        }
        if let Some(l) = self.limit {
            sql.push_str(" LIMIT ");
            sql.push_str(&l.to_string());
        }
        if let Some(lock) = self.lock_clause {
            sql.push(' ');
            sql.push_str(render_lock_clause(lock));
        }
        let mut all_binds = base_binds;
        all_binds.extend(join_binds);
        all_binds.extend(having_binds);
        (sql, all_binds)
    }

    /// Assemble a COUNT query.
    pub fn to_count_sql(
        &self,
        table: &str,
        has_soft_delete: bool,
        soft_delete_col: &str,
    ) -> (String, Vec<BindValue>) {
        let table_name = self.from_sql.as_deref().unwrap_or(table);
        let (where_sql, base_binds) = compile_predicates(
            table_name,
            &self.filters,
            &self.existence_relations,
            has_soft_delete,
            soft_delete_col,
            self.with_deleted,
            self.only_deleted,
            1,
        );

        let (join_sql, join_binds) = compile_joins(&self.joins, base_binds.len() + 1);
        let from_clause = if join_sql.is_empty() {
            format!("FROM {}", table_name)
        } else {
            format!("FROM {} {}", table_name, join_sql.join(" "))
        };
        let where_clause = if where_sql.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", where_sql.join(" AND "))
        };
        let count_expr = self
            .count_sql
            .clone()
            .unwrap_or_else(|| "COUNT(*)".to_string());

        let sql = if self.distinct || self.distinct_on.is_some() {
            let select_clause = Self::build_select_clause(
                self.distinct,
                self.distinct_on.as_deref(),
                &self.selects,
            );
            format!(
                "SELECT COUNT(*) FROM (SELECT {} {}{}) AS sub",
                select_clause, from_clause, where_clause
            )
        } else {
            format!("SELECT {} {}{}", count_expr, from_clause, where_clause)
        };

        let mut all_binds = base_binds;
        all_binds.extend(join_binds);
        (sql, all_binds)
    }

    /// Assemble a DELETE statement.
    pub fn to_delete_sql(
        &self,
        table: &str,
        has_soft_delete: bool,
        soft_delete_col: &str,
    ) -> (String, Vec<BindValue>) {
        let (where_sql, binds) = compile_predicates(
            table,
            &self.filters,
            &self.existence_relations,
            has_soft_delete,
            soft_delete_col,
            self.with_deleted,
            self.only_deleted,
            1,
        );

        let sql = if where_sql.is_empty() {
            format!("DELETE FROM {}", table)
        } else {
            format!("DELETE FROM {} WHERE {}", table, where_sql.join(" AND "))
        };
        (sql, binds)
    }

    pub fn predicate_parts(
        &self,
        table: &str,
        has_soft_delete: bool,
        soft_delete_col: &str,
    ) -> (Vec<String>, Vec<BindValue>) {
        compile_predicates(
            self.from_sql.as_deref().unwrap_or(table),
            &self.filters,
            &self.existence_relations,
            has_soft_delete,
            soft_delete_col,
            self.with_deleted,
            self.only_deleted,
            1,
        )
    }

    /// Extract WHERE parts for reuse.
    pub fn into_where_parts(self) -> (Vec<String>, Vec<BindValue>) {
        let table = self.from_sql.unwrap_or_default();
        compile_predicates(
            table.as_str(),
            &self.filters,
            &self.existence_relations,
            false,
            "",
            self.with_deleted,
            self.only_deleted,
            1,
        )
    }

    // ── Aggregate execution ───────────────────────────────────────────

    pub async fn aggregate_scalar(
        self,
        db: DbConn<'db>,
        agg_expr: &str,
        table: &str,
        has_soft_delete: bool,
        soft_delete_col: &str,
    ) -> Result<Option<f64>> {
        use crate::common::sql::{bind_scalar, PgQueryScalar};

        let table_name = self.from_sql.as_deref().unwrap_or(table);
        let (where_sql, binds) = compile_predicates(
            table_name,
            &self.filters,
            &self.existence_relations,
            has_soft_delete,
            soft_delete_col,
            self.with_deleted,
            self.only_deleted,
            1,
        );

        let (join_sql, join_binds) = compile_joins(&self.joins, binds.len() + 1);
        let from_clause = if join_sql.is_empty() {
            format!("FROM {}", table_name)
        } else {
            format!("FROM {} {}", table_name, join_sql.join(" "))
        };
        let where_clause = if where_sql.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", where_sql.join(" AND "))
        };
        let sql = format!("SELECT {} {}{}", agg_expr, from_clause, where_clause);

        let mut q: PgQueryScalar<'_, Option<f64>> = sqlx::query_scalar(&sql);
        for b in &binds {
            q = bind_scalar(q, b.clone());
        }
        for b in &join_binds {
            q = bind_scalar(q, b.clone());
        }
        let result = db.fetch_scalar(q).await?;
        Ok(result)
    }

    // ── Increment execution ───────────────────────────────────────────

    pub async fn execute_increment(
        self,
        db: DbConn<'db>,
        col_sql: &str,
        amount: BindValue,
        table: &str,
        has_soft_delete: bool,
        soft_delete_col: &str,
        has_updated_at: bool,
    ) -> Result<u64> {
        use crate::common::sql::{bind_query, renumber_placeholders};

        let (where_sql, binds) = compile_predicates(
            table,
            &self.filters,
            &self.existence_relations,
            has_soft_delete,
            soft_delete_col,
            self.with_deleted,
            self.only_deleted,
            1,
        );
        let where_binds = binds;

        let mut set_parts = format!("{} = {} + $1", col_sql, col_sql);

        if has_updated_at {
            set_parts.push_str(", updated_at = NOW()");
        }

        let mut sql = format!("UPDATE {} SET {}", table, set_parts);
        if !where_sql.is_empty() {
            let renumbered: Vec<String> = where_sql
                .iter()
                .map(|clause| renumber_placeholders(clause, 2))
                .collect();
            sql.push_str(" WHERE ");
            sql.push_str(&renumbered.join(" AND "));
        }

        let mut q = sqlx::query(&sql);
        q = bind_query(q, amount);
        for b in where_binds {
            q = bind_query(q, b);
        }
        let result = db.execute(q).await?;
        Ok(result.rows_affected())
    }

    // ── Restore (soft-delete undo) ────────────────────────────────────

    pub async fn execute_restore(
        self,
        db: DbConn<'db>,
        table: &str,
        soft_delete_col: &str,
        has_updated_at: bool,
    ) -> Result<u64> {
        use crate::common::sql::bind_query;

        if self.filters.is_empty() && self.existence_relations.is_empty() {
            anyhow::bail!("restore: no conditions set");
        }
        if self.limit.is_some() {
            anyhow::bail!("restore: does not support limit; add where clauses");
        }

        let (mut where_sql, mut where_binds) = compile_filters(&self.filters, 1);
        // Only restore records that ARE deleted
        where_sql.push(format!("{} IS NOT NULL", soft_delete_col));
        compile_existence_predicates(
            table,
            &mut where_sql,
            &mut where_binds,
            &self.existence_relations,
        );

        let mut set_clause = format!("{} = NULL", soft_delete_col);
        if has_updated_at {
            set_clause.push_str(", updated_at = NOW()");
        }

        let mut sql = format!("UPDATE {} SET {}", table, set_clause);
        if !where_sql.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&where_sql.join(" AND "));
        }

        let mut q = sqlx::query(&sql);
        for b in where_binds {
            q = bind_query(q, b);
        }
        let result = db.execute(q).await?;
        Ok(result.rows_affected())
    }
}

// ---------------------------------------------------------------------------
// CreateState — shared INSERT builder state (replaces per-model XxxCreateInner)
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct CreateState<'db> {
    pub base_url: Option<String>,
    pub table: &'static str,
    pub assignments: Vec<CreateAssignment>,
    pub conflict: Option<CreateConflictSpec>,
    pub translations: HashMap<&'static str, HashMap<String, String>>,
    pub meta: HashMap<String, serde_json::Value>,
    pub attachments_single: HashMap<&'static str, AttachmentInput>,
    pub attachments_multi: HashMap<&'static str, Vec<AttachmentInput>>,
    _scope: PhantomData<&'db ()>,
}

impl<'db> CreateState<'db> {
    pub fn new(base_url: Option<String>, table: &'static str) -> Self {
        Self {
            base_url,
            table,
            assignments: vec![],
            conflict: None,
            translations: HashMap::new(),
            meta: HashMap::new(),
            attachments_single: HashMap::new(),
            attachments_multi: HashMap::new(),
            _scope: PhantomData,
        }
    }

    pub fn set_col(mut self, col_sql: &'static str, val: BindValue) -> Self {
        self.assignments.push(CreateAssignment {
            col_sql,
            value: val,
        });
        self
    }

    pub fn on_conflict_do_nothing(mut self, cols: &[&'static str]) -> Self {
        self.conflict = Some(CreateConflictSpec {
            action: CreateConflictAction::DoNothing,
            cols: cols.to_vec(),
        });
        self
    }

    pub fn on_conflict_update(mut self, cols: &[&'static str]) -> Self {
        self.conflict = Some(CreateConflictSpec {
            action: CreateConflictAction::Update,
            cols: cols.to_vec(),
        });
        self
    }

    pub fn build_insert_sql(&self) -> (String, Vec<BindValue>) {
        let col_names: Vec<&'static str> = self
            .assignments
            .iter()
            .map(|assignment| assignment.col_sql)
            .collect();
        let binds: Vec<BindValue> = self
            .assignments
            .iter()
            .map(|assignment| assignment.value.clone())
            .collect();
        let placeholders: Vec<String> = (1..=binds.len()).map(|i| format!("${}", i)).collect();
        let mut sql = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            self.table,
            col_names.join(", "),
            placeholders.join(", ")
        );
        if let Some(conflict) = &self.conflict {
            if !conflict.cols.is_empty() {
                let action = match conflict.action {
                    CreateConflictAction::DoNothing => "DO NOTHING",
                    CreateConflictAction::Update => "DO UPDATE",
                };
                sql.push_str(&format!(
                    " ON CONFLICT ({}) {}",
                    conflict.cols.join(", "),
                    action
                ));
                if matches!(conflict.action, CreateConflictAction::Update) {
                    let set_clauses: Vec<String> = self
                        .assignments
                        .iter()
                        .zip(placeholders.iter())
                        .filter(|(assignment, _)| !conflict.cols.contains(&assignment.col_sql))
                        .map(|(assignment, ph)| format!("{} = {}", assignment.col_sql, ph))
                        .collect();
                    if !set_clauses.is_empty() {
                        sql.push_str(&format!(" SET {}", set_clauses.join(", ")));
                    }
                }
            }
        }
        sql.push_str(" RETURNING *");
        (sql, binds)
    }

    pub fn has_col(&self, col_sql: &'static str) -> bool {
        self.assignments
            .iter()
            .any(|assignment| assignment.col_sql == col_sql)
    }

    pub fn set_translation(
        mut self,
        field: &'static str,
        locale: String,
        value: String,
    ) -> Self {
        self.translations
            .entry(field)
            .or_default()
            .insert(locale, value);
        self
    }

    pub fn insert_meta_value(
        mut self,
        key: impl Into<String>,
        value: serde_json::Value,
    ) -> Self {
        self.meta.insert(key.into(), value);
        self
    }

    pub fn set_attachment_single(mut self, field: &'static str, input: AttachmentInput) -> Self {
        self.attachments_single.insert(field, input);
        self
    }

    pub fn add_attachment_multi(mut self, field: &'static str, input: AttachmentInput) -> Self {
        self.attachments_multi.entry(field).or_default().push(input);
        self
    }
}

// ---------------------------------------------------------------------------
// PatchState — shared UPDATE builder state (replaces per-model XxxPatchInner)
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct PatchSelection<'db> {
    pub pk_col: &'static str,
    pub source: QueryState<'db>,
}

pub struct PatchState<'db> {
    pub base_url: Option<String>,
    pub table: &'static str,
    pub has_soft_delete: bool,
    pub soft_delete_col: &'static str,
    pub assignments: Vec<PatchAssignment>,
    pub selection: Option<PatchSelection<'db>>,
    pub deferred_error: Option<String>,
    pub filters: Vec<FilterExpr>,
    pub with_deleted: bool,
    pub only_deleted: bool,
    pub translations: HashMap<&'static str, HashMap<String, String>>,
    pub meta: HashMap<String, serde_json::Value>,
    pub attachments_single: HashMap<&'static str, AttachmentInput>,
    pub attachments_multi: HashMap<&'static str, Vec<AttachmentInput>>,
    pub attachments_clear_single: Vec<&'static str>,
    pub attachments_delete_multi: HashMap<&'static str, Vec<Uuid>>,
    _scope: PhantomData<&'db ()>,
}

impl<'db> PatchState<'db> {
    pub fn new(
        base_url: Option<String>,
        table: &'static str,
        has_soft_delete: bool,
        soft_delete_col: &'static str,
    ) -> Self {
        Self {
            base_url,
            table,
            has_soft_delete,
            soft_delete_col,
            assignments: vec![],
            selection: None,
            deferred_error: None,
            filters: vec![],
            with_deleted: false,
            only_deleted: false,
            translations: HashMap::new(),
            meta: HashMap::new(),
            attachments_single: HashMap::new(),
            attachments_multi: HashMap::new(),
            attachments_clear_single: Vec::new(),
            attachments_delete_multi: HashMap::new(),
            _scope: PhantomData,
        }
    }

    pub fn assign_col(mut self, col_sql: &'static str, val: BindValue) -> Self {
        self.assignments.push(PatchAssignment {
            col_sql,
            value: val,
            mode: SetMode::Assign,
        });
        self
    }

    pub fn increment_col(mut self, col_sql: &'static str, val: BindValue) -> Self {
        self.assignments.push(PatchAssignment {
            col_sql,
            value: val,
            mode: SetMode::Increment,
        });
        self
    }

    pub fn decrement_col(mut self, col_sql: &'static str, val: BindValue) -> Self {
        self.assignments.push(PatchAssignment {
            col_sql,
            value: val,
            mode: SetMode::Decrement,
        });
        self
    }

    pub fn where_col_str(mut self, col_sql: &str, op: Op, val: BindValue) -> Self {
        if val.is_null() {
            return match op {
                Op::Eq => self.where_null_str(col_sql),
                Op::Ne => self.where_not_null_str(col_sql),
                _ => self,
            };
        }
        self.filters.push(FilterExpr::Comparison {
            col_sql: col_sql.to_string(),
            op,
            value: val,
        });
        self
    }

    pub fn where_null_str(mut self, col_sql: &str) -> Self {
        self.filters.push(FilterExpr::Null {
            col_sql: col_sql.to_string(),
            negated: false,
        });
        self
    }

    pub fn where_not_null_str(mut self, col_sql: &str) -> Self {
        self.filters.push(FilterExpr::Null {
            col_sql: col_sql.to_string(),
            negated: true,
        });
        self
    }

    pub fn where_raw(mut self, clause: String, raw_binds: Vec<BindValue>) -> Self {
        let mut clause = clause;
        let mut idx = 1usize;
        while let Some(pos) = clause.find('?') {
            let ph = format!("${idx}");
            clause.replace_range(pos..pos + 1, &ph);
            idx += 1;
        }
        self.filters.push(FilterExpr::Raw {
            clause,
            binds: raw_binds,
        });
        self
    }

    pub fn with_deleted(mut self) -> Self {
        self.with_deleted = true;
        self
    }

    pub fn only_deleted(mut self) -> Self {
        self.only_deleted = true;
        self
    }

    pub fn from_selected_query(
        qs: QueryState<'db>,
        table: &'static str,
        has_soft_delete: bool,
        soft_delete_col: &'static str,
        pk_col: &'static str,
        claim_mode: bool,
    ) -> Self {
        let base_url = qs.base_url.clone();
        let with_deleted = qs.with_deleted;
        let only_deleted = qs.only_deleted;
        let mut deferred_error = qs.deferred_error.clone();
        if claim_mode {
            match qs.lock_clause {
                Some(LockClause {
                    mode: LockMode::Update | LockMode::NoKeyUpdate,
                    ..
                }) => {}
                Some(_) => {
                    deferred_error = Some(
                        "claim() requires for_update() or for_no_key_update(), not a shared lock"
                            .to_string(),
                    );
                }
                None => {
                    deferred_error =
                        Some("claim() requires a lock mode such as for_update()".to_string());
                }
            }
        }
        Self {
            base_url,
            table,
            has_soft_delete,
            soft_delete_col,
            assignments: vec![],
            selection: Some(PatchSelection { pk_col, source: qs }),
            deferred_error,
            filters: vec![],
            with_deleted,
            only_deleted,
            translations: HashMap::new(),
            meta: HashMap::new(),
            attachments_single: HashMap::new(),
            attachments_multi: HashMap::new(),
            attachments_clear_single: Vec::new(),
            attachments_delete_multi: HashMap::new(),
            _scope: PhantomData,
        }
    }

    pub fn has_conditions(&self) -> bool {
        self.selection.is_some() || !self.filters.is_empty()
    }

    fn compile_direct_predicates(&self, bind_start: usize) -> (Vec<String>, Vec<BindValue>) {
        compile_predicates(
            self.table,
            &self.filters,
            &[],
            self.has_soft_delete,
            self.soft_delete_col,
            self.with_deleted,
            self.only_deleted,
            bind_start,
        )
    }

    fn compile_selection_clause(&self, bind_start: usize) -> (Option<String>, Vec<BindValue>) {
        use crate::common::sql::renumber_placeholders;

        let Some(selection) = &self.selection else {
            return (None, Vec::new());
        };

        let mut query = selection.source.clone().select_only_str(selection.pk_col);
        query.with_deleted = self.with_deleted;
        query.only_deleted = self.only_deleted;
        let (sql, binds) =
            query.to_select_sql(self.table, self.has_soft_delete, self.soft_delete_col);
        let sql = renumber_placeholders(&sql, bind_start);
        (Some(format!("{} IN ({sql})", selection.pk_col)), binds)
    }

    pub fn build_target_ids_select_sql(&self, pk_col: &str) -> (String, Vec<BindValue>) {
        let mut clauses = Vec::new();
        let mut binds = Vec::new();
        let mut current_offset = 0usize;

        let (selection_clause, selection_binds) = self.compile_selection_clause(current_offset + 1);
        if let Some(clause) = selection_clause {
            current_offset += selection_binds.len();
            binds.extend(selection_binds);
            clauses.push(clause);
        }

        let (direct_clauses, direct_binds) = self.compile_direct_predicates(current_offset + 1);
        if !direct_clauses.is_empty() {
            binds.extend(direct_binds);
            clauses.extend(direct_clauses);
        }

        let mut sql = format!("SELECT {pk_col} FROM {}", self.table);
        if !clauses.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&clauses.join(" AND "));
        }
        (sql, binds)
    }

    pub fn build_update_sql(&self) -> (String, Vec<BindValue>) {
        let mut parts: Vec<String> = Vec::new();
        for (i, assignment) in self.assignments.iter().enumerate() {
            let part = match assignment.mode {
                SetMode::Assign => format!("{} = ${}", assignment.col_sql, i + 1),
                SetMode::Increment => {
                    format!(
                        "{} = {} + ${}",
                        assignment.col_sql,
                        assignment.col_sql,
                        i + 1
                    )
                }
                SetMode::Decrement => {
                    format!(
                        "{} = {} - ${}",
                        assignment.col_sql,
                        assignment.col_sql,
                        i + 1
                    )
                }
            };
            parts.push(part);
        }
        let mut current_offset = parts.len();
        let mut where_clauses: Vec<String> = Vec::new();

        let (selection_clause, selection_binds) = self.compile_selection_clause(current_offset + 1);
        if let Some(clause) = selection_clause {
            current_offset += selection_binds.len();
            where_clauses.push(clause);
        }

        let mut sql = format!("UPDATE {} SET {}", self.table, parts.join(", "));
        let (direct_clauses, direct_binds) = self.compile_direct_predicates(current_offset + 1);
        where_clauses.extend(direct_clauses);
        if !where_clauses.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&where_clauses.join(" AND "));
        }

        let mut all_binds: Vec<BindValue> = self
            .assignments
            .iter()
            .map(|assignment| assignment.value.clone())
            .collect();
        all_binds.extend(selection_binds);
        all_binds.extend(direct_binds);
        (sql, all_binds)
    }

    pub fn has_assignment(&self, col_sql: &'static str) -> bool {
        self.assignments
            .iter()
            .any(|assignment| assignment.col_sql == col_sql)
    }

    pub fn set_translation(
        mut self,
        field: &'static str,
        locale: String,
        value: String,
    ) -> Self {
        self.translations
            .entry(field)
            .or_default()
            .insert(locale, value);
        self
    }

    pub fn insert_meta_value(
        mut self,
        key: impl Into<String>,
        value: serde_json::Value,
    ) -> Self {
        self.meta.insert(key.into(), value);
        self
    }

    pub fn set_attachment_single(mut self, field: &'static str, input: AttachmentInput) -> Self {
        self.attachments_single.insert(field, input);
        self
    }

    pub fn add_attachment_multi(mut self, field: &'static str, input: AttachmentInput) -> Self {
        self.attachments_multi.entry(field).or_default().push(input);
        self
    }

    pub fn clear_attachment_single(mut self, field: &'static str) -> Self {
        if !self.attachments_clear_single.contains(&field) {
            self.attachments_clear_single.push(field);
        }
        self
    }

    pub fn delete_attachment_multi_ids(
        mut self,
        field: &'static str,
        ids: impl IntoIterator<Item = Uuid>,
    ) -> Self {
        self.attachments_delete_multi
            .entry(field)
            .or_default()
            .extend(ids);
        self
    }

    pub fn build_update_sql_returning(&self, returning: &ReturningSpec) -> (String, Vec<BindValue>) {
        let (mut sql, binds) = self.build_update_sql();
        sql.push_str(" RETURNING ");
        sql.push_str(&render_returning_spec(returning));
        (sql, binds)
    }

    pub async fn execute_returning_scalar<'q, T>(
        &self,
        db: DbConn<'db>,
        returning: &ReturningSpec,
    ) -> Result<Vec<T>>
    where
        T: Send
            + Unpin
            + 'static
            + for<'r> sqlx::Decode<'r, sqlx::Postgres>
            + sqlx::Type<sqlx::Postgres>,
    {
        use crate::common::sql::bind_scalar;
        let (sql, binds) = self.build_update_sql_returning(returning);
        let mut query = sqlx::query_scalar::<_, T>(&sql);
        for bind in binds {
            query = bind_scalar(query, bind);
        }
        Ok(db.fetch_all_scalar(query).await?)
    }

    pub async fn execute_returning_json(
        &self,
        db: DbConn<'db>,
        returning: &ReturningSpec,
    ) -> Result<Vec<serde_json::Value>> {
        use crate::common::sql::bind_scalar;
        let (sql, binds) = self.build_update_sql_returning(returning);
        let mut query = sqlx::query_scalar::<_, serde_json::Value>(&sql);
        for bind in binds {
            query = bind_scalar(query, bind);
        }
        Ok(db.fetch_all_scalar(query).await?)
    }
}

// ---------------------------------------------------------------------------
// Blanket impls for Column<M, T>
// ---------------------------------------------------------------------------

impl<M, T> QueryField<M> for Column<M, T>
where
    M: QueryModel,
    T: Clone + Into<BindValue>,
{
    type Value = T;

    fn where_col<'db>(
        field: Self,
        state: QueryState<'db>,
        op: Op,
        value: Self::Value,
    ) -> QueryState<'db> {
        state.where_col_str(field.as_sql(), op, value.into())
    }

    fn or_where_col<'db>(
        field: Self,
        state: QueryState<'db>,
        op: Op,
        value: Self::Value,
    ) -> QueryState<'db> {
        state.or_where_col_str(field.as_sql(), op, value.into())
    }

    fn where_in<'db>(
        _field: Self,
        state: QueryState<'db>,
        values: &[Self::Value],
    ) -> QueryState<'db> {
        let bind_values: Vec<BindValue> = values.iter().map(|v| v.clone().into()).collect();
        state.where_in_str(_field.as_sql(), &bind_values)
    }

    fn order_by<'db>(field: Self, state: QueryState<'db>, dir: OrderDir) -> QueryState<'db> {
        state.order_by_str(field.as_sql(), dir)
    }

    fn where_null<'db>(field: Self, state: QueryState<'db>) -> QueryState<'db> {
        state.where_null_str(field.as_sql())
    }

    fn where_not_null<'db>(field: Self, state: QueryState<'db>) -> QueryState<'db> {
        state.where_not_null_str(field.as_sql())
    }
}

impl<M, T> CreateField<M> for Column<M, T>
where
    M: CreateModel,
    T: Into<BindValue>,
{
    type Value = T;

    fn set<'db>(
        field: Self,
        state: CreateState<'db>,
        value: Self::Value,
    ) -> Result<CreateState<'db>> {
        let value = M::transform_create_value(field.as_sql(), value.into())?;
        Ok(state.set_col(field.as_sql(), value))
    }
}

impl<M, T> CreateConflictField<M> for Column<M, T>
where
    M: CreateModel,
{
    fn on_conflict_do_nothing<'db>(state: CreateState<'db>, fields: &[Self]) -> CreateState<'db> {
        let cols: Vec<&'static str> = fields.iter().map(|f| f.as_sql()).collect();
        state.on_conflict_do_nothing(&cols)
    }

    fn on_conflict_update<'db>(state: CreateState<'db>, fields: &[Self]) -> CreateState<'db> {
        let cols: Vec<&'static str> = fields.iter().map(|f| f.as_sql()).collect();
        state.on_conflict_update(&cols)
    }
}

impl<M, T> PatchAssignField<M> for Column<M, T>
where
    M: PatchModel,
    T: Into<BindValue>,
{
    type Value = T;

    fn assign<'db>(
        field: Self,
        state: PatchState<'db>,
        value: Self::Value,
    ) -> Result<PatchState<'db>> {
        let value = M::transform_patch_value(field.as_sql(), value.into())?;
        Ok(state.assign_col(field.as_sql(), value))
    }
}

macro_rules! impl_patch_numeric_field {
    ($($ty:ty),*) => {
        $(
            impl<M> PatchNumericField<M> for Column<M, $ty>
            where
                M: PatchModel,
            {
                fn increment<'db>(
                    field: Self,
                    state: PatchState<'db>,
                    value: Self::Value,
                ) -> Result<PatchState<'db>> {
                    Ok(state.increment_col(field.as_sql(), value.into()))
                }

                fn decrement<'db>(
                    field: Self,
                    state: PatchState<'db>,
                    value: Self::Value,
                ) -> Result<PatchState<'db>> {
                    Ok(state.decrement_col(field.as_sql(), value.into()))
                }
            }
        )*
    };
}

impl_patch_numeric_field!(i16, i32, i64, f64, rust_decimal::Decimal);

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    struct FakeModel;
    struct SoftDeleteFakeModel;
    struct ChildModel;
    struct ChunkIterModel;
    #[derive(Debug, Clone, PartialEq, sqlx::FromRow)]
    struct ChunkRow {
        id: i64,
    }
    #[derive(Debug, Clone, PartialEq)]
    struct ChunkRecord {
        id: i64,
        relation_counts: HashMap<String, i64>,
        relation_aggregates: HashMap<String, f64>,
    }
    const STATUS_COL: Column<FakeModel, String> = Column::new("status");
    const STATE_COL: Column<FakeModel, String> = Column::new("processing_status");
    const CREATED_AT_COL: Column<FakeModel, time::OffsetDateTime> = Column::new("created_at");
    const ATTEMPTS_COL: Column<FakeModel, i64> = Column::new("send_attempt_count");
    const MAX_ATTEMPTS_COL: Column<FakeModel, i64> = Column::new("max_send_attempts");
    const FAILED_AT_COL: Column<FakeModel, time::OffsetDateTime> = Column::new("failed_at");
    const SD_STATUS_COL: Column<SoftDeleteFakeModel, String> = Column::new("status");
    const SD_STATE_COL: Column<SoftDeleteFakeModel, String> = Column::new("processing_status");
    const SD_CREATED_AT_COL: Column<SoftDeleteFakeModel, time::OffsetDateTime> =
        Column::new("created_at");
    const CHILD_STATUS_COL: Column<ChildModel, String> = Column::new("status");
    const DOWNLINES_REL: ManyRelation<SoftDeleteFakeModel, (), 0> =
        ManyRelation::new_with_soft_delete("children", "child_rows", "id", "parent_id");

    impl ModelDef for FakeModel {
        type Pk = i64;
        type Record = ();
        type Create = ();
        type Changes = ();

        const TABLE: &'static str = "fake_models";
        const MODEL_KEY: &'static str = "fake_model";
        const PK_COL: &'static str = "id";
    }

    impl QueryModel for FakeModel {
        const DEFAULT_SELECT: &'static str = "id";
        const HAS_SOFT_DELETE: bool = false;
        const SOFT_DELETE_COL: &'static str = "";
        const HAS_CREATED_AT: bool = false;
        const HAS_UPDATED_AT: bool = false;

        fn query_all<'db>(
            _: DbConn<'db>,
            _: QueryState<'db>,
        ) -> BoxModelFuture<'db, Vec<Self::Record>> {
            unreachable!()
        }

        fn query_first<'db>(
            _: DbConn<'db>,
            _: QueryState<'db>,
        ) -> BoxModelFuture<'db, Option<Self::Record>> {
            unreachable!()
        }

        fn query_find<'db>(
            _: DbConn<'db>,
            _: QueryState<'db>,
            _: Self::Pk,
        ) -> BoxModelFuture<'db, Option<Self::Record>> {
            unreachable!()
        }

        fn query_count<'db>(_: DbConn<'db>, _: QueryState<'db>) -> BoxModelFuture<'db, i64> {
            unreachable!()
        }

        fn query_delete<'db>(_: DbConn<'db>, _: QueryState<'db>) -> BoxModelFuture<'db, u64> {
            unreachable!()
        }

        fn query_paginate<'db>(
            _: DbConn<'db>,
            _: QueryState<'db>,
            _: i64,
            _: i64,
        ) -> BoxModelFuture<'db, Page<Self::Record>> {
            unreachable!()
        }
    }

    impl ModelDef for SoftDeleteFakeModel {
        type Pk = i64;
        type Record = ();
        type Create = ();
        type Changes = ();

        const TABLE: &'static str = "soft_delete_fake_models";
        const MODEL_KEY: &'static str = "soft_delete_fake_model";
        const PK_COL: &'static str = "id";
    }

    impl QueryModel for SoftDeleteFakeModel {
        const DEFAULT_SELECT: &'static str = "id";
        const HAS_SOFT_DELETE: bool = true;
        const SOFT_DELETE_COL: &'static str = "deleted_at";
        const HAS_CREATED_AT: bool = false;
        const HAS_UPDATED_AT: bool = false;

        fn query_all<'db>(
            _: DbConn<'db>,
            _: QueryState<'db>,
        ) -> BoxModelFuture<'db, Vec<Self::Record>> {
            unreachable!()
        }

        fn query_first<'db>(
            _: DbConn<'db>,
            _: QueryState<'db>,
        ) -> BoxModelFuture<'db, Option<Self::Record>> {
            unreachable!()
        }

        fn query_find<'db>(
            _: DbConn<'db>,
            _: QueryState<'db>,
            _: Self::Pk,
        ) -> BoxModelFuture<'db, Option<Self::Record>> {
            unreachable!()
        }

        fn query_count<'db>(_: DbConn<'db>, _: QueryState<'db>) -> BoxModelFuture<'db, i64> {
            unreachable!()
        }

        fn query_delete<'db>(_: DbConn<'db>, _: QueryState<'db>) -> BoxModelFuture<'db, u64> {
            unreachable!()
        }

        fn query_paginate<'db>(
            _: DbConn<'db>,
            _: QueryState<'db>,
            _: i64,
            _: i64,
        ) -> BoxModelFuture<'db, Page<Self::Record>> {
            unreachable!()
        }
    }

    impl ModelDef for ChildModel {
        type Pk = i64;
        type Record = ();
        type Create = ();
        type Changes = ();

        const TABLE: &'static str = "child_rows";
        const MODEL_KEY: &'static str = "child_model";
        const PK_COL: &'static str = "id";
    }

    impl QueryModel for ChildModel {
        const DEFAULT_SELECT: &'static str = "id";
        const HAS_SOFT_DELETE: bool = true;
        const SOFT_DELETE_COL: &'static str = "deleted_at";
        const HAS_CREATED_AT: bool = false;
        const HAS_UPDATED_AT: bool = false;

        fn query_all<'db>(
            _: DbConn<'db>,
            _: QueryState<'db>,
        ) -> BoxModelFuture<'db, Vec<Self::Record>> {
            unreachable!()
        }

        fn query_first<'db>(
            _: DbConn<'db>,
            _: QueryState<'db>,
        ) -> BoxModelFuture<'db, Option<Self::Record>> {
            unreachable!()
        }

        fn query_find<'db>(
            _: DbConn<'db>,
            _: QueryState<'db>,
            _: Self::Pk,
        ) -> BoxModelFuture<'db, Option<Self::Record>> {
            unreachable!()
        }

        fn query_count<'db>(_: DbConn<'db>, _: QueryState<'db>) -> BoxModelFuture<'db, i64> {
            unreachable!()
        }

        fn query_delete<'db>(_: DbConn<'db>, _: QueryState<'db>) -> BoxModelFuture<'db, u64> {
            unreachable!()
        }

        fn query_paginate<'db>(
            _: DbConn<'db>,
            _: QueryState<'db>,
            _: i64,
            _: i64,
        ) -> BoxModelFuture<'db, Page<Self::Record>> {
            unreachable!()
        }
    }

    impl ModelDef for ChunkIterModel {
        type Pk = i64;
        type Record = ChunkRecord;
        type Create = ();
        type Changes = ();

        const TABLE: &'static str = "chunk_iter_models";
        const MODEL_KEY: &'static str = "chunk_iter_model";
        const PK_COL: &'static str = "id";
    }

    impl QueryModel for ChunkIterModel {
        const DEFAULT_SELECT: &'static str = "id";
        const HAS_SOFT_DELETE: bool = false;
        const SOFT_DELETE_COL: &'static str = "";
        const HAS_CREATED_AT: bool = false;
        const HAS_UPDATED_AT: bool = false;

        fn query_all<'db>(
            _: DbConn<'db>,
            state: QueryState<'db>,
        ) -> BoxModelFuture<'db, Vec<Self::Record>> {
            Box::pin(async move {
                let mut rows = vec![
                    ChunkRecord {
                        id: 1,
                        relation_counts: HashMap::new(),
                        relation_aggregates: HashMap::new(),
                    },
                    ChunkRecord {
                        id: 2,
                        relation_counts: HashMap::new(),
                        relation_aggregates: HashMap::new(),
                    },
                    ChunkRecord {
                        id: 3,
                        relation_counts: HashMap::new(),
                        relation_aggregates: HashMap::new(),
                    },
                    ChunkRecord {
                        id: 4,
                        relation_counts: HashMap::new(),
                        relation_aggregates: HashMap::new(),
                    },
                    ChunkRecord {
                        id: 5,
                        relation_counts: HashMap::new(),
                        relation_aggregates: HashMap::new(),
                    },
                ];
                let mut min_id: Option<i64> = None;
                for filter in &state.filters {
                    if let FilterExpr::Comparison {
                        col_sql,
                        op: Op::Gt,
                        value: BindValue::I64(value),
                    } = filter
                    {
                        if col_sql == "id" {
                            min_id = Some(*value);
                        }
                    }
                }
                if let Some(min_id) = min_id {
                    rows.retain(|row| row.id > min_id);
                }
                if let Some(limit) = state.limit {
                    rows.truncate(limit as usize);
                }
                Ok(rows)
            })
        }

        fn query_first<'db>(
            _: DbConn<'db>,
            _: QueryState<'db>,
        ) -> BoxModelFuture<'db, Option<Self::Record>> {
            unreachable!()
        }

        fn query_find<'db>(
            _: DbConn<'db>,
            _: QueryState<'db>,
            _: Self::Pk,
        ) -> BoxModelFuture<'db, Option<Self::Record>> {
            unreachable!()
        }

        fn query_count<'db>(_: DbConn<'db>, _: QueryState<'db>) -> BoxModelFuture<'db, i64> {
            unreachable!()
        }

        fn query_delete<'db>(_: DbConn<'db>, _: QueryState<'db>) -> BoxModelFuture<'db, u64> {
            unreachable!()
        }

        fn query_paginate<'db>(
            _: DbConn<'db>,
            _: QueryState<'db>,
            _: i64,
            _: i64,
        ) -> BoxModelFuture<'db, Page<Self::Record>> {
            unreachable!()
        }
    }

    impl RuntimeModel for ChunkIterModel {
        type Row = ChunkRow;

        fn hydrate_records<'db>(
            _: DbConn<'db>,
            rows: Vec<Self::Row>,
            _: Option<String>,
        ) -> BoxModelFuture<'db, Vec<Self::Record>> {
            Box::pin(async move {
                Ok(rows
                    .into_iter()
                    .map(|row| ChunkRecord {
                        id: row.id,
                        relation_counts: HashMap::new(),
                        relation_aggregates: HashMap::new(),
                    })
                    .collect())
            })
        }
    }

    impl ChunkModel for ChunkIterModel {
        fn record_pk(record: &Self::Record) -> Self::Pk {
            record.id
        }
    }

    impl FeaturePersistenceModel for FakeModel {}
    impl FeaturePersistenceModel for SoftDeleteFakeModel {}
    impl FeaturePersistenceModel for ChildModel {}
    impl FeaturePersistenceModel for ChunkIterModel {}

    impl RelationMetricRecord for ChunkRecord {
        fn relation_counts(&self) -> &HashMap<String, i64> {
            &self.relation_counts
        }

        fn relation_aggregates(&self) -> &HashMap<String, f64> {
            &self.relation_aggregates
        }

        fn relation_counts_mut(&mut self) -> &mut HashMap<String, i64> {
            &mut self.relation_counts
        }

        fn relation_aggregates_mut(&mut self) -> &mut HashMap<String, f64> {
            &mut self.relation_aggregates
        }
    }

    impl CreateModel for FakeModel {
        fn create_save<'db>(
            _: DbConn<'db>,
            _: CreateState<'db>,
        ) -> BoxModelFuture<'db, Self::Record> {
            unreachable!()
        }

        fn transform_create_value(_: &str, value: BindValue) -> Result<BindValue> {
            Ok(value)
        }
    }

    impl PatchModel for FakeModel {
        fn patch_from_query<'db>(_: QueryState<'db>) -> PatchState<'db> {
            unreachable!()
        }

        fn patch_save<'db>(_: DbConn<'db>, _: PatchState<'db>) -> BoxModelFuture<'db, u64> {
            unreachable!()
        }

        fn patch_fetch<'db>(
            _: DbConn<'db>,
            _: PatchState<'db>,
        ) -> BoxModelFuture<'db, Vec<Self::Record>> {
            unreachable!()
        }

        fn patch_fetch_returning_all<'db>(
            _: DbConn<'db>,
            _: PatchState<'db>,
        ) -> BoxModelFuture<'db, Vec<Self::Record>> {
            unreachable!()
        }

        fn transform_patch_value(_: &str, value: BindValue) -> Result<BindValue> {
            Ok(value)
        }
    }

    impl PatchModel for SoftDeleteFakeModel {
        fn patch_from_query<'db>(state: QueryState<'db>) -> PatchState<'db> {
            PatchState::from_selected_query(
                state,
                Self::TABLE,
                Self::HAS_SOFT_DELETE,
                Self::SOFT_DELETE_COL,
                Self::PK_COL,
                false,
            )
        }

        fn patch_save<'db>(_: DbConn<'db>, _: PatchState<'db>) -> BoxModelFuture<'db, u64> {
            unreachable!()
        }

        fn patch_fetch<'db>(
            _: DbConn<'db>,
            _: PatchState<'db>,
        ) -> BoxModelFuture<'db, Vec<Self::Record>> {
            unreachable!()
        }

        fn patch_fetch_returning_all<'db>(
            _: DbConn<'db>,
            _: PatchState<'db>,
        ) -> BoxModelFuture<'db, Vec<Self::Record>> {
            unreachable!()
        }

        fn transform_patch_value(_: &str, value: BindValue) -> Result<BindValue> {
            Ok(value)
        }
    }

    impl IncludeRelation<SoftDeleteFakeModel> for ManyRelation<SoftDeleteFakeModel, (), 0> {
        fn load_spec<'db>(_relation: Self, _base_url: Option<String>) -> WithRelationSpec {
            WithRelationSpec {
                name: "children",
                kind: "has_many",
                target_table: "child_rows",
                target_pk: "id",
                foreign_key: "parent_id",
                local_key: "id",
                has_soft_delete: true,
                selects: vec![],
                filters: vec![],
                orders: vec![],
                limit: None,
                offset: None,
                with_deleted: false,
                only_deleted: false,
                nested: vec![],
                counts: vec![],
                aggregates: vec![],
            }
        }
    }

    impl WhereHasRelation<SoftDeleteFakeModel> for ManyRelation<SoftDeleteFakeModel, (), 0> {
        type Target = ChildModel;

        fn where_has<'db, F>(relation: Self, state: QueryState<'db>, scope: F) -> QueryState<'db>
        where
            F: FnOnce(Query<'db, Self::Target>) -> Query<'db, Self::Target>,
        {
            let scoped = scope(Query::<ChildModel>::new());
            let relation_spec = <Self as IncludeRelation<SoftDeleteFakeModel>>::load_spec(
                relation,
                state.base_url.clone(),
            );
            match relation_exists_from_query_state(
                relation_spec,
                scoped.into_inner(),
                <ChildModel as QueryModel>::DEFAULT_SELECT,
            ) {
                Ok(node) => state.push_relation_exists(ExistenceBoolean::And, node),
                Err(err) => state.defer_error(err.to_string()),
            }
        }

        fn or_where_has<'db, F>(relation: Self, state: QueryState<'db>, scope: F) -> QueryState<'db>
        where
            F: FnOnce(Query<'db, Self::Target>) -> Query<'db, Self::Target>,
        {
            let scoped = scope(Query::<ChildModel>::new());
            let relation_spec = <Self as IncludeRelation<SoftDeleteFakeModel>>::load_spec(
                relation,
                state.base_url.clone(),
            );
            match relation_exists_from_query_state(
                relation_spec,
                scoped.into_inner(),
                <ChildModel as QueryModel>::DEFAULT_SELECT,
            ) {
                Ok(node) => state.push_relation_exists(ExistenceBoolean::Or, node),
                Err(err) => state.defer_error(err.to_string()),
            }
        }
    }

    #[tokio::test]
    async fn query_create_and_patch_inherit_default_attachment_base_url() {
        set_default_attachment_base_url(Some("https://cdn.example.com/media/".to_string()));

        let query = Query::<FakeModel>::new();
        assert_eq!(
            query.into_inner().base_url.as_deref(),
            Some("https://cdn.example.com/media")
        );

        let create = Create::<FakeModel>::new();
        assert_eq!(
            create.into_inner().base_url.as_deref(),
            Some("https://cdn.example.com/media")
        );

        let patch = Patch::<FakeModel>::new();
        assert_eq!(
            patch.into_inner().base_url.as_deref(),
            Some("https://cdn.example.com/media")
        );

        let query = Query::<FakeModel>::new_with_base_url(Some(
            "https://images.example.com/custom/".to_string(),
        ));
        assert_eq!(
            query.into_inner().base_url.as_deref(),
            Some("https://images.example.com/custom")
        );
    }

    #[test]
    fn query_lock_modifier_sql_compiles() {
        let state = Query::<FakeModel>::new()
            .for_update()
            .skip_locked()
            .into_inner();
        let (sql, _) = state.to_select_sql(FakeModel::TABLE, false, "");
        assert!(sql.ends_with("FOR UPDATE SKIP LOCKED"));

        let state = Query::<FakeModel>::new().for_share().no_wait().into_inner();
        let (sql, _) = state.to_select_sql(FakeModel::TABLE, false, "");
        assert!(sql.ends_with("FOR SHARE NOWAIT"));
    }

    #[test]
    fn query_lock_modifier_requires_lock_mode() {
        let state = Query::<FakeModel>::new().skip_locked().into_inner();
        assert_eq!(
            state.deferred_error.as_deref(),
            Some("skip_locked() requires a lock mode such as for_update()")
        );
    }

    #[test]
    fn query_column_and_expr_comparisons_compile() {
        let state = Query::<FakeModel>::new()
            .where_col_cmp(ATTEMPTS_COL, Op::Lt, MAX_ATTEMPTS_COL)
            .where_expr_cmp(
                FAILED_AT_COL,
                Op::Lt,
                Expr::now_minus(time::Duration::seconds(30)),
            )
            .into_inner();
        let (sql, binds) = state.to_select_sql(FakeModel::TABLE, false, "");
        assert!(sql.contains("send_attempt_count < max_send_attempts"));
        assert!(sql.contains("failed_at < NOW() - ($1::double precision * INTERVAL '1 second')"));
        assert_eq!(binds.len(), 1);
        match &binds[0] {
            BindValue::F64(value) => assert!((*value - 30.0).abs() < f64::EPSILON),
            other => panic!("unexpected bind {other:?}"),
        }
    }

    #[test]
    fn claim_update_sql_compiles_with_returning() {
        let patch = Query::<FakeModel>::new()
            .where_col(STATUS_COL, Op::Eq, "queued".to_string())
            .order_by(CREATED_AT_COL, OrderDir::Asc)
            .limit(50)
            .for_update()
            .skip_locked()
            .claim()
            .assign(STATE_COL, "processing".to_string())
            .expect("assign should succeed");

        let state = patch.into_inner();
        let (sql, binds) =
            state.build_update_sql_returning(&ReturningSpec::Scalar(ReturnExpr::Column("id")));
        assert!(sql.contains("UPDATE fake_models SET processing_status = $1"));
        assert!(sql.contains("id IN (SELECT id FROM fake_models WHERE status = $2 ORDER BY created_at ASC LIMIT 50 FOR UPDATE SKIP LOCKED)"));
        assert!(sql.ends_with("RETURNING id"));
        assert_eq!(binds.len(), 2);
    }

    #[test]
    fn patch_query_excludes_deleted_by_default() {
        let state = Query::<SoftDeleteFakeModel>::new()
            .where_col(SD_STATUS_COL, Op::Eq, "queued".to_string())
            .patch()
            .assign(SD_STATE_COL, "processing".to_string())
            .expect("assign should succeed")
            .into_inner();
        let (sql, _) = state.build_update_sql();
        assert!(sql.contains("UPDATE soft_delete_fake_models SET processing_status = $1"));
        assert!(sql.contains("id IN (SELECT id FROM soft_delete_fake_models WHERE status = $2 AND deleted_at IS NULL)"));
        assert!(sql.contains("AND deleted_at IS NULL"));
    }

    #[test]
    fn patch_query_with_deleted_preserves_soft_delete_mode() {
        let state = Query::<SoftDeleteFakeModel>::new()
            .with_deleted()
            .where_col(SD_STATUS_COL, Op::Eq, "queued".to_string())
            .patch_selected()
            .assign(SD_STATE_COL, "processing".to_string())
            .expect("assign should succeed")
            .into_inner();
        let (sql, _) = state.build_update_sql();
        assert!(sql.contains("id IN (SELECT id FROM soft_delete_fake_models WHERE status = $2)"));
        assert!(!sql.contains("deleted_at IS NULL"));
        assert!(!sql.contains("deleted_at IS NOT NULL"));
    }

    #[test]
    fn patch_query_only_deleted_targets_only_deleted_rows() {
        let state = Query::<SoftDeleteFakeModel>::new()
            .only_deleted()
            .where_col(SD_STATUS_COL, Op::Eq, "queued".to_string())
            .patch()
            .assign(SD_STATE_COL, "processing".to_string())
            .expect("assign should succeed")
            .into_inner();
        let (sql, _) = state.build_update_sql();
        assert!(sql.contains("id IN (SELECT id FROM soft_delete_fake_models WHERE status = $2 AND deleted_at IS NOT NULL)"));
        assert!(sql.contains("AND deleted_at IS NOT NULL"));
    }

    #[test]
    fn claim_preserves_soft_delete_mode() {
        let state = Query::<SoftDeleteFakeModel>::new()
            .only_deleted()
            .where_col(SD_STATUS_COL, Op::Eq, "queued".to_string())
            .order_by(SD_CREATED_AT_COL, OrderDir::Asc)
            .limit(10)
            .for_update()
            .skip_locked()
            .claim()
            .assign(SD_STATE_COL, "processing".to_string())
            .expect("assign should succeed")
            .into_inner();
        let (sql, _) = state.build_update_sql();
        assert!(sql.contains("deleted_at IS NOT NULL"));
        assert!(sql.contains("FOR UPDATE SKIP LOCKED"));
    }

    #[test]
    fn patch_where_raw_compiles_with_soft_delete() {
        let state = Patch::<SoftDeleteFakeModel>::new()
            .where_raw("status = ?".to_string(), ["queued".to_string()])
            .assign(SD_STATE_COL, "processing".to_string())
            .expect("assign should succeed")
            .into_inner();
        let (sql, binds) = state.build_update_sql();
        assert!(
            sql.contains("UPDATE soft_delete_fake_models SET processing_status = $1 WHERE status = $2 AND deleted_at IS NULL"),
            "{sql}"
        );
        assert_eq!(binds.len(), 2);
    }

    #[test]
    fn patch_query_preserves_where_has_existence_tree() {
        let state = Query::<SoftDeleteFakeModel>::new()
            .where_has(DOWNLINES_REL, |query| {
                query.where_col(CHILD_STATUS_COL, Op::Eq, "queued".to_string())
            })
            .patch()
            .assign(SD_STATE_COL, "processing".to_string())
            .expect("assign should succeed")
            .into_inner();
        let (sql, _) = state.build_update_sql();
        assert!(sql.contains("EXISTS (SELECT 1 FROM child_rows AS __rf_rel_0"));
        assert!(sql.contains("__rf_rel_0.parent_id = soft_delete_fake_models.id"));
        assert!(sql.contains("status = $2"), "{sql}");
        assert!(sql.contains("deleted_at IS NULL"));
    }

    #[test]
    fn query_direct_raw_clause_helpers_compile_inside_typed_chain() {
        let where_clause = crate::common::sql::RawClause::new(
            "status = ? AND processing_status = ?",
            ["queued", "ready"],
        )
        .expect("valid raw clause");
        let join_on =
            crate::common::sql::RawClause::new("u.id = fake_models.id", Vec::<i32>::new())
                .expect("valid join clause");
        let join = crate::common::sql::RawJoinSpec::left("users u", join_on).expect("valid join");
        let select =
            crate::common::sql::RawSelectExpr::new("u.name AS user_name").expect("valid select");
        let order = crate::common::sql::RawOrderExpr::new("u.created_at DESC NULLS LAST")
            .expect("valid order");
        let group =
            crate::common::sql::RawGroupExpr::new("u.name").expect("valid group expression");

        let state = Query::<FakeModel>::new()
            .where_col(STATE_COL, Op::Eq, "processing".to_string())
            .where_raw(where_clause)
            .join_raw(join)
            .add_select_raw(select)
            .group_by_raw(group)
            .order_by_raw(order)
            .into_inner();

        let (sql, binds) = state.to_select_sql(FakeModel::TABLE, false, "");
        assert!(sql.contains("LEFT JOIN users u ON u.id = fake_models.id"));
        assert!(sql.contains("u.name AS user_name"));
        assert!(sql.contains("processing_status = $1"));
        assert!(sql.contains("status = $2 AND processing_status = $3"));
        assert!(sql.contains("GROUP BY u.name"));
        assert!(sql.contains("ORDER BY u.created_at DESC NULLS LAST"));
        assert_eq!(binds.len(), 3);
    }

    #[tokio::test]
    async fn chunk_by_id_iterates_using_primary_key_progression() {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .connect_lazy("postgres://postgres@localhost/chunk_test")
            .expect("lazy pool should build");
        let seen = Arc::new(Mutex::new(Vec::new()));
        let capture = Arc::clone(&seen);

        Query::<ChunkIterModel>::new()
            .chunk_by_id(DbConn::pool(&pool), 2, move |rows| {
                let capture = Arc::clone(&capture);
                async move {
                    capture
                        .lock()
                        .expect("chunk capture mutex poisoned")
                        .extend(rows.into_iter().map(|row| row.id));
                    Ok(true)
                }
            })
            .await
            .expect("chunk_by_id should succeed");

        assert_eq!(
            *seen.lock().expect("chunk capture mutex poisoned"),
            vec![1, 2, 3, 4, 5]
        );
    }

    #[tokio::test]
    async fn chunk_rejects_row_locks_and_chunk_by_id_requires_tx() {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .connect_lazy("postgres://postgres@localhost/chunk_test")
            .expect("lazy pool should build");

        let err = Query::<ChunkIterModel>::new()
            .for_update()
            .chunk(DbConn::pool(&pool), 2, |_rows| async { Ok(true) })
            .await
            .expect_err("chunk() should reject row locks");
        assert!(err
            .to_string()
            .contains("chunk() does not support row locks"));

        let err = Query::<ChunkIterModel>::new()
            .for_update()
            .chunk_by_id(DbConn::pool(&pool), 2, |_rows| async { Ok(true) })
            .await
            .expect_err("chunk_by_id() should require a transaction for row locks");
        assert!(err
            .to_string()
            .contains("chunk_by_id() with row locks requires a transaction"));
    }
}
