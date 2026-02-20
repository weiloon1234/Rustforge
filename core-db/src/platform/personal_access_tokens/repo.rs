use anyhow::Result;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::common::sql::DbConn;
use crate::platform::personal_access_tokens::model::PersonalAccessTokenRow;

pub struct PatRepo<'a> {
    db: DbConn<'a>,
}

impl<'a> PatRepo<'a> {
    pub fn new(db: DbConn<'a>) -> Self {
        Self { db }
    }

    pub async fn create(
        &self,
        tokenable_type: &str,
        tokenable_id: Uuid,
        name: &str,
        token_hash: &str,
        abilities: Option<Vec<String>>,
        expires_at: Option<OffsetDateTime>,
    ) -> Result<PersonalAccessTokenRow> {
        let abilities_json = abilities.map(sqlx::types::Json);
        let id = Uuid::new_v4();

        #[cfg(feature = "sqlx-checked")]
        let _ = sqlx::query!(
            r#"
            INSERT INTO personal_access_tokens
            (id, tokenable_type, tokenable_id, name, token, abilities, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, tokenable_type, tokenable_id, name, token, abilities, last_used_at, expires_at, created_at, updated_at
            "#,
            id,
            tokenable_type,
            tokenable_id,
            name,
            token_hash,
            abilities_json.clone(),
            expires_at
        );

        let q = sqlx::query_as::<_, PersonalAccessTokenRow>(
            r#"
            INSERT INTO personal_access_tokens 
            (id, tokenable_type, tokenable_id, name, token, abilities, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, tokenable_type, tokenable_id, name, token, abilities, last_used_at, expires_at, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(tokenable_type)
        .bind(tokenable_id)
        .bind(name)
        .bind(token_hash)
        .bind(abilities_json)
        .bind(expires_at);

        let pat = self.db.fetch_one(q).await?;
        Ok(pat)
    }

    pub async fn find_by_token(&self, token_hash: &str) -> Result<Option<PersonalAccessTokenRow>> {
        #[cfg(feature = "sqlx-checked")]
        let _ = sqlx::query!(
            r#"
            SELECT id, tokenable_type, tokenable_id, name, token, abilities, last_used_at, expires_at, created_at, updated_at
            FROM personal_access_tokens
            WHERE token = $1
            "#,
            token_hash
        );

        let q = sqlx::query_as::<_, PersonalAccessTokenRow>(
            r#"
            SELECT id, tokenable_type, tokenable_id, name, token, abilities, last_used_at, expires_at, created_at, updated_at
            FROM personal_access_tokens
            WHERE token = $1
            "#,
        )
        .bind(token_hash);

        let pat = self.db.fetch_optional(q).await?;
        Ok(pat)
    }

    pub async fn update_last_used(&self, id: Uuid) -> Result<()> {
        #[cfg(feature = "sqlx-checked")]
        let _ = sqlx::query!(
            r#"
            UPDATE personal_access_tokens
            SET last_used_at = NOW()
            WHERE id = $1
            "#,
            id
        );

        let q = sqlx::query(
            r#"
            UPDATE personal_access_tokens 
            SET last_used_at = NOW() 
            WHERE id = $1
            "#,
        )
        .bind(id);

        self.db.execute(q).await?;
        Ok(())
    }

    pub async fn delete_by_id(&self, id: Uuid) -> Result<()> {
        #[cfg(feature = "sqlx-checked")]
        let _ = sqlx::query!(
            r#"
            DELETE FROM personal_access_tokens
            WHERE id = $1
            "#,
            id
        );

        let q = sqlx::query(
            r#"
            DELETE FROM personal_access_tokens
            WHERE id = $1
            "#,
        )
        .bind(id);

        self.db.execute(q).await?;
        Ok(())
    }

    pub async fn list_by_subject(
        &self,
        tokenable_type: &str,
        tokenable_id: Uuid,
    ) -> Result<Vec<PersonalAccessTokenRow>> {
        #[cfg(feature = "sqlx-checked")]
        let _ = sqlx::query!(
            r#"
            SELECT id, tokenable_type, tokenable_id, name, token, abilities, last_used_at, expires_at, created_at, updated_at
            FROM personal_access_tokens
            WHERE tokenable_type = $1
              AND tokenable_id = $2
            ORDER BY created_at DESC
            "#,
            tokenable_type,
            tokenable_id
        );

        let q = sqlx::query_as::<_, PersonalAccessTokenRow>(
            r#"
            SELECT id, tokenable_type, tokenable_id, name, token, abilities, last_used_at, expires_at, created_at, updated_at
            FROM personal_access_tokens
            WHERE tokenable_type = $1
              AND tokenable_id = $2
            ORDER BY created_at DESC
            "#,
        )
        .bind(tokenable_type)
        .bind(tokenable_id);

        self.db.fetch_all(q).await.map_err(Into::into)
    }

    pub async fn revoke_by_subject(&self, tokenable_type: &str, tokenable_id: Uuid) -> Result<u64> {
        #[cfg(feature = "sqlx-checked")]
        let _ = sqlx::query!(
            r#"
            DELETE FROM personal_access_tokens
            WHERE tokenable_type = $1
              AND tokenable_id = $2
            "#,
            tokenable_type,
            tokenable_id
        );

        let q = sqlx::query(
            r#"
            DELETE FROM personal_access_tokens
            WHERE tokenable_type = $1
              AND tokenable_id = $2
            "#,
        )
        .bind(tokenable_type)
        .bind(tokenable_id);

        let result = self.db.execute(q).await?;
        Ok(result.rows_affected())
    }
}
