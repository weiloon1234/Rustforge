#![allow(dead_code)]
use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};

#[derive(Debug, Clone)]
pub struct Settings {
    pub app: AppSettings,
    pub server: ServerSettings,
    pub realtime: RealtimeSettings,
    pub db: DbSettings,
    pub redis: RedisSettings,
    pub s3: S3Settings,
    pub cdn: CdnSettings,
    pub worker: WorkerSettings,
    pub i18n: core_i18n::config::I18nSettings,
    pub middleware: MiddlewareSettings,
    pub auth: AuthSettings,
    pub mail: MailSettings,
    pub http_log: HttpLogSettings,
}

/// HTTP Traffic Logging Settings
#[derive(Debug, Clone)]
pub struct HttpLogSettings {
    /// Enable logging of incoming webhook requests
    pub webhook_enabled: bool,
    /// Path prefixes to log (e.g., "/wh/", "/webhook/")
    pub webhook_paths: Vec<String>,
    /// Enable logging of outgoing HTTP client requests
    pub client_enabled: bool,
    /// Days to keep logs (default: 7)
    pub retention_days: u64,
}

#[derive(Debug, Clone)]
pub struct MailSettings {
    pub enable: bool,
    pub driver: String, // smtp, log
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
    pub from_address: String,
}

#[derive(Debug, Clone)]
pub struct WorkerSettings {
    pub enabled: bool,
    pub concurrency: usize,
    pub sweep_interval: u64,
}

#[derive(Debug, Clone)]
pub struct MiddlewareSettings {
    pub rate_limit_per_second: u32,
    pub rate_limit_burst: u32,
    pub timeout_secs: u64,
    pub body_limit_mb: usize,
}

#[derive(Debug, Clone)]
pub struct AppSettings {
    pub name: String,
    pub env: String,
    pub key: String,
    pub enable_docs: bool,
    pub docs_path: String,
    pub enable_openapi_docs: bool,
    pub openapi_docs_path: String,
    pub openapi_json_path: String,
    pub default_per_page: u64,
    pub datatable_unknown_filter_mode: DataTableUnknownFilterMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataTableUnknownFilterMode {
    Ignore,
    Warn,
    Error,
}

impl DataTableUnknownFilterMode {
    fn parse(raw: &str) -> Result<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "ignore" | "0" | "false" => Ok(Self::Ignore),
            "warn" | "warning" => Ok(Self::Warn),
            "error" | "strict" | "1" | "true" => Ok(Self::Error),
            _ => Err(anyhow!(
                "Invalid DATATABLE_UNKNOWN_FILTER_MODE (expected ignore|warn|error)"
            )),
        }
    }
}

impl AppSettings {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            name: get_env("APP_NAME", "foundation"),
            env: get_env("APP_ENV", "local"),
            key: get_env(
                "APP_KEY",
                "base64:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=",
            ), // 32 bytes of zeros
            enable_docs: get_env_bool("ENABLE_FRAMEWORK_DOCS", false)?,
            docs_path: get_env("FRAMEWORK_DOCS_PATH", "/framework-documentation"),
            enable_openapi_docs: get_env_bool("ENABLE_OPENAPI_DOCS", false)?,
            openapi_docs_path: get_env("OPENAPI_DOCS_PATH", "/openapi"),
            openapi_json_path: get_env("OPENAPI_JSON_PATH", "/openapi.json"),
            default_per_page: get_env_u64("DEFAULT_PER_PAGE", 30)?,
            datatable_unknown_filter_mode: DataTableUnknownFilterMode::parse(&get_env(
                "DATATABLE_UNKNOWN_FILTER_MODE",
                "ignore",
            ))?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ServerSettings {
    pub host: String,
    pub port: u16,
}

impl ServerSettings {
    pub fn bind_addr(&self) -> Result<SocketAddr> {
        let addr: SocketAddr = format!("{}:{}", self.host, self.port)
            .parse()
            .context("Invalid SERVER_HOST/SERVER_PORT")?;
        Ok(addr)
    }
}

#[derive(Debug, Clone)]
pub struct RealtimeSettings {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    pub heartbeat_secs: u64,
    pub presence_ttl_secs: u64,
    pub max_connections: usize,
    pub max_message_bytes: usize,
    pub max_frame_bytes: usize,
    pub max_messages_per_sec: u32,
    pub send_queue_capacity: usize,
    pub require_auth: bool,
    pub checkpoint_enabled: bool,
    pub checkpoint_ttl_secs: u64,
    pub delivery_mode: RealtimeDeliveryMode,
    pub stream_max_len: usize,
    pub stream_retention_secs: u64,
    pub replay_limit_default: usize,
    pub replay_limit_max: usize,
    pub replay_gap_alert_threshold: u64,
    pub replay_gap_alert_window_secs: u64,
    pub channels: std::collections::HashMap<String, RealtimeChannelConfig>,
}

impl RealtimeSettings {
    pub fn bind_addr(&self) -> Result<SocketAddr> {
        let addr: SocketAddr = format!("{}:{}", self.host, self.port)
            .parse()
            .context("Invalid REALTIME_HOST/REALTIME_PORT")?;
        Ok(addr)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RealtimeDeliveryMode {
    AtMostOnce,
    Durable,
}

impl RealtimeDeliveryMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AtMostOnce => "at_most_once",
            Self::Durable => "durable",
        }
    }

