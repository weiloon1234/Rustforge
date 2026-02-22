import { useEffect } from 'react'
import Prism from 'prismjs'

export function Responses() {
    useEffect(() => {
        Prism.highlightAll()
    }, [])

    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">Responses</h1>
                <p className="text-xl text-gray-500">Standardized API Output.</p>
            </div>

            <div className="prose prose-orange max-w-none">
                <p>
                    To ensure consistency across all endpoints, the framework enforces a
                    unified success envelope via{' '}
                    <code className="language-rust">{'ApiResponse<T>'}</code>.
                </p>

                <h3>Success Response</h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`{
  "data": { ... },        // The generic payload T
  "message": "User created" // optional
}`}</code>
                </pre>

                <h3>Error Response</h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`{
  "message": "Validation failed", // always present
  "error_code": "VALIDATION_ERROR",
  "errors": {
      "email": ["must be a valid email address"]
  }
}`}</code>
                </pre>
                <p>
                    Validation errors use the <code>errors</code> map (field to message array).
                </p>

                <h3>Response Fields</h3>
                <div className="overflow-x-auto border rounded-lg mt-4">
                    <table className="min-w-full divide-y divide-gray-200 text-sm">
                        <thead className="bg-gray-50">
                            <tr>
                                <th className="px-4 py-3 text-left font-semibold text-gray-900">
                                    Field
                                </th>
                                <th className="px-4 py-3 text-left font-semibold text-gray-900">
                                    Type
                                </th>
                                <th className="px-4 py-3 text-left font-semibold text-gray-900">
                                    Description
                                </th>
                            </tr>
                        </thead>
                        <tbody className="divide-y divide-gray-200 bg-white">
                            <tr>
                                <td className="px-4 py-3 font-mono text-blue-600">data</td>
                                <td className="px-4 py-3 text-gray-500">T</td>
                                <td className="px-4 py-3 text-gray-700">Success payload</td>
                            </tr>
                            <tr>
                                <td className="px-4 py-3 font-mono text-blue-600">message</td>
                                <td className="px-4 py-3 text-gray-500">string?</td>
                                <td className="px-4 py-3 text-gray-700">Optional success message</td>
                            </tr>
                            <tr>
                                <td className="px-4 py-3 font-mono text-blue-600">message</td>
                                <td className="px-4 py-3 text-gray-500">string</td>
                                <td className="px-4 py-3 text-gray-700">
                                    Required in error payload. Defaults to HTTP status text when not provided.
                                </td>
                            </tr>
                            <tr>
                                <td className="px-4 py-3 font-mono text-blue-600">error_code</td>
                                <td className="px-4 py-3 text-gray-500">string?</td>
                                <td className="px-4 py-3 text-gray-700">
                                    Machine-readable app error code
                                </td>
                            </tr>
                            <tr>
                                <td className="px-4 py-3 font-mono text-blue-600">errors</td>
                                <td className="px-4 py-3 text-gray-500">{'{field: string[]}?'} </td>
                                <td className="px-4 py-3 text-gray-700">
                                    Validation field errors (typically 422, Laravel-style)
                                </td>
                            </tr>
                        </tbody>
                    </table>
                </div>

                <h3 className="mt-8">Usage in Handlers</h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`use core_web::response::ApiResponse;

// 1. Success with data
async fn get_user() -> ApiResponse<Json<User>> {
    let user = User::find(&db, id).await?;
    ApiResponse::success(Json(user), "User retrieved")
}

// 2. Created (201)
async fn create_user() -> ApiResponse<Json<User>> {
    let user = user_model.save(&db).await?;
    ApiResponse::created(Json(user), "User created")
}`}</code>
                </pre>

                <div className="bg-blue-50 border-l-4 border-blue-400 p-4 mt-6">
                    <p className="text-sm text-blue-700">
                        <strong>Pro Tip:</strong> The <code>ApiResponse</code> struct
                        automatically implements <code>IntoResponse</code> for Axum and{' '}
                        <code>OperationOutput</code> for OpenAPI generation.
                    </p>
                </div>

                <h3 className="mt-8">AppError Variants</h3>
                <div className="overflow-x-auto border rounded-lg mt-4">
                    <table className="min-w-full divide-y divide-gray-200 text-sm">
                        <thead className="bg-gray-50">
                            <tr>
                                <th className="px-4 py-3 text-left font-semibold text-gray-900">
                                    Variant
                                </th>
                                <th className="px-4 py-3 text-left font-semibold text-gray-900">
                                    Status
                                </th>
                                <th className="px-4 py-3 text-left font-semibold text-gray-900">
                                    error_code
                                </th>
                                <th className="px-4 py-3 text-left font-semibold text-gray-900">
                                    Description
                                </th>
                            </tr>
                        </thead>
                        <tbody className="divide-y divide-gray-200 bg-white">
                            <tr>
                                <td className="px-4 py-3 font-mono text-blue-600">Internal(anyhow::Error)</td>
                                <td className="px-4 py-3 text-gray-500">500</td>
                                <td className="px-4 py-3 font-mono text-gray-700">INTERNAL_ERROR</td>
                                <td className="px-4 py-3 text-gray-700">
                                    Wraps any error via <code>?</code> operator
                                </td>
                            </tr>
                            <tr>
                                <td className="px-4 py-3 font-mono text-blue-600">NotFound(String)</td>
                                <td className="px-4 py-3 text-gray-500">404</td>
                                <td className="px-4 py-3 font-mono text-gray-700">NOT_FOUND</td>
                                <td className="px-4 py-3 text-gray-700">Resource not found</td>
                            </tr>
                            <tr>
                                <td className="px-4 py-3 font-mono text-blue-600">BadRequest(String)</td>
                                <td className="px-4 py-3 text-gray-500">400</td>
                                <td className="px-4 py-3 font-mono text-gray-700">BAD_REQUEST</td>
                                <td className="px-4 py-3 text-gray-700">Invalid request</td>
                            </tr>
                            <tr>
                                <td className="px-4 py-3 font-mono text-blue-600">Unauthorized(String)</td>
                                <td className="px-4 py-3 text-gray-500">401</td>
                                <td className="px-4 py-3 font-mono text-gray-700">UNAUTHORIZED</td>
                                <td className="px-4 py-3 text-gray-700">Not authenticated</td>
                            </tr>
                            <tr>
                                <td className="px-4 py-3 font-mono text-blue-600">Forbidden(String)</td>
                                <td className="px-4 py-3 text-gray-500">403</td>
                                <td className="px-4 py-3 font-mono text-gray-700">FORBIDDEN</td>
                                <td className="px-4 py-3 text-gray-700">Not authorized</td>
                            </tr>
                            <tr>
                                <td className="px-4 py-3 font-mono text-blue-600">TooManyRequests(String)</td>
                                <td className="px-4 py-3 text-gray-500">429</td>
                                <td className="px-4 py-3 font-mono text-gray-700">RATE_LIMITED</td>
                                <td className="px-4 py-3 text-gray-700">Rate limit exceeded</td>
                            </tr>
                            <tr>
                                <td className="px-4 py-3 font-mono text-blue-600">UnprocessableEntity(String)</td>
                                <td className="px-4 py-3 text-gray-500">422</td>
                                <td className="px-4 py-3 font-mono text-gray-700">VALIDATION_ERROR</td>
                                <td className="px-4 py-3 text-gray-700">Validation failed (no field errors)</td>
                            </tr>
                            <tr>
                                <td className="px-4 py-3 font-mono text-blue-600">{'Validation { message, errors }'}</td>
                                <td className="px-4 py-3 text-gray-500">422</td>
                                <td className="px-4 py-3 font-mono text-gray-700">VALIDATION_ERROR</td>
                                <td className="px-4 py-3 text-gray-700">Validation with field-level errors</td>
                            </tr>
                        </tbody>
                    </table>
                </div>

                <h3 className="mt-8">Returning Errors from Handlers</h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-rust">{`use core_web::error::AppError;
