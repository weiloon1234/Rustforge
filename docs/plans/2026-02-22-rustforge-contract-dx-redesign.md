# Rustforge Contract DX Redesign

## Goal

Redesign `#[rustforge_contract]` to eliminate boilerplate, make all rules first-class, simplify adding new rules (framework and app-level), and unify OpenAPI metadata. No backward compatibility needed.

## Design Decisions

### 1. Auto-inject derives

`#[rustforge_contract]` auto-injects `Debug, Clone, serde::Deserialize, validator::Validate` on the original struct. `schemars::JsonSchema` goes only on the shadow struct (already the case). Users add extra derives if needed (`Serialize`, `PartialEq`, etc.) but never need the base set.

**Before:**
```rust
#[rustforge_contract]
#[derive(Debug, Clone, Deserialize, Validate, JsonSchema)]
pub struct CreateUserInput { ... }
```

**After:**
```rust
#[rustforge_contract]
pub struct CreateUserInput { ... }
```

### 2. All builtin rules are first-class syntax

Remove the `rule = "..."` string-key dispatch entirely. Every rule is a direct `#[rf(...)]` keyword.

**Parameterless rules** (Meta::Path):
- `required`, `required_trimmed`, `email`, `url`, `nested`
- `strong_password`, `alpha_dash`, `lowercase_slug`

**Parameterized rules** (Meta::List):
- `length(min = N, max = N, equal = N)`
- `range(min = N, max = N)`
- `regex(pattern = "...")`
- `contains(pattern = "...")`
- `does_not_contain(pattern = "...")`
- `must_match(other = "field_name")`
- `one_of("a", "b", "c")`
- `none_of("x", "y")`
- `date(format = "...")`
- `datetime(format = "...")`
- `phonenumber(field = "country_field")`

**Async DB rules** (Meta::List):
- `async_unique(table = "...", column = "...", ...modifiers)`
- `async_exists(table = "...", column = "...")`
- `async_not_exists(table = "...", column = "...")`

### 3. Inline `message` and `code` on any rule

Every rule (parameterless or parameterized) accepts optional `message` and `code` when written in list form:

```rust
#[rf(email)]                                          // default message
#[rf(email(message = "Please enter valid email"))]    // custom message
#[rf(strong_password(message = "Too weak", code = "weak_pw"))]
#[rf(one_of("a", "b", message = "Pick one"))]
#[rf(length(min = 3, max = 32, message = "Bad length"))]
```

Remove `rule_override(...)` entirely. Per-rule message is always inline.

