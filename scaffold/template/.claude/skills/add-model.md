---
name: add-model
description: Add a new database model with migration and code generation
---

# Add a New Database Model

Follow these steps to add a new database model to the application.

## Step 1: Create the model file

Create `app/models/{name}.rs` with the model struct and any associated enums or relationships.

```rust
use rustforge_prelude::prelude::*;

/// Optional enums used by this model.
#[derive(EnumIter, DeriveActiveEnum, Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS, Default)]
#[sea_orm(rs_type = "String")]
#[ts(export, export_to = "admin/types/")]
pub enum MyStatus {
    #[default]
    #[sea_orm(string_value = "active")]
    Active,
    #[sea_orm(string_value = "inactive")]
    Inactive,
}

#[rf_model]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub name: String,
    pub status: MyStatus,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}
```

Key conventions:
- Use `#[rf_model]` attribute macro on the struct.
- Primary key is typically `id: i64`.
- Always include `created_at` and `updated_at` timestamps.
- Enums that appear in API responses need `Serialize, Deserialize, JsonSchema, TS` derives.
- Set `#[ts(export, export_to = "{portal}/types/")]` on enums exported to the frontend.

## Step 2: Register the model module

Add `pub mod {name};` to `app/models/mod.rs` if it uses explicit module declarations. Some projects auto-discover models -- check how existing models are registered.

## Step 3: Create the database migration

Create a migration file at `migrations/{timestamp}_{name}.sql`. Determine the next migration number by checking the last file in `migrations/`.

```sql
CREATE TABLE IF NOT EXISTS {table_name} (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_{table_name}_status ON {table_name} (status);
```

Conventions:
- Table name is the plural snake_case form of the model name.
- Use `BIGSERIAL` for auto-incrementing primary keys.
- Use `TIMESTAMPTZ` for all timestamp columns.
- Add indexes for columns used in filters or lookups.
- Add foreign key constraints where applicable.

## Step 4: Run code generation

```bash
make gen
```

This regenerates the Rust types from the database schema and updates the generated crate.

## Step 5: Generate TypeScript types

```bash
make gen-types
```

This regenerates TypeScript types from Rust structs with `#[ts(export)]`. Run this whenever you add or change enums/structs that are exported to the frontend.

## Step 6: Add i18n translations for enums

If the model includes enums with user-facing labels, add translation keys for each variant.

Use the `enum.{model}.{variant}` dot-notation pattern for enum labels:

```json
// i18n/en.json — non-English keys MUST be in en.json
{
  "enum.my_status.active": "Active",
  "enum.my_status.inactive": "Inactive"
}
```

```json
// i18n/zh.json — every key needs translation
{
  "enum.my_status.active": "活跃",
  "enum.my_status.inactive": "非活跃"
}
```

Non-English keys (like `enum.my_status.active`) MUST exist in both `en.json` and `zh.json`. Plain English keys (like `"Submit"`) only need `zh.json`.

## Step 7: Verify

```bash
cargo check
```

Fix any compilation errors before proceeding. Common issues:
- Missing `use` imports for enum types.
- Enum variants not matching the `string_value` in the database.
- Missing `Default` impl on enums used with `#[serde(default)]`.
- Forgetting `make gen-types` after adding `#[ts(export)]` enums.
