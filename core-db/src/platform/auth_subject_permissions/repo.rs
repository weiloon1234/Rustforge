use anyhow::Result;
use std::collections::BTreeSet;
use uuid::Uuid;

use crate::common::auth::permissions::has_permission as has_permission_in_granted;
use crate::common::sql::DbConn;
use crate::platform::auth_subject_permissions::model::AuthSubjectPermissionRow;

pub struct AuthSubjectPermissionRepo<'a> {
    db: DbConn<'a>,
}

impl<'a> AuthSubjectPermissionRepo<'a> {
    pub fn new(db: DbConn<'a>) -> Self {
        Self { db }
    }

    pub async fn list(
        &self,
        guard: &str,
        subject_id: Uuid,
    ) -> Result<Vec<AuthSubjectPermissionRow>> {
        #[cfg(feature = "sqlx-checked")]
        let _ = sqlx::query!(
            r#"
            SELECT id, guard, subject_id, permission
            FROM auth_subject_permissions
            WHERE guard = $1
              AND subject_id = $2
            ORDER BY permission ASC
            "#,
            guard,
            subject_id
        );

        let q = sqlx::query_as::<_, AuthSubjectPermissionRow>(
            r#"
            SELECT id, guard, subject_id, permission
            FROM auth_subject_permissions
            WHERE guard = $1
              AND subject_id = $2
            ORDER BY permission ASC
            "#,
        )
        .bind(guard)
        .bind(subject_id);

        self.db.fetch_all(q).await.map_err(Into::into)
    }

    pub async fn list_permission_strings(
        &self,
        guard: &str,
        subject_id: Uuid,
    ) -> Result<Vec<String>> {
        let rows = self.list(guard, subject_id).await?;
        Ok(rows.into_iter().map(|row| row.permission).collect())
    }

    pub async fn has_permission(
        &self,
        guard: &str,
        subject_id: Uuid,
        required: &str,
    ) -> Result<bool> {
        let granted = self.list_permission_strings(guard, subject_id).await?;
        Ok(has_permission_in_granted(&granted, required))
    }

    pub async fn replace(
        &self,
        guard: &str,
        subject_id: Uuid,
        permissions: &[String],
    ) -> Result<()> {
        let mut unique = BTreeSet::new();
        for raw in permissions {
            let value = raw.trim();
            if !value.is_empty() {
                unique.insert(value.to_string());
            }
        }

        let delete_q = sqlx::query(
            r#"
            DELETE FROM auth_subject_permissions
            WHERE guard = $1
              AND subject_id = $2
            "#,
        )
        .bind(guard)
        .bind(subject_id);
        self.db.execute(delete_q).await?;

        for permission in unique {
            let insert_q = sqlx::query(
                r#"
                INSERT INTO auth_subject_permissions (id, guard, subject_id, permission)
                VALUES ($1, $2, $3, $4)
                "#,
            )
            .bind(Uuid::new_v4())
            .bind(guard)
            .bind(subject_id)
            .bind(permission);
            self.db.execute(insert_q).await?;
        }
        Ok(())
    }

    pub async fn grant(&self, guard: &str, subject_id: Uuid, permission: &str) -> Result<()> {
        let permission = permission.trim();
        if permission.is_empty() {
            return Ok(());
        }

        #[cfg(feature = "sqlx-checked")]
        let _ = sqlx::query!(
            r#"
            INSERT INTO auth_subject_permissions (id, guard, subject_id, permission)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (guard, subject_id, permission) DO NOTHING
            "#,
            Uuid::new_v4(),
            guard,
            subject_id,
            permission
        );

        let q = sqlx::query(
            r#"
            INSERT INTO auth_subject_permissions (id, guard, subject_id, permission)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (guard, subject_id, permission) DO NOTHING
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(guard)
        .bind(subject_id)
        .bind(permission);
        self.db.execute(q).await?;
        Ok(())
    }

    pub async fn revoke(&self, guard: &str, subject_id: Uuid, permission: &str) -> Result<()> {
        let permission = permission.trim();
        if permission.is_empty() {
            return Ok(());
        }

        #[cfg(feature = "sqlx-checked")]
        let _ = sqlx::query!(
            r#"
            DELETE FROM auth_subject_permissions
            WHERE guard = $1
              AND subject_id = $2
              AND permission = $3
            "#,
            guard,
            subject_id,
            permission
        );

        let q = sqlx::query(
            r#"
            DELETE FROM auth_subject_permissions
            WHERE guard = $1
              AND subject_id = $2
              AND permission = $3
            "#,
        )
        .bind(guard)
        .bind(subject_id)
        .bind(permission);
        self.db.execute(q).await?;
        Ok(())
    }
}
