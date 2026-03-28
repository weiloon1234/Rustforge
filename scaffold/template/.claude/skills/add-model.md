---
name: add-model
description: Author and use Rustforge models with the current SSOT model style and query runtime
---

# Add Or Extend A Model

Use this when changing `app/models/*.rs`.

The handwritten model file is the single source of truth for:
- table name
- database fields
- enum storage
- relationships
- scoped relationships
- computed record fields
- record helper methods
- model/query scopes

Do not hand-edit generated files.

## Current Model Split

Keep the current Rustforge authoring style:
- `#[rf_model(...)]` on the model struct
- relation fields directly on the struct
- `#[rf_record_impl]` for computed fields and record helpers
- `#[rf_model_impl]` for query scopes and relation scopes

One `#[rf_model]` per file.

The same file may also contain:
- enums used by the model
- small helper structs
- helper free functions

## Canonical Shape

```rust
#[rf_db_enum(storage = "i16")]
pub enum UserBanStatus {
    No = 0,
    Yes = 1,
}

#[rf_model(table = "users")]
pub struct User {
    #[rf(pk(strategy = snowflake))]
    pub id: i64,

    pub uuid: String,
    pub username: String,
    pub email: Option<String>,
    pub introducer_user_id: Option<i64>,
    pub ban: UserBanStatus,
    pub credit_1: rust_decimal::Decimal,
    pub created_at: time::OffsetDateTime,
    pub updated_at: time::OffsetDateTime,

    #[rf(foreign_key = "introducer_user_id")]
    pub introducer: BelongsTo<User>,

    #[rf(foreign_key = "introducer_user_id")]
    pub downlines: HasMany<User>,

    #[rf(foreign_key = "introducer_user_id", scope = active_downlines_scope)]
    pub active_downlines: HasMany<User>,
}

#[rf_record_impl]
impl UserRecord {
    #[rf_computed]
    pub fn display_name(&self) -> String {
        self.username.clone()
    }

    pub fn has_credit(&self) -> bool {
        self.credit_1 > rust_decimal::Decimal::ZERO
    }
}

#[rf_model_impl]
impl UserModel {
    pub fn active(query: Query<UserModel>) -> Query<UserModel> {
        query.where_col(UserCol::BAN, Op::Eq, UserBanStatus::No)
    }

    pub fn active_downlines_scope(query: Query<UserModel>) -> Query<UserModel> {
        query.where_col(UserCol::BAN, Op::Eq, UserBanStatus::No)
    }
}
```

## Table And Struct Rules

- Always declare the table explicitly: `#[rf_model(table = "...")]`.
- Use one model struct per file.
- Match DB column nullability exactly with `Option<T>`.
- Use `time::OffsetDateTime` for timestamps.
- Use `rust_decimal::Decimal` for money, fee, rate, credit, and balance-like values.
- Use `serde_json::Value` for JSON blobs stored in DB columns.
- Use `#[rf(hashed)]` for persisted password hashes.

## Primary Key And Snowflake Rules

Snowflake DB primary keys should be written like this:

```rust
#[rf(pk(strategy = snowflake))]
pub id: i64,
```

Important:
- DB model PK fields still use `i64` today.
- The framework generates the snowflake ID.
- Do not write model PK fields as `SnowflakeId`.

Use `core_web::ids::SnowflakeId` in:
- contracts
- API DTOs
- datatable rows
- request path/query params
- any frontend-exported ID surface

Use `i64` in:
- `#[rf_model]` DB structs
- direct DB relation FK fields
- migration column definitions

For migrations:
- use `BIGINT` for snowflake PK/FK columns
- do not use `BIGSERIAL` for snowflake-generated IDs

## Enum Rules

Declare model-owned enums in the same model file.

```rust
#[rf_db_enum(storage = "string")]
pub enum DepositStatus {
    Pending,
    Approved,
    Rejected,
}
```

Supported enum storage in this codebase:
- `storage = "string"`
- `storage = "i16"`

Enum behavior is generated and affects:
- DB persistence
- explained labels
- filters/datatables
- frontend type generation

Preserve enum names and storage carefully.

## Relationship Rules

Supported relationship field types:
- `BelongsTo<T>`
- `HasOne<T>`
- `HasMany<T>`

Declare relationships directly on the model struct.

```rust
#[rf(foreign_key = "user_id")]
pub profile: HasOne<Profile>,

#[rf(foreign_key = "country_iso2", local_key = "country_iso2")]
pub country: BelongsTo<Country>,

#[rf(foreign_key = "user_id")]
pub orders: HasMany<Order>,
```

