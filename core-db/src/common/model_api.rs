use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;

use crate::common::sql::{BindValue, DbConn, Op, OrderDir, SetMode};

pub type BoxModelFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T>> + Send + 'a>>;

pub trait ModelDef: Sized + 'static {
    type Pk: Clone + Send + Sync + 'static;
    type Record: Clone + Send + Sync + 'static;
    type Create: Send + 'static;
    type Changes: Send + 'static;

    const TABLE: &'static str;
    const MODEL_KEY: &'static str;
    const PK_COL: &'static str;
}

pub trait QueryModel: ModelDef {
    type InnerQuery<'db>: Clone + Send + 'db;

    fn query_root<'db>(db: DbConn<'db>, base_url: Option<String>) -> Self::InnerQuery<'db>;
    fn query_limit<'db>(query: Self::InnerQuery<'db>, limit: i64) -> Self::InnerQuery<'db>;
    fn query_offset<'db>(query: Self::InnerQuery<'db>, offset: i64) -> Self::InnerQuery<'db>;
    fn query_for_update<'db>(query: Self::InnerQuery<'db>) -> Self::InnerQuery<'db>;
    fn query_for_update_skip_locked<'db>(query: Self::InnerQuery<'db>) -> Self::InnerQuery<'db>;
    fn query_for_no_key_update<'db>(query: Self::InnerQuery<'db>) -> Self::InnerQuery<'db>;
    fn query_where_group<'db, F>(query: Self::InnerQuery<'db>, scope: F) -> Self::InnerQuery<'db>
    where
        F: FnOnce(Query<'db, Self>) -> Query<'db, Self>;
    fn query_or_where_group<'db, F>(
        query: Self::InnerQuery<'db>,
        scope: F,
    ) -> Self::InnerQuery<'db>
    where
        F: FnOnce(Query<'db, Self>) -> Query<'db, Self>;
    fn query_all<'db>(query: Self::InnerQuery<'db>) -> BoxModelFuture<'db, Vec<Self::Record>>;
    fn query_first<'db>(query: Self::InnerQuery<'db>) -> BoxModelFuture<'db, Option<Self::Record>>;
    fn query_find<'db>(
        query: Self::InnerQuery<'db>,
        id: Self::Pk,
    ) -> BoxModelFuture<'db, Option<Self::Record>>;
    fn query_count<'db>(query: Self::InnerQuery<'db>) -> BoxModelFuture<'db, i64>;
    fn query_delete<'db>(query: Self::InnerQuery<'db>) -> BoxModelFuture<'db, u64>;
    fn query_paginate<'db>(
        query: Self::InnerQuery<'db>,
        page: i64,
        per_page: i64,
    ) -> BoxModelFuture<'db, Page<Self::Record>>;
}

pub trait UnsafeQueryModel: QueryModel {
    fn query_where_raw<'db>(
        query: Self::InnerQuery<'db>,
        clause: String,
        binds: Vec<BindValue>,
    ) -> Self::InnerQuery<'db>;
    fn query_where_exists<'db>(
        query: Self::InnerQuery<'db>,
        clause: String,
        binds: Vec<BindValue>,
    ) -> Self::InnerQuery<'db>;
    fn query_order_raw<'db>(query: Self::InnerQuery<'db>, expr: String) -> Self::InnerQuery<'db>;
    fn query_select_raw<'db>(query: Self::InnerQuery<'db>, expr: String) -> Self::InnerQuery<'db>;
    fn query_join_raw<'db>(
        query: Self::InnerQuery<'db>,
        table: String,
        on_clause: String,
        binds: Vec<BindValue>,
    ) -> Self::InnerQuery<'db>;
}

pub trait QueryField<M: QueryModel>: Copy {
    type Value: Clone + Into<BindValue>;

