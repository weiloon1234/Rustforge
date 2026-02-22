# Rustforge Contract DX Redesign - Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Redesign `#[rustforge_contract]` to eliminate derive boilerplate, make all rules first-class `#[rf(...)]` syntax, reduce adding new rules to 2 crates, support app-level custom rules with OpenAPI metadata, and group OpenAPI attributes. No backward compat.

**Architecture:** The proc macro (`rustforge-contract-macros`) is an attribute macro that generates a shadow struct for schemars + validator attributes on the original struct. The meta crate (`rustforge-contract-meta`) defines the rule registry. The rewrite changes the parsing layer to auto-discover rules from the registry rather than hardcoding ident matches, auto-injects derives, removes the `rule = "..."` dispatch, and adds `custom(...)` + `openapi(...)` grouped syntax.

**Tech Stack:** syn 2, quote, proc-macro2, schemars 0.8, validator (vendored), aide 0.14

---

### Task 1: Update `rustforge-contract-meta` - add `BuiltinRuleArgs`

**Files:**
- Modify: `rustforge-contract-meta/src/lib.rs`

**Step 1: Rewrite `rustforge-contract-meta/src/lib.rs`**

Add `BuiltinRuleArgs` enum to tell the proc macro how to parse each rule's arguments. Update all `BuiltinRuleMeta` entries. The proc macro depends on this crate, so it can use `args` to generically handle any rule without hardcoded match arms.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinRuleArgs {
    /// No arguments: `#[rf(strong_password)]`
    /// With message only: `#[rf(strong_password(message = "..."))]`
    None,
    /// Accepts list of string values: `#[rf(one_of("a", "b"))]`
    Values,
    /// Accepts `format = "..."`: `#[rf(date(format = "..."))]`
    Format,
    /// Accepts `field = "..."`: `#[rf(phonenumber(field = "country_iso2"))]`
    Field,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinRuleKind {
    CustomFnPath(&'static str),
    GeneratedOneOf,
    GeneratedNoneOf,
    GeneratedDate,
    GeneratedDateTime,
    PhoneNumberByIso2Field,
}

#[derive(Debug, Clone, Copy)]
pub struct BuiltinRuleMeta {
    pub key: &'static str,
    pub kind: BuiltinRuleKind,
    pub args: BuiltinRuleArgs,
    pub default_code: &'static str,
    pub default_message: &'static str,
    pub openapi_description_template: &'static str,
    pub pattern: Option<&'static str>,
    pub format: Option<&'static str>,
}
```

Update all 8 entries to include `args`. E.g.:
- `required_trimmed` -> `args: BuiltinRuleArgs::None`
- `one_of` -> `args: BuiltinRuleArgs::Values`
- `date` -> `args: BuiltinRuleArgs::Format`
- `phonenumber` -> `args: BuiltinRuleArgs::Field`

**Step 2: Run `cargo check -p rustforge-contract-meta`**

Expected: PASS (no consumers changed yet; `args` field added, proc macro not yet updated to use it).

**Step 3: Commit**

```
git add rustforge-contract-meta/src/lib.rs
git commit -m "feat(meta): add BuiltinRuleArgs enum for generic rule parsing"
```

---

### Task 2: Rewrite proc macro - auto-inject derives + new parsing

**Files:**
- Rewrite: `rustforge-contract-macros/src/lib.rs`

This is the largest task. The codegen layer (build_validate_*, build_patch_block, RuleExtensionSpec, JsonParam, generate_*_wrapper_fn, generate_async_db_rule_block, apply_async_db_modifier, mk_attr) stays mostly intact. The major changes are:

**Step 1: Auto-inject derives in `expand_rustforge_contract`**

Replace the derive-handling logic. Instead of preserving user's derives (minus JsonSchema), always ensure the required derives are present:

```rust
// After parsing the struct, before emitting:
// 1. Collect any user-supplied derives (strip JsonSchema, Validate, Deserialize, Debug, Clone)
// 2. The original struct always gets: #[derive(Debug, Clone, serde::Deserialize, validator::Validate, {user_extras})]
// 3. The shadow struct always gets: #[derive(schemars::JsonSchema)]
```

The key change in the derive handling loop:

```rust
// Collect user extra derives (anything NOT in the auto-inject set)
let auto_inject_set = ["Debug", "Clone", "Deserialize", "Validate", "JsonSchema"];
let mut user_extra_derives: Vec<Meta> = Vec::new();

for attr in item_attrs_without_rf.iter() {
    if attr.path().is_ident("derive") {
        let metas = attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
        for m in metas {
            if let Meta::Path(p) = &m {
                let ident = p.segments.last().map(|s| s.ident.to_string()).unwrap_or_default();
                if !auto_inject_set.contains(&ident.as_str()) {
                    user_extra_derives.push(m);
                }
            }
        }
        continue; // Don't add raw derive to original_container_attrs
    }
    // ... rest of attr handling
}

// Build the derive attr for the original struct
let extra_derives = if user_extra_derives.is_empty() {
    quote! {}
} else {
    quote! { , #(#user_extra_derives),* }
};

// In the expanded output:
// #[derive(::std::fmt::Debug, ::std::clone::Clone, ::serde::Deserialize, ::validator::Validate #extra_derives)]
```

**Step 2: Rewrite `parse_rf_field` - registry-driven builtin matching**

Replace the giant match chain with registry-driven dispatch:

```rust
// For Meta::Path (parameterless):
Meta::Path(path) => {
    let name = path.get_ident().map(|i| i.to_string()).unwrap_or_default();
    match name.as_str() {
        // Core rules handled directly (they have special validator attrs)
        "email" | "url" | "required" | "nested" => { /* existing logic */ }
        // Everything else: check meta registry
        _ => {
            if let Some(meta) = builtin_rule_meta(&name) {
                if meta.args != BuiltinRuleArgs::None {
                    return Err(syn::Error::new_spanned(path,
                        format!("#[rf({})] requires arguments", name)));
                }
                // Create BuiltinRuleUse with no args
                local_builtin = Some(BuiltinRuleUse { key: name, values: vec![], format: None, field: None });
            } else {
                return Err(syn::Error::new_spanned(path, "unsupported #[rf(...)] syntax"));
            }
        }
    }
}

// For Meta::List (parameterized):
Meta::List(list) => {
    let name = list.path.get_ident().map(|i| i.to_string()).unwrap_or_default();
    match name.as_str() {
        // Core parameterized rules handled directly
        "length" | "range" | "regex" | "contains" | "does_not_contain" | "must_match" => { /* existing */ }
        // Async DB rules
        "async_unique" | "async_exists" | "async_not_exists" => { /* existing */ }
        // New: custom(), openapi()
        "custom" => { /* new: parse function, description, pattern, code, message */ }
        "openapi" => { /* new: parse description, hint, example, format */ }
        // Builtins that were previously string-key or parameterless-with-message
        _ => {
            if let Some(meta) = builtin_rule_meta(&name) {
                // Parse args based on meta.args discriminant + optional message/code
                let builtin = parse_builtin_from_list(&list, meta)?;
                local_builtin = Some(builtin);
            } else if name == "email" || name == "url" || name == "required" || name == "nested" {
                // Core rules in list form for message override: #[rf(email(message = "..."))]
                // Parse message/code from list args, set the bool flag + override
            } else {
                return Err(syn::Error::new_spanned(list, format!("unknown #[rf({}(...))] rule", name)));
            }
        }
    }
}
```

**Step 3: Add `parse_builtin_from_list` helper**

```rust
fn parse_builtin_from_list(list: &MetaList, meta: &BuiltinRuleMeta) -> syn::Result<BuiltinRuleUse> {
    let metas = list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
    let mut values = Vec::new();
    let mut format = None;
    let mut field = None;
    let mut message = None;
    let mut code = None;

    for m in &metas {
        match m {
            // Bare string literals are values (for one_of/none_of)
            // Named args: format, field, message, code
            Meta::NameValue(nv) if nv.path.is_ident("format") => {
                format = Some(lit_str_from_expr(&nv.value, "format")?.value());
            }
            Meta::NameValue(nv) if nv.path.is_ident("field") => {
                field = Some(lit_str_from_expr(&nv.value, "field")?.value());
            }
            Meta::NameValue(nv) if nv.path.is_ident("message") => {
                message = Some(lit_str_from_expr(&nv.value, "message")?.value());
            }
            Meta::NameValue(nv) if nv.path.is_ident("code") => {
                code = Some(lit_str_from_expr(&nv.value, "code")?.value());
            }
            _ => {
                return Err(syn::Error::new_spanned(m, "unexpected argument"));
            }
        }
    }

    // For Values args, parse positional string exprs from the list
    if meta.args == BuiltinRuleArgs::Values {
        // Re-parse as Expr list to capture bare strings: one_of("a", "b", message = "...")
        let exprs = list.parse_args_with(Punctuated::<Expr, Token![,]>::parse_terminated)?;
        for expr in &exprs {
            // Only take string literals, skip named key=value
            if let Expr::Lit(ExprLit { lit: Lit::Str(s), .. }) = expr {
                values.push(s.value());
            }
        }
    }

    // Validate required args based on meta.args
    match meta.args {
        BuiltinRuleArgs::Format if format.is_none() => {
            return Err(syn::Error::new_spanned(list,
                format!("#[rf({}(...))] requires format = \"...\"", meta.key)));
        }
        BuiltinRuleArgs::Field if field.is_none() => {
            return Err(syn::Error::new_spanned(list,
                format!("#[rf({}(...))] requires field = \"...\"", meta.key)));
        }
        BuiltinRuleArgs::Values if values.is_empty() => {
            return Err(syn::Error::new_spanned(list,
                format!("#[rf({}(...))] requires at least one value", meta.key)));
        }
        _ => {}
    }

    Ok(BuiltinRuleUse { key: meta.key.to_string(), values, format, field })
    // message and code handled via rule_overrides or inline
}
```

Note: for `BuiltinRuleArgs::Values`, the list contains a mix of positional string exprs and named `message = "..."` args. The parser needs to handle this mixed form. Parse with `Punctuated<Expr, Token![,]>` first to get values, then re-parse with `Punctuated<Meta, Token![,]>` for named args. OR parse as a custom token stream that handles both.

A cleaner approach for `one_of("a", "b", message = "...")`:
- Parse the token stream manually: iterate tokens, string literals become values, `ident = lit` become named args.

**Step 4: Add `custom(...)` parsing**

```rust
// In the Meta::List match for "custom":
"custom" => {
    let metas = list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
    let mut function: Option<Path> = None;
    let mut description: Option<String> = None;
    let mut pattern: Option<String> = None;
    let mut custom_code: Option<String> = None;
    let mut custom_message: Option<String> = None;

    for m in metas {
        match m {
            Meta::NameValue(nv) if nv.path.is_ident("function") => {
                function = Some(path_from_expr(&nv.value, "function")?);
            }
            Meta::NameValue(nv) if nv.path.is_ident("description") => {
                description = Some(lit_str_from_expr(&nv.value, "description")?.value());
            }
            Meta::NameValue(nv) if nv.path.is_ident("pattern") => {
                pattern = Some(lit_str_from_expr(&nv.value, "pattern")?.value());
            }
            Meta::NameValue(nv) if nv.path.is_ident("code") => {
                custom_code = Some(lit_str_from_expr(&nv.value, "code")?.value());
            }
            Meta::NameValue(nv) if nv.path.is_ident("message") => {
                custom_message = Some(lit_str_from_expr(&nv.value, "message")?.value());
            }
            other => return Err(syn::Error::new_spanned(other, "unsupported custom(...) arg")),
        }
    }
    let function = function.ok_or_else(||
        syn::Error::new_spanned(&list, "custom(...) requires function = \"...\""))?;

    // Store as CustomRuleUse in FieldRfConfig
    cfg.custom_rules.push(CustomRuleUse { function, description, pattern, code: custom_code, message: custom_message });
}
```

**Step 5: Add `openapi(...)` grouped parsing**

```rust
"openapi" => {
    let metas = list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
    for m in metas {
        match m {
            Meta::NameValue(nv) if nv.path.is_ident("description") => {
                cfg.openapi_description = Some(lit_str_from_expr(&nv.value, "description")?.value());
            }
            Meta::NameValue(nv) if nv.path.is_ident("hint") => {
                cfg.openapi_hint = Some(lit_str_from_expr(&nv.value, "hint")?.value());
            }
            Meta::NameValue(nv) if nv.path.is_ident("example") => {
                cfg.openapi_example = Some(nv.value.clone());
            }
            Meta::NameValue(nv) if nv.path.is_ident("format") => {
                cfg.openapi_format = Some(lit_str_from_expr(&nv.value, "format")?.value());
            }
            other => return Err(syn::Error::new_spanned(other, "unsupported openapi(...) arg")),
        }
    }
}
```

**Step 6: Remove dead code**

- Remove `rule = "..."` / `format = ...` / `field = ...` context-dependent NameValue handling
- Remove `rule_override(...)` parsing
- Remove `values(...)` as standalone Meta::List
- Remove flat `openapi_*` NameValue keys
- Remove `pending_builtin` pattern (replaced by `local_builtin`)

**Step 7: Add `CustomRuleUse` to `FieldRfConfig` and codegen**

```rust
struct CustomRuleUse {
    function: Path,
    description: Option<String>,
    pattern: Option<String>,
    code: Option<String>,
    message: Option<String>,
}
```

In the codegen loop, for each `CustomRuleUse`:
- Generate `#[validate(custom(function = <path>, message = "...", code = "..."))]`
- Add `RuleExtensionSpec` with key = "custom", source = "app"
- If `description` provided, add to `field_desc_parts`
- If `pattern` provided, set `field_pattern_patch`

**Step 8: Run `cargo check -p rustforge-contract-macros`**

Expected: PASS (compiles but tests will fail since test code uses old syntax).

**Step 9: Commit**

```
git add rustforge-contract-macros/src/lib.rs
git commit -m "feat(macros): rewrite parsing for first-class rules, auto-derive, custom(), openapi()"
```

---

### Task 3: Update `rustforge_string_rule_type!`

**Files:**
- Modify: `core-web/src/contracts.rs`

**Step 1: Update the inner helper struct**

The `rustforge_string_rule_type!` macro creates an inner struct with `#[rustforge_contract]` + derives. Since `#[rustforge_contract]` now auto-injects derives, remove the explicit derive list:

```rust
const _: () = {
    #[::core_web::contracts::rustforge_contract]
    struct __RustforgeStringRuleTypeSchemaHelper {
        $(#[$field_attr])*
        value: String,
    }
    // ... rest stays the same
};
```

Remove `#[derive(Debug, Clone, ::serde::Deserialize, ::serde::Serialize, ::validator::Validate, ::schemars::JsonSchema)]` from the inner struct. The `#[rustforge_contract]` macro will auto-inject the needed derives.

Note: the inner struct also needs `Serialize` since it derives it currently. Check if `Serialize` is actually needed by the helper - it's only used for `Validate` and `JsonSchema`. If not needed, just let auto-inject handle it. If needed, add `#[derive(::serde::Serialize)]` alongside the macro.

**Step 2: Update `#[rf(...)]` syntax in consumer macro invocations**

The macro body uses `#[rf(rule = "alpha_dash")]` syntax. Now it should use `#[rf(alpha_dash)]`.

But wait - `rustforge_string_rule_type!` takes raw attributes from the user. The user's attributes will already use the new syntax. No change needed in the macro body itself, only the inner helper struct's derive handling.

**Step 3: Run `cargo check -p core-web`**

Expected: may fail until tests updated.

**Step 4: Commit**

```
git add core-web/src/contracts.rs
git commit -m "feat(contracts): simplify rustforge_string_rule_type inner struct"
```

---

### Task 4: Rewrite tests to new syntax

**Files:**
- Rewrite: `core-web/tests/rustforge_contract.rs`

**Step 1: Rewrite all test structs**

Convert every test contract from old syntax to new:

| Old | New |
|-----|-----|
| `#[derive(Debug, Clone, Deserialize, Validate, schemars::JsonSchema)]` | (removed - auto-injected) |
| `#[rf(rule = "alpha_dash")]` | `#[rf(alpha_dash)]` |
| `#[rf(rule = "phonenumber", field = "...")]` | `#[rf(phonenumber(field = "..."))]` |
| `#[rf(openapi_hint = "...")]` | `#[rf(openapi(hint = "..."))]` |
| `#[rf(openapi_description = "...")]` | `#[rf(openapi(description = "..."))]` |
| `#[rf(openapi_example = ...)]` | `#[rf(openapi(example = ...))]` |
| `#[rf(message = "Field-level default message")]` + `#[rf(rule_override(...))]` | `#[rf(alpha_dash(message = "Alpha-dash failed"))]` |

Remove imports that are no longer needed: `schemars::JsonSchema`, `validator::Validate`, `serde::Deserialize` (all auto-injected).

Keep `use core_web::contracts::{rustforge_contract, rustforge_string_rule_type};` and `use core_web::extract::AsyncValidate;`.

Example of the first test struct rewritten:

```rust
#[rustforge_contract]
struct DemoInput {
    #[rf(length(min = 3, max = 32))]
    #[rf(alpha_dash)]
    username: String,

    #[rf(alpha_dash)]
    optional_handle: Option<String>,

    #[rf(phonenumber(field = "contact_country_iso2"))]
    #[rf(openapi(hint = "Store raw input; server validates by country."))]
    phone: String,

    contact_country_iso2: String,
}
```

Rewrite `UsernameString` in the test:

```rust
rustforge_string_rule_type! {
    /// Username wrapper type (project-level SSOT example).
    pub struct UsernameString {
        #[validate(custom(function = "validate_username_wrapper"))]
        #[rf(length(min = 3, max = 64))]
        #[rf(alpha_dash)]
        #[rf(openapi(description = "Lowercase username using letters, numbers, _ and -.", example = "admin_user"))]
    }
}
```

Rewrite `OverrideMessageInput`:

```rust
#[rustforge_contract]
struct OverrideMessageInput {
    #[rf(length(min = 3, max = 32, message = "Bad length"))]
    #[rf(alpha_dash(message = "Alpha-dash failed"))]
    username: String,
}
```

All test assertions stay the same - they test runtime behavior and schema output, which shouldn't change.

**Step 2: Run `cargo test -p core-web`**

Expected: PASS

**Step 3: Commit**

```
git add core-web/tests/rustforge_contract.rs
git commit -m "test: rewrite contract tests to new #[rf(...)] syntax"
```

---

### Task 5: Update scaffold templates

**Files:**
- Modify: `scaffold/src/templates.rs`

**Step 1: Update all template string constants**

In every template constant that contains `#[rustforge_contract]` + `#[derive(...)]`:

1. Remove `#[derive(Debug, Clone, Deserialize, Validate, JsonSchema)]` lines
2. Remove `use schemars::JsonSchema;`, `use serde::Deserialize;`, `use validator::Validate;` imports (keep Serialize imports for response DTOs)
3. Replace `#[rf(rule = "alpha_dash")]` with `#[rf(alpha_dash)]`
4. Replace `#[rf(openapi_description = "...")]` with `#[rf(openapi(description = "..."))]`
5. Replace `#[rf(openapi_example = ...)]` with `#[rf(openapi(example = ...))]`

Affected constants:
- `APP_CONTRACTS_TYPES_USERNAME_RS`
- `APP_CONTRACTS_DATATABLE_ADMIN_ADMIN_RS`
- `APP_CONTRACTS_API_V1_ADMIN_RS`
- `APP_CONTRACTS_API_V1_ADMIN_AUTH_RS`

Note: Response DTOs (`AdminOutput`, `AdminAuthOutput`, `AdminDeleteOutput`) are NOT contracts - they use `#[derive(Serialize, JsonSchema)]` directly and don't use `#[rustforge_contract]`. Leave those unchanged.

**Step 2: Run `cargo check -p scaffold`**

Expected: PASS (templates are string constants, not compiled Rust)

**Step 3: Commit**

```
git add scaffold/src/templates.rs
git commit -m "feat(scaffold): update templates to new contract syntax"
```

---

### Task 6: Full workspace verification

**Step 1: Run full workspace check**

```
cargo check --workspace
```

Expected: PASS

**Step 2: Run full workspace tests**

```
cargo test --workspace
```

Expected: All tests PASS

**Step 3: Verify trybuild tests if any exist**

```
ls target/tests/trybuild/rustforge-contract-macros/
```

If there are compile-fail test cases, update them to match new error messages.

**Step 4: Final commit**

```
git commit -m "chore: verify full workspace builds and tests pass"
```

---

## File Change Summary

| File | Change |
|------|--------|
| `rustforge-contract-meta/src/lib.rs` | Add `BuiltinRuleArgs` enum, update all entries |
| `rustforge-contract-macros/src/lib.rs` | Rewrite parsing: auto-derives, registry-driven builtins, custom(), openapi() |
| `core-web/src/contracts.rs` | Simplify `rustforge_string_rule_type!` inner struct |
| `core-web/tests/rustforge_contract.rs` | Rewrite all test contracts to new syntax |
| `scaffold/src/templates.rs` | Update all template contracts to new syntax |

## Estimated scope

- Meta crate: ~30 lines changed
- Proc macro: ~400 lines changed (mostly parsing; codegen stays)
- Contracts: ~20 lines changed
- Tests: ~100 lines changed (syntax only, assertions unchanged)
- Scaffold: ~50 lines changed (string template syntax updates)