Use:
- `foreign_key` when the default inferred FK is not enough
- `local_key` when the source-side link field is not the default
- `scope = some_scope_name` for a conditional relationship

### Conditional Relationships

Scoped relationships define default conditions on the relation itself.

```rust
#[rf(foreign_key = "user_id", scope = paid_orders_scope)]
pub paid_orders: HasMany<Order>,

#[rf_model_impl]
impl UserModel {
    pub fn paid_orders_scope(query: Query<OrderModel>) -> Query<OrderModel> {
        query.where_col(OrderCol::STATUS, Op::Eq, OrderStatus::Paid)
    }
}
```

Final effective relation filter is:
- relation join condition
- plus model-defined relation scope
- plus runtime `with_scope(...)`
- plus runtime `with_count_scope(...)`
- plus runtime aggregate scope

All merge with `AND`.

## `#[rf_record_impl]`: Computed Fields And Record Helpers

Use `#[rf_record_impl]` for:
- `#[rf_computed]` fields
- lightweight record helpers
- formatting helpers
- convenience methods that only need the hydrated record

```rust
#[rf_record_impl]
impl DepositRecord {
    #[rf_computed]
    pub fn status_badge(&self) -> String {
        self.status.explained_label()
    }

    pub fn is_pending(&self) -> bool {
        matches!(self.status, DepositStatus::Pending)
    }
}
```

Rules:
- `#[rf_computed]` belongs on `XxxRecord`, not `XxxModel`
- computed methods must take `&self` or `self`
- computed methods must not take extra arguments
- computed methods must return a value

## `#[rf_model_impl]`: Query Scopes And Relation Scopes

Use `#[rf_model_impl]` for:
- reusable query scopes
- relation scope functions
- model-level static helpers

```rust
#[rf_model_impl]
impl WithdrawalModel {
    pub fn pending(query: Query<WithdrawalModel>) -> Query<WithdrawalModel> {
        query.where_col(WithdrawalCol::STATUS, Op::Eq, WithdrawalStatus::Pending)
    }

    pub fn reviewed(query: Query<WithdrawalModel>) -> Query<WithdrawalModel> {
        query.where_not_null(WithdrawalCol::REVIEWED_AT)
    }
}
```

## Query Usage

Use the current query surface only:
- `Model::query()`
- `where_col(...)`
- `where_col_cmp(...)`
- `where_expr_cmp(...)`
- `order_by(...)`
- `with(Rel::X)`
- `with_scope(Rel::X, |q| ...)`
- `where_has(Rel::X, |q| ...)`
- `all(db)`, `first(db)`, `find(db, id)`, `paginate(db, page, per_page)`

Do not use stale helper styles like:
- `with_author()`
- `where_has_author(...)`
- `get_with_relations()`
- `get()`

### Relation Loading

Relations are opt-in loaded.

No `.with(...)` means:
- `BelongsTo` / `HasOne` stays `None`
- `HasMany` stays `[]`

```rust
let users = UserModel::query()
    .with(UserRel::INTRODUCER)
    .with(UserRel::DOWNLINES)
    .all(db)
    .await?;
```

### Nested Eager Loading

Nested relation trees are supported without a fixed runtime depth cap.

```rust
let users = UserModel::query()
    .with_scope(UserRel::DOWNLINES, |q| {
        q.with(UserRel::INTRODUCER)
            .with_scope(UserRel::DOWNLINES, |q| {
                q.order_by(UserCol::CREATED_AT, OrderDir::Desc).limit(3)
            })
    })
    .all(db)
    .await?;
```

### Scoped Order / Limit / Offset / Select

Use `with_scope(...)` for conditional or shaped relation loads.

```rust
let user = UserModel::query()
    .with_scope(UserRel::DOWNLINES, |q| {
        q.where_col(UserCol::BAN, Op::Eq, UserBanStatus::No)
            .order_by(UserCol::CREATED_AT, OrderDir::Desc)
            .limit(5)
    })
    .find(db, user_id)
    .await?;
```

Current behavior:
- `HasMany`: scoped order / limit / offset apply per parent
- `HasOne`: scoped order / limit / offset also apply per parent, but the loaded value is singular
- `BelongsTo`: scoped order / select/filter are fine, but scoped `limit/offset` is invalid and will fail explicitly

### `where_has(...)` And Nested Existence

