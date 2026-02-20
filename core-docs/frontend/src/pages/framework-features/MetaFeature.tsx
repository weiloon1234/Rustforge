import { useEffect } from 'react'
import Prism from 'prismjs'

export function MetaFeature() {
    useEffect(() => {
        Prism.highlightAll()
    }, [])

    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">Feature: Meta</h1>
                <p className="text-xl text-gray-500">
                    JSONB-backed flexible fields with typed helper methods.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <p>
                    Meta is for model-specific dynamic fields that should not become top-level
                    columns immediately. It keeps runtime flexibility while preserving typed
                    access for declared keys.
                </p>

                <h2>Schema</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-toml">{`[model.article]
fields = ["id:i64", "status:ArticleStatus"]
meta = [
  "seo_title:string",
  "is_featured:bool",
  "priority:i32",
  "score:i64",
  "confidence:f64",
  "extra:ExtraMeta",
  "published_at:datetime"
]`}</code>
                </pre>
                <p className="text-sm text-gray-600">
                    Define <code>ExtraMeta</code> in your model type module (for example{' '}
                    <code>generated/src/extensions.rs</code> under{' '}
                    <code>extensions::article::types</code>) with serde derives.
                </p>

                <h2>Generated API Surface</h2>
                <ul>
                    <li>
                        <code>XxxView.meta: HashMap&lt;String, serde_json::Value&gt;</code> as raw
                        fallback.
                    </li>
                    <li>
                        Typed readers on <code>XxxView</code>: <code>meta_&lt;field&gt;()</code> for
                        declared scalar/datetime fields.
                    </li>
                    <li>
                        For custom meta types (for example <code>extra:ExtraMeta</code>):{' '}
                        <code>meta_&lt;field&gt;()</code> returns{' '}
                        <code>anyhow::Result&lt;Option&lt;Type&gt;&gt;</code> directly (no cast call).
                    </li>
                    <li>
                        Typed writers on <code>XxxInsert</code>:{' '}
                        <code>set_meta_&lt;field&gt;(...)</code>.
                    </li>
                    <li>
                        For <code>:json</code> fields, read/write helpers stay available:{' '}
                        <code>meta_&lt;field&gt;_as&lt;T&gt;()</code> and{' '}
                        <code>set_meta_&lt;field&gt;_as(&amp;T)</code>.
                    </li>
                </ul>

                <h2>Write Example</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`use serde_json::json;

let row = article_model
    .insert()
    .set_status(ArticleStatus::Draft)
    .set_meta_seo_title("Rust ORM Guide")
    .set_meta_is_featured(true)
    .set_meta_priority(10)
    .set_meta_score(9000)
    .set_meta_confidence(0.98)
    .set_meta_extra(json!({
        "source": "editor",
        "labels": ["orm", "rust", "dx"]
    }))
    .set_meta_published_at(time::OffsetDateTime::now_utc())
    .save()
    .await?;`}</code>
                </pre>

                <h2>Typed JSON Shape Write Example</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`use core_db::common::sql::Op;

#[derive(serde::Serialize, serde::Deserialize)]
struct ExtraMeta {
    source: String,
    labels: Vec<String>,
}

let payload = ExtraMeta {
    source: "editor".to_string(),
    labels: vec!["orm".to_string(), "rust".to_string()],
};

article_model
    .insert()
    .set_meta_extra_as(&payload)?
    .save()
    .await?;

article_model
    .update()
    .where_id(Op::Eq, 123)
    .set_meta_extra_as(&payload)?
    .save()
    .await?;`}</code>
                </pre>

                <h2>Read Example</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`#[derive(serde::Deserialize)]
struct ExtraMeta {
    source: String,
    labels: Vec<String>,
}

if let Some(view) = article_model.find(123).await? {
    let featured = view.meta_is_featured().unwrap_or(false);
    let published_at = view.meta_published_at();

    // No cast helper needed when schema uses extra:ExtraMeta
    let extra: Option<ExtraMeta> = view.meta_extra()?;

    // Raw fallback for keys not declared in schema
    let custom = view.meta.get("custom_runtime_key");

    println!("featured={featured} published_at={published_at:?} extra={extra:?} custom={custom:?}");
}`}</code>
                </pre>

                <h2>Typed-First Rule</h2>
                <p>
                    Prefer declaring expected keys in schema and using generated typed methods.
                    Use raw <code>view.meta</code> access only as an escape hatch for truly dynamic
                    data.
                </p>
            </div>
        </div>
    )
}
