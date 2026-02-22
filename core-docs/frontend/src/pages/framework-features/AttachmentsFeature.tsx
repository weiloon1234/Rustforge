import { useEffect } from 'react'
import Prism from 'prismjs'

export function AttachmentsFeature() {
    useEffect(() => {
        Prism.highlightAll()
    }, [])

    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">Feature: Attachments</h1>
                <p className="text-xl text-gray-500">
                    Typed attachment inputs, generated URL fields, and atomic persistence.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <p>
                    Attachments are declared in schema and validated by type definitions in
                    <code>app/configs.toml</code>. The generated model handles read hydration and
                    write orchestration for single and multi attachment fields.
                </p>

                <h2>Config + Schema</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-toml">{`# app/configs.toml
[attachment_type.image]
allowed = ["image/jpeg", "image/png", "image/webp"]
[attachment_type.image.resize]
width = 1920
height = 1080
quality = 82

# app/schemas/article.toml
[model.article]
fields = ["id:i64", "status:ArticleStatus"]
attachment = ["cover:image"]      # single
attachments = ["gallery:image"]   # multiple`}</code>
                </pre>

                <h2>Generated API Surface</h2>
                <ul>
                    <li>
                        <code>XxxView.cover: Option&lt;Attachment&gt;</code> and{' '}
                        <code>XxxView.cover_url: Option&lt;String&gt;</code> for single attachment.
                    </li>
                    <li>
                        <code>XxxView.gallery: Vec&lt;Attachment&gt;</code> and{' '}
                        <code>XxxView.gallery_urls: Vec&lt;String&gt;</code> for multi attachments.
                    </li>
                    <li>
                        Insert methods: <code>set_attachment_&lt;single&gt;</code>,{' '}
                        <code>add_attachment_&lt;multi&gt;</code>.
                    </li>
                    <li>
                        Update methods: <code>set_attachment_&lt;single&gt;</code>,{' '}
                        <code>clear_attachment_&lt;single&gt;</code>,{' '}
                        <code>add_attachment_&lt;multi&gt;</code>,{' '}
                        <code>delete_attachment_&lt;multi&gt;</code>.
                    </li>
                </ul>

                <h2>Create Example</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`use core_db::platform::attachments::types::AttachmentInput;

let cover = AttachmentInput::new(
    "uploads/articles/cover.jpg",
    "image/jpeg",
    125_012,
    Some(1200),
    Some(630),
);

let gallery_1 = AttachmentInput::new(
    "uploads/articles/gallery-1.jpg",
    "image/jpeg",
    98_311,
    Some(1024),
    Some(768),
);

let article = article_model
    .insert()
    .set_attachment_cover(cover)
    .add_attachment_gallery(gallery_1)
    .save()
    .await?;

println!("cover url = {:?}", article.cover_url);
println!("gallery urls = {:?}", article.gallery_urls);`}</code>
                </pre>

                <h2>Update Example</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`use core_db::common::sql::Op;
use core_db::platform::attachments::types::AttachmentInput;
use uuid::Uuid;

let new_cover = AttachmentInput::new(
    "uploads/articles/new-cover.jpg",
    "image/jpeg",
    111_000,
    Some(1200),
    Some(630),
);

let old_gallery_ids: Vec<Uuid> = vec![/* existing attachment ids */];

article_model
    .update()
    .where_id(Op::Eq, article_id)
    .clear_attachment_cover()
    .set_attachment_cover(new_cover)
    .delete_attachment_gallery(old_gallery_ids)
    .add_attachment_gallery(AttachmentInput::new(
        "uploads/articles/new-gallery.jpg",
        "image/jpeg",
        87_000,
        Some(960),
        Some(540),
    ))
    .save()
    .await?;`}</code>
                </pre>

                <h2>Atomicity</h2>
                <p>
                    Pool-backed <code>save()</code> writes are auto-atomic: base row, localized
                    data, meta, and attachments run in one transaction by default.
                </p>

                <h2>End-to-End: Multipart Upload to Attachment</h2>
                <p>
                    The full flow starts with the client uploading a file via a multipart form request.
                    The handler receives the multipart field, uploads the raw bytes to S3 through the
                    Storage service, constructs an <code>AttachmentInput</code> with the resulting path
                    and metadata, and passes it to the generated model setter. The model persists the
                    attachment record atomically alongside any other field changes.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`use axum::extract::Multipart;
use core_db::infra::storage::Storage;
use core_db::platform::attachments::types::AttachmentInput;
use core_web::error::AppError;

async fn upload_article_cover(
    State(state): State<AppApiState>,
    Path(article_id): Path<i64>,
    mut multipart: Multipart,
) -> ApiResult<serde_json::Value> {
    let field = multipart.next_field().await?.ok_or(
        AppError::BadRequest("No file provided".to_string()),
    )?;

    let file_name = field.file_name().unwrap_or("upload").to_string();
    let content_type = field.content_type().unwrap_or("application/octet-stream").to_string();
    let data = field.bytes().await?;

    // 1. Upload to S3
    let path = format!("uploads/articles/{}/{}", article_id, file_name);
    state.storage.put(&path, &data, &content_type).await?;

    // 2. Build attachment input
    let input = AttachmentInput::new(
        &path,
        &content_type,
        data.len() as u64,
        None,
        None,
    );

    // 3. Attach to model
    let article = Article::new(&state.db)
        .update()
        .where_id(Op::Eq, article_id)
        .set_attachment_cover(input)
        .save()
        .await?;

    Ok(ApiResponse::success(
        serde_json::json!({ "cover_url": article.cover_url }),
        &t("Cover uploaded"),
    ))
}`}</code>
                </pre>

                <h3>Route Registration</h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`Router::new()
    .route("/articles/{id}/cover", post(upload_article_cover))`}</code>
                </pre>
                <p>
                    The multipart body limit is controlled by <code>MIDDLEWARE_BODY_LIMIT</code> in{' '}
                    <code>.env</code> (default 10MB).
                </p>
            </div>
        </div>
    )
}