Use `where_has(...)` to filter parent rows by related rows.

```rust
let users = UserModel::query()
    .where_has(UserRel::DOWNLINES, |q| {
        q.where_col(UserCol::BAN, Op::Eq, UserBanStatus::No)
    })
    .all(db)
    .await?;
```

Nested existence is also supported:

```rust
let users = UserModel::query()
    .where_has(UserRel::DOWNLINES, |q| {
        q.with(UserRel::INTRODUCER)
    })
    .all(db)
    .await?;
```

Inside `where_has(...)`:
- nested `with(...)` / `with_scope(...)` means nested existence, not eager load
- relation counts/aggregates are invalid there
- scoped relation `limit/offset` is invalid there
- explicit relation select/projection is invalid there

If you need actual relation data loaded, also add `.with(...)` / `.with_scope(...)` on the main query.

### Counts And Aggregates

Count or aggregate related rows without loading the full relation.

```rust
let users = UserModel::query()
    .with_count(UserRel::DOWNLINES)
    .with_count_scope(UserRel::DOWNLINES, |q| {
        q.where_col(UserCol::BAN, Op::Eq, UserBanStatus::No)
    })
    .with_sum(UserRel::DOWNLINES, UserCol::CREDIT_1)
    .with_avg_scope(UserRel::DOWNLINES, UserCol::CREDIT_1, |q| {
        q.where_col(UserCol::BAN, Op::Eq, UserBanStatus::No)
    })
    .all(db)
    .await?;

let count = users[0].count(UserRel::DOWNLINES);
let sum = users[0].sum(UserRel::DOWNLINES, UserCol::CREDIT_1);
let avg = users[0].avg(UserRel::DOWNLINES, UserCol::CREDIT_1);
let min = users[0].min(UserRel::DOWNLINES, UserCol::CREDIT_1);
let max = users[0].max(UserRel::DOWNLINES, UserCol::CREDIT_1);
```

Prefer the typed readers above over `record.aggregate("sum:downlines:credit_1")`.
Keep `aggregate(key)` for internal or advanced cases that do not have a typed helper.

Custom aggregate expressions are also supported:

```rust
let users = UserModel::query()
    .with_sum_expr(
        UserRel::DOWNLINES,
        AggregateTarget::<UserModel>::expr("COALESCE(credit_1 + credit_2, 0)")
    )
    .all(db)
    .await?;
```

## Datatable Expectations

Datatables must explicitly load every relation used by `row_to_record`.

Put the `.with(...)` / `.with_scope(...)` calls in the datatable `scope()` hook.

```rust
fn scope<'db>(&'db self, query: Query<'db, MyModel>, ..) -> Query<'db, MyModel> {
    query
        .with(MyRel::USER)
        .with_scope(MyRel::ITEMS, |q| {
            q.order_by(ItemCol::CREATED_AT, OrderDir::Desc).limit(5)
        })
}
```

### Column Comparison And Time Expressions

Use column-to-column comparison when the predicate depends on two DB columns:

```rust
let ready = MessageModel::query()
    .where_col_cmp(
        MessageCol::SEND_ATTEMPT_COUNT,
        Op::Lt,
        MessageCol::MAX_SEND_ATTEMPTS,
    )
    .all(db)
    .await?;
```

Use typed time expressions for “now plus/minus duration” comparisons:

```rust
let retryable = MessageModel::query()
    .where_expr_cmp(
        MessageCol::FAILED_AT,
        Op::Lt,
        Expr::now_minus(time::Duration::seconds(30)),
    )
    .all(db)
    .await?;
```

### Partial Raw Clauses

If the builder cannot express one clause yet, keep the rest of the chain typed and use a clause-level raw escape hatch instead of switching the whole query to an unsafe/raw mode.

Example:

```rust
use core_db::common::sql::RawClause;

let rows = MessageModel::query()
    .where_col(MessageCol::DIRECTION, Op::Eq, MessageDirection::Outbound)
    .where_raw(RawClause::new("send_attempt_count < max_send_attempts", [])?)
    .order_by(MessageCol::CREATED_AT, OrderDir::Asc)
    .all(db)
    .await?;
```

Available partial raw helpers:
- `.where_raw(...)`
- `.select_raw(...)`
- `.order_by_raw(...)`
- `.join_raw(...)`
- `.group_by_raw(...)`
- `.having_raw(...)`
- `.returning_raw(...)`

Use them only for the clause that genuinely cannot be expressed yet.

