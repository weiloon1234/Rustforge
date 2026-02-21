export function Responses() {
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

                <h3>Error Response (RFC 9457 Problem Details)</h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`{
  "type": "about:blank",
  "title": "Unprocessable Entity",
  "status": 422,
  "detail": "Validation failed",
  "error_code": "VALIDATION_ERROR",
  "errors": {
      "email": ["must be a valid email address"]
  }
}`}</code>
                </pre>
                <p>
                    Error content type is <code>application/problem+json</code>.
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
                                <td className="px-4 py-3 font-mono text-blue-600">type</td>
                                <td className="px-4 py-3 text-gray-500">string</td>
                                <td className="px-4 py-3 text-gray-700">
                                    Problem type URI (defaults to <code>about:blank</code>)
                                </td>
                            </tr>
                            <tr>
                                <td className="px-4 py-3 font-mono text-blue-600">title</td>
                                <td className="px-4 py-3 text-gray-500">string</td>
                                <td className="px-4 py-3 text-gray-700">HTTP error title</td>
                            </tr>
                            <tr>
                                <td className="px-4 py-3 font-mono text-blue-600">status</td>
                                <td className="px-4 py-3 text-gray-500">number</td>
                                <td className="px-4 py-3 text-gray-700">HTTP status code</td>
                            </tr>
                            <tr>
                                <td className="px-4 py-3 font-mono text-blue-600">detail</td>
                                <td className="px-4 py-3 text-gray-500">string?</td>
                                <td className="px-4 py-3 text-gray-700">
                                    Human-readable error detail
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
                                    Validation field errors (typically for 422)
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
            </div>
        </div>
    )
}
