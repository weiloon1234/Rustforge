import { useEffect } from 'react'
import Prism from 'prismjs'

export function AttachmentsFeature() {
    useEffect(() => {
        Prism.highlightAll()
    }, [])

    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">Attachments</h1>
                <p className="text-xl text-gray-500">
                    Schema-declared attachment fields, typed upload DTOs, and atomic model persistence.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>Where the SSOT lives</h2>
                <p>
                    Attachment behavior is split across two framework-owned sources of truth:
                </p>
                <ul>
                    <li>
                        <code>app/configs.toml</code> declares attachment types and validation rules.
                    </li>
                    <li>
                        <code>app/models/*.rs</code> declares which model fields are single or multi attachment slots.
                    </li>
                </ul>
                <p>
                    Generation owns the model APIs from those declarations. App code should pass typed upload DTOs
                    into generated setters instead of manually writing attachment rows.
                </p>

                <h2>Config + model source example</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-text">{`# app/configs.toml
[attachment_type.image]
allowed = ["image/jpeg", "image/png", "image/webp"]
[attachment_type.image.resize]
width = 1920
height = 1080
quality = 82

# app/models/article.rs
#[rf_model(table = "articles")]
pub struct Article {
    pub id: i64,
    pub status: ArticleStatus,
    #[rf(kind = "image")]
    pub cover: Attachment,
    #[rf(kind = "image")]
    pub gallery: Attachments,
}`}</code>
                </pre>

                <h2>Generated runtime surface</h2>
                <ul>
                    <li>
                        <code>XxxView.cover</code> and <code>XxxView.cover_url</code> for single attachment slots
                    </li>
                    <li>
                        <code>XxxView.gallery</code> and <code>XxxView.gallery_urls</code> for multi attachment slots
                    </li>
                    <li>
                        Insert/update setters such as <code>set_attachment_cover</code>,{' '}
                        <code>clear_attachment_cover</code>, <code>add_attachment_gallery</code>, and{' '}
                        <code>delete_attachment_gallery</code>
                    </li>
                    <li>
                        Shared platform TS exports for attachment payloads, including{' '}
                        <code>AttachmentUploadDto</code> / <code>AttachmentInput</code>
                    </li>
                </ul>

                <h2>Typed upload DTO</h2>
                <p>
                    The canonical Rust-side upload payload is <code>AttachmentUploadDto</code>. The framework also
                    exposes <code>AttachmentInput</code> as an alias, so app code can use the shorter name without
                    changing the underlying shape.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`use core_db::platform::attachments::types::AttachmentInput;

let cover = AttachmentInput::new(
    "uploads/articles/cover.jpg",
    "image/jpeg",
    125_012,
    Some(1200),
    Some(630),
);`}</code>
                </pre>

                <h2>Generated model write path</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`let article = article_model
    .insert()
    .set_attachment_cover(cover)
    .add_attachment_gallery(AttachmentInput::new(
        "uploads/articles/gallery-1.jpg",
        "image/jpeg",
        98_311,
        Some(1024),
        Some(768),
    ))
    .save()
    .await?;

println!("cover url = {:?}", article.cover_url);
println!("gallery urls = {:?}", article.gallery_urls);`}</code>
                </pre>

                <h2>Atomic persistence</h2>
                <p>
                    Generated <code>save()</code> handles the base row, localized fields, meta, and attachment
                    persistence in one transaction. Keep attachment writes inside the generated insert/update flow
                    instead of splitting them across multiple handlers.
                </p>

                <h2>Starter handoff</h2>
                <p>
                    Upload transport remains app-owned. Typical starter flow is:
                </p>
                <ol>
                    <li>receive multipart or presigned-upload completion in the API layer</li>
                    <li>store bytes via the storage service</li>
                    <li>build <code>AttachmentUploadDto</code> / <code>AttachmentInput</code></li>
                    <li>pass it into generated model setters</li>
                </ol>
                <p>
                    For starter-side examples, see the admin multipart handlers under
                    <code> scaffold/template/app/src/internal/api/v1/admin/</code>.
                </p>

                <h2>Cross-links</h2>
                <ul>
                    <li>
                        <a href="#/requests">Requests &amp; Validation</a> for contract boundary guidance around upload DTOs.
                    </li>
                    <li>
                        <a href="#/model-api-features">Framework Features on Models</a> for how attachments combine with localized/meta features.
                    </li>
                    <li>
                        <a href="#/openapi">OpenAPI</a> for transport contract generation around attachment DTOs.
                    </li>
                </ul>
            </div>
        </div>
    )
}
