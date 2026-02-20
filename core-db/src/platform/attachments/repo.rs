#![allow(dead_code)]

use std::collections::HashMap;

use anyhow::Result;

use uuid::Uuid;

use crate::common::sql::DbConn;
use crate::platform::attachments::model::AttachmentRow;
use crate::platform::attachments::types::{Attachment, AttachmentMap, AttachmentUploadDto};

pub struct AttachmentRepo<'a> {
    db: DbConn<'a>,
}

impl<'a> AttachmentRepo<'a> {
    pub fn new(db: DbConn<'a>) -> Self {
        Self { db }
    }

    pub async fn load_for_owners(
        &self,
        owner_type: &str,
        owner_ids: &[i64],
        fields: &[&str],
    ) -> Result<AttachmentMap> {
        if owner_ids.is_empty() || fields.is_empty() {
            return Ok(AttachmentMap::default());
        }

        let fields_vec: Vec<String> = fields.iter().map(|s| s.to_string()).collect();

        #[cfg(feature = "sqlx-checked")]
        let _ = sqlx::query!(
            r#"
            SELECT id, owner_type, owner_id, field, path, content_type, size, width, height, created_at, updated_at, deleted_at
            FROM attachments
            WHERE owner_type = $1
              AND owner_id = ANY($2)
              AND field = ANY($3)
              AND deleted_at IS NULL
            ORDER BY created_at ASC
            "#,
            owner_type,
            owner_ids,
            &fields_vec
        );

        let q = sqlx::query_as::<_, AttachmentRow>(
            r#"
            SELECT id, owner_type, owner_id, field, path, content_type, size, width, height, created_at, updated_at, deleted_at
            FROM attachments
            WHERE owner_type = $1
              AND owner_id = ANY($2)
              AND field = ANY($3)
              AND deleted_at IS NULL
            ORDER BY created_at ASC
            "#,
        )
        .bind(owner_type)
        .bind(owner_ids)
        .bind(&fields_vec);
        let rows = self.db.fetch_all(q).await?;

        let mut map: HashMap<String, HashMap<i64, Vec<Attachment>>> = HashMap::new();

        for r in rows {
            let entry = map
                .entry(r.field.clone())
                .or_default()
                .entry(r.owner_id)
                .or_default();
            entry.push(Attachment {
                id: r.id,
                path: r.path,
                content_type: r.content_type,
                size: r.size,
                width: r.width,
                height: r.height,
                created_at: r.created_at,
            });
        }

        Ok(AttachmentMap::new(map))
    }

    pub async fn replace_single(
        &self,
        owner_type: &str,
        owner_id: i64,
        field: &str,
        attachment: &AttachmentUploadDto,
    ) -> Result<()> {
        #[cfg(feature = "sqlx-checked")]
        let _ = sqlx::query!(
            r#"DELETE FROM attachments WHERE owner_type = $1 AND owner_id = $2 AND field = $3"#,
            owner_type,
            owner_id,
            field
        );

        let q1 = sqlx::query(
            r#"DELETE FROM attachments WHERE owner_type = $1 AND owner_id = $2 AND field = $3"#,
        )
        .bind(owner_type)
        .bind(owner_id)
        .bind(field);
        self.db.execute(q1).await?;

        #[cfg(feature = "sqlx-checked")]
        let _ = sqlx::query!(
            r#"
            INSERT INTO attachments (id, owner_type, owner_id, field, path, content_type, size, width, height)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
            Uuid::nil(),
            owner_type,
            owner_id,
            field,
            &attachment.path,
            &attachment.content_type,
            attachment.size,
            attachment.width,
            attachment.height
        );

        let q2 = sqlx::query(
            r#"
            INSERT INTO attachments (id, owner_type, owner_id, field, path, content_type, size, width, height)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(owner_type)
        .bind(owner_id)
        .bind(field)
        .bind(&attachment.path)
        .bind(&attachment.content_type)
        .bind(attachment.size)
        .bind(attachment.width)
        .bind(attachment.height);
        self.db.execute(q2).await?;

        Ok(())
    }

    pub async fn add_many(
        &self,
        owner_type: &str,
        owner_id: i64,
        field: &str,
        attachments: &[AttachmentUploadDto],
    ) -> Result<()> {
        if attachments.is_empty() {
            return Ok(());
        }

        for att in attachments {
            #[cfg(feature = "sqlx-checked")]
            let _ = sqlx::query!(
                r#"
                INSERT INTO attachments (id, owner_type, owner_id, field, path, content_type, size, width, height)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                "#,
                Uuid::nil(),
                owner_type,
                owner_id,
                field,
                &att.path,
                &att.content_type,
                att.size,
                att.width,
                att.height
            );

            let q = sqlx::query(
                r#"
                INSERT INTO attachments (id, owner_type, owner_id, field, path, content_type, size, width, height)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                "#,
            )
            .bind(Uuid::new_v4())
            .bind(owner_type)
            .bind(owner_id)
            .bind(field)
            .bind(&att.path)
            .bind(&att.content_type)
            .bind(att.size)
            .bind(att.width)
            .bind(att.height);
            self.db.execute(q).await?;
        }
        Ok(())
    }

    pub async fn delete_by_ids(
        &self,
        owner_type: &str,
        owner_id: i64,
        field: &str,
        ids: &[Uuid],
    ) -> Result<()> {
        if ids.is_empty() {
            return Ok(());
        }
        #[cfg(feature = "sqlx-checked")]
        let _ = sqlx::query!(
            r#"
            DELETE FROM attachments
            WHERE owner_type = $1
              AND owner_id = $2
              AND field = $3
              AND id = ANY($4)
            "#,
            owner_type,
            owner_id,
            field,
            ids
        );

        let q = sqlx::query(
            r#"
            DELETE FROM attachments
            WHERE owner_type = $1
              AND owner_id = $2
              AND field = $3
              AND id = ANY($4)
            "#,
        )
        .bind(owner_type)
        .bind(owner_id)
        .bind(field)
        .bind(ids);
        self.db.execute(q).await?;
        Ok(())
    }

    pub async fn clear_field(&self, owner_type: &str, owner_id: i64, field: &str) -> Result<()> {
        #[cfg(feature = "sqlx-checked")]
        let _ = sqlx::query!(
            r#"
            DELETE FROM attachments
            WHERE owner_type = $1
              AND owner_id = $2
              AND field = $3
            "#,
            owner_type,
            owner_id,
            field
        );

        let q = sqlx::query(
            r#"
            DELETE FROM attachments
            WHERE owner_type = $1
              AND owner_id = $2
              AND field = $3
            "#,
        )
        .bind(owner_type)
        .bind(owner_id)
        .bind(field);
        self.db.execute(q).await?;
        Ok(())
    }
}