    fn parse(raw: &str) -> Result<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "at_most_once" => Ok(Self::AtMostOnce),
            "durable" => Ok(Self::Durable),
            _ => Err(anyhow!(
                "Invalid REALTIME_DELIVERY_MODE (expected at_most_once|durable)"
            )),
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize, Default)]
pub struct RealtimeChannelConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub guard: Option<String>,
    #[serde(default = "default_true")]
    pub presence_enabled: bool,
    #[serde(default)]
    pub max_message_bytes: Option<usize>,
    #[serde(default)]
    pub max_frame_bytes: Option<usize>,
    #[serde(default)]
    pub max_messages_per_sec: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct DbSettings {
    pub url: String,
    pub max_connections: u32,
    pub connect_timeout: Duration,
}

#[derive(Debug, Clone)]
pub struct RedisSettings {
    pub url: String,
    pub prefix: Option<String>,
}

#[derive(Debug, Clone)]
pub struct S3Settings {
    pub endpoint: String,
    pub region: String,
    pub bucket: String,
    pub access_key: String,
    pub secret_key: String,
    pub force_path_style: bool,
}

#[derive(Debug, Clone)]
pub struct CdnSettings {
    /// Public base URL (or CDN CNAME) for attachments, e.g. https://cdn.example.com
    pub base_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AuthSettings {
    pub default_guard: String,
    pub guards: std::collections::HashMap<String, GuardConfig>,
}

impl AuthSettings {
    pub fn guard(&self, name: &str) -> Option<&GuardConfig> {
        self.guards.get(name)
    }

