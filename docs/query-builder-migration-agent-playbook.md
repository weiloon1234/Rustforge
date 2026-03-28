# Query Builder Migration Agent Playbook

Use this document when migrating existing framework or app code onto the current Rustforge query/runtime model.

This is for AI agents and engineers doing migration work, not for end-user model authoring.

## Purpose And Invariants

Goal:
- migrate code, tests, docs, and generated expectations onto the current query/runtime system
- keep behavior correct
- keep builds and tests warning-free
- preserve relation semantics:
  - unloaded `BelongsTo` / `HasOne` => `None`
  - unloaded `HasMany` => `[]`

Hard rules:
- do not touch old migration files
- do not hand-edit generated files
- do not hide unfinished work
- do not assume a stale old pattern is “close enough”
- do not suppress warnings except existing unused-family cases already allowed by the framework

## Mental Model Of The Current Runtime

The important part is the engine underneath:
- the normal typed query path is AST-driven
- relation loads are tree-based
- relation existence (`where_has`) is tree-based
- relation counts and aggregates are tree-based
- runtime nesting is not capped at a fixed depth

Current read model:
- root queries use typed query builder methods
- relation loads use `.with(Rel::X)` and `.with_scope(Rel::X, |q| ...)`
- relation existence uses `.where_has(Rel::X, |q| ...)`
- relation counts and aggregates use `.with_count(...)`, `.with_*_scope(...)`, and typed record readers
- row locking uses typed lock helpers instead of raw SQL suffixes
- selected updates and claim flows build on `patch_selected()` / `claim()`

Important semantics:
- relations are opt-in loaded
- `where_has(... |q| q.with(...))` means nested existence, not eager load
- `unsafe_sql` still exists as an escape hatch, but normal migration should stay on the typed path unless the typed path genuinely cannot express the query

## Required Migration Patterns

### Root Query Construction

Before:

```rust
let rows = Model::query(db)
    .where_col(ModelCol::STATUS, Op::Eq, Status::Active)
    .all()
    .await?;
```

After:

```rust
let rows = Model::query()
    .where_col(ModelCol::STATUS, Op::Eq, Status::Active)
    .all(db)
    .await?;
```

Also update:
- `first()` -> `first(db)`
- `find(id)` -> `find(db, id)`
- `count()` -> `count(db)`
- `paginate(page, per_page)` -> `paginate(db, page, per_page)`
- `delete()` -> `delete(db)`

Add new typed predicates when they replace raw SQL:
- `where_col_cmp(lhs, op, rhs)`
- `where_expr_cmp(col, op, Expr::now_minus(...))`

### Create / Patch / Save

Before:

```rust
Model::create(db)
    .set(...)
    .save()
    .await?;
```

After:

```rust
Model::create()
    .set(...)
    .save(db)
    .await?;
```

Before:

```rust
Model::query(db)
    .where_col(...)
    .patch()
    .assign(...)
    .save()
    .await?;
```

After:

```rust
Model::query()
    .where_col(...)
    .patch()
    .assign(...)
    .save(db)
    .await?;
```

Selected update:

```rust
Model::query()
    .where_col(...)
    .limit(50)
    .patch_selected()
    .assign(...)
    .save(db)
    .await?;
```

Atomic claim:

```rust
let ids: Vec<i64> = Model::query()
    .where_col(...)
    .for_update()
    .skip_locked()
    .limit(50)
    .claim()
    .assign(...)
    .returning(ModelCol::ID)
    .fetch_scalars(db)
    .await?;
```

Returning variants:
- `.returning(ModelCol::ID).fetch_scalars(db)` for one typed column
- `.returning_many([ModelCol::ID.into(), ModelCol::STATUS.into()]).fetch_json(db)` for mixed columns
- `.returning_raw("id").fetch_json(db)` when a raw returning expression is genuinely needed
- `.returning_all().fetch(db)` for full updated records

### Relation Loading

Before old helper style:

```rust
query.with_author().with_comments()
```

After:

```rust
query.with(ArticleRel::AUTHOR)
     .with(ArticleRel::COMMENTS)
```

For filtered or shaped relation loads:

```rust
query.with_scope(UserRel::DOWNLINES, |q| {
    q.where_col(UserCol::BAN, Op::Eq, UserBanStatus::No)
        .order_by(UserCol::CREATED_AT, OrderDir::Desc)
        .limit(5)
})
```

### Row Locking

Old raw SQL suffixes should migrate to typed lock helpers:

```rust
query.for_update().skip_locked()
query.for_no_key_update().no_wait()
query.for_share()
query.for_key_share()
```

