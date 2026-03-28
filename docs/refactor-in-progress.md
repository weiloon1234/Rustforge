# Refactor In Progress

This file is the current checkpoint for the Rustforge query/model/codegen refactor.

It is intentionally blunt:
- what is already done
- what is still not done
- what the actual objective is
- the recommended order to finish it without creating more generator-owned behavior

## Current Objective

The objective is not just to split [`db-gen/src/gen_models.rs`](/Users/weiloon/Projects/personal/Rust/Rustforge/db-gen/src/gen_models.rs) into smaller files.

The real objective is:
- make the model/query runtime the single source of truth for behavior
- keep generated code mostly as metadata and typed public surface
- reduce `writeln!`-driven embedded runtime logic
- keep scaffold/template and framework consumers compiling warning-free

The acceptance bar is:
- less generator-owned execution logic
- more trait/default/runtime-owned behavior
- no regression in relations, soft delete, datatable behavior, or query ergonomics

## What Is Done

### Query Runtime

- Read/query path is tree-based for the normal typed path.
- Root query state is no longer string-first for the main typed filter/order/select path.
- Relation load / existence / count / aggregate trees are typed.
- `PatchState` predicate/selection side is typed.
- `CreateState` / `PatchState` assignment state is now typed:
  - `CreateAssignment`
  - `CreateConflictSpec`
  - `PatchAssignment`
- `CreateState` now also owns create-side feature payloads:
  - localized translations
  - meta values
  - attachment inputs
- `PatchState` now also owns patch-side feature payloads:
  - localized translations
  - meta values
  - attachment add/replace inputs
  - attachment clear/delete intent
- Patch returning state is now typed internally:
  - `ReturnExpr`
  - `JsonReturnField`
  - `ReturningSpec`

Strictly speaking, the AST story is:
- read/query AST is mostly done for the normal typed path
- write AST is improved a lot, but still not fully done
- raw escape hatches still exist by design
- `QueryState` still has raw/string escape fields for:
  - `from_sql`
  - `count_sql`
  - `group_by: Vec<String>`
  - raw joins / raw havings

### Runtime Defaults Moved Out Of Codegen

In [`core-db/src/common/model_api.rs`](/Users/weiloon/Projects/personal/Rust/Rustforge/core-db/src/common/model_api.rs):

- generic query runtime helpers now exist for:
  - `query_all_runtime`
  - `query_count_runtime`
  - `query_paginate_runtime`
  - `query_delete_runtime`
  - `create_save_runtime`
  - `create_save_with_db_runtime`
  - `patch_save_runtime`
  - `patch_fetch_runtime`
  - `patch_fetch_returning_all_runtime`
- `QueryModel` now owns default implementations for:
  - `query_all`
  - `query_first`
  - `query_find`
  - `query_count`
  - `query_paginate`
  - `query_delete`
- `CreateModel` now owns the generic create/save orchestration.
- `PatchModel` now owns the generic patch/update orchestration.
- added runtime traits:
  - `ChunkModel`
  - `DeleteModel`

Generated create model code no longer owns:
- transaction wrapping for inserts
- create observer orchestration
- insert SQL execution
- post-insert hydration orchestration
- generated create builders now store their feature payloads directly in `CreateState`

Generated patch builders also no longer need separate side maps for:
- localized payloads
- meta payloads
- attachment add/replace payloads
- attachment clear/delete payloads

Generated patch model code no longer owns:
- transaction wrapping for updates
- update observer sequencing
- target-id selection and refetch orchestration
- update SQL execution / profiler plumbing
- patch returning-all orchestration
- generated patch output is now model metadata plus model-specific hook emitters

This means generated models no longer need to emit the full per-model async bodies for the common read/query path, delete path, create path, or patch execution path.

### Chunking / Lock Behavior

- `chunk()` is still the simple offset loop and now explicitly rejects row locks.
- `chunk_by_id()` exists for keyset-style iteration by primary key.
- `chunk_by_id()` with row locks requires `DbConn::Tx`; pool-backed locked iteration is rejected explicitly.

### Direct Raw Clause Ergonomics

