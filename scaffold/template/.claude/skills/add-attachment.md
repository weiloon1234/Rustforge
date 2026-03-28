---
name: add-attachment
description: Add file upload attachment fields to a model
---

# Add Attachment Fields to a Model

## Step 1: Define attachment type in settings.toml

If the attachment type doesn't exist yet, add it to `app/settings.toml`:

```toml
[attachment_type.image]
allowed = ["image/jpeg", "image/png", "image/webp", "image/gif"]
max_size = 5242880

[attachment_type.document]
allowed = ["application/pdf", "application/msword"]
max_size = 10485760
```

## Step 2: Add attachment field to the model

In `app/models/{model}.rs`, add the field with `#[rf(kind = "type_name")]`:

```rust
#[rf_model(table = "my_models")]
pub struct MyModel {
    #[rf(pk(strategy = snowflake))]
    pub id: i64,
    pub name: String,
    // Single attachment
    #[rf(kind = "image")]
    pub avatar: Attachment,
    // Multiple attachments
    #[rf(kind = "image")]
    pub gallery: Attachments,
    pub created_at: time::OffsetDateTime,
    pub updated_at: time::OffsetDateTime,
}
```

The `kind` value must match an `[attachment_type.*]` key in settings.toml.

## Step 3: Regenerate

```bash
make gen
```

This generates:
- **Record fields:** `avatar: Option<Attachment>`, `avatar_url: Option<String>`, `gallery: Vec<Attachment>`, `gallery_urls: Vec<String>`
- **Builder methods:** `set_attachment_single("avatar", att)` / `add_attachment_multi("gallery", att)`
- **Hydration:** attachments are batch-loaded from the `attachments` table automatically

## Step 4: Handle upload in API handler

```rust
use core_web::extract::file_upload::FileUpload;

async fn upload_avatar(
    State(state): State<AppApiState>,
    auth: AuthUser<AdminGuard>,
    file: FileUpload,
) -> Result<ApiResponse<MyOutput>, AppError> {
    // Upload to S3/R2 — validates against attachment_type rules automatically
    let attachment = file.upload(&*state.storage, "my_model", "avatar").await?;

    // Optionally add meta (alt text, captions, etc.)
    let attachment = attachment.with_meta(serde_json::json!({
        "alt": "User avatar",
        "uploaded_by": auth.user.id,
    }));

    // Set on model
    MyModel::query()
        .where_col(MyCol::ID, Op::Eq, id)
        .patch()
        .set_attachment_single("avatar", attachment)
        .save(DbConn::pool(&state.db))
        .await?;

    Ok(ApiResponse::success(output, &t("Avatar uploaded")))
}
```

## Step 5: Access in workflows/datatables

```rust
// The record has the attachment hydrated
let record = MyModel::find(DbConn::pool(&db), id).await?.unwrap();

// Single attachment
if let Some(avatar) = &record.avatar {
    let url = avatar.url_with_base(base.as_deref());
    let content_type = &avatar.content_type;
    let meta = &avatar.meta; // Option<serde_json::Value>
}

// Multiple attachments
for photo in &record.gallery {
    let url = photo.url_with_base(base.as_deref());
}

// URL helpers (pre-built with CDN base)
let avatar_url: Option<String> = record.avatar_url;
let gallery_urls: Vec<String> = record.gallery_urls;
```

## Step 6: Frontend usage

TypeScript types (auto-generated in `@shared/types`):

```typescript
interface Attachment {
  id: string;
  path: string;
  url: string;
  content_type: string;
  size: number;
  width: number | null;
  height: number | null;
  meta?: Record<string, unknown> | null;
  created_at: string;
}
```

Upload with `FileInput` component:

```tsx
<FileInput
  label={t("Avatar")}
  accept="image/*"
  files={fileValues.avatar ?? []}
  onChange={(e) => setFiles("avatar", Array.from(e.target.files ?? []))}
/>
```

Display attachment URL:

```tsx
import { attachmentUrl } from "@shared/components";
<img src={attachmentUrl(record.avatar_url)} alt="" />
```

## Step 7: Verify

```bash
make gen
cargo check
```

## Notes

- Attachments are stored in the `attachments` table (polymorphic via `owner_type` + `owner_id` + `field`)
- Files are uploaded to S3/R2 with path: `owner_type/YYYY/MM/DD/field/uuid.ext`
- Soft-delete enabled — attachments are logically deleted, not physically removed
- `meta` field stores arbitrary JSON: alt text, captions, processing status, original filenames, etc.
- Image dimensions (`width`, `height`) are populated automatically for image uploads
- Validation (MIME type, max size) happens automatically based on `[attachment_type.*]` config