Rules:
- `skip_locked()` and `no_wait()` require a lock mode first
- `claim()` requires `for_update()` or `for_no_key_update()`

### Chunk Processing

Use `chunk()` for simple read iteration:

```rust
query.chunk(db, 500, |rows| async move {
    // process rows
    Ok(true)
}).await?;
```

Use `chunk_by_id()` for mutable or lock-sensitive processing:

```rust
let scope = DbConn::pool(db).begin_scope().await?;
let conn = scope.conn();

query
    .for_update()
    .skip_locked()
    .chunk_by_id(conn, 100, |rows| async move {
        // process rows while the transaction keeps locks alive
        Ok(true)
    })
    .await?;

scope.commit().await?;
```

Rules:
- `chunk()` rejects row locks
- `chunk_by_id()` always pages by primary key ascending
- `chunk_by_id()` rejects custom order and pre-set limit/offset
- `chunk_by_id()` with row locks requires `DbConn::Tx`; pool-backed statements release locks before the callback runs

### Nested Relations

Use nested `.with(...)` / `.with_scope(...)` inside relation scopes:

```rust
query.with_scope(UserRel::DOWNLINES, |q| {
    q.with(UserRel::INTRODUCER)
        .with_scope(UserRel::DOWNLINES, |q| {
            q.order_by(UserCol::CREATED_AT, OrderDir::Desc).limit(3)
        })
})
```

### `where_has(...)`

Before old helper style:

```rust
query.where_has_author(|q| ...)
```

After:

```rust
query.where_has(ArticleRel::AUTHOR, |q| {
    q.where_col(UserCol::IS_ACTIVE, Op::Eq, true)
})
```

Nested existence:

```rust
query.where_has(UserRel::DOWNLINES, |q| {
    q.with(UserRel::INTRODUCER)
})
```

Inside `where_has(...)`, do not use:
- relation counts
- relation aggregates
- scoped limit/offset
- explicit relation select/projection

Those are invalid there and should fail explicitly.

### Relation Counts And Aggregates

Before:

```rust
record.__relation_counts.get("groups.items")
record.aggregate("sum:downlines:credit_1")
```

After:

```rust
record.count(UserRel::DOWNLINES)
record.sum(UserRel::DOWNLINES, UserCol::CREDIT_1)
record.avg(UserRel::DOWNLINES, UserCol::CREDIT_1)
record.min(UserRel::DOWNLINES, UserCol::CREDIT_1)
record.max(UserRel::DOWNLINES, UserCol::CREDIT_1)
```

Prefer the typed readers above over `record.aggregate("sum:downlines:credit_1")`.
Keep `aggregate(key)` for internal or advanced cases that do not have a typed helper.

### Column Comparison, Time Expressions, And Partial Raw Clauses

Prefer typed clauses first:

```rust
query.where_col_cmp(
    MessageCol::SEND_ATTEMPT_COUNT,
    Op::Lt,
    MessageCol::MAX_SEND_ATTEMPTS,
)
```

```rust
query.where_expr_cmp(
    MessageCol::FAILED_AT,
    Op::Lt,
    Expr::now_minus(time::Duration::seconds(30)),
)
```

If one clause still cannot be expressed, keep the rest of the chain typed and use a clause-level raw escape hatch:

```rust
use core_db::common::sql::RawClause;

query
    .where_col(MessageCol::DIRECTION, Op::Eq, MessageDirection::Outbound)
    .where_raw(RawClause::new("send_attempt_count < max_send_attempts", [])?)
```

Available partial raw escapes:
- `where_raw(...)`
- `select_raw(...)`
- `order_by_raw(...)`
- `join_raw(...)`
- `group_by_raw(...)`
- `having_raw(...)`
- `returning_raw(...)`

Only use `unsafe_sql()` when the query shape itself cannot stay on the typed path.

Custom aggregate expressions:

```rust
query.with_sum_expr(
    UserRel::DOWNLINES,
    AggregateTarget::<UserModel>::expr("COALESCE(credit_1 + credit_2, 0)")
)
```

### `BelongsTo` / `HasOne` / `HasMany`

Access expectations:
- `BelongsTo` and `HasOne` are singular and load into `Option<Box<T>>`
- `HasMany` is plural and loads into `Vec<T>`

If a relation is accessed in app code or datatable mapping:
- ensure the query has an explicit `.with(...)` / `.with_scope(...)`

### Datatable `scope()` Updates

Every relation touched by `row_to_record` must be loaded in `scope()`.