The typed query chain now supports direct raw clause helpers:
- `where_raw(RawClause)`
- `or_where_raw(RawClause)`
- `select_raw(RawSelectExpr)`
- `add_select_raw(RawSelectExpr)`
- `join_raw(RawJoinSpec)`
- `group_by_raw(RawGroupExpr)`
- `having_raw(RawClause)`
- `order_by_raw(RawOrderExpr)`

This removed the need for current scaffold consumers to use `unsafe_sql().done()` for normal mixed typed/raw cases.

### Generator Reduction So Far

Current rough numbers:
- `db-gen/src/gen_models.rs`: about `6604` lines before this phase
- `db-gen/src/gen_models.rs`: about `4638` lines now (was `4753` before simplification pass)
- `render_model()` is still the main hotspot, but it is smaller than before
- dead template scaffolding (empty query/insert/update sections) removed
- duplicate `_transform_create_value` / `_transform_patch_value` extracted into `emit_transform_value_body()`
- redundant identical if/else branches collapsed

The reduction is real because behavior moved into runtime traits/defaults, not only because the file was reorganized.

### Feature Persistence Delegation Macro

`core_db::impl_feature_persistence_delegates!` macro added to `core-db/src/common/model_api.rs`.

This macro generates the 6 boilerplate delegation methods (`upsert_localized_many`, `upsert_meta_many`, `clear_attachment_field`, `replace_single_attachment`, `add_attachments`, `delete_attachment_ids`) inside a `FeaturePersistenceModel` impl block.

Usage:
```rust
impl FeaturePersistenceModel for ArticleModel {
    fn localized_owner_type() -> Option<&'static str> { Some(localized::ARTICLE_OWNER_TYPE) }
    core_db::impl_feature_persistence_delegates!(localized, localized, meta, attachment);
}
```

The macro accepts a module path and feature flags (`localized`, `meta`, `attachment`). Each model invokes it with only the features it uses.

**Status:** DONE. Macro defined and generator updated. Per-model output now uses `core_db::impl_feature_persistence_delegates!(localized, localized, meta, attachment);` instead of 6 inline delegation methods. Test fixtures updated.

## What Is Not Done

### Feature Descriptors Are Partially Generator-Specific

The repetitive persistence loops are no longer emitted per model.

That work is now shared in `core-db` through:
- `FeaturePersistenceModel`
- generic localized/meta/attachment persistence helpers
- `impl_feature_persistence_delegates!` macro for delegation boilerplate

What is still generated per model:
- owner-type constants (unavoidable metadata, not logic)
- owner-type return methods (`localized_owner_type()`, etc.) — compact, metadata-only
- parent touch/update hooks (`persist_create_related`, `persist_patch_related`) — actual per-model logic

What has been eliminated from per-model generation:
- persistence delegation methods (`upsert_localized_many`, etc.) — replaced by macro
- persistence loops — already moved to shared runtime

The remaining per-model code is either metadata constants or model-specific FK-based touch logic.
The shared functions (`upsert_localized_many`, etc.) must stay in the generated localized module because they use generated types (`LocalizedModel`, `MetaModel`, `AttachmentModel`).

### Thin Generated Builder Wrappers Are Gone

- generated `XxxCreateInner` / `XxxPatchInner` wrappers are removed
- `Model::create()` / `Model::patch()` now expose the shared `Create` / `Patch` surface only
- generic feature helpers now live on the shared builders:
  - `set_translation(...)`
  - `insert_meta_value(...)`
  - `set_attachment_single(...)`
  - `add_attachment_multi(...)`
  - `clear_attachment_single(...)`
  - `delete_attachment_multi_ids(...)`

This is a real reduction in codegen, not just a rename.

### Full Write AST Is Still Not Complete

Even after the typed assignment work, the write path is still not fully at the same level as the read path.

Still incomplete:
- `CreateState` is typed for assignments, conflicts, and feature payloads, but create returning still does not have a generalized returning AST surface
- create/update SQL assembly still compiles typed nodes into SQL strings at execution time rather than having a larger expression IR for every SQL construct
- feature ownership still hangs off generated hooks rather than compact typed descriptors
- create-side and patch-side returning still are not one unified typed write projection system

