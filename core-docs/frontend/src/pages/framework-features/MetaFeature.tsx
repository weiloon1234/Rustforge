import { useEffect } from 'react'
import Prism from 'prismjs'

export function MetaFeature() {
    useEffect(() => {
        Prism.highlightAll()
    }, [])

    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">Meta</h1>
                <p className="text-xl text-gray-500">
                    Schema-declared model metadata with typed readers/writers and a narrow raw JSON fallback.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>Where the SSOT lives</h2>
                <p>
                    Meta fields are declared in <code>app/models/*.rs</code>. The generated model API owns the
                    typed write and read helpers from that declaration. App code should not duplicate the same
                    field catalog in workflows or frontend constants.
                </p>

                <h2>Model source example</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`#[rf_model(table = "articles")]
pub struct Article {
    pub id: i64,
    pub status: ArticleStatus,
    pub seo_title: Meta<String>,
    pub is_featured: Meta<bool>,
    pub priority: Meta<i32>,
    pub score: Meta<i64>,
    pub confidence: Meta<f64>,
    pub extra: Meta<ExtraMeta>,
    pub published_at: Meta<time::OffsetDateTime>,
}`}</code>
                </pre>

                <h2>Generated runtime surface</h2>
                <ul>
                    <li>
                        <code>XxxInsert</code> / <code>XxxUpdate</code> writers such as{' '}
                        <code>set_meta_&lt;field&gt;(...)</code>
                    </li>
                    <li>
                        <code>XxxView</code> readers such as <code>meta_&lt;field&gt;()</code>
                    </li>
                    <li>
                        Raw fallback bag on the view for truly dynamic keys: <code>view.meta</code>
                    </li>
                    <li>
                        For JSON/custom types, typed serialization helpers stay available via{' '}
                        <code>_as&lt;T&gt;</code> forms
                    </li>
                </ul>

                <h2>Typed custom shapes</h2>
                <p>
                    For model-declared custom types such as <code>extra: Meta&lt;ExtraMeta&gt;</code>, define the
                    Rust type in the same model source file instead of scattering private JSON structs across handlers.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`#[derive(serde::Serialize, serde::Deserialize)]
pub struct ExtraMeta {
    pub source: String,
    pub labels: Vec<String>,
}`}</code>
                </pre>

                <h2>Write example</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`let article = article_model
    .insert()
    .set_status(ArticleStatus::Draft)
    .set_meta_seo_title("Rust ORM Guide")
    .set_meta_is_featured(true)
    .set_meta_priority(10)
    .set_meta_extra_as(&ExtraMeta {
        source: "editor".to_string(),
        labels: vec!["orm".to_string(), "rust".to_string()],
    })?
    .save()
    .await?;`}</code>
                </pre>

                <h2>Read example</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`if let Some(view) = article_model.find(123).await? {
    let featured = view.meta_is_featured().unwrap_or(false);
    let published_at = view.meta_published_at();
    let extra: Option<ExtraMeta> = view.meta_extra()?;
    let custom_runtime_key = view.meta.get("custom_runtime_key");
}`}</code>
                </pre>

                <h2>Typed-first rule</h2>
                <p>
                    Prefer declaring expected meta keys in model source and using generated typed methods. Reach for raw
                    <code>view.meta</code> only when the key is genuinely dynamic and not part of the stable model contract.
                </p>

                <h2>Cross-links</h2>
                <ul>
                    <li>
                        <a href="#/model-api-view">`XxxView` &amp; model methods</a> for the app-facing model surface.
                    </li>
                    <li>
                        <a href="#/model-api-features">Framework Features on Models</a> for how meta combines with localized and attachments.
                    </li>
                    <li>
                        <a href="#/cookbook/build-crud-admin-resource">Build a CRUD Admin Resource</a> for the starter recipe.
                    </li>
                </ul>
            </div>
        </div>
    )
}