```rust
fn scope<'db>(&'db self, query: Query<'db, MyModel>, ..) -> Query<'db, MyModel> {
    query
        .with(MyRel::USER)
        .with_scope(MyRel::ITEMS, |q| {
            q.order_by(ItemCol::CREATED_AT, OrderDir::Desc).limit(5)
        })
}
```

Do not rely on implicit loading.

## Code And Test Updates Required

When migrating a feature area, update all relevant consumers:
- workflows
- datatables
- API handlers if they construct queries directly
- contracts if computed/read fields changed
- generator fixtures when generated output intentionally changes
- docs/examples that still mention old query styles

If a model/query migration changes:
- relation loading shape
- generated read methods
- aggregate access
- lock/claim patterns
- patch returning behavior

then update:
- checked-in generated fixture files
- scaffold smoke expectations
- any user-facing skill or guide that shows the old pattern

## Strict Do / Don’t Rules

Do:
- use the current Rustforge API only
- keep migrations append-only
- keep model files as SSOT
- use typed relation constants (`Rel::X`)
- use typed aggregate readers where available
- keep builds and tests warning-free
- verify generated consumer crates after framework changes

Don’t:
- do not touch migration history
- do not hand-edit generated code
- do not hide unfinished work
- do not leave old helper API usage in docs/tests
- do not access unloaded relations without adding `.with(...)`
- do not use relation counts/aggregates inside `where_has(...)`
- do not convert DB model PK fields to `SnowflakeId`
- do not assume a relation is loaded because a previous version of the framework did so
- do not keep raw queue-claim SQL if the typed lock/claim path now expresses it cleanly
- do not call `skip_locked()` / `no_wait()` without a lock mode

## Verification Checklist

A migration is not complete unless all relevant checks pass.

Required framework checks:

```bash
cargo test -p core-db
cargo test -p db-gen
make scaffold-template-clean && cargo test -p scaffold
```

Required local-patched starter checks:

```bash
cargo check -p rustforge-starter-generated --config 'patch."https://github.com/weiloon1234/Rustforge.git".bootstrap.path="/abs/bootstrap"' ...
cargo check -p rustforge-starter --config 'patch."https://github.com/weiloon1234/Rustforge.git".bootstrap.path="/abs/bootstrap"' ...
```

If generated output changed intentionally:

```bash
UPDATE_DB_GEN_FIXTURES=1 cargo test -p db-gen --test template_generation
```

If model/contract TS exports changed:

```bash
make gen
cargo run -p app --bin export-types
```

## Common Failure Signatures And Fixes

### Relation unexpectedly `None` or `[]`

Cause:
- query forgot `.with(...)` / `.with_scope(...)`

Fix:
- add the explicit relation load in the query or datatable `scope()`

### Stale old helper API usage

Examples:
- `with_author()`
- `where_has_author(...)`
- `get_with_relations()`
- `get()`

Fix:
- replace with `with(Rel::X)`, `where_has(Rel::X, ...)`, `all(db)`, `find(db, id)`, `paginate(db, ...)`

### Generated fixture drift

Cause:
- framework generation changed intentionally

Fix:
- refresh with `UPDATE_DB_GEN_FIXTURES=1 cargo test -p db-gen --test template_generation`
- verify the diff is intended before keeping it

### Trait import / generated const access issues

Examples:
- trait-provided associated items referenced as if inherent
- generated code assuming old helper imports

Fix:
- use fully qualified trait paths in generated code when needed
- keep checked-in fixtures in sync with the current generator

### Invalid relation scope usage inside `where_has(...)`

Cause:
- using eager-load-only features in an existence predicate

Fix:
- move that logic to `.with(...)` / `.with_scope(...)` on the main query
- keep `where_has(...)` focused on parent filtering only

### Claim/update builder fails with “no conditions set”

Cause:
- selected update path lost its source query during migration

Fix:
- use `patch_selected()` or `claim()` from the built query
- do not rebuild the update from a blank `Patch::new()` if the source is a selected query

### Lock modifier error

Cause:
- `skip_locked()` or `no_wait()` was called before `for_update()` / `for_no_key_update()` / share lock

Fix:
- attach the modifier after a lock mode

### Claim lock error

Cause:
- `claim()` used a share lock or no lock

Fix:
- use `for_update()` or `for_no_key_update()` before `claim()`

### Relation unexpectedly filtered or ordered differently than intended

Cause:
- model-defined relation scope merged with runtime scope

Fix:
- check the relation field for `scope = ...`
- check `#[rf_model_impl]` for the relation scope function
- remember relation join + default scope + runtime scope all merge together