So the honest status is:
- write AST is improved materially
- write AST is not fully finished

### Full Read AST Is Also Not Absolute

If the standard is “all read/query concepts are represented as typed AST nodes”, the read side is still not absolute either.

Still string/raw-based by design:
- `from_sql`
- `count_sql`
- raw joins
- raw select expressions
- raw group expressions
- raw having clauses
- raw where/exists clauses

So the honest read status is:
- the normal typed path is largely AST-driven
- the full read side is not 100% pure-AST because explicit raw escape hatches remain

### Datatable Generator Slimming

The datatable section is still heavily generator-owned.

Still generated per model:
- bind parsing
- locale field parsing
- relation-path filter helper glue
- large `ParsedFilter` match dispatch
- sort/cursor/count/fetch adapter glue

`core-datatable` still does not own enough of this logic.

### `render_model()` Is Still Too Big

`render_model()` still owns too much assembly logic (~3,190 lines):
- row/record/view generation (~579 lines)
- column model section (~207 lines)
- datatable generation (~1,430 lines — 44.8% of render_model, the largest section)
- model runtime section (~580 lines)
- active record section (~22 lines)

**DONE:** `ModelCtx` struct + 7 extracted section functions:
- `ModelCtx<'a>` struct with 40 fields, `ModelCtx::new()` constructor
- `render_imports_and_constants(&ModelCtx)` — imports + constants
- `render_row_view_json_section(&ModelCtx)` — row/record/view structs (~586 lines)
- `render_column_model_section(&ModelCtx)` — column enum + model metadata (~207 lines)
- `render_datatable_section(&ModelCtx)` — datatable adapter (~1,431 lines)
- `render_active_record_section(&ModelCtx)` — ActiveRecord impl (~22 lines)
- `render_model_runtime_section(&ModelCtx)` — model/create/patch trait impls (~580 lines)
- `render_model()` is now a 66-line orchestrator
- Existing sub-helpers (`render_create_model_impl`, `render_patch_model_impl`, etc.) unchanged

The structural hotspot is resolved. Each section is independently readable and testable.

## Recommended Next Order

### 1. ~~Shrink Feature Ownership Hooks Further~~ — PARTIALLY DONE

~~Create runtime traits or descriptors for localized/meta/attachments/parent-touch/update targets.~~

**Done:**
- `impl_feature_persistence_delegates!` macro created in core-db
- Macro eliminates 6 delegation methods per model (~50 lines per model with features)
- Generated code now only provides owner-type constants and the macro invocation

**Remaining:**
- Parent-touch/update hooks could potentially use a descriptor trait (lower priority)

### 2. ~~Move Datatable Filter / Parser Logic Into `core-datatable`~~ — PARTIALLY DONE

**Done:**
- `DataTableColumnResolver` trait added to `core-datatable/src/traits.rs` (7 methods: resolve_col_sql, resolve_like_col_sql, parse_bind_for_col, resolve_locale_field, parse_datetime, table_name, pk_column)
- `apply_standard_filter()` function in `core-datatable/src/filters.rs` handles 10 of 14 ParsedFilter variants generically (Eq, Like, Gte, Lte, DateFrom, DateTo, LocaleEq, LocaleLike, LikeAny, Any)
- Generated `apply_auto_filter` now delegates to `apply_standard_filter` first, only falls through to per-model code for 4 relation variants (Has, HasLike, LocaleHas, LocaleHasLike)
- Generated code implements `DataTableColumnResolver` per model, delegating to existing static methods

**Still per-model (by necessity):**
- Relation filter dispatch (Has, HasLike, LocaleHas, LocaleHasLike) — requires typed `where_has` with model-specific relation enums
- Column descriptor arrays — static metadata
- Config, hooks trait, adapter wrapper — extensibility points

### 3. ~~Split `render_model()`~~ — DONE

**Completed.** `render_model()` is now a 66-line orchestrator calling 7 extracted section functions via `ModelCtx<'a>` struct.

## Suggested Concrete Trait Targets

### `core-db`