### Row Locking And Atomic Claim Patterns

Current lock helpers:
- `.for_update()`
- `.for_update_skip_locked()`
- `.for_no_key_update()`
- `.for_share()`
- `.for_key_share()`
- `.skip_locked()`
- `.no_wait()`

Examples:

```rust
let rows = MessageModel::query()
    .where_col(MessageCol::PROCESSING_STATUS, Op::Eq, MessageProcessingStatus::Queued)
    .order_by(MessageCol::CREATED_AT, OrderDir::Asc)
    .limit(50)
    .for_update()
    .skip_locked()
    .all(db)
    .await?;
```

Atomic selected update:

```rust
let affected = MessageModel::query()
    .where_col(MessageCol::DIRECTION, Op::Eq, MessageDirection::Outbound)
    .for_update()
    .skip_locked()
    .limit(50)
    .patch_selected()
    .assign(MessageCol::PROCESSING_STATUS, MessageProcessingStatus::Processing)?
    .save(db)
    .await?;
```

Atomic claim with typed returning:

```rust
let claimed_ids: Vec<i64> = MessageModel::query()
    .where_col(MessageCol::DIRECTION, Op::Eq, MessageDirection::Outbound)
    .for_update()
    .skip_locked()
    .limit(50)
    .claim()
    .assign(MessageCol::PROCESSING_STATUS, MessageProcessingStatus::Processing)?
    .returning(MessageCol::ID)
    .fetch_scalars(db)
    .await?;
```

Notes:
- `claim()` requires `for_update()` or `for_no_key_update()`
- `skip_locked()` / `no_wait()` require a lock mode first
- use `patch_selected()` for generic selected-row updates
- use `claim()` for queue/outbox claim flows

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

Notes:
- `chunk()` rejects row locks
- `chunk_by_id()` always pages by primary key ascending
- `chunk_by_id()` rejects custom order and pre-set limit/offset
- `chunk_by_id()` with row locks requires `DbConn::Tx`; pool-backed statements release locks before the callback runs

### Update Returning

Returning options on patch/claim builders:
- `.returning(MessageCol::ID).fetch_scalars(db)` for one typed column
- `.returning_many([MessageCol::ID.into(), MessageCol::STATUS.into()]).fetch_json(db)` for mixed columns
- `.returning_raw("id").fetch_json(db)` for a raw returning expression when needed
- `.returning_all().fetch(db)` for full updated records

Do not rely on `row_to_record` access to magically load relations.

## `find` Versus Query Builder

`Model::find(db, id)` does not support relation loading.

Use:

```rust
Model::query()
    .with(Rel::X)
    .find(db, id)
    .await?;
```

Not:

```rust
Model::find(db, id)
```

when you need relations loaded.

## Migration Rules

When adding or changing a model:
- add a new migration
- do not edit old migration files unless explicitly required
- keep table names aligned with `#[rf_model(table = "...")]`
- use `BIGINT` for snowflake PK/FK columns
- index foreign keys and common filter columns

Example:

```sql
CREATE TABLE users (
    id BIGINT PRIMARY KEY,
    uuid VARCHAR(36) NOT NULL,
    username VARCHAR(255) NOT NULL,
    introducer_user_id BIGINT NULL,
    ban SMALLINT NOT NULL DEFAULT 0,
    credit_1 NUMERIC(20, 8) NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_users_username ON users (username);
CREATE INDEX idx_users_introducer_user_id ON users (introducer_user_id);
```

## Do / Don’t

Do:
- keep one model per file
- keep model files as SSOT
- use `#[rf_record_impl]` for computed fields and record helpers
- use `#[rf_model_impl]` for scopes and relation scopes
- use typed relation constants like `UserRel::DOWNLINES`
- use typed aggregate readers where available
- load relations explicitly in workflows and datatables

Don’t:
- do not edit generated files
- do not assume relations auto-load
- do not use old helper styles like `with_author()` or `get_with_relations()`
- do not put computed methods in `#[rf_model_impl]`
- do not suppress warnings
- do not modify old migrations
- do not use relation counts/aggregates inside `where_has(...)`
- do not convert DB model PK fields to `SnowflakeId`

## Verification

After changing a model:

```bash
make gen
cargo check -p app
cargo run -p app --bin export-types
```

When the model affects real consumers, also run what is relevant:

```bash
cargo test -p generated
cargo test -p app
```

If datatables, contracts, or generated behavior changed, verify those paths too before closing the work.