    fn where_col<'db>(
        field: Self,
        query: M::InnerQuery<'db>,
        op: Op,
        value: Self::Value,
    ) -> M::InnerQuery<'db>;
    fn or_where_col<'db>(
        field: Self,
        query: M::InnerQuery<'db>,
        op: Op,
        value: Self::Value,
    ) -> M::InnerQuery<'db>;
    fn where_in<'db>(
        field: Self,
        query: M::InnerQuery<'db>,
        values: &[Self::Value],
    ) -> M::InnerQuery<'db>;
    fn order_by<'db>(field: Self, query: M::InnerQuery<'db>, dir: OrderDir) -> M::InnerQuery<'db>;
    fn where_null<'db>(field: Self, query: M::InnerQuery<'db>) -> M::InnerQuery<'db>;
    fn where_not_null<'db>(field: Self, query: M::InnerQuery<'db>) -> M::InnerQuery<'db>;
}

pub trait IncludeRelation<M: QueryModel>: Copy {
    fn include<'db>(relation: Self, query: M::InnerQuery<'db>) -> M::InnerQuery<'db>;
}

pub trait WhereHasRelation<M: QueryModel>: Copy {
    type Target: QueryModel;

    fn where_has<'db, F>(relation: Self, query: M::InnerQuery<'db>, scope: F) -> M::InnerQuery<'db>
    where
        F: FnOnce(Query<'db, Self::Target>) -> Query<'db, Self::Target>;

    fn or_where_has<'db, F>(
        relation: Self,
        query: M::InnerQuery<'db>,
        scope: F,
    ) -> M::InnerQuery<'db>
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
    fn foreign_key(relation: Self) -> &'static str;
}

pub trait CreateModel: ModelDef {
    type InnerCreate<'db>: Send + 'db;

    fn create_root<'db>(db: DbConn<'db>, base_url: Option<String>) -> Self::InnerCreate<'db>;
    fn create_save<'db>(builder: Self::InnerCreate<'db>) -> BoxModelFuture<'db, Self::Record>;
}

pub trait CreateField<M: CreateModel>: Copy {
    type Value;

    fn set<'db>(
        field: Self,
        builder: M::InnerCreate<'db>,
        value: Self::Value,
    ) -> Result<M::InnerCreate<'db>>;
}

pub trait CreateConflictField<M: CreateModel>: Copy {
    fn on_conflict_do_nothing<'db>(
        builder: M::InnerCreate<'db>,
        fields: &[Self],
    ) -> M::InnerCreate<'db>;

    fn on_conflict_update<'db>(
        builder: M::InnerCreate<'db>,
        fields: &[Self],
    ) -> M::InnerCreate<'db>;
}

pub trait PatchModel: ModelDef {
    type InnerQuery<'db>: Clone + Send + 'db;
    type InnerPatch<'db>: Send + 'db;

    fn patch_root<'db>(db: DbConn<'db>, base_url: Option<String>) -> Self::InnerPatch<'db>;
    fn patch_from_query<'db>(query: Self::InnerQuery<'db>) -> Self::InnerPatch<'db>;
    fn patch_save<'db>(builder: Self::InnerPatch<'db>) -> BoxModelFuture<'db, u64>;
    fn patch_fetch<'db>(builder: Self::InnerPatch<'db>) -> BoxModelFuture<'db, Vec<Self::Record>>;
}

pub trait PatchAssignField<M: PatchModel>: Copy {
    type Value;

    fn assign<'db>(
        field: Self,
        builder: M::InnerPatch<'db>,
        value: Self::Value,
    ) -> Result<M::InnerPatch<'db>>;
}

pub trait PatchNumericField<M: PatchModel>: PatchAssignField<M> {
    fn increment<'db>(
        field: Self,
        builder: M::InnerPatch<'db>,
        value: Self::Value,
    ) -> Result<M::InnerPatch<'db>>;
    fn decrement<'db>(
        field: Self,
        builder: M::InnerPatch<'db>,
        value: Self::Value,
    ) -> Result<M::InnerPatch<'db>>;
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
    foreign_key: &'static str,
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
        foreign_key: &'static str,
    ) -> Self {
        Self {
            name,
            target_table,
            foreign_key,
            _marker: PhantomData,
        }
    }

    pub const fn name(self) -> &'static str {
        self.name
    }

    pub const fn target_table(self) -> &'static str {
        self.target_table
    }

    pub const fn foreign_key(self) -> &'static str {
        self.foreign_key
    }
}

