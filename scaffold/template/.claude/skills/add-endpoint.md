---
name: add-endpoint
description: Add a new API endpoint with contract, workflow, and handler
---

# Add a New API Endpoint

Follow these steps to add a complete API endpoint with contract, workflow, and handler.

## Step 1: Create contract DTOs

Create or update `app/src/contracts/api/v1/{portal}/{domain}.rs` with input and output structs.

**Input struct** (request body):
```rust
use rustforge_prelude::prelude::*;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct CreateMyDomainInput {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    pub description: Option<String>,
}
```

**Output struct** (response body):
```rust
use core_web::ids::SnowflakeId;

#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[ts(export, export_to = "{portal}/types/")]
pub struct MyDomainOutput {
    pub id: SnowflakeId,
    pub name: String,
    pub description: Option<String>,
    pub created_at: String,
}
```

Conventions:
- Input structs derive `Deserialize, JsonSchema` (for OpenAPI docs and validation).
- Output structs derive `Serialize, JsonSchema, TS` with `#[ts(export, export_to = "{portal}/types/")]`.
- Use `#[validate(...)]` attributes for input validation.
- Use `SnowflakeId` for ID fields in output structs, NOT `i64`.
- Use typed enums for status fields, NOT strings.
- Group related DTOs in the same file.

## Step 2: Create the workflow

Create `app/src/internal/workflows/{domain}.rs` with the business logic.

```rust
use crate::contracts::api::v1::{portal}::{domain}::*;

pub async fn create(
    state: &AppApiState,
    actor_id: i64,
    input: CreateMyDomainInput,
) -> anyhow::Result<MyDomainOutput> {
    // Business logic here
    // Use state.db for database access
    // Use state.queue for job dispatch
    Ok(MyDomainOutput { ... })
}
```

Conventions:
- Workflows contain business logic, not HTTP concerns.
- Accept `&AppApiState` as the first parameter, `actor_id: i64` for the authenticated user.
- Return `anyhow::Result<T>` for error handling.
- Keep workflows testable by avoiding direct HTTP types.

## Step 3: Create the handler

Create `app/src/internal/api/v1/{portal}/{domain}.rs` with the route handler and router.

```rust
use core_web::{auth::AuthUser, contracts::ContractJson, error::AppError, response::ApiResponse};
use core_web::ids::SnowflakeId;
use crate::contracts::api::v1::{portal}::{domain}::*;
use crate::internal::workflows::{domain} as workflow;

pub fn routes() -> ApiRouter<AppApiState> {
    ApiRouter::new()
        .api_route("/my-domain", post(create_handler))
        .api_route("/my-domain/:id", get(get_handler))
}

async fn create_handler(
    State(state): State<AppApiState>,
    auth: AuthUser<AdminGuard>,
    ContractJson(req): ContractJson<CreateMyDomainInput>,
) -> Result<ApiResponse<MyDomainOutput>, AppError> {
    let result = workflow::create(&state, auth.user.id, req).await?;
    Ok(ApiResponse::success(result, t("Created successfully")))
}

async fn get_handler(
    State(state): State<AppApiState>,
    auth: AuthUser<AdminGuard>,
    Path(id): Path<SnowflakeId>,
) -> Result<ApiResponse<MyDomainOutput>, AppError> {
    let result = workflow::get(&state, id).await?;
    Ok(ApiResponse::success(result, t("Success")))
}
```

Conventions:
- Use `ContractJson<T>` for request body extraction, NOT `Json<T>`. `ContractJson` handles validation automatically.
- Use `SnowflakeId` for path ID parameters, NOT `i64`.
- Use `ApiResponse::success(data, t("message"))` for responses. The `t()` function enables i18n for API response messages.
- Use `AuthUser<AdminGuard>` or `AuthUser<UserGuard>` for authentication.
- Define a `routes()` function that returns `ApiRouter<AppApiState>`.
- Use `.api_route()` (not `.route()`) for OpenAPI integration.
- Handlers delegate to workflows -- keep them thin.

## Step 4: Wire the route

Add the route to `app/src/internal/api/v1/{portal}/mod.rs`:

```rust
pub mod {domain};

// In the router function:
.merge({domain}::routes())
```

## Step 5: Add module exports

Ensure `mod` declarations exist in:
- `app/src/contracts/api/v1/{portal}/mod.rs` -- `pub mod {domain};`
- `app/src/internal/workflows/mod.rs` -- `pub mod {domain};`
- `app/src/internal/api/v1/{portal}/mod.rs` -- `pub mod {domain};`

## Step 6: Add i18n translations

API response messages use `t("key")` for translation. Add translation keys for any user-facing messages:
- `i18n/en.json`: Only add if the key differs from the value (skip entries like `"Name": "Name"`).
- `i18n/zh.json`: Always add the Chinese translation.

Example:
```json
// i18n/zh.json
{
  "Created successfully": "创建成功",
  "Updated successfully": "更新成功",
  "Deleted successfully": "删除成功"
}
```

## Step 7: Regenerate types and verify

```bash
make gen-types
cargo check
```

Fix any compilation errors. Common issues:
- Missing permission enum variant -- add it to `app/permissions.toml` and run `make gen`.
- Missing mod declarations in parent modules.
- Import path mismatches.
