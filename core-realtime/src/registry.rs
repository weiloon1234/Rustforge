use crate::types::ErrorCode;
use core_web::auth::AuthIdentity;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ChannelPolicy {
    pub enabled: bool,
    pub guard: Option<String>,
    pub presence_enabled: bool,
}

impl Default for ChannelPolicy {
    fn default() -> Self {
        Self {
            enabled: true,
            guard: None,
            presence_enabled: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PolicyDecision {
    pub allowed: bool,
    pub code: ErrorCode,
    pub reason: Option<String>,
}

impl PolicyDecision {
    pub fn allow() -> Self {
        Self {
            allowed: true,
            code: ErrorCode::Forbidden,
            reason: None,
        }
    }

    pub fn deny(code: ErrorCode, reason: impl Into<String>) -> Self {
        Self {
            allowed: false,
            code,
            reason: Some(reason.into()),
        }
    }

    pub fn into_error(self, fallback: impl Into<String>) -> (ErrorCode, String) {
        (self.code, self.reason.unwrap_or_else(|| fallback.into()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyAction {
    Subscribe,
    Publish,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicySource {
    WebSocket,
    HttpApi,
    Internal,
}

#[derive(Debug, Clone, Default)]
pub struct PolicyMetadata {
    pub request_id: Option<String>,
    pub remote_addr: Option<String>,
    pub user_agent: Option<String>,
    pub tenant_id: Option<String>,
    pub extras: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct PolicyContext {
    pub action: PolicyAction,
    pub source: PolicySource,
    pub channel: String,
    pub room: Option<String>,
    pub connection_id: Option<Uuid>,
    pub identity: Option<AuthIdentity>,
    pub metadata: PolicyMetadata,
}

impl PolicyContext {
    pub fn new(
        action: PolicyAction,
        source: PolicySource,
        channel: impl Into<String>,
        room: Option<String>,
    ) -> Self {
        Self {
            action,
            source,
            channel: channel.into(),
            room,
            connection_id: None,
            identity: None,
            metadata: PolicyMetadata::default(),
        }
    }

    pub fn with_identity(mut self, identity: Option<&AuthIdentity>) -> Self {
        self.identity = identity.cloned();
        self
    }

    pub fn with_connection_id(mut self, connection_id: Uuid) -> Self {
        self.connection_id = Some(connection_id);
        self
    }

    pub fn with_metadata(mut self, metadata: PolicyMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn channel(&self) -> &str {
        &self.channel
    }

    pub fn room(&self) -> Option<&str> {
        self.room.as_deref()
    }

    pub fn guard(&self) -> Option<&str> {
        self.identity.as_ref().map(|i| i.guard.as_str())
    }

    pub fn subject_id(&self) -> Option<&str> {
        self.identity.as_ref().map(|i| i.subject_id.as_str())
    }

    pub fn has_ability(&self, ability: &str) -> bool {
        self.identity
            .as_ref()
            .map(|i| i.can(ability))
            .unwrap_or(false)
    }
}

pub trait SubscribeAuthorizer: Send + Sync + 'static {
    fn authorize_subscribe(&self, _context: &PolicyContext) -> PolicyDecision {
        PolicyDecision::allow()
    }
}

pub trait PublishAuthorizer: Send + Sync + 'static {
    fn authorize_publish(&self, _context: &PolicyContext) -> PolicyDecision {
        PolicyDecision::allow()
    }
}

#[derive(Default)]
pub struct AllowAllSubscribeAuthorizer;

impl SubscribeAuthorizer for AllowAllSubscribeAuthorizer {}

#[derive(Default)]
pub struct AllowAllPublishAuthorizer;

impl PublishAuthorizer for AllowAllPublishAuthorizer {}

#[derive(Clone)]
pub struct ChannelPolicyRegistry {
    global_enabled: bool,
    policies: Arc<HashMap<String, ChannelPolicy>>,
    authorizer: Arc<dyn SubscribeAuthorizer>,
    publish_authorizer: Arc<dyn PublishAuthorizer>,
}

impl ChannelPolicyRegistry {
    pub fn new(
        global_enabled: bool,
        policies: HashMap<String, ChannelPolicy>,
        authorizer: Arc<dyn SubscribeAuthorizer>,
    ) -> Self {
        let policies = policies
            .into_iter()
            .map(|(k, v)| (k.to_ascii_lowercase(), v))
            .collect::<HashMap<_, _>>();
        Self {
            global_enabled,
            policies: Arc::new(policies),
            authorizer,
            publish_authorizer: Arc::new(AllowAllPublishAuthorizer),
        }
    }

    pub fn with_publish_authorizer(
        mut self,
        publish_authorizer: Arc<dyn PublishAuthorizer>,
    ) -> Self {
        self.publish_authorizer = publish_authorizer;
        self
    }

    pub fn global_enabled(&self) -> bool {
        self.global_enabled
    }

    pub fn policy(&self, channel: &str) -> ChannelPolicy {
        self.policies
            .get(&channel.to_ascii_lowercase())
            .cloned()
            .unwrap_or_default()
    }

    pub fn authorize_subscribe(&self, context: &PolicyContext) -> PolicyDecision {
        self.authorizer.authorize_subscribe(context)
    }

    pub fn authorize_publish(&self, context: &PolicyContext) -> PolicyDecision {
        self.publish_authorizer.authorize_publish(context)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AllowAllSubscribeAuthorizer, ChannelPolicy, ChannelPolicyRegistry, PolicyAction,
        PolicyContext, PolicyDecision, PolicySource,
    };
    use crate::types::ErrorCode;
    use core_web::auth::AuthIdentity;
    use std::collections::HashMap;
    use std::sync::Arc;
    use uuid::Uuid;

    struct DenySubscribe;
    struct DenyPublish;

    impl super::SubscribeAuthorizer for DenySubscribe {
        fn authorize_subscribe(&self, _context: &PolicyContext) -> PolicyDecision {
            PolicyDecision::deny(ErrorCode::Forbidden, "denied")
        }
    }

    impl super::PublishAuthorizer for DenyPublish {
        fn authorize_publish(&self, _context: &PolicyContext) -> PolicyDecision {
            PolicyDecision::deny(ErrorCode::RateLimited, "too many publish requests")
        }
    }

    #[test]
    fn policy_defaults_when_not_configured() {
        let reg =
            ChannelPolicyRegistry::new(true, HashMap::new(), Arc::new(AllowAllSubscribeAuthorizer));
        let policy = reg.policy("missing");
        assert!(policy.enabled);
        assert!(policy.presence_enabled);
    }

    #[test]
    fn policy_uses_case_insensitive_lookup() {
        let mut map = HashMap::new();
        map.insert(
            "Admin.Notifications".to_string(),
            ChannelPolicy {
                enabled: false,
                guard: Some("admin".to_string()),
                presence_enabled: false,
            },
        );
        let reg = ChannelPolicyRegistry::new(true, map, Arc::new(AllowAllSubscribeAuthorizer));
        let policy = reg.policy("admin.notifications");
        assert!(!policy.enabled);
        assert_eq!(policy.guard.as_deref(), Some("admin"));
    }

    #[test]
    fn subscribe_authorizer_decision_is_exposed() {
        let reg = ChannelPolicyRegistry::new(true, HashMap::new(), Arc::new(DenySubscribe));
        let context =
            PolicyContext::new(PolicyAction::Subscribe, PolicySource::Internal, "x", None);
        let decision = reg.authorize_subscribe(&context);
        assert!(!decision.allowed);
        assert!(matches!(decision.code, ErrorCode::Forbidden));
        assert_eq!(decision.reason.as_deref(), Some("denied"));
    }

    #[test]
    fn publish_authorizer_decision_is_exposed() {
        let reg =
            ChannelPolicyRegistry::new(true, HashMap::new(), Arc::new(AllowAllSubscribeAuthorizer))
                .with_publish_authorizer(Arc::new(DenyPublish));
        let context = PolicyContext::new(PolicyAction::Publish, PolicySource::Internal, "x", None);
        let decision = reg.authorize_publish(&context);
        assert!(!decision.allowed);
        assert!(matches!(decision.code, ErrorCode::RateLimited));
        assert_eq!(
            decision.reason.as_deref(),
            Some("too many publish requests")
        );
    }

    struct ContextAwareDeny;

    impl super::SubscribeAuthorizer for ContextAwareDeny {
        fn authorize_subscribe(&self, context: &PolicyContext) -> PolicyDecision {
            assert!(matches!(context.source, PolicySource::WebSocket));
            assert!(matches!(context.action, PolicyAction::Subscribe));
            assert_eq!(context.channel(), "private_notifications");
            assert_eq!(context.room(), Some("tenant:42"));
            assert_eq!(context.guard(), Some("admin"));
            assert!(context.has_ability("rt:subscribe"));
            assert!(context.connection_id.is_some());
            assert_eq!(context.metadata.tenant_id.as_deref(), Some("42"));
            PolicyDecision::deny(ErrorCode::Forbidden, "blocked by context policy")
        }
    }

    #[test]
    fn context_is_passed_to_authorizer() {
        let identity = AuthIdentity {
            subject_id: "u1".to_string(),
            guard: "admin".to_string(),
            abilities: vec!["rt:subscribe".to_string()],
            token_id: None,
        };
        let context = PolicyContext::new(
            PolicyAction::Subscribe,
            PolicySource::WebSocket,
            "private_notifications",
            Some("tenant:42".to_string()),
        )
        .with_identity(Some(&identity))
        .with_connection_id(Uuid::new_v4())
        .with_metadata(super::PolicyMetadata {
            tenant_id: Some("42".to_string()),
            ..Default::default()
        });

        let reg = ChannelPolicyRegistry::new(true, HashMap::new(), Arc::new(ContextAwareDeny));
        let decision = reg.authorize_subscribe(&context);
        assert!(!decision.allowed);
        assert_eq!(
            decision.reason.as_deref(),
            Some("blocked by context policy")
        );
    }
}