impl<M: ModelDef, T, const KEY: usize> CountRelation<M> for ManyRelation<M, T, KEY> {
    fn name(relation: Self) -> &'static str {
        relation.name()
    }

    fn target_table(relation: Self) -> &'static str {
        relation.target_table()
    }

    fn foreign_key(relation: Self) -> &'static str {
        relation.foreign_key()
    }
}

pub struct Query<'db, M: QueryModel> {
    inner: M::InnerQuery<'db>,
}

impl<'db, M: QueryModel> Clone for Query<'db, M> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<'db, M: QueryModel> Query<'db, M> {
    pub fn new(db: impl Into<DbConn<'db>>) -> Self {
        Self {
            inner: M::query_root(db.into(), None),
        }
    }

    pub fn new_with_base_url(db: impl Into<DbConn<'db>>, base_url: Option<String>) -> Self {
        Self {
            inner: M::query_root(db.into(), base_url),
        }
    }

    pub fn from_inner(inner: M::InnerQuery<'db>) -> Self {
        Self { inner }
    }

    pub fn into_inner(self) -> M::InnerQuery<'db> {
        self.inner
    }

    pub fn where_col<F, V>(self, field: F, op: Op, value: V) -> Self
    where
        F: QueryField<M>,
        V: Into<F::Value>,
    {
        Self {
            inner: F::where_col(field, self.inner, op, value.into()),
        }
    }

    pub fn or_where_col<F, V>(self, field: F, op: Op, value: V) -> Self
    where
        F: QueryField<M>,
        V: Into<F::Value>,
    {
        Self {
            inner: F::or_where_col(field, self.inner, op, value.into()),
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
            inner: F::where_in(field, self.inner, &values),
        }
    }

    pub fn where_null<F>(self, field: F) -> Self
    where
        F: QueryField<M>,
    {
        Self {
            inner: F::where_null(field, self.inner),
        }
    }

    pub fn where_not_null<F>(self, field: F) -> Self
    where
        F: QueryField<M>,
    {
        Self {
            inner: F::where_not_null(field, self.inner),
        }
    }

    pub fn limit(self, limit: i64) -> Self {
        Self {
            inner: M::query_limit(self.inner, limit),
        }
    }

    pub fn offset(self, offset: i64) -> Self {
        Self {
            inner: M::query_offset(self.inner, offset),
        }
    }

    pub fn for_update(self) -> Self {
        Self {
            inner: M::query_for_update(self.inner),
        }
    }

    pub fn for_update_skip_locked(self) -> Self {
        Self {
            inner: M::query_for_update_skip_locked(self.inner),
        }
    }

    pub fn for_no_key_update(self) -> Self {
        Self {
            inner: M::query_for_no_key_update(self.inner),
        }
    }

    pub fn where_group<F>(self, scope: F) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        Self {
            inner: M::query_where_group(self.inner, scope),
        }
    }

    pub fn or_where_group<F>(self, scope: F) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        Self {
            inner: M::query_or_where_group(self.inner, scope),
        }
    }

    pub fn order_by<F>(self, field: F, dir: OrderDir) -> Self
    where
        F: QueryField<M>,
    {
        Self {
            inner: F::order_by(field, self.inner, dir),
        }
    }

    pub fn with<R>(self, relation: R) -> Self
    where
        R: IncludeRelation<M>,
    {
        Self {
            inner: R::include(relation, self.inner),
        }
    }

    pub fn where_has<R, F>(self, relation: R, scope: F) -> Self
    where
        R: WhereHasRelation<M>,
        F: FnOnce(Query<'db, R::Target>) -> Query<'db, R::Target>,
    {
        Self {
            inner: R::where_has(relation, self.inner, scope),
        }
    }

    pub fn or_where_has<R, F>(self, relation: R, scope: F) -> Self
    where
        R: WhereHasRelation<M>,
        F: FnOnce(Query<'db, R::Target>) -> Query<'db, R::Target>,
    {
        Self {
            inner: R::or_where_has(relation, self.inner, scope),
        }
    }

    pub fn unsafe_sql(self) -> UnsafeQuery<'db, M>
    where
        M: UnsafeQueryModel,
    {
        UnsafeQuery { inner: self }
    }

    pub fn where_exists_raw<T>(
        self,
        clause: impl Into<String>,
        binds: impl IntoIterator<Item = T>,
    ) -> Self
    where
        M: UnsafeQueryModel,
        T: Into<BindValue>,
    {
        Self {
            inner: M::query_where_exists(
                self.inner,
                clause.into(),
                binds.into_iter().map(Into::into).collect(),
            ),
        }
    }

    pub async fn all(self) -> Result<Vec<M::Record>> {
        M::query_all(self.inner).await
    }

    pub async fn first(self) -> Result<Option<M::Record>> {
        M::query_first(self.inner).await
    }

    pub async fn find(self, id: M::Pk) -> Result<Option<M::Record>> {
        M::query_find(self.inner, id).await
    }

    pub async fn count(self) -> Result<i64> {
        M::query_count(self.inner).await
    }

    pub async fn delete(self) -> Result<u64> {
        M::query_delete(self.inner).await
    }

    pub async fn paginate(self, page: i64, per_page: i64) -> Result<Page<M::Record>> {
        M::query_paginate(self.inner, page, per_page).await
    }

    pub fn patch(self) -> Patch<'db, M>
    where
        M: PatchModel<InnerQuery<'db> = <M as QueryModel>::InnerQuery<'db>>,
    {
        Patch {
            inner: M::patch_from_query(self.inner),
        }
    }
}

pub struct UnsafeQuery<'db, M: UnsafeQueryModel> {
    inner: Query<'db, M>,
}

