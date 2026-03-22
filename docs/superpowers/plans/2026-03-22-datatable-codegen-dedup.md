# DataTable Codegen Deduplication Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Eliminate N×M code duplication in DataTable relation filter/sort generation by extracting per-target-model helper functions, reducing generated code from ~18K to ~3K lines for complex models.

**Architecture:** Currently, every `(relation_path, column)` combination generates its own match arm — duplicating column-level logic for every relation that targets the same model. Extract the column dispatch into a per-target-model function generated once, then have relation dispatch call it.

**Tech Stack:** Rust code generation via `db-gen/src/gen_models.rs`

---

## Problem

Five generated functions use the same N×M loop pattern (`for rel_path { for target_field { emit match arm } }`):

| Generated function | Purpose | Lines in user.rs |
|--------------------|---------|------------------|
| `parse_bind_for_relation()` | Convert raw string to typed BindValue per (relation, column) | ~4,478 |
| `relation_column_descriptors()` | Static metadata per (relation, column) | ~4,474 |
| `ParsedFilter::Has` | Filter by exact match | ~N×M arms |
| `ParsedFilter::HasLike` | Filter by LIKE match (string cols only) | ~N×M arms |
| `ParsedFilter::LocaleHas/HasLike` | Filter localized fields | ~N×M arms |

For User with 10 relations × 19 fields, each function has ~190 match arms. Total: ~950+ arms across the 5 functions.

**Key insight:** Multiple relations target the SAME model (e.g., `introducer → User`, `downlines → User`). The column matching for User is duplicated for each. Extracting it into a per-model helper changes N×M to N+M.

## File Structure

All changes are in a single file: `db-gen/src/gen_models.rs`

No changes to:
- `core-db/` (runtime library)
- `core-datatable/` (trait definitions)
- App code (controllers, React frontend)
- Generated API surface (same trait methods, same behavior)

---

## Task 1: Generate per-target-model `parse_bind_for_{model}` helpers

**File:** `db-gen/src/gen_models.rs` (around lines 4455-4485)

Currently generates:
```rust
fn parse_bind_for_relation(relation: &str, column: &str, raw: &str) -> Option<BindValue> {
    match (relation, column) {
        ("introducer", "id") => raw.trim().parse::<i64>().ok().map(Into::into),
        ("introducer", "name") => Some(raw.trim().to_string().into()),
        ("downlines", "id") => raw.trim().parse::<i64>().ok().map(Into::into),  // DUPLICATE
        ("downlines", "name") => Some(raw.trim().to_string().into()),            // DUPLICATE
        _ => None,
    }
}
```

Change to generate:
```rust
// Generated ONCE per unique target model
fn parse_bind_for_user_cols(column: &str, raw: &str) -> Option<BindValue> {
    match column {
        "id" => raw.trim().parse::<i64>().ok().map(Into::into),
        "name" => Some(raw.trim().to_string().into()),
        // ... 19 arms (one per User field)
        _ => None,
    }
}

fn parse_bind_for_relation(relation: &str, column: &str, raw: &str) -> Option<BindValue> {
    match relation {
        "introducer" => Self::parse_bind_for_user_cols(column, raw),
        "downlines" => Self::parse_bind_for_user_cols(column, raw),  // reuses same function
        "galleries" => Self::parse_bind_for_gallery_cols(column, raw),
        // ... one arm per relation (NOT per column)
        _ => None,
    }
}
```

### Implementation:

- [ ] **Step 1:** Before the `parse_bind_for_relation` generation loop, collect unique target models from `relation_paths` into a `BTreeMap<String, &ModelSpec>` (dedup by target model name).

- [ ] **Step 2:** For each unique target model, generate a `parse_bind_for_{snake_model}_cols(column, raw)` function with one match arm per field. Use the existing `parse_bind_expr()` closure for the expression.

- [ ] **Step 3:** Replace the `parse_bind_for_relation` N×M loop with an N-only loop that matches on `relation` and delegates to the per-model helper.

- [ ] **Step 4:** Verify compilation: `cargo check -p db-gen`

- [ ] **Step 5:** Commit

---

## Task 2: Generate per-target-model `filter_has_{model}` helpers

**File:** `db-gen/src/gen_models.rs` (around lines 4925-4965)

Currently the `ParsedFilter::Has` block generates `(relation, column)` match arms. Change to:

```rust
// Generated ONCE per unique target model
fn filter_has_for_user_cols<'db>(column: &str, rq: Query<'db, UserModel>, bind: BindValue) -> Option<Query<'db, UserModel>> {
    match column {
        "id" => Some(rq.where_col(UserDbCol::Id, Op::Eq, bind)),
        "name" => Some(rq.where_col(UserDbCol::Name, Op::Eq, bind)),
        _ => None,
    }
}

// In ParsedFilter::Has:
match relation.as_str() {
    "introducer" => {
        let Some(bind) = Self::parse_bind_for_user_cols(column, trimmed) else { return Ok(None); };
        // build_nested_where_has_expr wraps the inner call
        Ok(Some(query.where_has(Rel::INTRODUCER, |rq| Self::filter_has_for_user_cols(column, rq, bind).unwrap_or(rq))))
    }
    // ...
}
```

