export function ModelApiFeatures() {
    return (
        <div className="space-y-8">
            <div className="space-y-3">
                <h1 className="text-4xl font-extrabold text-gray-900">Meta, Attachments, Localized</h1>
                <p className="text-xl text-gray-500">
                    Dedicated framework model features available from generated APIs.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>Meta (JSONB)</h2>
                <p>
                    Keep storage flexible in JSONB while exposing typed readers/writers from schema.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-toml">{`meta = [
  "seo_title:string",
  "priority:i32",
  "extra:ExtraMeta", # strongly typed shape
  "debug_blob:json"  # dynamic payload
]`}</code>
                </pre>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`let extra = view.meta_extra()?; // Option<ExtraMeta>
let debug = view.meta_debug_blob_as::<serde_json::Value>()?;

let insert = article.insert().set_meta_extra(&payload)?;
let update = article.update().set_meta_extra(&payload)?;`}</code>
                </pre>

                <h2>Attachments</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-toml">{`attachment = ["cover:image"]
attachments = ["gallery:image"]`}</code>
                </pre>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`article.insert().set_attachment_cover(input).save().await?;
article.update().add_attachment_gallery(input).save().await?;

let cover_url = view.cover_url.clone();
let gallery_urls = view.gallery_urls.clone();`}</code>
                </pre>

                <h2>Localized + Relations</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-toml">{`multilang = ["title", "body"]
relations = ["author:belongs_to:User:author_id:id"]`}</code>
                </pre>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`let row = article
    .insert()
    .set_title_lang(localized::Locale::En, "Hello")
    .set_title_lang(localized::Locale::Zh, "你好")
    .save()
    .await?;

let rows = article.query().with_author().get_with_relations().await?;`}</code>
                </pre>
            </div>
        </div>
    )
}
