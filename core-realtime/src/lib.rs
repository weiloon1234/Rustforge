pub mod auth;
pub mod idempotency;
pub mod presence;
pub mod publisher;
pub mod pubsub;
pub mod registry;
pub mod server;
pub mod types;

pub use auth::{AuthResolver, ConnectionAuthState};
pub use idempotency::RealtimeIdempotency;
pub use presence::PresenceManager;
pub use publisher::{RealtimePublishSettings, RealtimePublisher};
pub use pubsub::RealtimeSubscriber;
pub use registry::{
    AllowAllPublishAuthorizer, AllowAllSubscribeAuthorizer, ChannelPolicy, ChannelPolicyRegistry,
    PolicyAction, PolicyContext, PolicyDecision, PolicyMetadata, PolicySource, PublishAuthorizer,
    SubscribeAuthorizer,
};
pub use server::{
    ws_handler, RealtimeChannelMetricsSnapshot, RealtimeHistogramSnapshot, RealtimeMetricsSnapshot,
    RealtimeReplayGapSloSnapshot, RealtimeScopeMetricsSnapshot, WsServerState,
};
pub use types::{
    ClientMessage, ErrorCode, PubSubEnvelope, RealtimeEvent, RealtimeTarget, ServerMessage,
};