use core_i18n::t;

// 404 — resource not found
async fn get_article(
    State(state): State<AppApiState>,
    Path(id): Path<i64>,
) -> Result<ApiResponse<ArticleView>, AppError> {
    let article = Article::find(&state.db, id).await?
        .ok_or_else(|| AppError::NotFound(t("Article not found")))?;
    Ok(ApiResponse::success(article, ""))
}

// 403 — business logic authorization
async fn delete_article(
    State(state): State<AppApiState>,
    auth: Auth<WebGuard>,
    Path(id): Path<i64>,
) -> Result<ApiResponse<()>, AppError> {
    if !auth.user.can_manage_articles() {
        return Err(AppError::Forbidden(t("Not allowed to delete articles")));
    }
    // ... delete logic
    Ok(ApiResponse::success((), &t("Article deleted")))
}

// 422 — manual validation with field errors
async fn check_availability() -> Result<ApiResponse<()>, AppError> {
    let mut errors = std::collections::HashMap::new();
    errors.insert("email".to_string(), vec!["Email already taken".to_string()]);
    Err(AppError::Validation {
        message: "Validation failed".to_string(),
        errors,
    })
}`}</code>
                </pre>

                <h3 className="mt-8">Auto-Conversion</h3>
                <div className="bg-blue-50 border-l-4 border-blue-400 p-4 mt-4">
                    <p className="text-sm text-blue-700">
                        The blanket impl{' '}
                        <code>{'From<E: Into<anyhow::Error>> for AppError'}</code> means any{' '}
                        <code>?</code> on a <code>Result</code> automatically wraps the error into{' '}
                        <code>AppError::Internal</code>. This means expressions like{' '}
                        <code>{'sqlx::query(...).execute(&db).await?'}</code> just work -- the
                        database error is captured, logged, and returned as a 500 with{' '}
                        <code>INTERNAL_ERROR</code> without any manual conversion.
                    </p>
                </div>
            </div>
        </div>
    )
}
