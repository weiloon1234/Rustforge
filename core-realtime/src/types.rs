use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RealtimeTarget {
    pub room: Option<String>,
}

pub trait RealtimeEvent: Serialize {
    const CHANNEL: &'static str;
    const EVENT: &'static str;
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    Unauthorized,
    ChannelDisabled,
    Forbidden,
    InvalidMessage,
    RateLimited,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AuthIdentityPublic {
    pub subject_id: String,
    pub guard: String,
    pub abilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum ClientMessage {
    Auth {
        token: String,
    },
    Subscribe {
        channel: String,
        room: Option<String>,
        #[serde(default)]
        since_id: Option<String>,
        #[serde(default)]
        replay_limit: Option<u32>,
    },
    Unsubscribe {
        channel: String,
        room: Option<String>,
    },
    Ack {
        channel: String,
        room: Option<String>,
        delivery_id: String,
    },
    Ping,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum ServerMessage {
    AuthOk {
        identity: AuthIdentityPublic,
    },
    Event {
        channel: String,
        event: String,
        room: Option<String>,
        payload: Value,
        sent_at_unix_ms: i64,
        #[serde(default)]
        delivery_id: Option<String>,
    },
    Presence {
        channel: String,
        room: String,
        online: u64,
    },
    ReplayGap {
        channel: String,
        room: Option<String>,
        requested_since_id: String,
        first_available_id: String,
        last_available_id: String,
    },
    Error {
        code: ErrorCode,
        message: String,
    },
    AckOk {
        channel: String,
        room: Option<String>,
        delivery_id: String,
    },
    Pong,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PubSubEnvelope {
    pub channel: String,
    pub event: String,
    pub room: Option<String>,
    pub payload: Value,
    pub sent_at_unix_ms: i64,
    pub delivery_id: Option<String>,
}
