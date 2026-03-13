export function ModelApiFeatures() {
    return (
        <div className="space-y-8">
            <div className="space-y-3">
                <h1 className="text-4xl font-extrabold text-gray-900">Framework Features on Models</h1>
                <p className="text-xl text-gray-500">
                    Localized, meta, and attachment features are model-source-owned capabilities with generated read/write APIs, not ad hoc side tables app code should manage manually.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>Why these features live on the model API</h2>
                <p>
                    These features are framework-owned storage patterns. The Rust model source declares them once, db-gen emits the Rust model API, and app code consumes them through <code>XxxView</code>, <code>XxxInsert</code>, and <code>XxxUpdate</code>. That keeps the model surface SSOT instead of splitting feature logic across repositories and raw queries.
                </p>

                <h2>Meta</h2>
                <p>
                    Meta keeps flexible storage in JSON while still emitting typed accessors and writers when the model source declares known keys.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`#[rf_model(table = "articles")]
pub struct Article {
    pub seo_title: Meta<String>,
    pub priority: Meta<i32>,
    pub extra: Meta<ExtraMeta>,
    pub debug_blob: Meta<serde_json::Value>,
}`}</code>
                </pre>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`let priority = view.meta_priority;
let extra = view.meta_extra()?;
let payload = view.meta_debug_blob_as::<serde_json::Value>()?;

article.insert().set_meta_extra(&payload)?;
article.update().set_meta_priority(10).save().await?;`}</code>
                </pre>

                <h2>Attachments</h2>
                <p>
                    Attachment fields generate typed write helpers and app-facing URL fields. App code should not own the attachment side tables directly in normal flows.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`#[rf_model(table = "articles")]
pub struct Article {
    #[rf(kind = "image")]
    pub cover: Attachment,
    #[rf(kind = "image")]
    pub gallery: Attachments,
}`}</code>
                </pre>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`article.insert().set_attachment_cover(input).save().await?;
article.update().add_attachment_gallery(input).save().await?;

let cover_url = view.cover_url.clone();
let gallery_urls = view.gallery_urls.clone();`}</code>
                </pre>

                <h2>Localized</h2>
                <p>
                    Localized fields generate locale-aware write helpers and a stable app-facing localized text shape. The current locale is applied when hydrating view-facing values, while the full localized structure remains available when the model surface exposes it.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`#[rf_model(table = "articles")]
pub struct Article {
    pub title: Localized<String>,
    pub body: Localized<String>,
}`}</code>
                </pre>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`let row = article
    .insert()
    .set_title_lang(localized::Locale::En, "Hello")
    .set_title_lang(localized::Locale::Zh, "你好")
    .save()
    .await?;`}</code>
                </pre>

                <h2>How to extend safely</h2>
                <ul>
                    <li>Add computed app-facing helpers on <a href="#/model-api-view"><code>XxxView</code> methods</a>.</li>
                    <li>Keep feature declarations in Rust model sources under <code>app/models/*.rs</code> or framework model sources.</li>
                    <li>Use generated insert/update/query APIs for normal reads and writes; do not fork these features into handwritten repositories unless there is a real gap.</li>
                </ul>

                <h2>Cross-links</h2>
                <ul>
                    <li><a href="#/attachments">Attachments</a> for the framework-level attachment runtime surface.</li>
                    <li><a href="#/meta">Meta</a> for the framework-level meta feature documentation.</li>
                    <li><a href="#/localized-relations">Localized Relations</a> for locale-aware relation behavior.</li>
                </ul>
            </div>
        </div>
    )
}
