---
name: add-datatable
description: Add a new admin datatable with hooks, registration, and frontend page
---

# Add a New Admin Datatable

Follow these steps to add a server-side datatable with backend hooks and a frontend page.

## Step 1: Create the datatable hooks module

Create `app/src/internal/datatables/v1/admin/{domain}.rs`:

```rust
use core_datatable::{DataTableContext, DataTableInput, DataTableRegistry};
use core_db::common::{model_api::Query, sql::Op};
use core_web::authz::{has_required_permissions, PermissionMode};
use core_web::datatable::{routes_for_scoped_contract_with_options, DataTableRouteOptions, DataTableRouteState};
use core_web::openapi::ApiRouter;
use generated::{models::*, permissions::Permission};

#[derive(Default, Clone)]
pub struct MyDomainDataTableAppHooks;

impl MyDomainDataTableHooks for MyDomainDataTableAppHooks {
    fn scope<'db>(
        &'db self,
        query: Query<'db, MyDomainModel>,
        _input: &DataTableInput,
        _ctx: &DataTableContext,
    ) -> Query<'db, MyDomainModel> {
        query
    }

    fn authorize(&self, input: &DataTableInput, ctx: &DataTableContext) -> anyhow::Result<bool> {
        let Some(actor) = ctx.actor.as_ref() else {
            return Ok(false);
        };
        let base_authorized = has_required_permissions(
            &actor.permissions,
            &[Permission::MyDomainRead.as_str(), Permission::MyDomainManage.as_str()],
            PermissionMode::Any,
        );
        Ok(authorize_with_optional_export(base_authorized, input, ctx))
    }

    fn filter_query<'db>(
        &'db self,
        query: Query<'db, MyDomainModel>,
        filter_key: &str,
        value: &str,
        _input: &DataTableInput,
        _ctx: &DataTableContext,
    ) -> anyhow::Result<Option<Query<'db, MyDomainModel>>> {
        match filter_key {
            "q" => Ok(Some(apply_keyword_filter(query, value))),
            "f-status" => {
                if let Some(s) = MyDomainStatus::from_storage(value) {
                    Ok(Some(query.where_col(MyDomainCol::STATUS, Op::Eq, s)))
                } else {
                    Ok(Some(query))
                }
            }
            _ => Ok(None),
        }
    }

    fn map_row(
        &self,
        _row: &mut MyDomainRecord,
        _input: &DataTableInput,
        _ctx: &DataTableContext,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    fn row_to_record(
        &self,
        row: MyDomainRecord,
        _input: &DataTableInput,
        _ctx: &DataTableContext,
    ) -> anyhow::Result<serde_json::Map<String, serde_json::Value>> {
        let mut record = self.default_row_to_record(row.clone())?;
        // Add computed fields to the record:
        record.insert("status_label".into(), serde_json::Value::String(row.status_label()));
        Ok(record)
    }
}

fn apply_keyword_filter<'db>(
    query: Query<'db, MyDomainModel>,
    value: &str,
) -> Query<'db, MyDomainModel> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return query;
    }
    if let Ok(id) = trimmed.parse::<i64>() {
        return query.where_col(MyDomainCol::ID, Op::Eq, id);
    }
    // Add additional keyword search columns as needed:
    // query.where_col(MyDomainCol::NAME, Op::ILike, format!("%{trimmed}%"))
    query
}

pub type AppMyDomainDataTable = MyDomainDataTable<MyDomainDataTableAppHooks>;

pub fn app_my_domain_datatable(db: sqlx::PgPool) -> AppMyDomainDataTable {
    MyDomainDataTable::new(db).with_hooks(MyDomainDataTableAppHooks::default())
}

pub fn register_scoped(registry: &mut DataTableRegistry, db: sqlx::PgPool) {
    registry.register_as(SCOPED_KEY, app_my_domain_datatable(db));
}
```

Conventions:
- The hooks struct implements the generated `{Model}DataTableHooks` trait, not a generic trait.
- `authorize()` uses `has_required_permissions` with `PermissionMode::Any` for checking multiple permissions.
- `filter_query()` returns `Ok(None)` for unrecognized filter keys (lets the framework handle them).
- `row_to_record()` uses `default_row_to_record()` then inserts computed fields.
- `apply_keyword_filter()` handles empty input and numeric ID lookups.
- The type alias `AppMyDomainDataTable` and constructor function follow the naming convention.

## Step 2: Register in the admin datatable module

Update `app/src/internal/datatables/v1/admin/mod.rs`:

```rust
pub mod {domain};

// Add to the register function:
pub fn register_scoped(registry: &mut DataTableRegistry, db: sqlx::PgPool) {
    // ... existing registrations
    {domain}::register_scoped(registry, db.clone());
}
```

## Step 3: Create the frontend page

Create `frontend/src/{portal}/pages/{group}/{Domain}Page.tsx`:

```tsx
import { DataTable } from "@shared/components/DataTable";
import { useTranslation } from "react-i18next";

export default function MyDomainPage() {
  const { t } = useTranslation();

  return (
    <div>
      <h1 className="text-2xl font-bold mb-4">{t("My Domain")}</h1>
      <DataTable
        url="datatable/my-domain/query"
        columns={[
          { key: "id", label: t("ID"), sortable: true },
          { key: "name", label: t("Name"), sortable: true },
          { key: "status_label", label: t("Status"), sortable: true },
          { key: "created_at", label: t("Created At"), sortable: true },
        ]}
      />
    </div>
  );
}
```

Conventions:
- Use `useTranslation()` for all user-visible text.
- Use canonical Tailwind CSS classes (never arbitrary values when a built-in utility exists).
- Column keys must match the fields returned by `row_to_record()` in the hooks.

## Step 4: Add the route

Update `frontend/src/{portal}/App.tsx`:

```tsx
import MyDomainPage from "@{portal}/pages/{group}/MyDomainPage";

// Inside the route tree:
<Route path="/{group}/my-domain" element={<MyDomainPage />} />
```

## Step 5: Add sidebar navigation

Update `frontend/src/{portal}/nav.ts`:

```ts
{
  label: "My Domain",
  path: "/{group}/my-domain",
  icon: IconName,
  permission: "my_domain.read",
}
```

## Step 6: Add i18n translations

Edit `i18n/en.json`: Only add entries where the key differs from the value.

Edit `i18n/zh.json`: Always add Chinese translations for the page title, column labels, and filter labels.

## Step 7: Generate types and verify

```bash
make gen-types
make check
```

This regenerates TypeScript types and runs full verification.