impl<'db, M: UnsafeQueryModel> UnsafeQuery<'db, M> {
    pub fn where_raw<T>(self, clause: impl Into<String>, binds: impl IntoIterator<Item = T>) -> Self
    where
        T: Into<BindValue>,
    {
        Self {
            inner: Query::from_inner(M::query_where_raw(
                self.inner.into_inner(),
                clause.into(),
                binds.into_iter().map(Into::into).collect(),
            )),
        }
    }

    pub fn order_raw(self, expr: impl Into<String>) -> Self {
        Self {
            inner: Query::from_inner(M::query_order_raw(self.inner.into_inner(), expr.into())),
        }
    }

    pub fn select_raw(self, expr: impl Into<String>) -> Self {
        Self {
            inner: Query::from_inner(M::query_select_raw(self.inner.into_inner(), expr.into())),
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
            inner: Query::from_inner(M::query_join_raw(
                self.inner.into_inner(),
                table.into(),
                on_clause.into(),
                binds.into_iter().map(Into::into).collect(),
            )),
        }
    }

    pub fn done(self) -> Query<'db, M> {
        self.inner
    }
}

pub struct Create<'db, M: CreateModel> {
    inner: M::InnerCreate<'db>,
}

impl<'db, M: CreateModel> Create<'db, M> {
    pub fn new(db: impl Into<DbConn<'db>>) -> Self {
        Self {
            inner: M::create_root(db.into(), None),
        }
    }

    pub fn new_with_base_url(db: impl Into<DbConn<'db>>, base_url: Option<String>) -> Self {
        Self {
            inner: M::create_root(db.into(), base_url),
        }
    }

    pub fn from_inner(inner: M::InnerCreate<'db>) -> Self {
        Self { inner }
    }

    pub fn into_inner(self) -> M::InnerCreate<'db> {
        self.inner
    }

    pub fn set<F, V>(self, field: F, value: V) -> Result<Self>
    where
        F: CreateField<M>,
        V: Into<F::Value>,
    {
        Ok(Self {
            inner: F::set(field, self.inner, value.into())?,
        })
    }

    pub fn on_conflict_do_nothing<F>(self, fields: &[F]) -> Self
    where
        F: CreateConflictField<M>,
    {
        Self {
            inner: F::on_conflict_do_nothing(self.inner, fields),
        }
    }

    pub fn on_conflict_update<F>(self, fields: &[F]) -> Self
    where
        F: CreateConflictField<M>,
    {
        Self {
            inner: F::on_conflict_update(self.inner, fields),
        }
    }

    pub async fn save(self) -> Result<M::Record> {
        M::create_save(self.inner).await
    }
}

