use async_trait::async_trait;
use sqlx::PgPool;

#[async_trait]
pub trait ActiveRecord: Sized + Send + Sync + 'static {
    type Id: Send + Sync + 'static;

    /// Find a record by its ID
    async fn find(db: &PgPool, id: Self::Id) -> anyhow::Result<Option<Self>>;
}