Good next shared helpers:
- `create_save_runtime::<M>(...)`
- `patch_save_runtime::<M>(...)`
- `patch_fetch_runtime::<M>(...)`
- `patch_fetch_returning_all_runtime::<M>(...)`

Likely needed support traits:
- a feature descriptor trait for localized/meta/attachment ownership
- a parent-touch descriptor trait
- a thinner patch/update hook trait for override/change decoding only

### `core-datatable`

Good next shared helpers:
- generic bind parsing by descriptor
- generic locale filter application by descriptor
- generic relation-path `has` / `has_like` filter application
- generic sort/cursor helpers where model metadata is enough

## Risks / Constraints

- Do not reintroduce per-model builder wrapper types or convenience setter shells.
  The canonical builder surface is now the shared `Create` / `Patch` API plus typed column methods.
  Keep:
  - relation constants
  - datatable public types/hooks

- Do not move behavior into runtime by reintroducing string-based query internals.
  Runtime extraction must stay aligned with the typed query/write direction.

- Do not silently regress observer behavior.
  The observer hooks are one of the easiest places to accidentally change semantics while slimming codegen.

- Do not count fixture updates as proof of improvement by themselves.
  The real proof is less behavior emitted per model and more shared runtime defaults.

## Current Verification Status

This checkpoint was last refreshed after the simplification pass and feature persistence macro addition. The full current matrix is green:
- `cargo test -p core-db` — 28 pass, 0 fail
- `cargo test -p db-gen` — 23 pass, 0 fail (1 skip)
- `make scaffold-template-clean && cargo test -p scaffold`
- local-patched `cargo check -p rustforge-starter-generated`
- local-patched `cargo check -p rustforge-starter`
- local-patched `make gen && make gen-types && make release` — all green, zero warnings

Current line counts:
- `core-db/src/common/model_api.rs`: `7288` lines (was `7427` before simplification; +macro/touch runtime)
- `db-gen/src/gen_models.rs`: `4528` lines (was `6604` at start of refactor; net -2076 lines, -31%)
  - `render_model()`: 66 lines (was ~3,190 lines before `ModelCtx` split)
- `core-datatable/src/filters.rs`: `375` lines (+194 for shared filter dispatch)
- `core-datatable/src/traits.rs`: `318` lines (+35 for DataTableColumnResolver)

If continuing from here, re-run the full matrix after the next behavior move:
- `cargo test -p core-db`
- `cargo test -p db-gen`
- `make scaffold-template-clean && cargo test -p scaffold`
- local-patched `cargo check -p rustforge-starter-generated`
- local-patched `cargo check -p rustforge-starter`

## Short Honest Summary

The refactor is no longer in the “ideas only” stage.

Real progress already landed:
- read/query execution moved into shared runtime defaults
- delete execution moved into shared runtime defaults
- create/save execution moved into shared runtime defaults
- patch/save/fetch execution moved into shared runtime defaults
- write assignment state is typed
- create feature payload state is typed
- patch feature payload state is typed
- patch returning state is typed
- generated create/patch wrapper structs are gone
- generator size is materially reduced (6604 → 4638 lines)
- runtime simplified: `check_deferred()`, `normalize_raw_placeholders()`, `make_placeholders()`, aggregate method dedup via `push_aggregate`/`push_aggregate_scoped`
- observer dedup in `create_save_runtime` (Pool/Tx branches share notification tail)
- `impl_feature_persistence_delegates!` macro added to eliminate per-model delegation boilerplate
- critical bugs fixed: `from_selected_query` missing FROM clause, `chunk()` missing deferred error check

The remaining refactor work:
1. ~~**Generator update to use the persistence macro**~~ — DONE
2. ~~**Split `render_model()`**~~ — DONE (66-line orchestrator + 7 section functions + `ModelCtx` struct)
3. ~~**Datatable slimming**~~ — DONE: 10/14 filter variants moved to `apply_standard_filter()` in core-datatable. 4 relation variants remain per-model (typed where_has requirement).
4. ~~**Parent-touch descriptor**~~ — DONE: `TouchTarget` struct + `persist_create_touch_targets()` / `persist_patch_touch_targets()` runtime functions. Generated code now emits descriptors instead of bespoke FK-update logic.
