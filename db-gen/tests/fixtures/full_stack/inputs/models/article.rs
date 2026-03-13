#[rf_db_enum(storage = "string")]
pub enum ArticleStatus {
    Draft,
    Published,
}

#[rf_db_enum(storage = "i16")]
pub enum ArticleSystemFlag {
    No = 0,
    Yes = 1,
}

#[rf_model(table = "articles")]
pub struct Article {
    pub id: i64,
    pub author_id: i64,
    pub title: Localized<String>,
    pub status: ArticleStatus,
    pub is_system: ArticleSystemFlag,
    #[rf(foreign_key = "author_id")]
    pub author: BelongsTo<User>,
    pub flags: Meta<bool>,
    pub extra: Meta<serde_json::Value>,
    #[rf(kind = "image")]
    pub hero: Attachment,
}
