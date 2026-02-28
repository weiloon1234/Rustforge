# Validation

Custom validation rules — both sync and async (DB).

## Sync Validators

Return `Result<(), ValidationError>`. Use in contracts with `#[validate(custom(function = "path"))]`.

```rust
use std::borrow::Cow;
use validator::ValidationError;

pub fn validate_slug(value: &str) -> Result<(), ValidationError> {
    if value.contains("--") {
        let mut err = ValidationError::new("slug");
        err.message = Some(Cow::from("Slug cannot contain consecutive hyphens"));
        return Err(err);
    }
    Ok(())
}
```

## Async Validators (DB)

For `async_unique` / `async_exists` rules, the `#[rf(...)]` macro generates async validation automatically. For custom async checks, implement `AsyncValidate`:

```rust
use core_web::extract::AsyncValidate;

#[async_trait]
impl AsyncValidate for MyInput {
    async fn validate_async(&self, db: &sqlx::PgPool) -> Result<(), validator::ValidationErrors> {
        // Custom DB checks
        Ok(())
    }
}
```
