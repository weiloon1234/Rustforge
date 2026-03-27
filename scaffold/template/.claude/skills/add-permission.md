---
name: add-permission
description: Add new permissions to the permission catalog
---

# Add New Permissions

Follow these steps to add permissions for a new domain or feature.

## Step 1: Add entries to the permission catalog

Edit `app/permissions.toml` and add permission entries. Group related permissions together.

```toml
[[permissions]]
key = "my_domain.read"
guard = "admin"
label = "Read My Domain"
group = "my_domain"
description = "View my domain records"

[[permissions]]
key = "my_domain.create"
guard = "admin"
label = "Create My Domain"
group = "my_domain"
description = "Create new my domain records"

[[permissions]]
key = "my_domain.update"
guard = "admin"
label = "Update My Domain"
group = "my_domain"
description = "Edit existing my domain records"

[[permissions]]
key = "my_domain.delete"
guard = "admin"
label = "Delete My Domain"
group = "my_domain"
description = "Delete my domain records"
```

Conventions:
- `key`: Dot-separated, lowercase. Format: `{domain}.{action}`.
- `guard`: Which portal this permission applies to (`admin` or `user`).
- `label`: Human-readable name shown in the permission management UI.
- `group`: Groups permissions together in the UI. Usually matches the domain.
- `description`: Longer explanation of what the permission grants.
- Standard actions: `read`, `create`, `update`, `delete`. Add custom actions as needed (e.g., `export`, `approve`).

## Step 2: Add i18n translations

Permission keys like `my_domain` (group names) and labels like `"Read My Domain"` are used in the UI and need translations.

Edit `i18n/en.json`: Add the permission group key and any label where the key differs from the English value.
```json
{
  "my_domain": "My Domain"
}
```

Non-English keys (like `my_domain`) do NOT match their display value, so they MUST be added to `en.json` as well as `zh.json`.

Edit `i18n/zh.json`: Always add Chinese translations for the group key and all permission labels.
```json
{
  "my_domain": "我的域名",
  "Read My Domain": "查看我的域名",
  "Create My Domain": "创建我的域名",
  "Update My Domain": "更新我的域名",
  "Delete My Domain": "删除我的域名"
}
```

## Step 3: Regenerate the Permission enum

```bash
make gen
```

This regenerates the `Permission` enum from `permissions.toml`. The generated enum will include variants like `MyDomainRead`, `MyDomainCreate`, etc.

## Step 4: Use permissions in handlers

Reference the generated permission in handler code:

```rust
// In a handler using AuthUser guard:
async fn my_handler(
    auth: AuthUser<AdminGuard>,
) -> Result<ApiResponse<T>, AppError> {
    // Permission checked via guard or middleware
}

// In datatable hooks:
fn authorize(&self, input: &DataTableInput, ctx: &DataTableContext) -> anyhow::Result<bool> {
    let Some(actor) = ctx.actor.as_ref() else { return Ok(false); };
    let base_authorized = has_required_permissions(
        &actor.permissions,
        &[Permission::MyDomainRead.as_str()],
        PermissionMode::Any,
    );
    Ok(base_authorized)
}

// In nav.ts for frontend sidebar:
{
    permission: "my_domain.read",
}
```

## Step 5: Verify

```bash
cargo check
```

Common issues:
- Permission key format mismatch between `permissions.toml` and handler usage.
- Forgotten `make gen` step -- the `Permission` enum variant won't exist until generation runs.
- Typo in permission key string -- use `Permission::MyDomainRead.as_str()` instead of hardcoding strings.
- Missing i18n entry for non-English keys (like group names) in `en.json` -- these will show as raw keys in the UI.