For field-level fallback message (applies to all rules on that field if they don't have their own), keep `#[rf(message = "...")]` as a standalone attribute.

### 4. Two-crate rule registration (eliminate proc macro changes)

Adding a new framework rule requires only:
1. Add validation fn in `core-web/src/rules/mod.rs`
2. Add `BuiltinRuleMeta` entry in `rustforge-contract-meta/src/lib.rs`

The proc macro uses the meta registry to auto-discover valid rule names. No match-arm additions needed in the proc macro.

**Meta registry enhancement** - add `args` field to `BuiltinRuleMeta`:

```rust
pub enum BuiltinRuleArgs {
    None,                        // strong_password, alpha_dash
    Values,                      // one_of, none_of
    Format,                      // date, datetime
    Field,                       // phonenumber
}
```

The proc macro checks `builtin_rule_meta(name)` for any unknown ident. If found, it knows how to parse its args based on the `BuiltinRuleArgs` discriminant.

### 5. App-level custom rules with OpenAPI metadata

New `#[rf(custom(...))]` syntax for app-defined rules:

```rust
#[rf(custom(
    function = "crate::validation::validate_tax_id",
    description = "Valid tax identification number",
    pattern = "^[A-Z]{2}[0-9]{8}$",
    code = "tax_id",
    message = "Invalid tax ID format"
))]
pub tax_id: String,
```

Generates `#[validate(custom(function = ...))]` + OpenAPI metadata on the schema.

**App-level reusable rules via `rustforge_string_rule_type!`** already exist and keep working for wrapper types. For non-wrapper custom rules, `#[rf(custom(...))]` is the escape hatch with full OpenAPI support.

### 6. Grouped OpenAPI attributes

Replace flat `openapi_*` keys with grouped syntax:

```rust
#[rf(openapi(description = "...", hint = "...", example = "...", format = "..."))]
```

Remove: `openapi_description`, `openapi_hint`, `openapi_example`, `openapi_format` as separate keys.

### 7. End-state contract example

```rust
#[rustforge_contract]
pub struct CreateUserInput {
    #[rf(length(min = 1, max = 120))]
    pub name: String,

    #[rf(email(message = "Please enter a valid email"))]
    #[rf(async_unique(table = "users", column = "email"))]
    pub email: String,

    #[rf(strong_password)]
    pub password: String,

    #[rf(must_match(other = "password"))]
    pub password_confirmation: String,

    #[rf(one_of("admin", "editor", "viewer"))]
    pub role: String,

    #[rf(date(format = "[year]-[month]-[day]"))]
    pub birth_date: String,

    #[rf(phonenumber(field = "country_iso2"))]
    #[rf(openapi(hint = "Raw input; server validates by country"))]
    pub phone: String,

    pub country_iso2: String,

    #[rf(custom(
        function = "crate::validation::validate_tax_id",
        description = "Valid tax ID",
        pattern = "^[A-Z]{2}[0-9]{8}$"
    ))]
    pub tax_id: String,
}
```

## Implementation Plan

### Step 1: Update `rustforge-contract-meta`

- Add `BuiltinRuleArgs` enum to `BuiltinRuleMeta`
- Update all 8 existing entries with their arg type
- Remove: nothing yet (proc macro still references these)

### Step 2: Rewrite proc macro parsing layer

- Auto-inject derives (Debug, Clone, Deserialize, Validate)
- Remove `rule = "..."` string dispatch, `values(...)` as separate key, `format`/`field` as context-dependent keys
- Remove `rule_override(...)` syntax
- Remove flat `openapi_*` keys
- Add registry-driven builtin matching: any `Meta::Path` or `Meta::List` ident checked against `builtin_rule_meta()`
- Add `custom(...)` parsing
- Add `openapi(...)` grouped parsing
- Add inline `message`/`code` support on all parameterized rules
- Parameterless rules with message become `Meta::List`: `strong_password(message = "...")`

### Step 3: Update code generation layer

- Minimal changes needed - `build_validate_*` functions stay mostly the same
- `build_rules_json_expr` and `RuleExtensionSpec` stay the same
- Adjust how `BuiltinRuleUse` is constructed (from parsed list args instead of pending_builtin + loose keys)

### Step 4: Update all consumers

- `core-web/tests/rustforge_contract.rs` - update all test contracts to new syntax
- `scaffold/src/templates.rs` - update all template contracts
- `core-web/src/datatable.rs` - update any contracts there

### Step 5: Update `rustforge_string_rule_type!` macro

- Remove the manual derive injection (now handled by `#[rustforge_contract]` auto-inject)
- Update inner helper struct syntax to match new `#[rf(...)]` API

### Step 6: Verify

- `cargo test` across workspace
- `cargo check` on scaffold output
- Trybuild tests if any

## Security note

The `Unique`/`Exists`/`NotExists` rules in `core-web/src/rules/mod.rs` interpolate table/column names via `format!()`. Currently safe because they're `&'static str` from macro expansion. This design doesn't change that - table/column still come from string literals in `#[rf(async_unique(table = "...", column = "..."))]` which are compile-time constants. No new risk.

## Files affected

- `rustforge-contract-meta/src/lib.rs` - add args enum, update entries
- `rustforge-contract-macros/src/lib.rs` - major rewrite of parsing layer
- `core-web/src/contracts.rs` - update `rustforge_string_rule_type!`
- `core-web/tests/rustforge_contract.rs` - rewrite tests to new syntax
- `scaffold/src/templates.rs` - update all template contracts
