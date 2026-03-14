use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;

use crate::common::sql::{BindValue, DbConn, Op, OrderDir};

pub type BoxModelFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T>> + Send + 'a>>;

pub trait ModelDef: Sized + 'static {
    type Pk: Clone + Send + Sync + 'static;
    type Record: Clone + Send + Sync + 'static;
    type Create: Send + 'static;
    type Changes: Send + 'static;

    const TABLE: &'static str;
    const MODEL_KEY: &'static str;
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