### Implementation:

- [ ] **Step 1:** For each unique target model, generate a `filter_has_for_{model}_cols<'db>(column, rq, bind)` function that matches column name to `rq.where_col(DbCol::Variant, Op::Eq, bind)`.

- [ ] **Step 2:** Replace the `ParsedFilter::Has` N×M loop. For each relation_path, generate a single match arm that:
  1. Calls `parse_bind_for_{target}_cols(column, trimmed)` to get the bind
  2. Uses `build_nested_where_has_expr` to wrap the call to `filter_has_for_{target}_cols`

- [ ] **Step 3:** Do the same for `ParsedFilter::HasLike` — generate `filter_has_like_for_{model}_cols` that uses `Op::Like` for String columns.

- [ ] **Step 4:** Verify compilation: `cargo check -p db-gen`

- [ ] **Step 5:** Commit

---

## Task 3: Deduplicate LocaleHas/LocaleHasLike filter arms

**File:** `db-gen/src/gen_models.rs` (around lines 5025-5160)

Same pattern as Task 2 but for localized fields. Extract per-target-model locale filter helpers.

- [ ] **Step 1:** For each unique target model with localized fields, generate locale filter helpers.

- [ ] **Step 2:** Replace the LocaleHas/LocaleHasLike N×M loops with relation-only dispatch.

- [ ] **Step 3:** Verify compilation: `cargo check -p db-gen`

- [ ] **Step 4:** Commit

---

## Task 4: Deduplicate `relation_column_descriptors`

**File:** `db-gen/src/gen_models.rs` (around lines 4607-4657)

Currently generates one `DataTableRelationColumnDescriptor` per `(relation, column)`. The descriptors for the same target model are identical except for the `relation` field.

Extract per-target-model descriptor arrays and reference them:

```rust
// Generated once per target model
const USER_REL_COL_DESCRIPTORS: &[(&str, &str)] = &[
    ("id", "i64"), ("name", "String"), ...
];

fn relation_column_descriptors(&self) -> &'static [DataTableRelationColumnDescriptor] {
    &[
        // Expand per relation × per target descriptor
        // Still N×M entries in the array (needed for the trait contract)
        // but the GENERATION code is N+M, not N×M
    ]
}
```

Note: `relation_column_descriptors` returns a flat `&[DataTableRelationColumnDescriptor]` — the trait contract requires this. The dedup here is in the generator code (fewer writeln! loops), not in the generated output. The output remains the same, but the generator runs faster and the code is cleaner.

Actually, for this function the generated output IS the same flat array — the N×M is inherent in the data structure. The real savings come from Tasks 1-3 (the match arms). This task is lower priority.

- [ ] **Step 1:** Assess if deduplication is worthwhile here (the output is inherently N×M). Skip if marginal.

- [ ] **Step 2:** If skipped, commit a comment explaining why.

---

## Task 5: Update snapshots and verify

- [ ] **Step 1:** `UPDATE_DB_GEN_FIXTURES=1 cargo test -p db-gen` — regenerate snapshots

- [ ] **Step 2:** Verify snapshot diff — the generated code should show:
  - New per-model helper functions (`parse_bind_for_user_cols`, `filter_has_for_user_cols`, etc.)
  - Smaller match statements in `parse_bind_for_relation` and `apply_auto_filter`
  - Same external behavior

- [ ] **Step 3:** `cargo test` — all tests pass

- [ ] **Step 4:** Commit

---

## Task 6: Verify in mediaforge

- [ ] **Step 1:** Count lines before: `wc -l mediaforge/generated/src/models/user.rs`

- [ ] **Step 2:** `cd mediaforge && cargo update && make gen`

- [ ] **Step 3:** Count lines after: `wc -l mediaforge/generated/src/models/user.rs`

- [ ] **Step 4:** Time the build: `time make gen`

Expected: user.rs drops from ~18K to ~3-4K lines. Build time drops from 5+ min to under 1 min.

---

## Expected Impact

| Metric | Before | After (depth fix + dedup) |
|--------|--------|--------------------------|
| user.rs lines | 18,265 | ~2,000-3,000 |
| Total generated lines | 141,028 | ~30,000-40,000 |
| `make gen` time | 5+ min | <1 min |
| Compile-time safety | Full | Full (unchanged) |
| Runtime behavior | N/A | Identical |
| App/React changes | N/A | Zero |