    pub fn default_guard_config(&self) -> Option<&GuardConfig> {
        self.guard(&self.default_guard)
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct GuardConfig {
    // Driver is implied (Token/Stateful)
    pub provider: String,
    pub ttl_min: u64,
    pub refresh_ttl_days: u64,
}

#[derive(Debug, serde::Deserialize, Default)]
struct TomlConfig {
    #[serde(default)]
    auth: TomlAuth,
    #[serde(default)]
    realtime: TomlRealtime,
}

#[derive(Debug, serde::Deserialize, Default)]
struct TomlAuth {
    #[serde(default)]
    default: Option<String>,
    // secret: Option<String>, // Removed
    #[serde(default)]
    guards: std::collections::HashMap<String, GuardConfig>,
}

#[derive(Debug, serde::Deserialize, Default)]
struct TomlRealtime {
    #[serde(default)]
    channels: std::collections::HashMap<String, RealtimeChannelConfig>,
}

impl Settings {
    pub fn load() -> Result<Self> {
        // Assumes dotenv is already loaded in main.rs via dotenvy::dotenv()

        let app = AppSettings::from_env()?;

        let server = ServerSettings {
            host: get_env("SERVER_HOST", "0.0.0.0"),
            port: get_env_u16("SERVER_PORT", 3000)?,
        };

        let realtime = RealtimeSettings {
            enabled: get_env_bool("REALTIME_ENABLED", true)?,
            host: get_env("REALTIME_HOST", "0.0.0.0"),
            port: get_env_u16("REALTIME_PORT", 3010)?,
            heartbeat_secs: get_env_u64("REALTIME_HEARTBEAT_SECS", 20)?,
            presence_ttl_secs: get_env_u64("REALTIME_PRESENCE_TTL_SECS", 60)?,
            max_connections: get_env_u64("REALTIME_MAX_CONNECTIONS", 10000)? as usize,
            max_message_bytes: get_env_u64("REALTIME_MAX_MESSAGE_BYTES", 64 * 1024)? as usize,
            max_frame_bytes: get_env_u64("REALTIME_MAX_FRAME_BYTES", 64 * 1024)? as usize,
            max_messages_per_sec: get_env_u32("REALTIME_MAX_MESSAGES_PER_SEC", 150)?,
            send_queue_capacity: get_env_u64("REALTIME_SEND_QUEUE_CAPACITY", 1024)? as usize,
            require_auth: get_env_bool("REALTIME_REQUIRE_AUTH", true)?,
            checkpoint_enabled: get_env_bool("REALTIME_CHECKPOINT_ENABLED", false)?,
            checkpoint_ttl_secs: get_env_u64("REALTIME_CHECKPOINT_TTL_SECS", 2_592_000)?,
            delivery_mode: RealtimeDeliveryMode::parse(&get_env(
                "REALTIME_DELIVERY_MODE",
                "at_most_once",
            ))?,
            stream_max_len: get_env_u64("REALTIME_STREAM_MAX_LEN", 100000)? as usize,
            stream_retention_secs: get_env_u64("REALTIME_STREAM_RETENTION_SECS", 0)?,
            replay_limit_default: get_env_u64("REALTIME_REPLAY_LIMIT_DEFAULT", 200)? as usize,
            replay_limit_max: get_env_u64("REALTIME_REPLAY_LIMIT_MAX", 1000)? as usize,
            replay_gap_alert_threshold: get_env_u64("REALTIME_REPLAY_GAP_ALERT_THRESHOLD", 100)?,
            replay_gap_alert_window_secs: get_env_u64("REALTIME_REPLAY_GAP_ALERT_WINDOW_SECS", 60)?,
            channels: std::collections::HashMap::new(),
        };

        let db = DbSettings {
            url: get_env_required("DATABASE_URL")?,
            max_connections: get_env_u32("DB_MAX_CONNECTIONS", 10)?,
            connect_timeout: Duration::from_secs(get_env_u64("DB_CONNECT_TIMEOUT_SECS", 5)?),
        };

        let redis_url = if let Ok(url) = env::var("REDIS_URL") {
            url
        } else {
            let host = get_env("REDIS_HOST", "127.0.0.1");
            let port = get_env("REDIS_PORT", "6379");
            let password = env::var("REDIS_PASSWORD").unwrap_or_default();
            let db = get_env("REDIS_DB", "0");

            if password.is_empty() {
                format!("redis://{}:{}/{}", host, port, db)
            } else {
                format!("redis://:{}@{}:{}/{}", password, host, port, db)
            }
        };

        let redis = RedisSettings {
            url: redis_url,
            prefix: resolve_redis_prefix(&app.name, &app.env),
        };

        let s3 = S3Settings {
            endpoint: get_env("S3_ENDPOINT", ""),
            region: get_env("S3_REGION", "auto"),
            bucket: get_env("S3_BUCKET", ""),
            access_key: get_env("S3_ACCESS_KEY", ""),
            secret_key: get_env("S3_SECRET_KEY", ""),
            force_path_style: get_env_bool("S3_FORCE_PATH_STYLE", false)?,
        };

        let cdn = CdnSettings {
            base_url: env::var("S3_URL").ok(),
        };

        let worker = WorkerSettings {
            enabled: get_env_bool("RUN_WORKER", false)?,
            concurrency: get_env_u64("WORKER_CONCURRENCY", 10)? as usize,
            sweep_interval: get_env_u64("WORKER_SWEEP_INTERVAL", 30)?,
        };

        let tz_str = get_env("APP_TIMEZONE", "+08:00");
        let i18n = core_i18n::config::I18nSettings {
            default_locale: "en",
            supported_locales: &["en", "zh"], // Fallback if not configured
            default_timezone: core_i18n::config::I18nSettings::parse_utc_offset(&tz_str)
                .context("Invalid APP_TIMEZONE; expected format like +08:00")?,
            default_timezone_str: tz_str,
        };

        let middleware = MiddlewareSettings {
            rate_limit_per_second: get_env_u32("MW_RATE_LIMIT_PER_SEC", 2)?,
            rate_limit_burst: get_env_u32("MW_RATE_LIMIT_BURST", 60)?,
            timeout_secs: get_env_u64("MW_TIMEOUT_SECS", 30)?,
            body_limit_mb: get_env_u64("MW_BODY_LIMIT_MB", 10)? as usize,
        };

        let toml_config: TomlConfig = load_toml_config()?;

        let auth = AuthSettings {
            default_guard: toml_config
                .auth
                .default
                .unwrap_or_else(|| "web".to_string()),
            guards: toml_config.auth.guards,
        };
        let mut realtime = realtime;
        realtime.channels = toml_config.realtime.channels;

        // Minimal sanity checks (optional but recommended)
        if db.url.is_empty() {
            return Err(anyhow!("DATABASE_URL is required"));
        }

        Ok(Self {
            app,
            server,
            realtime,
            db,
            redis,
            s3,
            cdn,
            worker,
            i18n,
            middleware,
            auth,
            mail: MailSettings {
                enable: get_env_bool("MAIL_ENABLE", true)?,
                driver: get_env("MAIL_DRIVER", "log"),
                host: get_env("MAIL_HOST", "smtp.mailtrap.io"),
                port: get_env_u16("MAIL_PORT", 2525)?,
                username: env::var("MAIL_USERNAME").ok().filter(|s| !s.is_empty()),
                password: env::var("MAIL_PASSWORD").ok().filter(|s| !s.is_empty()),
                from_address: get_env("MAIL_FROM_ADDRESS", "hello@example.com"),
            },
            http_log: HttpLogSettings {
                webhook_enabled: get_env_bool("HTTP_LOG_WEBHOOK_ENABLED", false)?,
                webhook_paths: get_env("HTTP_LOG_WEBHOOK_PATHS", "/wh/,/webhook/")
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect(),
                client_enabled: get_env_bool("HTTP_LOG_CLIENT_ENABLED", false)?,
                retention_days: get_env_u64("HTTP_LOG_RETENTION_DAYS", 7)?,
            },
        })
    }
}

fn default_true() -> bool {
    true
}

/* -----------------------------
   Helpers (private)
------------------------------ */

fn load_toml_config() -> Result<TomlConfig> {
    let path = env::var("APP_CONFIGS_PATH")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| "configs.toml".to_string());

    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read config file at {}", path))?;

    if content.trim().is_empty() {
        return Ok(TomlConfig::default());
    }

    toml::from_str(&content).with_context(|| format!("Invalid TOML in {}", path))
}

fn get_env(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

fn get_env_required(key: &str) -> Result<String> {
    env::var(key).with_context(|| format!("{key} is required"))
}

fn get_env_u16(key: &str, default: u16) -> Result<u16> {
    let raw = env::var(key).unwrap_or_else(|_| default.to_string());
    raw.parse::<u16>()
        .with_context(|| format!("Invalid {key} (expected u16)"))
}

fn get_env_u32(key: &str, default: u32) -> Result<u32> {
    let raw = env::var(key).unwrap_or_else(|_| default.to_string());
    raw.parse::<u32>()
        .with_context(|| format!("Invalid {key} (expected u32)"))
}

fn get_env_u64(key: &str, default: u64) -> Result<u64> {
    let raw = env::var(key).unwrap_or_else(|_| default.to_string());
    raw.parse::<u64>()
        .with_context(|| format!("Invalid {key} (expected u64)"))
}

fn get_env_bool(key: &str, default: bool) -> Result<bool> {
    let raw = env::var(key).unwrap_or_else(|_| default.to_string());
    match raw.trim().to_lowercase().as_str() {
        "1" | "true" | "yes" | "y" | "on" => Ok(true),
        "0" | "false" | "no" | "n" | "off" => Ok(false),
        _ => Err(anyhow!("Invalid {key} (expected bool)")),
    }
}

fn resolve_redis_prefix(app_name: &str, app_env: &str) -> Option<String> {
    let explicit = env::var("REDIS_CACHE_PREFIX")
        .ok()
        .map(|value| normalize_prefix_token(&value))
        .filter(|value| !value.is_empty());

    if explicit.is_some() {
        return explicit;
    }

    let app_name = normalize_prefix_token(app_name);
    let app_env = normalize_prefix_token(app_env);
    let parts = [app_name, app_env]
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();

    if parts.is_empty() {
        None
    } else {
        Some(parts.join("_"))
    }
}

fn normalize_prefix_token(raw: &str) -> String {
    raw.trim()
        .to_ascii_lowercase()
        .replace([' ', '-', ':'], "_")
        .chars()
        .filter_map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                Some(ch)
            } else {
                None
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .to_string()
}
