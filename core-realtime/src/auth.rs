use core_web::auth::AuthIdentity;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use time::OffsetDateTime;

pub type AuthResolverFuture = Pin<Box<dyn Future<Output = Option<AuthIdentity>> + Send>>;
pub type AuthResolver = Arc<dyn Fn(String) -> AuthResolverFuture + Send + Sync>;

#[derive(Debug, Clone, Default)]
pub struct ConnectionAuthState {
    pub identity: Option<AuthIdentity>,
    pub authenticated_at: Option<OffsetDateTime>,
}

impl ConnectionAuthState {
    pub fn is_authenticated(&self) -> bool {
        self.identity.is_some()
    }
}
