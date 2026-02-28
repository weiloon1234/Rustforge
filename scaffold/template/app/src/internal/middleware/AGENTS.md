# Middleware

Custom middleware functions. Framework applies standard stack (CORS, rate limit, timeout, compression, auth headers) automatically.

## Auth Middleware Pattern

```rust
use axum::{extract::State, http::Request, middleware::Next, response::Response};
use core_web::{auth, error::AppError};
use generated::guards::AdminGuard;

pub async fn require_admin<B>(
    State(state): State<AppApiState>,
    mut req: Request<B>,
    next: Next<B>,
) -> Result<Response, AppError> {
    let token = auth::extract_bearer_token(req.headers())
        .ok_or_else(|| AppError::Unauthorized(t("Missing token")))?;
    let auth_user = auth::authenticate_token::<AdminGuard>(&state.db, &token).await?;
    req.extensions_mut().insert(auth_user);
    Ok(next.run(req).await)
}
```

Apply to routes via `from_fn_with_state(state, require_admin)`.
