use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::{OnceLock, RwLock};

use crate::common::sql::{BindValue, DbConn, Op, OrderDir, SetMode};

pub type BoxModelFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T>> + Send + 'a>>;

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

/// Execute count queries for requested relations, returning name → (fk_value → count).
/// Called by generated `query_all`/`query_paginate` when `state.count_relations` is non-empty.
pub async fn execute_relation_counts(
    db: &DbConn<'_>,
    parent_ids: &[BindValue],
    specs: &[CountRelationSpec],
) -> Result<std::collections::HashMap<String, std::collections::HashMap<i64, i64>>> {
    use std::collections::HashMap;

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

        let (sql, extra_binds) = if let Some(nested) = &spec.nested {
            // Nested count: JOIN through intermediate table
            // SELECT t1.{fk}, COUNT(t2.{nested_target_pk}) as cnt
            // FROM {table} t1
            // JOIN {nested_table} t2 ON t2.{nested_fk} = t1.{target_pk}
            // WHERE t1.{fk} IN (...)
            // [AND t1.deleted_at IS NULL]
            // [AND t2.deleted_at IS NULL]
            // [AND t2.extra_where]
            // GROUP BY t1.{fk}
            let t1_soft = if spec.has_soft_delete { " AND t1.deleted_at IS NULL" } else { "" };
            let t2_soft = if nested.has_soft_delete { " AND t2.deleted_at IS NULL" } else { "" };

            let extra_clause = match &nested.extra_where {
                Some(clause) => {
                    let mut renumbered = clause.clone();
                    for (i, _) in nested.extra_binds.iter().enumerate().rev() {
                        let old = format!("${}", i + 1);
                        let new = format!("${}", bind_idx + i);
                        renumbered = renumbered.replace(&old, &new);
                    }
                    format!(" {renumbered}")
                }
                None => String::new(),
            };

            // The count key uses "parent_rel.child_rel" naming for __relation_counts
            let sql = format!(
                "SELECT t1.{fk}, COUNT(t2.{nested_pk}) as cnt \
                 FROM {table} t1 \
                 JOIN {nested_table} t2 ON t2.{nested_fk} = t1.{target_pk} \
                 WHERE t1.{fk} IN ({placeholders}){t1_soft}{t2_soft}{extra} \
                 GROUP BY t1.{fk}",
                fk = spec.foreign_key,
                table = spec.target_table,
                target_pk = spec.target_pk,
                nested_table = nested.target_table,
                nested_fk = nested.foreign_key,
                nested_pk = nested.target_pk,
                placeholders = placeholders.join(", "),
                t1_soft = t1_soft,
                t2_soft = t2_soft,
                extra = extra_clause,
            );
            (sql, &nested.extra_binds)
        } else {
            // Simple count: direct relation
            let soft_delete_clause = if spec.has_soft_delete {
                " AND deleted_at IS NULL"
            } else {
                ""
            };
            let extra_clause = match &spec.extra_where {
                Some(clause) => {
                    let mut renumbered = clause.clone();
                    for (i, _) in spec.extra_binds.iter().enumerate().rev() {
                        let old = format!("${}", i + 1);
                        let new = format!("${}", bind_idx + i);
                        renumbered = renumbered.replace(&old, &new);
                    }
                    format!(" {renumbered}")
                }
                None => String::new(),
            };
            let sql = format!(
                "SELECT {fk}, COUNT(*) as cnt FROM {table} WHERE {fk} IN ({placeholders}){soft_delete}{extra} GROUP BY {fk}",
                fk = spec.foreign_key,
                table = spec.target_table,
                placeholders = placeholders.join(", "),
                soft_delete = soft_delete_clause,
                extra = extra_clause,
            );
            (sql, &spec.extra_binds)
        };

        let mut q = sqlx::query_as::<_, (i64, i64)>(&sql);
        for id in parent_ids {
            q = crate::common::sql::bind(q, id.clone());
        }
        for bind in extra_binds {
            q = crate::common::sql::bind(q, bind.clone());
        }

        let rows: Vec<(i64, i64)> = db.fetch_all(q).await?;

        let mut counts: HashMap<i64, i64> = HashMap::new();
        for (fk_val, cnt) in rows {
            counts.insert(fk_val, cnt);
        }

        // For nested, use "parent.child" as the key
        let count_key = if spec.nested.is_some() {
            format!("{}.{}", spec.name, spec.nested.as_ref().unwrap().name)
        } else {
            spec.name.to_string()
        };
        result.insert(count_key, counts);
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

    fn query_all<'db>(state: QueryState<'db>) -> BoxModelFuture<'db, Vec<Self::Record>>;
    fn query_first<'db>(state: QueryState<'db>) -> BoxModelFuture<'db, Option<Self::Record>>;
    fn query_find<'db>(
        state: QueryState<'db>,
        id: Self::Pk,
    ) -> BoxModelFuture<'db, Option<Self::Record>>;
    fn query_count<'db>(state: QueryState<'db>) -> BoxModelFuture<'db, i64>;
    fn query_delete<'db>(state: QueryState<'db>) -> BoxModelFuture<'db, u64>;
    fn query_paginate<'db>(
        state: QueryState<'db>,
        page: i64,
        per_page: i64,
    ) -> BoxModelFuture<'db, Page<Self::Record>>;
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
    fn include<'db>(relation: Self, state: QueryState<'db>) -> QueryState<'db>;
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

pub trait CountRelation<M: ModelDef>: Copy {
    fn name(relation: Self) -> &'static str;
    fn target_table(relation: Self) -> &'static str;
    fn target_pk(relation: Self) -> &'static str;
    fn foreign_key(relation: Self) -> &'static str;
    fn has_soft_delete(_relation: Self) -> bool {
        false
    }
}

/// Specification for a relation count to be loaded alongside the main query.
#[derive(Debug, Clone)]
pub struct CountRelationSpec {
    pub name: &'static str,
    pub target_table: &'static str,
    pub target_pk: &'static str,
    pub foreign_key: &'static str,
    pub has_soft_delete: bool,
    /// Optional extra WHERE clause (e.g., "AND status = $N")
    pub extra_where: Option<String>,
    /// Bind values for extra_where placeholders
    pub extra_binds: Vec<BindValue>,
    /// Optional nested relation for 2-level counting (e.g., count grandchildren through children)
    pub nested: Option<Box<CountRelationSpec>>,
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
    pub extra_where: Option<String>,
    pub extra_binds: Vec<BindValue>,
    /// Nested relations to load on the loaded records (max 1 level deep).
    pub nested: Vec<WithRelationSpec>,
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
pub fn get_relation_nested<'a>(name: &str, with_relations: &'a Option<Vec<WithRelationSpec>>) -> &'a [WithRelationSpec] {
    with_relations
        .as_ref()
        .and_then(|list| list.iter().find(|s| s.name == name))
        .map(|s| s.nested.as_slice())
        .unwrap_or(&[])
}

