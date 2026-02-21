use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PersonalAccessTokenKind {
    Access,
    Refresh,
}

impl PersonalAccessTokenKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Access => "access",
            Self::Refresh => "refresh",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "access" => Some(Self::Access),
            "refresh" => Some(Self::Refresh),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct PersonalAccessTokenRow {
    pub id: Uuid,
    pub tokenable_type: String,
    pub tokenable_id: String,
    pub name: String,
    pub token: String,
    pub token_kind: String,
    pub family_id: Uuid,
    pub parent_token_id: Option<Uuid>,
    pub abilities: Option<sqlx::types::Json<Vec<String>>>,
    pub last_used_at: Option<OffsetDateTime>,
    pub expires_at: Option<OffsetDateTime>,
    pub revoked_at: Option<OffsetDateTime>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl PersonalAccessTokenRow {
    pub fn kind(&self) -> Option<PersonalAccessTokenKind> {
        PersonalAccessTokenKind::parse(&self.token_kind)
    }

    pub fn is_revoked(&self) -> bool {
        self.revoked_at.is_some()
    }

    pub fn is_expired(&self, now: OffsetDateTime) -> bool {
        self.expires_at.is_some_and(|exp| exp <= now)
    }

    pub fn abilities_vec(&self) -> Vec<String> {
        self.abilities
            .as_ref()
            .map(|items| items.0.clone())
            .unwrap_or_default()
    }
}
