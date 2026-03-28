#[rf_db_enum(storage = "string")]
pub enum PersonalAccessTokenKind {
    Access,
    Refresh,
}

#[rf_model(table = "personal_access_tokens")]
pub struct PersonalAccessToken {
    #[rf(pk(strategy = manual))]
    pub id: uuid::Uuid,
    pub tokenable_type: String,
    pub tokenable_id: String,
    pub name: String,
    pub token: String,
    pub token_kind: PersonalAccessTokenKind,
    pub family_id: uuid::Uuid,
    pub parent_token_id: Option<uuid::Uuid>,
    pub abilities: Option<serde_json::Value>,
    pub last_used_at: Option<time::OffsetDateTime>,
    pub expires_at: Option<time::OffsetDateTime>,
    pub revoked_at: Option<time::OffsetDateTime>,
    pub created_at: time::OffsetDateTime,
    pub updated_at: time::OffsetDateTime,
}

#[rf_record_impl]
impl PersonalAccessTokenRecord {
    #[rf_computed]
    pub fn is_revoked(&self) -> bool {
        self.revoked_at.is_some()
    }

    #[rf_computed]
    pub fn is_expired(&self) -> bool {
        match self.expires_at {
            Some(exp) => exp <= time::OffsetDateTime::now_utc(),
            None => false,
        }
    }

    #[rf_computed]
    pub fn is_valid(&self) -> bool {
        !self.is_revoked() && !self.is_expired()
    }
}

#[rf_model_impl]
impl PersonalAccessTokenModel {
    /// Revoke a single token by setting revoked_at = now.
    pub async fn revoke_token<'db>(
        db: impl Into<DbConn<'db>>,
        token_id: uuid::Uuid,
    ) -> Result<u64> {
        let db = db.into();
        Query::<PersonalAccessTokenModel>::new()
            .where_col(PersonalAccessTokenCol::ID, Op::Eq, token_id)
            .where_null(PersonalAccessTokenCol::REVOKED_AT)
            .patch()
            .assign(PersonalAccessTokenCol::REVOKED_AT, time::OffsetDateTime::now_utc())?
            .save(db)
            .await
    }

    /// Revoke all tokens in a token family.
    pub async fn revoke_family<'db>(
        db: impl Into<DbConn<'db>>,
        family_id: uuid::Uuid,
    ) -> Result<u64> {
        let db = db.into();
        Query::<PersonalAccessTokenModel>::new()
            .where_col(PersonalAccessTokenCol::FAMILY_ID, Op::Eq, family_id)
            .where_null(PersonalAccessTokenCol::REVOKED_AT)
            .patch()
            .assign(PersonalAccessTokenCol::REVOKED_AT, time::OffsetDateTime::now_utc())?
            .save(db)
            .await
    }

    /// Revoke all tokens for a tokenable entity (e.g., user logout).
    pub async fn revoke_all_for<'db>(
        db: impl Into<DbConn<'db>>,
        tokenable_type: &str,
        tokenable_id: &str,
    ) -> Result<u64> {
        let db = db.into();
        Query::<PersonalAccessTokenModel>::new()
            .where_col(PersonalAccessTokenCol::TOKENABLE_TYPE, Op::Eq, tokenable_type.to_string())
            .where_col(PersonalAccessTokenCol::TOKENABLE_ID, Op::Eq, tokenable_id.to_string())
            .where_null(PersonalAccessTokenCol::REVOKED_AT)
            .patch()
            .assign(PersonalAccessTokenCol::REVOKED_AT, time::OffsetDateTime::now_utc())?
            .save(db)
            .await
    }

    /// Touch the last_used_at timestamp for a token.
    pub async fn touch_last_used<'db>(
        db: impl Into<DbConn<'db>>,
        token_id: uuid::Uuid,
    ) -> Result<u64> {
        let db = db.into();
        Query::<PersonalAccessTokenModel>::new()
            .where_col(PersonalAccessTokenCol::ID, Op::Eq, token_id)
            .patch()
            .assign(PersonalAccessTokenCol::LAST_USED_AT, time::OffsetDateTime::now_utc())?
            .save(db)
            .await
    }

    /// Find a valid (non-revoked, non-expired) token by its hashed value.
    pub async fn find_valid_by_hash<'db>(
        db: impl Into<DbConn<'db>>,
        token_hash: &str,
    ) -> Result<Option<PersonalAccessTokenRecord>> {
        let db = db.into();
        let now = time::OffsetDateTime::now_utc();
        Query::<PersonalAccessTokenModel>::new()
            .where_col(PersonalAccessTokenCol::TOKEN, Op::Eq, token_hash.to_string())
            .where_null(PersonalAccessTokenCol::REVOKED_AT)
            .where_group(|q| {
                q.where_null(PersonalAccessTokenCol::EXPIRES_AT)
                    .or_where_col(PersonalAccessTokenCol::EXPIRES_AT, Op::Gt, now)
            })
            .first(db)
            .await
    }
}