pub struct Patch<'db, M: PatchModel> {
    inner: M::InnerPatch<'db>,
}

impl<'db, M: PatchModel> Patch<'db, M> {
    pub fn new(db: impl Into<DbConn<'db>>) -> Self {
        Self {
            inner: M::patch_root(db.into(), None),
        }
    }

    pub fn new_with_base_url(db: impl Into<DbConn<'db>>, base_url: Option<String>) -> Self {
        Self {
            inner: M::patch_root(db.into(), base_url),
        }
    }

    pub fn from_inner(inner: M::InnerPatch<'db>) -> Self {
        Self { inner }
    }

    pub fn into_inner(self) -> M::InnerPatch<'db> {
        self.inner
    }

    pub fn assign<F, V>(self, field: F, value: V) -> Result<Self>
    where
        F: PatchAssignField<M>,
        V: Into<F::Value>,
    {
        Ok(Self {
            inner: F::assign(field, self.inner, value.into())?,
        })
    }

    pub fn increment<F, V>(self, field: F, value: V) -> Result<Self>
    where
        F: PatchNumericField<M>,
        V: Into<F::Value>,
    {
        Ok(Self {
            inner: F::increment(field, self.inner, value.into())?,
        })
    }

    pub fn decrement<F, V>(self, field: F, value: V) -> Result<Self>
    where
        F: PatchNumericField<M>,
        V: Into<F::Value>,
    {
        Ok(Self {
            inner: F::decrement(field, self.inner, value.into())?,
        })
    }

    pub async fn save(self) -> Result<u64> {
        M::patch_save(self.inner).await
    }

    pub async fn fetch(self) -> Result<Vec<M::Record>> {
        M::patch_fetch(self.inner).await
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
        }
    }

    // ── WHERE helpers ──────────────────────────────────────────────────

    pub fn where_col_str(mut self, col_sql: &str, op: Op, val: BindValue) -> Self {
        let idx = self.binds.len() + 1;
        self.where_sql
            .push(format!("{} {} ${}", col_sql, op.as_sql(), idx));
        self.binds.push(val);
        self
    }

    pub fn or_where_col_str(mut self, col_sql: &str, op: Op, val: BindValue) -> Self {
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
                result.where_sql.push(format!("({} OR {})", last, grouped_sql));
            } else {
                result.where_sql.push(grouped_sql);
            }
        }
        result
    }

    // ── ORDER BY ───────────────────────────────────────────────────────

    pub fn order_by_str(mut self, col_sql: &str, dir: OrderDir) -> Self {
        self.order_sql
            .push(format!("{} {}", col_sql, dir.as_sql()));
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
        let select_clause = Self::build_select_clause(
            self.distinct,
            self.distinct_on.as_deref(),
            self.select_sql,
        );
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
}

// ---------------------------------------------------------------------------
// CreateState — shared INSERT builder state (replaces per-model XxxCreateInner)
// ---------------------------------------------------------------------------

pub struct CreateState<'db> {
    pub(crate) db: DbConn<'db>,
    pub(crate) base_url: Option<String>,
    pub(crate) table: &'static str,
    pub(crate) col_names: Vec<&'static str>,
    pub(crate) binds: Vec<BindValue>,
    pub(crate) conflict_action: Option<&'static str>,
    pub(crate) conflict_cols: Vec<&'static str>,
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
        let placeholders: Vec<String> = (1..=self.binds.len())
            .map(|i| format!("${}", i))
            .collect();
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
    pub(crate) db: DbConn<'db>,
    pub(crate) base_url: Option<String>,
    pub(crate) table: &'static str,
    pub(crate) set_cols: Vec<&'static str>,
    pub(crate) set_binds: Vec<BindValue>,
    pub(crate) set_modes: Vec<SetMode>,
    pub(crate) where_sql: Vec<String>,
    pub(crate) where_binds: Vec<BindValue>,
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
