import { useEffect } from 'react'
import Prism from 'prismjs'

export function Chapter11TestingRecipe() {
    useEffect(() => {
        Prism.highlightAll()
    }, [])

    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">
                    Chapter 11: Testing Recipe
                </h1>
                <p className="text-xl text-gray-500">
                    Standard patterns for testing contracts, handlers, jobs, and full integration
                    flows using cargo test.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>Step 0: Scope</h2>
                <ul>
                    <li>
                        Uses <code>cargo test</code> with <code>#[tokio::test]</code>.
                    </li>
                    <li>
                        Test database: separate DB via <code>TEST_DATABASE_URL</code>.
                    </li>
                    <li>
                        Framework provides no test harness — build on standard Rust testing.
                    </li>
                    <li>
                        This chapter covers: contracts, handlers, jobs, integration tests.
                    </li>
                </ul>

                <h2>Step 1: Test Helper Setup</h2>
                <h3>
                    File: <code>app/tests/helpers/mod.rs</code>
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-rust">{`use core_db::infra::cache::{Cache, create_cache};
use core_config::RedisSettings;
use sqlx::PgPool;

pub async fn test_db() -> PgPool {
    let url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/myapp_test".to_string());
    PgPool::connect(&url).await.expect("Failed to connect to test DB")
}

pub async fn test_cache() -> Cache {
    let settings = RedisSettings {
        url: std::env::var("TEST_REDIS_URL")
            .unwrap_or_else(|_| "redis://127.0.0.1/1".to_string()),
        prefix: Some("test".to_string()),
    };
    create_cache(&settings).await.expect("Failed to connect to test Redis")
}`}</code>
                </pre>

                <h3>
                    File: <code>app/tests/helpers/state.rs</code>
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-rust">{`use crate::helpers;
use crate::internal::api::state::AppApiState;
use core_jobs::queue::RedisQueue;

pub async fn test_state() -> AppApiState {
    AppApiState {
        db: helpers::test_db().await,
        queue: RedisQueue::new("redis://127.0.0.1/1").unwrap(),
        cache: helpers::test_cache().await,
        cdn_base: None,
    }
}`}</code>
                </pre>

                <h2>Step 2: Test Contracts (Validation)</h2>
                <h3>
                    File: <code>app/tests/contracts/article_test.rs</code>
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-rust">{`use validator::Validate;
use crate::contracts::api::v1::article::CreateArticleInput;

#[test]
fn create_article_valid_input_passes() {
    let input = CreateArticleInput {
        title: "My Article".to_string(),
        status: "draft".to_string(),
    };
    assert!(input.validate().is_ok());
}

#[test]
fn create_article_empty_title_fails() {
    let input = CreateArticleInput {
        title: "".to_string(),
        status: "draft".to_string(),
    };
    let result = input.validate();
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.field_errors().contains_key("title"));
}

#[test]
fn create_article_invalid_status_fails() {
    let input = CreateArticleInput {
        title: "My Article".to_string(),
        status: "invalid_status".to_string(),
    };
    let result = input.validate();
    assert!(result.is_err());
}`}</code>
                </pre>

                <h2>Step 3: Test Handlers (HTTP layer)</h2>
                <h3>
                    File: <code>app/tests/api/article_test.rs</code>
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-rust">{`use axum::{body::Body, http::{Request, StatusCode}};
use tower::ServiceExt;

#[tokio::test]
async fn list_articles_returns_200() {
    let state = helpers::state::test_state().await;
    let app = crate::internal::api::v1::article::router()
        .with_state(state);

    let req = Request::builder()
        .uri("/articles")
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn create_article_with_valid_body_returns_201() {
    let state = helpers::state::test_state().await;
    let app = crate::internal::api::v1::article::router()
        .with_state(state);

    let body = serde_json::json!({
        "title": "Test Article",
        "status": "draft"
    });

    let req = Request::builder()
        .method("POST")
        .uri("/articles")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn create_article_with_invalid_body_returns_422() {
    let state = helpers::state::test_state().await;
    let app = crate::internal::api::v1::article::router()
        .with_state(state);

    let body = serde_json::json!({ "title": "" });

    let req = Request::builder()
        .method("POST")
        .uri("/articles")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
}`}</code>
                </pre>

                <h2>Step 4: Test Jobs</h2>
                <h3>
                    File: <code>app/tests/jobs/rebuild_index_test.rs</code>
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-rust">{`use core_jobs::JobContext;
use crate::internal::jobs::definitions::rebuild_article_index::RebuildArticleIndexJob;

async fn test_job_context() -> JobContext {
    JobContext {
        db: helpers::test_db().await,
        redis: helpers::test_cache().await,
        settings: std::sync::Arc::new(core_config::Settings::from_env()),
        extensions: axum::http::Extensions::new(),
    }
}

#[tokio::test]
async fn rebuild_index_job_succeeds() {
    let ctx = test_job_context().await;
    let job = RebuildArticleIndexJob { article_id: 1 };
    let result = job.handle(&ctx).await;
    assert!(result.is_ok());
}`}</code>
                </pre>

                <h2>Step 5: Integration Test</h2>
                <h3>
                    File: <code>app/tests/integration/article_crud_test.rs</code>
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-rust">{`#[tokio::test]
async fn article_crud_lifecycle() {
    let db = helpers::test_db().await;
    let mut tx = db.begin().await.unwrap();

    // Create
    let article = Article::new(&mut *tx)
        .insert()
        .set_title("Integration Test Article")
        .save()
        .await
        .unwrap();
    assert!(article.id > 0);

    // Read
    let found = Article::find(&mut *tx, article.id).await.unwrap();
    assert!(found.is_some());

    // Update
    Article::new(&mut *tx)
        .update()
        .where_id(Op::Eq, article.id)
        .set_title("Updated Title")
        .save()
        .await
        .unwrap();

    // Delete
    Article::new(&mut *tx)
        .delete()
        .where_id(Op::Eq, article.id)
        .execute()
        .await
        .unwrap();

    // Rollback (clean up test data)
    tx.rollback().await.unwrap();
}`}</code>
                </pre>

                <h2>Step 6: Run Tests</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-bash">{`# Run all app tests
cargo test -p app

# Run single test file
cargo test -p app --test article_test

# Sequential (if tests share DB state)
cargo test -p app -- --test-threads=1

# With test database
TEST_DATABASE_URL=postgres://localhost/myapp_test cargo test -p app`}</code>
                </pre>

                <h2>Chapter Decision Rule</h2>
                <ul>
                    <li>Unit test: contracts, pure logic, job handlers.</li>
                    <li>Integration test: full request → DB → response cycle.</li>
                    <li>Don't test: generated code, framework internals.</li>
                    <li>Always test: custom validation rules, business logic in workflows.</li>
                    <li>Use transaction rollback to keep test DB clean.</li>
                    <li>
                        Keep tests independent — no test should depend on another test's side
                        effects.
                    </li>
                </ul>
            </div>
        </div>
    )
}
