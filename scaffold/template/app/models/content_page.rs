#[rf_db_enum(storage = "i16")]
pub enum ContentPageSystemFlag {
    No = 0,
    Yes = 1,
}

#[rf_model(table = "content_pages", soft_delete)]
pub struct ContentPage {
    #[rf(pk(strategy = snowflake))]
    pub id: i64,
    pub tag: String,
    pub is_system: ContentPageSystemFlag,
    pub created_at: time::OffsetDateTime,
    pub updated_at: time::OffsetDateTime,
    pub title: Localized<String>,
    pub content: Localized<String>,
    pub cover: Localized<String>,
}
