use anyhow::Result;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::common::sql::DbConn;
use crate::platform::personal_access_tokens::model::{
    PersonalAccessTokenKind, PersonalAccessTokenRow,
};

pub struct PatRepo<'a> {
    db: DbConn<'a>,
}

#[derive(Debug, Clone)]
pub struct CreatePatInput {
    pub tokenable_type: String,
    pub tokenable_id: String,
    pub name: String,
    pub token_hash: String,
    pub token_kind: PersonalAccessTokenKind,
    pub family_id: Uuid,
    pub parent_token_id: Option<Uuid>,
    pub abilities: Option<Vec<String>>,
    pub expires_at: Option<OffsetDateTime>,
}

impl<'a> PatRepo<'a> {
    pub fn new(db: DbConn<'a>) -> Self {
        Self { db }
    }

    pub async fn create(&self, input: CreatePatInput) -> Result<PersonalAccessTokenRow> {
        let abilities_json = input.abilities.map(sqlx::types::Json);
        let id = Uuid::new_v4();

        #[cfg(feature = "sqlx-checked")]
        let _ = sqlx::query!(
            r#"
            INSERT INTO personal_access_tokens
            (id, tokenable_type, tokenable_id, name, token, token_kind, family_id, parent_token_id, abilities, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id, tokenable_type, tokenable_id, name, token, token_kind, family_id, parent_token_id, abilities, last_used_at, expires_at, revoked_at, created_at, updated_at
            "#,
            id,
            &input.tokenable_type,
            &input.tokenable_id,
            &input.name,
            &input.token_hash,
            input.token_kind.as_str(),
            input.family_id,
            input.parent_token_id,
            abilities_json.clone(),
            input.expires_at,
        );

        let q = sqlx::query_as::<_, PersonalAccessTokenRow>(
            r#"
            INSERT INTO personal_access_tokens
            (id, tokenable_type, tokenable_id, name, token, token_kind, family_id, parent_token_id, abilities, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id, tokenable_type, tokenable_id, name, token, token_kind, family_id, parent_token_id, abilities, last_used_at, expires_at, revoked_at, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(input.tokenable_type)
        .bind(input.tokenable_id)
        .bind(input.name)
        .bind(input.token_hash)
        .bind(input.token_kind.as_str())
        .bind(input.family_id)
        .bind(input.parent_token_id)
        .bind(abilities_json)
        .bind(input.expires_at);

        let pat = self.db.fetch_one(q).await?;
        Ok(pat)
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<PersonalAccessTokenRow>> {
        #[cfg(feature = "sqlx-checked")]
        let _ = sqlx::query!(
            r#"
            SELECT id, tokenable_type, tokenable_id, name, token, token_kind, family_id, parent_token_id, abilities, last_used_at, expires_at, revoked_at, created_at, updated_at
            FROM personal_access_tokens
            WHERE id = $1
            "#,
            id
        );

        let q = sqlx::query_as::<_, PersonalAccessTokenRow>(
            r#"
            SELECT id, tokenable_type, tokenable_id, name, token, token_kind, family_id, parent_token_id, abilities, last_used_at, expires_at, revoked_at, created_at, updated_at
            FROM personal_access_tokens
            WHERE id = $1
            "#,
        )
        .bind(id);

        self.db.fetch_optional(q).await.map_err(Into::into)
    }

    pub async fn find_by_token(&self, token_hash: &str) -> Result<Option<PersonalAccessTokenRow>> {
        #[cfg(feature = "sqlx-checked")]
        let _ = sqlx::query!(
            r#"
            SELECT id, tokenable_type, tokenable_id, name, token, token_kind, family_id, parent_token_id, abilities, last_used_at, expires_at, revoked_at, created_at, updated_at
            FROM personal_access_tokens
            WHERE token = $1
            "#,
            token_hash
        );

        let q = sqlx::query_as::<_, PersonalAccessTokenRow>(
            r#"
            SELECT id, tokenable_type, tokenable_id, name, token, token_kind, family_id, parent_token_id, abilities, last_used_at, expires_at, revoked_at, created_at, updated_at
            FROM personal_access_tokens
            WHERE token = $1
            "#,
        )
        .bind(token_hash);

        self.db.fetch_optional(q).await.map_err(Into::into)
    }

    pub async fn find_by_token_and_kind(
        &self,
        token_hash: &str,
        token_kind: PersonalAccessTokenKind,
    ) -> Result<Option<PersonalAccessTokenRow>> {
        #[cfg(feature = "sqlx-checked")]
        let _ = sqlx::query!(
            r#"
            SELECT id, tokenable_type, tokenable_id, name, token, token_kind, family_id, parent_token_id, abilities, last_used_at, expires_at, revoked_at, created_at, updated_at
            FROM personal_access_tokens
            WHERE token = $1
              AND token_kind = $2
            "#,
            token_hash,
            token_kind.as_str()
        );

        let q = sqlx::query_as::<_, PersonalAccessTokenRow>(
            r#"
            SELECT id, tokenable_type, tokenable_id, name, token, token_kind, family_id, parent_token_id, abilities, last_used_at, expires_at, revoked_at, created_at, updated_at
            FROM personal_access_tokens
            WHERE token = $1
              AND token_kind = $2
            "#,
        )
        .bind(token_hash)
        .bind(token_kind.as_str());

        self.db.fetch_optional(q).await.map_err(Into::into)
    }

    pub async fn update_last_used(&self, id: Uuid) -> Result<()> {
        #[cfg(feature = "sqlx-checked")]
        let _ = sqlx::query!(
            r#"
            UPDATE personal_access_tokens
            SET last_used_at = NOW(), updated_at = NOW()
            WHERE id = $1
            "#,
            id
        );

        let q = sqlx::query(
            r#"
            UPDATE personal_access_tokens
            SET last_used_at = NOW(), updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id);

        self.db.execute(q).await?;
        Ok(())
    }

    pub async fn revoke_by_id(&self, id: Uuid) -> Result<()> {
        #[cfg(feature = "sqlx-checked")]
        let _ = sqlx::query!(
            r#"
            UPDATE personal_access_tokens
            SET revoked_at = NOW(), updated_at = NOW()
            WHERE id = $1
            "#,
            id
        );

        let q = sqlx::query(
            r#"
            UPDATE personal_access_tokens
            SET revoked_at = NOW(), updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id);

        self.db.execute(q).await?;
        Ok(())
    }

    pub async fn revoke_family(&self, family_id: Uuid) -> Result<u64> {
        #[cfg(feature = "sqlx-checked")]
        let _ = sqlx::query!(
            r#"
            UPDATE personal_access_tokens
            SET revoked_at = COALESCE(revoked_at, NOW()), updated_at = NOW()
            WHERE family_id = $1
            "#,
            family_id
        );

        let q = sqlx::query(
            r#"
            UPDATE personal_access_tokens
            SET revoked_at = COALESCE(revoked_at, NOW()), updated_at = NOW()
            WHERE family_id = $1
            "#,
        )
        .bind(family_id);

        let result = self.db.execute(q).await?;
        Ok(result.rows_affected())
    }

    pub async fn list_by_subject(
        &self,
        tokenable_type: &str,
        tokenable_id: &str,
    ) -> Result<Vec<PersonalAccessTokenRow>> {
        #[cfg(feature = "sqlx-checked")]
        let _ = sqlx::query!(
            r#"
            SELECT id, tokenable_type, tokenable_id, name, token, token_kind, family_id, parent_token_id, abilities, last_used_at, expires_at, revoked_at, created_at, updated_at
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
            SELECT id, tokenable_type, tokenable_id, name, token, token_kind, family_id, parent_token_id, abilities, last_used_at, expires_at, revoked_at, created_at, updated_at
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

    pub async fn revoke_by_subject(&self, tokenable_type: &str, tokenable_id: &str) -> Result<u64> {
        #[cfg(feature = "sqlx-checked")]
        let _ = sqlx::query!(
            r#"
            UPDATE personal_access_tokens
            SET revoked_at = COALESCE(revoked_at, NOW()), updated_at = NOW()
            WHERE tokenable_type = $1
              AND tokenable_id = $2
            "#,
            tokenable_type,
            tokenable_id
        );

        let q = sqlx::query(
            r#"
            UPDATE personal_access_tokens
            SET revoked_at = COALESCE(revoked_at, NOW()), updated_at = NOW()
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
