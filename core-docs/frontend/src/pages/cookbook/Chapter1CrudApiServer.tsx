import { useEffect, useState } from 'react'
import Prism from 'prismjs'

export function Chapter1CrudApiServer() {
    const [handlerPatternTab, setHandlerPatternTab] = useState<'thin' | 'fat'>('thin')

    useEffect(() => {
        Prism.highlightAll()
    }, [handlerPatternTab])

    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">
                    Chapter 1: CRUD API Server (Portal + Typed AuthZ)
                </h1>
                <p className="text-xl text-gray-500">
                    Build article category + article CRUD with relationships, eager loading,
                    localized fields, attachments, meta fields, and typed permission metadata.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>Step 0: Scope</h2>
                <ul>
                    <li>
                        SSOT: <code>app/schemas/*.toml</code>, <code>app/permissions.toml</code>, and{' '}
                        <code>app/src/contracts/api/*</code>.
                    </li>
                    <li>
                        Baseline framework migrations for <code>localized</code>, <code>attachments</code>, and{' '}
                        <code>meta</code> are expected to already exist.
                    </li>
                    <li>
                        API portals: <code>/api/v1/user/*</code> and <code>/api/v1/admin/*</code>.
                    </li>
                    <li>
                        User-facing strings use <code>core_i18n::t("...")</code>.
                    </li>
                </ul>

                <h2>Step 1: Define Schemas</h2>
                <h3>
                    File: <code>app/schemas/article_category.toml</code>
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-toml">{`[CategoryStatus]
type = "enum"
storage = "string"
variants = ["Draft", "Published"]

[model.article_category]
table = "article_category"
pk = "id"
pk_type = "i64"
id_strategy = "snowflake"

fields = [
  "id:i64",
  "status:CategoryStatus",
  "created_at:datetime",
  "updated_at:datetime"
]

multilang = ["name"]
meta = [
  "seo_title:string",
  "priority:i32"
]
relations = [
  "articles:has_many:article:category_id:id"
]`}</code>
                </pre>
                <h3>
                    File: <code>app/schemas/article.toml</code>
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-toml">{`[ArticleStatus]
type = "enum"
storage = "string"
variants = ["Draft", "Published"]

[model.article]
table = "article"
pk = "id"
pk_type = "i64"
id_strategy = "snowflake"

fields = [
  "id:i64",
  "category_id:i64",
  "status:ArticleStatus",
  "created_at:datetime",
  "updated_at:datetime"
]

multilang = ["title", "summary"]
meta = [
  "seo_title:string",
  "reading_minutes:i32",
  "is_featured:bool"
]
attachment = ["cover:image"]
attachments = ["galleries:image"]
relations = [
  "category:belongs_to:article_category:category_id:id"
]`}</code>
                </pre>
                <p>
                    This schema generates meta APIs like <code>set_meta_seo_title(...)</code>,{' '}
                    <code>set_meta_reading_minutes(...)</code>, and view helpers like{' '}
                    <code>meta_seo_title()</code>.
                </p>

                <h2>Step 2: Add Migration</h2>
                <p>
                    Add <code>migrations/&lt;new&gt;_create_article_category_and_article.sql</code>{' '}
                    with FK + indexes + enum check constraints. Do not add custom columns for
                    localized/meta/attachments data; these are stored in framework tables.
                </p>

                <h2>Step 3: Define Permission Catalog</h2>
                <h3>
                    File: <code>app/permissions.toml</code>
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-toml">{`[[permissions]]
key = "article.read"
guard = "admin"
label = "Read Articles"
group = "article"
description = "View article list and detail."

[[permissions]]
key = "article.manage"
guard = "admin"
label = "Manage Articles"
group = "article"
description = "Create, update, and delete articles."

[[permissions]]
key = "article.export"
guard = "admin"
label = "Export Articles"
group = "article"
description = "Export article datasets."

[[permissions]]
key = "article_category.manage"
guard = "admin"
label = "Manage Categories"
group = "article_category"
description = "Create, update, and delete categories."`}</code>
                </pre>

                <h2>Step 4: Regenerate + Migrate</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-bash">{`cargo check -p generated
./console migrate run`}</code>
                </pre>
                <p>
                    Generated outputs now include <code>generated::permissions::Permission</code>.
                </p>

                <h2>Step 5: Route Collections (Module-first)</h2>
                <p>
                    Use <code>with_permission_check_*</code> helpers so permission declaration
                    becomes single-source for both runtime enforcement and OpenAPI extensions.
                </p>
                <h3>
                    File: <code>app/src/internal/api/v1/article.rs</code> (admin excerpt)
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`use core_web::{
    authz::PermissionMode,
    openapi::{with_permission_check_delete, with_permission_check_patch_with, with_permission_check_post, ApiRouter},
};
use generated::{guards::AdminGuard, permissions::Permission};

pub fn admin_routes(state: AppApiState) -> ApiRouter<AppApiState> {
    ApiRouter::new()
        .api_route(
            "/articles",
            with_permission_check_post(
                create,
                AdminGuard,
                PermissionMode::Any,
                [Permission::ArticleManage],
            ),
        )
        .api_route(
            "/articles/{id}",
            with_permission_check_patch_with(
                update,
                AdminGuard,
                PermissionMode::Any,
                [Permission::ArticleManage],
                |op| op.summary("Update article"),
            )
            .merge(
                with_permission_check_delete(
                    remove,
                    AdminGuard,
                    PermissionMode::Any,
                    [Permission::ArticleManage],
                ),
            ),
        ) // require_admin middleware still required to set AuthUser<AdminGuard>
        .layer(axum::middleware::from_fn_with_state(
            state,
            crate::middleware::auth::require_admin,
        ))
}`}</code>
                </pre>
                <p>
                    For special GET actions (example CSV export), bind a dedicated permission key:
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`use core_web::openapi::with_permission_check_get;

with_permission_check_get(
    export_csv,
    AdminGuard,
    PermissionMode::Any,
    [Permission::ArticleExport],
)`}</code>
                </pre>

                <h2>Step 6: Portal Composer</h2>
                <h3>
                    File: <code>app/src/internal/api/v1/mod.rs</code>
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`pub fn router(state: AppApiState) -> ApiRouter {
    ApiRouter::new()
        .nest("/user", user_router(state.clone()))
        .nest("/admin", admin_router(state.clone()))
        .with_state(state)
}`}</code>
                </pre>

                <h2>Step 7: Thin Handler + Optional Workflow</h2>
                <p>
                    Pick one structure per module. Both patterns can use the same route-level
                    authz declaration and OpenAPI metadata.
                </p>
                <p>
                    <code>ContractJson&lt;T&gt;</code> (alias of{' '}
                    <code>ValidatedJson&lt;T&gt;</code>) and <code>AsyncValidatedJson&lt;T&gt;</code>{' '}
                    are boundary validators. If extraction succeeds, <code>req</code> can be used
                    as trusted validated input in handlers/workflows.
                </p>
                <p>
                    Write paths below use explicit transaction scope: commit on success, rollback
                    on failure.
                </p>
                <div className="not-prose rounded-xl border border-gray-200 bg-white p-4">
                    <div className="inline-flex rounded-full border border-gray-200 bg-gray-100 p-1">
                        <button
                            type="button"
                            onClick={() => setHandlerPatternTab('thin')}
                            className={`rounded-full px-4 py-1.5 text-xs font-semibold transition ${
                                handlerPatternTab === 'thin'
                                    ? 'bg-white text-gray-900 shadow-sm'
                                    : 'text-gray-600 hover:text-gray-900'
                            }`}
                        >
                            Thin handler + Fat workflow sample
                        </button>
                        <button
                            type="button"
                            onClick={() => setHandlerPatternTab('fat')}
                            className={`rounded-full px-4 py-1.5 text-xs font-semibold transition ${
                                handlerPatternTab === 'fat'
                                    ? 'bg-white text-gray-900 shadow-sm'
                                    : 'text-gray-600 hover:text-gray-900'
                            }`}
                        >
                            Fat handler (without workflow sample)
                        </button>
                    </div>

                    <div className="mt-4 space-y-4">
                        <p className="text-sm text-gray-700">
                            Shared files for both patterns (create once):
                        </p>
                        <ul className="text-sm text-gray-700 list-disc pl-5">
                            <li>
                                Keep one canonical app state in{' '}
                                <code>app/src/internal/api/state.rs</code>.
                            </li>
                            <li>
                                Keep API DTO contracts in{' '}
                                <code>app/src/contracts/api/v1/article.rs</code> and{' '}
                                <code>app/src/contracts/api/v1/article_category.rs</code>.
                            </li>
                            <li>
                                DTO rule (default): use <code>#[rustforge_contract]</code> +{' '}
                                <code>#[rf(...)]</code> so runtime validation + OpenAPI stay
                                aligned from one field-attribute style.
                            </li>
                            <li>
                                Write flows use explicit transaction scope:{' '}
                                <code>DbConn::pool(&amp;state.db).begin_scope()</code> +{' '}
                                <code>commit()</code>/<code>rollback()</code>.
                            </li>
                        </ul>
                        <h3 className="text-sm font-semibold text-gray-900">
                            File: <code>app/src/contracts/api/v1/article.rs</code> (input excerpt)
                        </h3>
                        <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                            <code className="language-rust">{`use core_web::contracts::rustforge_contract;

#[rustforge_contract]
#[derive(Debug, Clone, serde::Deserialize, validator::Validate, schemars::JsonSchema)]
pub struct ArticleMetaInput {
    #[rf(length(min = 1, max = 120))]
    #[rf(rule = "required_trimmed")]
    pub seo_title: String,
    #[rf(range(min = 0, max = 600))]
    pub reading_minutes: i32,
    pub is_featured: bool,
}

#[rustforge_contract]
#[derive(Debug, Clone, serde::Deserialize, validator::Validate, schemars::JsonSchema)]
pub struct ArticleCreateInput {
    pub category_id: i64,
    pub status: generated::models::ArticleStatus,
    pub title: generated::localized::MultiLang,
    pub summary: generated::localized::MultiLang,
    #[validate(nested)]
    pub meta: ArticleMetaInput,
}`}</code>
                        </pre>
                    </div>

                    {handlerPatternTab === 'thin' ? (
                        <div className="mt-4 space-y-4">
                            <p className="text-sm text-gray-700">
                                Create these files:
                            </p>
                            <h3 className="text-sm font-semibold text-gray-900">
                                File: <code>app/src/internal/workflows/article/mod.rs</code>
                            </h3>
                            <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                                <code className="language-rust">{`pub mod create_article;`}</code>
                            </pre>

                            <h3 className="text-sm font-semibold text-gray-900">
                                File: <code>app/src/internal/workflows/article/create_article.rs</code>
                            </h3>
                            <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                                <code className="language-rust">{`use core_i18n::t;
use contracts::api::v1::article::{ArticleCreateInput, ArticleCreateOutput};
use core_db::common::sql::DbConn;
use core_web::{error::AppError, response::ApiResponse};
use generated::{
    guards::AdminGuard,
    localized::MultiLang,
    models::Article,
};

use crate::internal::api::state::AppApiState;

fn to_lang_map(value: &MultiLang) -> std::collections::BTreeMap<String, String> {
    std::collections::BTreeMap::from([
        ("en".to_string(), value.en.clone()),
        ("zh".to_string(), value.zh.clone()),
    ])
}

pub async fn run(
    state: &AppApiState,
    _auth: &core_web::auth::AuthUser<AdminGuard>,
    req: ArticleCreateInput,
) -> Result<ApiResponse<ArticleCreateOutput>, AppError> {
    let scope = DbConn::pool(&state.db)
        .begin_scope()
        .await
        .map_err(AppError::from)?;
    let conn = scope.conn();

    let created_result = Article::new(conn, state.cdn_base.clone())
        .insert()
        .set_category_id(req.category_id)
        .set_status(req.status)
        .set_title_langs(to_lang_map(&req.title))
        .set_summary_langs(to_lang_map(&req.summary))
        .set_meta_seo_title(req.meta.seo_title)
        .set_meta_reading_minutes(req.meta.reading_minutes)
        .set_meta_is_featured(req.meta.is_featured)
        .save()
        .await;

    let created = match created_result {
        Ok(view) => {
            scope.commit().await.map_err(AppError::from)?;
            view
        }
        Err(err) => {
            scope.rollback().await.map_err(AppError::from)?;
            return Err(AppError::from(err));
        }
    };

    let _seo_title = created.meta_seo_title();

    Ok(ApiResponse::success(
        ArticleCreateOutput { id: created.id },
        &t("Article created"),
    ))
}`}</code>
                            </pre>

                            <h3 className="text-sm font-semibold text-gray-900">
                                File: <code>app/src/internal/api/v1/article.rs</code>
                            </h3>
                            <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                                <code className="language-rust">{`use axum::extract::State;
use contracts::api::v1::article::{ArticleCreateInput, ArticleCreateOutput};
use core_web::{
    authz::PermissionMode,
    contracts::ContractJson,
    error::AppError,
    openapi::{with_permission_check_post, ApiRouter},
    response::ApiResponse,
};
use generated::{guards::AdminGuard, permissions::Permission};
use crate::internal::api::state::AppApiState;

pub fn admin_routes(state: AppApiState) -> ApiRouter<AppApiState> {
    ApiRouter::new()
        .api_route(
            "/articles",
            with_permission_check_post(
                create,
                AdminGuard,
                PermissionMode::Any,
                [Permission::ArticleManage],
            ),
        )
        .layer(axum::middleware::from_fn_with_state(
            state,
            crate::middleware::auth::require_admin,
        ))
}

async fn create(
    State(state): State<AppApiState>,
    auth: core_web::auth::AuthUser<AdminGuard>,
    ContractJson(req): ContractJson<ArticleCreateInput>,
) -> Result<ApiResponse<ArticleCreateOutput>, AppError> {
    // req is already validated by ContractJson<ArticleCreateInput>
    crate::workflows::article::create_article::run(&state, &auth, req).await
}`}</code>
                            </pre>
                        </div>
                    ) : (
                        <div className="mt-4 space-y-4">
                            <p className="text-sm text-gray-700">
                                Create this file:
                            </p>
                            <h3 className="text-sm font-semibold text-gray-900">
                                File: <code>app/src/internal/api/v1/article.rs</code>
                            </h3>
                            <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                                <code className="language-rust">{`use axum::extract::State;
use contracts::api::v1::article::{ArticleCreateInput, ArticleCreateOutput};
use core_db::common::sql::DbConn;
use core_i18n::t;
use core_web::{
    authz::PermissionMode,
    contracts::ContractJson,
    error::AppError,
    openapi::{with_permission_check_post, ApiRouter},
    response::ApiResponse,
};
use generated::{guards::AdminGuard, localized::MultiLang, models::Article, permissions::Permission};
use crate::internal::api::state::AppApiState;

fn to_lang_map(value: &MultiLang) -> std::collections::BTreeMap<String, String> {
    std::collections::BTreeMap::from([
        ("en".to_string(), value.en.clone()),
        ("zh".to_string(), value.zh.clone()),
    ])
}

pub fn admin_routes(state: AppApiState) -> ApiRouter<AppApiState> {
    ApiRouter::new()
        .api_route(
            "/articles",
            with_permission_check_post(
                create,
                AdminGuard,
                PermissionMode::Any,
                [Permission::ArticleManage],
            ),
        )
        .layer(axum::middleware::from_fn_with_state(
            state,
            crate::middleware::auth::require_admin,
        ))
}

async fn create(
    State(state): State<AppApiState>,
    _auth: core_web::auth::AuthUser<AdminGuard>,
    ContractJson(req): ContractJson<ArticleCreateInput>,
) -> Result<ApiResponse<ArticleCreateOutput>, AppError> {
    // req is already validated by ContractJson<ArticleCreateInput>
    let scope = DbConn::pool(&state.db)
        .begin_scope()
        .await
        .map_err(AppError::from)?;
    let conn = scope.conn();

    let created_result = Article::new(conn, state.cdn_base.clone())
        .insert()
        .set_category_id(req.category_id)
        .set_status(req.status)
        .set_title_langs(to_lang_map(&req.title))
        .set_summary_langs(to_lang_map(&req.summary))
        .set_meta_seo_title(req.meta.seo_title)
        .set_meta_reading_minutes(req.meta.reading_minutes)
        .set_meta_is_featured(req.meta.is_featured)
        .save()
        .await;

    let created = match created_result {
        Ok(view) => {
            scope.commit().await.map_err(AppError::from)?;
            view
        }
        Err(err) => {
            scope.rollback().await.map_err(AppError::from)?;
            return Err(AppError::from(err));
        }
    };

    let _seo_title = created.meta_seo_title();

    Ok(ApiResponse::success(
        ArticleCreateOutput { id: created.id },
        &t("Article created"),
    ))
}`}</code>
                            </pre>
                        </div>
                    )}
                </div>

                <h2>Step 8: Model Extension Pattern (DX Default)</h2>
                <p>
                    Use generated model types with this app-level extension rule:
                </p>
                <ul>
                    <li>
                        <code>Article&lt;'db&gt;</code>: model gateway (query/insert/update entrypoint).
                    </li>
                    <li>
                        <code>ArticleView</code>: app-facing read model (default extension target).
                    </li>
                    <li>
                        <code>ArticleRow</code>: internal DB hydration shape (do not extend in app layer).
                    </li>
                </ul>
                <h3>
                    File: <code>app/src/internal/article_ext.rs</code>
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`use core_db::common::sql::OrderDir;
use generated::models::{ArticleCol, ArticleQuery, ArticleView};

pub trait ArticleViewExt {
    fn public_id(&self) -> String;
    fn is_featured_meta(&self) -> bool;
}

impl ArticleViewExt for ArticleView {
    fn public_id(&self) -> String {
        format!("ART-{}", self.id)
    }

    fn is_featured_meta(&self) -> bool {
        self.meta_is_featured().unwrap_or(false)
    }
}

pub trait ArticleQueryExt<'db> {
    fn latest_first(self) -> Self;
}

impl<'db> ArticleQueryExt<'db> for ArticleQuery<'db> {
    fn latest_first(self) -> Self {
        self.order_by(ArticleCol::CreatedAt, OrderDir::Desc)
    }
}`}</code>
                </pre>
                <h3>
                    File: <code>app/src/internal/lib.rs</code>
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`pub mod article_ext;`}</code>
                </pre>
                <p>
                    Typical use cases: computed output fields, typed meta helper methods, and
                    reusable query scopes shared by handlers/workflows.
                </p>

                <h2>Step 9: Verify</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-bash">{`./console route list
./console route list --json

curl http://127.0.0.1:3000/api/v1/user/articles
curl -X POST http://127.0.0.1:3000/api/v1/admin/articles \
  -H 'Authorization: Bearer <ACCESS_TOKEN>' \
  -H 'Content-Type: application/json' \
  -d '{"category_id":1,"status":"Draft","title":{"en":"Hello","zh":"你好"},"summary":{"en":"Sample","zh":"示例"},"meta":{"seo_title":"Hello SEO","reading_minutes":4,"is_featured":true}}'`}</code>
                </pre>
            </div>
        </div>
    )
}