/// Get the extra WHERE clause for a relation, if any.
pub fn get_relation_extra_where<'a>(name: &str, with_relations: &'a Option<Vec<WithRelationSpec>>) -> Option<(&'a str, &'a [BindValue])> {
    with_relations
        .as_ref()
        .and_then(|list| list.iter().find(|s| s.name == name))
        .and_then(|s| s.extra_where.as_ref().map(|w| (w.as_str(), s.extra_binds.as_slice())))
}

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

pub trait CreateModel: ModelDef {
    fn create_save<'db>(state: CreateState<'db>) -> BoxModelFuture<'db, Self::Record>;
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

pub trait PatchModel: ModelDef {
    fn patch_from_query<'db>(state: QueryState<'db>) -> PatchState<'db>;
    fn patch_save<'db>(state: PatchState<'db>) -> BoxModelFuture<'db, u64>;
    fn patch_fetch<'db>(state: PatchState<'db>) -> BoxModelFuture<'db, Vec<Self::Record>>;
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

impl<M: ModelDef, T, const KEY: usize> CountRelation<M> for ManyRelation<M, T, KEY> {
    fn name(relation: Self) -> &'static str {
        relation.name()
    }

    fn target_table(relation: Self) -> &'static str {
        relation.target_table()
    }

    fn target_pk(relation: Self) -> &'static str {
        relation.target_pk()
    }

    fn foreign_key(relation: Self) -> &'static str {
        relation.foreign_key()
    }

    fn has_soft_delete(relation: Self) -> bool {
        relation.soft_delete
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
    pub fn new(db: impl Into<DbConn<'db>>) -> Self {
        Self {
            state: QueryState::new(
                db.into(),
                resolve_attachment_base_url(None),
                M::DEFAULT_SELECT,
            ),
            _marker: PhantomData,
        }
    }

    pub fn new_with_base_url(db: impl Into<DbConn<'db>>, base_url: Option<String>) -> Self {
        Self {
            state: QueryState::new(
                db.into(),
                resolve_attachment_base_url(base_url),
                M::DEFAULT_SELECT,
            ),
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

    pub fn for_no_key_update(self) -> Self {
        Self {
            state: self.state.for_no_key_update(),
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

    pub fn order_by<F>(self, field: F, dir: OrderDir) -> Self
    where
        F: QueryField<M>,
    {
        Self {
            state: F::order_by(field, self.state, dir),
            _marker: PhantomData,
        }
    }

    pub fn with<R>(self, relation: R) -> Self
    where
        R: IncludeRelation<M>,
    {
        Self {
            state: R::include(relation, self.state),
            _marker: PhantomData,
        }
    }

    /// Eager-load a relation with additional WHERE conditions.
    /// The scope closure adds conditions that filter which related records are loaded.
    ///
    /// Example: `.with_where(UserRel::DOWNLINES, |q| q.where_col(UserCol::HAS_PURCHASED, Op::Eq, true))`
    pub fn with_where<R, T, F>(mut self, relation: R, scope: F) -> Self
    where
        R: IncludeRelation<M> + RelationName,
        T: QueryModel,
        F: FnOnce(Query<'db, T>) -> Query<'db, T>,
    {
        let name = relation.relation_name();
        let dummy_state = QueryState::new(self.state.db.clone(), None, "");
        let scoped = scope(Query::<T>::from_inner(dummy_state));
        let inner = scoped.into_inner();

        let extra_where = if inner.where_sql.is_empty() {
            None
        } else {
            Some(
                inner
                    .where_sql
                    .iter()
                    .map(|w| format!("AND {w}"))
                    .collect::<Vec<_>>()
                    .join(" "),
            )
        };

        let list = self.state.with_relations.get_or_insert_with(Vec::new);
        list.retain(|s| s.name != name);
        list.push(WithRelationSpec {
            name,
            extra_where,
            extra_binds: inner.binds,
            nested: vec![],
        });
        self
    }

    /// Eager-load a relation AND a nested sub-relation on the loaded records.
    ///
    /// Example: `.with_nested(ArticleRel::COMMENTS, CommentRel::USER)`
    /// Loads comments for each article, then loads each comment's user.
    pub fn with_nested<R1, R2>(mut self, parent_rel: R1, child_rel: R2) -> Self
    where
        R1: IncludeRelation<M> + RelationName,
        R2: RelationName,
    {
        let parent_name = parent_rel.relation_name();
        let child_name = child_rel.relation_name();

        // First ensure parent relation is marked for loading
        let list = self.state.with_relations.get_or_insert_with(Vec::new);
        if let Some(existing) = list.iter_mut().find(|s| s.name == parent_name) {
            // Parent already registered — add nested
            if !existing.nested.iter().any(|n| n.name == child_name) {
                existing.nested.push(WithRelationSpec {
                    name: child_name,
                    extra_where: None,
                    extra_binds: vec![],
                    nested: vec![],
                });
            }
        } else {
            // Register parent with nested
            list.push(WithRelationSpec {
                name: parent_name,
                extra_where: None,
                extra_binds: vec![],
                nested: vec![WithRelationSpec {
                    name: child_name,
                    extra_where: None,
                    extra_binds: vec![],
                    nested: vec![],
                }],
            });
        }
        self
    }

    pub fn where_has<R, F>(self, relation: R, scope: F) -> Self
    where
        R: WhereHasRelation<M>,
        F: FnOnce(Query<'db, R::Target>) -> Query<'db, R::Target>,
    {
        Self {
            state: R::where_has(relation, self.state, scope),
            _marker: PhantomData,
        }
    }

    pub fn or_where_has<R, F>(self, relation: R, scope: F) -> Self
    where
        R: WhereHasRelation<M>,
        F: FnOnce(Query<'db, R::Target>) -> Query<'db, R::Target>,
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
        self.state.count_relations.push(CountRelationSpec {
            name: R::name(relation),
            target_table: R::target_table(relation),
            target_pk: R::target_pk(relation),
            foreign_key: R::foreign_key(relation),
            has_soft_delete: R::has_soft_delete(relation),
            extra_where: None,
            extra_binds: vec![],
            nested: None,
        });
        self
    }

    /// Request a conditional count for a HasMany relation.
    /// The scope closure receives a query builder for the target model to add WHERE conditions.
    ///
    /// Example: `.with_count_where(Rel::ITEMS, |q| q.where_col(ItemCol::STATUS, Op::Eq, ItemStatus::Active))`
    pub fn with_count_where<R, T, F>(mut self, relation: R, scope: F) -> Self
    where
        R: CountRelation<M>,
        T: QueryModel,
        F: FnOnce(Query<'db, T>) -> Query<'db, T>,
    {
        let dummy_state = QueryState::new(self.state.db.clone(), None, "");
        let scoped = scope(Query::<T>::from_inner(dummy_state));
        let inner = scoped.into_inner();

        let extra_where = if inner.where_sql.is_empty() {
            None
        } else {
            Some(inner.where_sql.iter().map(|w| format!("AND {w}")).collect::<Vec<_>>().join(" "))
        };

        self.state.count_relations.push(CountRelationSpec {
            name: R::name(relation),
            target_table: R::target_table(relation),
            target_pk: R::target_pk(relation),
            foreign_key: R::foreign_key(relation),
            has_soft_delete: R::has_soft_delete(relation),
            extra_where,
            extra_binds: inner.binds,
            nested: None,
        });
        self
    }

    /// Request a count of grandchildren through a HasMany → HasMany chain.
    ///
    /// Example: `.with_count_nested(ArticleRel::COMMENTS, CommentRel::REPLIES)`
    /// SQL: `SELECT c.article_id, COUNT(r.id) FROM comments c JOIN replies r ON r.comment_id = c.id WHERE c.article_id IN (...) GROUP BY c.article_id`
    pub fn with_count_nested<R1, R2, T>(mut self, parent_rel: R1, child_rel: R2) -> Self
    where
        R1: CountRelation<M>,
        T: ModelDef,
        R2: CountRelation<T>,
    {
        self.state.count_relations.push(CountRelationSpec {
            name: R1::name(parent_rel),
            target_table: R1::target_table(parent_rel),
            target_pk: R1::target_pk(parent_rel),
            foreign_key: R1::foreign_key(parent_rel),
            has_soft_delete: R1::has_soft_delete(parent_rel),
            extra_where: None,
            extra_binds: vec![],
            nested: Some(Box::new(CountRelationSpec {
                name: R2::name(child_rel),
                target_table: R2::target_table(child_rel),
                target_pk: R2::target_pk(child_rel),
                foreign_key: R2::foreign_key(child_rel),
                has_soft_delete: R2::has_soft_delete(child_rel),
                extra_where: None,
                extra_binds: vec![],
                nested: None,
            })),
        });
        self
    }

    /// Request a conditional count of grandchildren through a HasMany → HasMany chain.
    ///
    /// Example: `.with_count_nested_where(ArticleRel::COMMENTS, CommentRel::REPLIES, |q| q.where_col(ReplyCol::APPROVED, Op::Eq, true))`
    pub fn with_count_nested_where<R1, R2, T, T2, F>(mut self, parent_rel: R1, child_rel: R2, scope: F) -> Self
    where
        R1: CountRelation<M>,
        T: ModelDef,
        R2: CountRelation<T>,
        T2: QueryModel,
        F: FnOnce(Query<'db, T2>) -> Query<'db, T2>,
    {
        let dummy_state = QueryState::new(self.state.db.clone(), None, "");
        let scoped = scope(Query::<T2>::from_inner(dummy_state));
        let inner = scoped.into_inner();

        let extra_where = if inner.where_sql.is_empty() {
            None
        } else {
            Some(inner.where_sql.iter().map(|w| format!("AND {w}")).collect::<Vec<_>>().join(" "))
        };

        self.state.count_relations.push(CountRelationSpec {
            name: R1::name(parent_rel),
            target_table: R1::target_table(parent_rel),
            target_pk: R1::target_pk(parent_rel),
            foreign_key: R1::foreign_key(parent_rel),
            has_soft_delete: R1::has_soft_delete(parent_rel),
            extra_where: None,
            extra_binds: vec![],
            nested: Some(Box::new(CountRelationSpec {
                name: R2::name(child_rel),
                target_table: R2::target_table(child_rel),
                target_pk: R2::target_pk(child_rel),
                foreign_key: R2::foreign_key(child_rel),
                has_soft_delete: R2::has_soft_delete(child_rel),
                extra_where,
                extra_binds: inner.binds,
                nested: None,
            })),
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

    pub async fn all(self) -> Result<Vec<M::Record>> {
        M::query_all(self.state).await
    }

    pub async fn first(self) -> Result<Option<M::Record>> {
        M::query_first(self.state).await
    }

    pub async fn find(self, id: M::Pk) -> Result<Option<M::Record>> {
        M::query_find(self.state, id).await
    }

    pub async fn count(self) -> Result<i64> {
        M::query_count(self.state).await
    }

    pub async fn delete(self) -> Result<u64> {
        M::query_delete(self.state).await
    }

    pub async fn paginate(self, page: i64, per_page: i64) -> Result<Page<M::Record>> {
        M::query_paginate(self.state, page, per_page).await
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

    // ── Aggregate terminal methods ────────────────────────────────────

    pub async fn sum(self, col: impl ColExpr) -> Result<Option<f64>> {
        self.state
            .aggregate_scalar(
                &format!("SUM({}::DOUBLE PRECISION)", col.col_sql()),
                M::TABLE,
                M::HAS_SOFT_DELETE,
                M::SOFT_DELETE_COL,
            )
            .await
    }

    pub async fn avg(self, col: impl ColExpr) -> Result<Option<f64>> {
        self.state
            .aggregate_scalar(
                &format!("AVG({}::DOUBLE PRECISION)", col.col_sql()),
                M::TABLE,
                M::HAS_SOFT_DELETE,
                M::SOFT_DELETE_COL,
            )
            .await
    }

    pub async fn min_val(self, col: impl ColExpr) -> Result<Option<f64>> {
        self.state
            .aggregate_scalar(
                &format!("MIN({}::DOUBLE PRECISION)", col.col_sql()),
                M::TABLE,
                M::HAS_SOFT_DELETE,
                M::SOFT_DELETE_COL,
            )
            .await
    }

    pub async fn max_val(self, col: impl ColExpr) -> Result<Option<f64>> {
        self.state
            .aggregate_scalar(
                &format!("MAX({}::DOUBLE PRECISION)", col.col_sql()),
                M::TABLE,
                M::HAS_SOFT_DELETE,
                M::SOFT_DELETE_COL,
            )
            .await
    }

    pub async fn exists(self) -> Result<bool> {
        Ok(self.count().await? > 0)
    }

    // ── Increment / Decrement ─────────────────────────────────────────

    pub async fn increment(self, col: impl ColExpr, amount: i64) -> Result<u64> {
        self.state
            .execute_increment(
                col.col_sql(),
                BindValue::I64(amount),
                M::TABLE,
                M::HAS_SOFT_DELETE,
                M::SOFT_DELETE_COL,
                M::HAS_UPDATED_AT,
            )
            .await
    }

    pub async fn decrement(self, col: impl ColExpr, amount: i64) -> Result<u64> {
        self.increment(col, -amount).await
    }

    // ── Fail-fast terminal methods ────────────────────────────────────

    pub async fn first_or_fail(self) -> Result<M::Record> {
        self.first()
            .await?
            .ok_or_else(|| anyhow::anyhow!("{}: record not found", M::TABLE))
    }

    pub async fn find_or_fail(self, id: M::Pk) -> Result<M::Record> {
        self.find(id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("{}: record not found", M::TABLE))
    }

    pub async fn sole(self) -> Result<M::Record> {
        let mut rows = self.limit(2).all().await?;
        match rows.len() {
            0 => anyhow::bail!("{}: no record found", M::TABLE),
            1 => Ok(rows.remove(0)),
            _ => anyhow::bail!("{}: multiple records found", M::TABLE),
        }
    }

    // ── Chunk iteration ───────────────────────────────────────────────

    pub async fn chunk<F, Fut>(self, size: i64, mut callback: F) -> Result<()>
    where
        F: FnMut(Vec<M::Record>) -> Fut,
        Fut: std::future::Future<Output = Result<bool>>,
    {
        let mut page = 0i64;
        loop {
            let page_state = {
                let mut s = self.state.clone();
                s.offset = Some(page * size);
                s.limit = Some(size);
                s
            };
            let rows = M::query_all(page_state).await?;
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

    pub async fn restore(self) -> Result<u64> {
        if !M::HAS_SOFT_DELETE {
            anyhow::bail!("{}: restore() not supported (no soft delete)", M::TABLE);
        }
        self.state
            .execute_restore(M::TABLE, M::SOFT_DELETE_COL, M::HAS_UPDATED_AT)
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

    pub async fn pluck<K, E>(self, extract: E) -> Result<Vec<K>>
    where
        E: Fn(&M::Record) -> K,
    {
        let rows = self.all().await?;
        Ok(rows.iter().map(|r| extract(r)).collect())
    }

    pub async fn pluck_map<K, V, E>(self, extract: E) -> Result<std::collections::HashMap<K, V>>
    where
        K: Eq + std::hash::Hash,
        E: Fn(&M::Record) -> (K, V),
    {
        let rows = self.all().await?;
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
    pub fn new(db: impl Into<DbConn<'db>>) -> Self {
        Self {
            state: CreateState::new(db.into(), resolve_attachment_base_url(None), M::TABLE),
            _marker: PhantomData,
        }
    }

    pub fn new_with_base_url(db: impl Into<DbConn<'db>>, base_url: Option<String>) -> Self {
        Self {
            state: CreateState::new(db.into(), resolve_attachment_base_url(base_url), M::TABLE),
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

    pub async fn save(self) -> Result<M::Record> {
        M::create_save(self.state).await
    }
}

// ---------------------------------------------------------------------------
// Patch wrapper
// ---------------------------------------------------------------------------

pub struct Patch<'db, M: PatchModel> {
    state: PatchState<'db>,
    _marker: PhantomData<M>,
}

impl<'db, M: PatchModel> Patch<'db, M> {
    pub fn new(db: impl Into<DbConn<'db>>) -> Self {
        Self {
            state: PatchState::new(db.into(), resolve_attachment_base_url(None), M::TABLE),
            _marker: PhantomData,
        }
    }

    pub fn new_with_base_url(db: impl Into<DbConn<'db>>, base_url: Option<String>) -> Self {
        Self {
            state: PatchState::new(db.into(), resolve_attachment_base_url(base_url), M::TABLE),
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

    pub async fn save(self) -> Result<u64> {
        M::patch_save(self.state).await
    }

    pub async fn fetch(self) -> Result<Vec<M::Record>> {
        M::patch_fetch(self.state).await
    }
}

// ---------------------------------------------------------------------------
// QueryState — shared SQL builder state (replaces per-model XxxQueryInner)
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct QueryState<'db> {
    pub db: DbConn<'db>,
    pub base_url: Option<String>,
    pub select_sql: Option<String>,
    pub from_sql: Option<String>,
    pub count_sql: Option<String>,
    pub distinct: bool,
    pub distinct_on: Option<String>,
    pub lock_sql: Option<&'static str>,
    pub join_sql: Vec<String>,
    pub join_binds: Vec<BindValue>,
    pub where_sql: Vec<String>,
    pub order_sql: Vec<String>,
    pub group_by_sql: Vec<String>,
    pub having_sql: Vec<String>,
    pub having_binds: Vec<BindValue>,
    pub offset: Option<i64>,
    pub limit: Option<i64>,
    pub binds: Vec<BindValue>,
    pub with_deleted: bool,
    pub only_deleted: bool,
    pub count_relations: Vec<CountRelationSpec>,
    /// Which relations to eager-load. `None` = none. `Some(vec)` = only listed.
    pub with_relations: Option<Vec<WithRelationSpec>>,
}

impl<'db> QueryState<'db> {
    pub fn new(db: DbConn<'db>, base_url: Option<String>, default_select: &str) -> Self {
        Self {
            db,
            base_url,
            select_sql: Some(default_select.to_string()),
            from_sql: None,
            count_sql: None,
            distinct: false,
            distinct_on: None,
            lock_sql: None,
            join_sql: vec![],
            join_binds: vec![],
            where_sql: vec![],
            order_sql: vec![],
            group_by_sql: vec![],
            having_sql: vec![],
            having_binds: vec![],
            offset: None,
            limit: None,
            binds: vec![],
            with_deleted: false,
            only_deleted: false,
            count_relations: vec![],
            with_relations: None,
        }
    }

    // ── WHERE helpers ──────────────────────────────────────────────────

    pub fn where_col_str(mut self, col_sql: &str, op: Op, val: BindValue) -> Self {
        // NULL-safe: `col = NULL` never matches in SQL; use `IS NULL` / `IS NOT NULL` instead.
        if val.is_null() {
            let null_clause = match op {
                Op::Eq => format!("{} IS NULL", col_sql),
                Op::Ne => format!("{} IS NOT NULL", col_sql),
                _ => {
                    // For other ops (>, <, etc.) with NULL, just skip — these are no-ops in SQL.
                    return self;
                }
            };
            self.where_sql.push(null_clause);
            return self;
        }
        let idx = self.binds.len() + 1;
        self.where_sql
            .push(format!("{} {} ${}", col_sql, op.as_sql(), idx));
        self.binds.push(val);
        self
    }

    pub fn or_where_col_str(mut self, col_sql: &str, op: Op, val: BindValue) -> Self {
        // NULL-safe: same logic as where_col_str for OR clauses.
        if val.is_null() {
            let null_clause = match op {
                Op::Eq => format!("{} IS NULL", col_sql),
                Op::Ne => format!("{} IS NOT NULL", col_sql),
                _ => return self,
            };
            if let Some(last) = self.where_sql.pop() {
                self.where_sql.push(format!("({} OR {})", last, null_clause));
            } else {
                self.where_sql.push(null_clause);
            }
            return self;
        }
        let idx = self.binds.len() + 1;
        let clause = format!("{} {} ${}", col_sql, op.as_sql(), idx);
        if let Some(last) = self.where_sql.pop() {
            self.where_sql.push(format!("({} OR {})", last, clause));
        } else {
            self.where_sql.push(clause);
        }
        self.binds.push(val);
        self
    }

    pub fn where_in_str(mut self, col_sql: &str, vals: &[BindValue]) -> Self {
        if vals.is_empty() {
            self.where_sql.push("1=0".to_string());
            return self;
        }
        let start = self.binds.len() + 1;
        let mut placeholders = Vec::with_capacity(vals.len());
        for (i, v) in vals.iter().enumerate() {
            placeholders.push(format!("${}", start + i));
            self.binds.push(v.clone());
        }
        self.where_sql
            .push(format!("{} IN ({})", col_sql, placeholders.join(", ")));
        self
    }

    pub fn where_not_in_str(mut self, col_sql: &str, vals: &[BindValue]) -> Self {
        if vals.is_empty() {
            return self;
        }
        let start = self.binds.len() + 1;
        let mut placeholders = Vec::with_capacity(vals.len());
        for (i, v) in vals.iter().enumerate() {
            placeholders.push(format!("${}", start + i));
            self.binds.push(v.clone());
        }
        self.where_sql
            .push(format!("{} NOT IN ({})", col_sql, placeholders.join(", ")));
        self
    }

    pub fn where_between_str(mut self, col_sql: &str, low: BindValue, high: BindValue) -> Self {
        let idx1 = self.binds.len() + 1;
        let idx2 = idx1 + 1;
        self.where_sql
            .push(format!("{} BETWEEN ${} AND ${}", col_sql, idx1, idx2));
        self.binds.push(low);
        self.binds.push(high);
        self
    }

    pub fn where_null_str(mut self, col_sql: &str) -> Self {
        self.where_sql.push(format!("{} IS NULL", col_sql));
        self
    }

    pub fn where_not_null_str(mut self, col_sql: &str) -> Self {
        self.where_sql.push(format!("{} IS NOT NULL", col_sql));
        self
    }

    pub fn where_raw(mut self, clause: String, raw_binds: Vec<BindValue>) -> Self {
        let mut clause = clause;
        let mut idx = self.binds.len() + 1;
        while let Some(pos) = clause.find('?') {
            let ph = format!("${}", idx);
            clause.replace_range(pos..pos + 1, &ph);
            idx += 1;
        }
        self.where_sql.push(clause);
        self.binds.extend(raw_binds);
        self
    }

    pub fn or_where_raw(mut self, clause: String, raw_binds: Vec<BindValue>) -> Self {
        let mut clause = clause;
        let mut idx = self.binds.len() + 1;
        while let Some(pos) = clause.find('?') {
            let ph = format!("${}", idx);
            clause.replace_range(pos..pos + 1, &ph);
            idx += 1;
        }
        if let Some(last) = self.where_sql.pop() {
            self.where_sql.push(format!("({} OR {})", last, clause));
        } else {
            self.where_sql.push(clause);
        }
        self.binds.extend(raw_binds);
        self
    }

    pub fn where_exists_raw(mut self, clause: String, raw_binds: Vec<BindValue>) -> Self {
        let mut clause = clause;
        let mut idx = self.binds.len() + 1;
        while let Some(pos) = clause.find('?') {
            let ph = format!("${}", idx);
            clause.replace_range(pos..pos + 1, &ph);
            idx += 1;
        }
        self.where_sql.push(format!("EXISTS ({})", clause));
        self.binds.extend(raw_binds);
        self
    }

    pub fn where_group(self, f: impl FnOnce(Self) -> Self) -> Self {
        let start_where = self.where_sql.len();
        let mut result = f(self);
        if result.where_sql.len() > start_where {
            let group_clauses: Vec<String> = result.where_sql.drain(start_where..).collect();
            let grouped_sql = format!("({})", group_clauses.join(" AND "));
            result.where_sql.push(grouped_sql);
        }
        result
    }

    pub fn or_where_group(self, f: impl FnOnce(Self) -> Self) -> Self {
        let start_where = self.where_sql.len();
        let mut result = f(self);
        if result.where_sql.len() > start_where {
            let group_clauses: Vec<String> = result.where_sql.drain(start_where..).collect();
            let grouped_sql = format!("({})", group_clauses.join(" AND "));
            if let Some(last) = result.where_sql.pop() {
                result
                    .where_sql
                    .push(format!("({} OR {})", last, grouped_sql));
            } else {
                result.where_sql.push(grouped_sql);
            }
        }
        result
    }

    // ── ORDER BY ───────────────────────────────────────────────────────

    pub fn order_by_str(mut self, col_sql: &str, dir: OrderDir) -> Self {
        self.order_sql.push(format!("{} {}", col_sql, dir.as_sql()));
        self
    }

    pub fn order_by_nulls_first_str(mut self, col_sql: &str, dir: OrderDir) -> Self {
        self.order_sql
            .push(format!("{} {} NULLS FIRST", col_sql, dir.as_sql()));
        self
    }

    pub fn order_by_nulls_last_str(mut self, col_sql: &str, dir: OrderDir) -> Self {
        self.order_sql
            .push(format!("{} {} NULLS LAST", col_sql, dir.as_sql()));
        self
    }

    pub fn order_raw(mut self, expr: String) -> Self {
        self.order_sql.push(expr);
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
        self.lock_sql = Some("FOR UPDATE");
        self
    }

    pub fn for_update_skip_locked(mut self) -> Self {
        self.lock_sql = Some("FOR UPDATE SKIP LOCKED");
        self
    }

    pub fn for_no_key_update(mut self) -> Self {
        self.lock_sql = Some("FOR NO KEY UPDATE");
        self
    }

    pub fn for_share(mut self) -> Self {
        self.lock_sql = Some("FOR SHARE");
        self
    }

    pub fn for_key_share(mut self) -> Self {
        self.lock_sql = Some("FOR KEY SHARE");
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
        let mut clause = format!("{} {} ON {}", kind, table, on_clause);
        let mut idx = self.join_binds.len() + self.binds.len() + 1;
        while let Some(pos) = clause.find('?') {
            let ph = format!("${}", idx);
            clause.replace_range(pos..pos + 1, &ph);
            idx += 1;
        }
        self.join_sql.push(clause);
        self.join_binds.extend(raw_binds);
        self
    }

    // ── SELECT ─────────────────────────────────────────────────────────

    pub fn select_raw(mut self, expr: String) -> Self {
        if expr.is_empty() {
            return self;
        }
        let mut base = self.select_sql.take().unwrap_or_else(|| "*".to_string());
        if !base.is_empty() {
            base.push_str(", ");
        }
        base.push_str(&expr);
        self.select_sql = Some(base);
        self
    }

    pub fn add_select_raw(mut self, expr: String) -> Self {
        if expr.is_empty() {
            return self;
        }
        let mut base = self.select_sql.take().unwrap_or_else(|| "*".to_string());
        if !base.is_empty() {
            base.push_str(", ");
        }
        base.push_str(&expr);
        self.select_sql = Some(base);
        self
    }

    // ── GROUP BY / HAVING ──────────────────────────────────────────────

    pub fn group_by_str(mut self, cols: &[&str]) -> Self {
        for c in cols {
            self.group_by_sql.push(c.to_string());
        }
        self
    }

    pub fn having_raw(mut self, clause: String, raw_binds: Vec<BindValue>) -> Self {
        let mut clause = clause;
        let mut idx = self.having_binds.len() + 1;
        while let Some(pos) = clause.find('?') {
            let ph = format!("${}", idx);
            clause.replace_range(pos..pos + 1, &ph);
            idx += 1;
        }
        self.having_sql.push(clause);
        self.having_binds.extend(raw_binds);
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

    fn apply_soft_delete(
        where_sql: &mut Vec<String>,
        has_soft_delete: bool,
        soft_delete_col: &str,
        with_deleted: bool,
        only_deleted: bool,
    ) {
        if has_soft_delete {
            if only_deleted {
                where_sql.push(format!("{} IS NOT NULL", soft_delete_col));
            } else if !with_deleted {
                where_sql.push(format!("{} IS NULL", soft_delete_col));
            }
        }
    }

    fn build_select_clause(
        distinct: bool,
        distinct_on: Option<&str>,
        select_sql: Option<String>,
    ) -> String {
        let cols = select_sql.unwrap_or_else(|| "*".to_string());
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
        let mut where_sql = self.where_sql.clone();
        Self::apply_soft_delete(
            &mut where_sql,
            has_soft_delete,
            soft_delete_col,
            self.with_deleted,
            self.only_deleted,
        );

        let select_clause = Self::build_select_clause(
            self.distinct,
            self.distinct_on.as_deref(),
            self.select_sql.clone(),
        );
        let table_name = self.from_sql.as_deref().unwrap_or(table);
        let mut sql = format!("SELECT {} FROM {}", select_clause, table_name);
        if !self.join_sql.is_empty() {
            sql.push(' ');
            sql.push_str(&self.join_sql.join(" "));
        }
        if !where_sql.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&where_sql.join(" AND "));
        }
        if !self.group_by_sql.is_empty() {
            sql.push_str(" GROUP BY ");
            sql.push_str(&self.group_by_sql.join(", "));
        }
        if !self.having_sql.is_empty() {
            sql.push_str(" HAVING ");
            sql.push_str(&self.having_sql.join(" AND "));
        }
        if !self.order_sql.is_empty() {
            sql.push_str(" ORDER BY ");
            sql.push_str(&self.order_sql.join(", "));
        }
        if let Some(off) = self.offset {
            sql.push_str(" OFFSET ");
            sql.push_str(&off.to_string());
        }
        if let Some(l) = self.limit {
            sql.push_str(" LIMIT ");
            sql.push_str(&l.to_string());
        }
        if let Some(lock) = self.lock_sql {
            sql.push(' ');
            sql.push_str(lock);
        }
        let mut all_binds = self.binds.clone();
        all_binds.extend(self.join_binds.clone());
        all_binds.extend(self.having_binds.clone());
        (sql, all_binds)
    }

    /// Assemble a COUNT query.
    pub fn to_count_sql(
        &self,
        table: &str,
        has_soft_delete: bool,
        soft_delete_col: &str,
    ) -> (String, Vec<BindValue>) {
        let mut where_sql = self.where_sql.clone();
        Self::apply_soft_delete(
            &mut where_sql,
            has_soft_delete,
            soft_delete_col,
            self.with_deleted,
            self.only_deleted,
        );

        let table_name = self.from_sql.as_deref().unwrap_or(table);
        let from_clause = if self.join_sql.is_empty() {
            format!("FROM {}", table_name)
        } else {
            format!("FROM {} {}", table_name, self.join_sql.join(" "))
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
                self.select_sql.clone(),
            );
            format!(
                "SELECT COUNT(*) FROM (SELECT {} {}{}) AS sub",
                select_clause, from_clause, where_clause
            )
        } else {
            format!("SELECT {} {}{}", count_expr, from_clause, where_clause)
        };

        let mut all_binds = self.binds.clone();
        all_binds.extend(self.join_binds.clone());
        (sql, all_binds)
    }

    /// Assemble a DELETE statement.
    pub fn to_delete_sql(
        &self,
        table: &str,
        has_soft_delete: bool,
        soft_delete_col: &str,
    ) -> (String, Vec<BindValue>) {
        let mut where_sql = self.where_sql.clone();
        Self::apply_soft_delete(
            &mut where_sql,
            has_soft_delete,
            soft_delete_col,
            self.with_deleted,
            self.only_deleted,
        );

        let sql = if where_sql.is_empty() {
            format!("DELETE FROM {}", table)
        } else {
            format!("DELETE FROM {} WHERE {}", table, where_sql.join(" AND "))
        };
        (sql, self.binds.clone())
    }

    /// Extract WHERE parts for reuse.
    pub fn into_where_parts(self) -> (Vec<String>, Vec<BindValue>) {
        (self.where_sql, self.binds)
    }

    /// Assemble a full SELECT statement and return it with binds (consuming self).
    pub fn to_sql(self) -> (String, Vec<BindValue>) {
        let select_clause =
            Self::build_select_clause(self.distinct, self.distinct_on.as_deref(), self.select_sql);
        let table_name = self.from_sql.unwrap_or_default();
        let mut sql = if table_name.is_empty() {
            format!("SELECT {}", select_clause)
        } else {
            format!("SELECT {} FROM {}", select_clause, table_name)
        };
        if !self.join_sql.is_empty() {
            sql.push(' ');
            sql.push_str(&self.join_sql.join(" "));
        }
        if !self.where_sql.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&self.where_sql.join(" AND "));
        }
        if !self.group_by_sql.is_empty() {
            sql.push_str(" GROUP BY ");
            sql.push_str(&self.group_by_sql.join(", "));
        }
        if !self.having_sql.is_empty() {
            sql.push_str(" HAVING ");
            sql.push_str(&self.having_sql.join(" AND "));
        }
        if !self.order_sql.is_empty() {
            sql.push_str(" ORDER BY ");
            sql.push_str(&self.order_sql.join(", "));
        }
        if let Some(off) = self.offset {
            sql.push_str(" OFFSET ");
            sql.push_str(&off.to_string());
        }
        if let Some(l) = self.limit {
            sql.push_str(" LIMIT ");
            sql.push_str(&l.to_string());
        }
        if let Some(lock) = self.lock_sql {
            sql.push(' ');
            sql.push_str(lock);
        }
        let mut all_binds = self.binds;
        all_binds.extend(self.join_binds);
        all_binds.extend(self.having_binds);
        (sql, all_binds)
    }

    // ── Aggregate execution ───────────────────────────────────────────

    pub async fn aggregate_scalar(
        self,
        agg_expr: &str,
        table: &str,
        has_soft_delete: bool,
        soft_delete_col: &str,
    ) -> Result<Option<f64>> {
        use crate::common::sql::{bind_scalar, PgQueryScalar};

        let mut where_sql = self.where_sql;
        Self::apply_soft_delete(
            &mut where_sql,
            has_soft_delete,
            soft_delete_col,
            self.with_deleted,
            self.only_deleted,
        );

        let table_name = self.from_sql.as_deref().unwrap_or(table);
        let from_clause = if self.join_sql.is_empty() {
            format!("FROM {}", table_name)
        } else {
            format!("FROM {} {}", table_name, self.join_sql.join(" "))
        };
        let where_clause = if where_sql.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", where_sql.join(" AND "))
        };
        let sql = format!("SELECT {} {}{}", agg_expr, from_clause, where_clause);

        let mut q: PgQueryScalar<'_, Option<f64>> = sqlx::query_scalar(&sql);
        for b in &self.binds {
            q = bind_scalar(q, b.clone());
        }
        for b in &self.join_binds {
            q = bind_scalar(q, b.clone());
        }
        let result = self.db.fetch_scalar(q).await?;
        Ok(result)
    }

    // ── Increment execution ───────────────────────────────────────────

    pub async fn execute_increment(
        self,
        col_sql: &str,
        amount: BindValue,
        table: &str,
        has_soft_delete: bool,
        soft_delete_col: &str,
        has_updated_at: bool,
    ) -> Result<u64> {
        use crate::common::sql::{bind_query, renumber_placeholders};

        let mut where_sql = self.where_sql;
        Self::apply_soft_delete(
            &mut where_sql,
            has_soft_delete,
            soft_delete_col,
            self.with_deleted,
            self.only_deleted,
        );

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
        for b in self.binds {
            q = bind_query(q, b);
        }
        let result = self.db.execute(q).await?;
        Ok(result.rows_affected())
    }

    // ── Restore (soft-delete undo) ────────────────────────────────────

    pub async fn execute_restore(
        self,
        table: &str,
        soft_delete_col: &str,
        has_updated_at: bool,
    ) -> Result<u64> {
        use crate::common::sql::bind_query;

        if self.where_sql.is_empty() {
            anyhow::bail!("restore: no conditions set");
        }
        if self.limit.is_some() {
            anyhow::bail!("restore: does not support limit; add where clauses");
        }

        let mut where_sql = self.where_sql;
        // Only restore records that ARE deleted
        where_sql.push(format!("{} IS NOT NULL", soft_delete_col));

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
        for b in self.binds {
            q = bind_query(q, b);
        }
        let result = self.db.execute(q).await?;
        Ok(result.rows_affected())
    }
}

// ---------------------------------------------------------------------------
// CreateState — shared INSERT builder state (replaces per-model XxxCreateInner)
// ---------------------------------------------------------------------------

pub struct CreateState<'db> {
    pub db: DbConn<'db>,
    pub base_url: Option<String>,
    pub table: &'static str,
    pub col_names: Vec<&'static str>,
    pub binds: Vec<BindValue>,
    pub conflict_action: Option<&'static str>,
    pub conflict_cols: Vec<&'static str>,
}

impl<'db> CreateState<'db> {
    pub fn new(db: DbConn<'db>, base_url: Option<String>, table: &'static str) -> Self {
        Self {
            db,
            base_url,
            table,
            col_names: vec![],
            binds: vec![],
            conflict_action: None,
            conflict_cols: vec![],
        }
    }

    pub fn set_col(mut self, col_sql: &'static str, val: BindValue) -> Self {
        self.col_names.push(col_sql);
        self.binds.push(val);
        self
    }

    pub fn on_conflict_do_nothing(mut self, cols: &[&'static str]) -> Self {
        self.conflict_action = Some("DO NOTHING");
        self.conflict_cols = cols.to_vec();
        self
    }

    pub fn on_conflict_update(mut self, cols: &[&'static str]) -> Self {
        self.conflict_action = Some("DO UPDATE");
        self.conflict_cols = cols.to_vec();
        self
    }

    pub fn build_insert_sql(&self) -> (String, Vec<BindValue>) {
        let placeholders: Vec<String> = (1..=self.binds.len()).map(|i| format!("${}", i)).collect();
        let mut sql = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            self.table,
            self.col_names.join(", "),
            placeholders.join(", ")
        );
        if let Some(action) = self.conflict_action {
            if !self.conflict_cols.is_empty() {
                sql.push_str(&format!(
                    " ON CONFLICT ({}) {}",
                    self.conflict_cols.join(", "),
                    action
                ));
                if action == "DO UPDATE" {
                    let set_clauses: Vec<String> = self
                        .col_names
                        .iter()
                        .zip(placeholders.iter())
                        .filter(|(col, _)| !self.conflict_cols.contains(col))
                        .map(|(col, ph)| format!("{} = {}", col, ph))
                        .collect();
                    if !set_clauses.is_empty() {
                        sql.push_str(&format!(" SET {}", set_clauses.join(", ")));
                    }
                }
            }
        }
        sql.push_str(" RETURNING *");
        (sql, self.binds.clone())
    }
}

// ---------------------------------------------------------------------------
// PatchState — shared UPDATE builder state (replaces per-model XxxPatchInner)
// ---------------------------------------------------------------------------

pub struct PatchState<'db> {
    pub db: DbConn<'db>,
    pub base_url: Option<String>,
    pub table: &'static str,
    pub set_cols: Vec<&'static str>,
    pub set_binds: Vec<BindValue>,
    pub set_modes: Vec<SetMode>,
    pub where_sql: Vec<String>,
    pub where_binds: Vec<BindValue>,
}

impl<'db> PatchState<'db> {
    pub fn new(db: DbConn<'db>, base_url: Option<String>, table: &'static str) -> Self {
        Self {
            db,
            base_url,
            table,
            set_cols: vec![],
            set_binds: vec![],
            set_modes: vec![],
            where_sql: vec![],
            where_binds: vec![],
        }
    }

    pub fn assign_col(mut self, col_sql: &'static str, val: BindValue) -> Self {
        self.set_cols.push(col_sql);
        self.set_binds.push(val);
        self.set_modes.push(SetMode::Assign);
        self
    }

    pub fn increment_col(mut self, col_sql: &'static str, val: BindValue) -> Self {
        self.set_cols.push(col_sql);
        self.set_binds.push(val);
        self.set_modes.push(SetMode::Increment);
        self
    }

    pub fn decrement_col(mut self, col_sql: &'static str, val: BindValue) -> Self {
        self.set_cols.push(col_sql);
        self.set_binds.push(val);
        self.set_modes.push(SetMode::Decrement);
        self
    }

    pub fn where_col_str(mut self, col_sql: &str, op: Op, val: BindValue) -> Self {
        if val.is_null() {
            let null_clause = match op {
                Op::Eq => format!("{} IS NULL", col_sql),
                Op::Ne => format!("{} IS NOT NULL", col_sql),
                _ => return self,
            };
            self.where_sql.push(null_clause);
            return self;
        }
        let idx = self.where_binds.len() + 1;
        self.where_sql
            .push(format!("{} {} ${}", col_sql, op.as_sql(), idx));
        self.where_binds.push(val);
        self
    }

    pub fn where_raw(mut self, clause: String, raw_binds: Vec<BindValue>) -> Self {
        let mut clause = clause;
        let mut idx = self.where_binds.len() + 1;
        while let Some(pos) = clause.find('?') {
            let ph = format!("${}", idx);
            clause.replace_range(pos..pos + 1, &ph);
            idx += 1;
        }
        self.where_sql.push(clause);
        self.where_binds.extend(raw_binds);
        self
    }

    pub fn from_query_state(qs: QueryState<'db>, table: &'static str) -> Self {
        Self {
            db: qs.db,
            base_url: qs.base_url,
            table,
            set_cols: vec![],
            set_binds: vec![],
            set_modes: vec![],
            where_sql: qs.where_sql,
            where_binds: qs.binds,
        }
    }

    pub fn build_update_sql(&self) -> (String, Vec<BindValue>) {
        use crate::common::sql::renumber_placeholders;

        let mut parts: Vec<String> = Vec::new();
        for (i, (col, mode)) in self.set_cols.iter().zip(self.set_modes.iter()).enumerate() {
            let part = match mode {
                SetMode::Assign => format!("{} = ${}", col, i + 1),
                SetMode::Increment => format!("{} = {} + ${}", col, col, i + 1),
                SetMode::Decrement => format!("{} = {} - ${}", col, col, i + 1),
            };
            parts.push(part);
        }
        let offset = parts.len();

        let mut renumbered_where: Vec<String> = Vec::with_capacity(self.where_sql.len());
        for clause in &self.where_sql {
            renumbered_where.push(renumber_placeholders(clause, offset + 1));
        }

        let mut sql = format!("UPDATE {} SET {}", self.table, parts.join(", "));
        if !renumbered_where.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&renumbered_where.join(" AND "));
        }

        let mut all_binds = self.set_binds.clone();
        all_binds.extend(self.where_binds.clone());
        (sql, all_binds)
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
    use sqlx::postgres::PgPoolOptions;

    struct FakeModel;

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

        fn query_all<'db>(_: QueryState<'db>) -> BoxModelFuture<'db, Vec<Self::Record>> {
            unreachable!()
        }

        fn query_first<'db>(_: QueryState<'db>) -> BoxModelFuture<'db, Option<Self::Record>> {
            unreachable!()
        }

        fn query_find<'db>(
            _: QueryState<'db>,
            _: Self::Pk,
        ) -> BoxModelFuture<'db, Option<Self::Record>> {
            unreachable!()
        }

        fn query_count<'db>(_: QueryState<'db>) -> BoxModelFuture<'db, i64> {
            unreachable!()
        }

        fn query_delete<'db>(_: QueryState<'db>) -> BoxModelFuture<'db, u64> {
            unreachable!()
        }

        fn query_paginate<'db>(
            _: QueryState<'db>,
            _: i64,
            _: i64,
        ) -> BoxModelFuture<'db, Page<Self::Record>> {
            unreachable!()
        }
    }

    impl CreateModel for FakeModel {
        fn create_save<'db>(_: CreateState<'db>) -> BoxModelFuture<'db, Self::Record> {
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

        fn patch_save<'db>(_: PatchState<'db>) -> BoxModelFuture<'db, u64> {
            unreachable!()
        }

        fn patch_fetch<'db>(_: PatchState<'db>) -> BoxModelFuture<'db, Vec<Self::Record>> {
            unreachable!()
        }

        fn transform_patch_value(_: &str, value: BindValue) -> Result<BindValue> {
            Ok(value)
        }
    }

    fn fake_pool() -> sqlx::PgPool {
        PgPoolOptions::new()
            .connect_lazy("postgres://postgres:postgres@127.0.0.1/test")
            .expect("connect_lazy should succeed")
    }

    #[tokio::test]
    async fn query_create_and_patch_inherit_default_attachment_base_url() {
        set_default_attachment_base_url(Some("https://cdn.example.com/media/".to_string()));
        let db = fake_pool();

        let query = Query::<FakeModel>::new(&db);
        assert_eq!(
            query.into_inner().base_url.as_deref(),
            Some("https://cdn.example.com/media")
        );

        let create = Create::<FakeModel>::new(&db);
        assert_eq!(
            create.into_inner().base_url.as_deref(),
            Some("https://cdn.example.com/media")
        );

        let patch = Patch::<FakeModel>::new(&db);
        assert_eq!(
            patch.into_inner().base_url.as_deref(),
            Some("https://cdn.example.com/media")
        );

        let query = Query::<FakeModel>::new_with_base_url(
            &db,
            Some("https://images.example.com/custom/".to_string()),
        );
        assert_eq!(
            query.into_inner().base_url.as_deref(),
            Some("https://images.example.com/custom")
        );
    }
}
