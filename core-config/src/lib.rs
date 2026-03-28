#![allow(dead_code)]
pub mod global_config;

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};

// ─── Top-level Settings ─────────────────────────────────────────────────────

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
    pub cors: CorsSettings,
    pub auth: AuthSettings,
    pub mail: MailSettings,
    pub http_log: HttpLogSettings,
    /// Raw TOML tree (after env overlay) for app-level custom sections.
    /// Access custom sections via `settings.section::<T>("name")`.
    pub tree: Arc<toml::Value>,
}

impl Settings {
    /// Load settings from TOML file with env var overlay.
    ///
    /// 1. Reads TOML file (path from `SETTINGS_PATH` env var, default `app/settings.toml`)
    /// 2. Walks every leaf and overlays matching env vars (`[section].field` → `SECTION_FIELD`)
    /// 3. Deserializes typed framework sections
    /// 4. Preserves full tree for `section::<T>()` access
    pub fn load() -> Result<Self> {
        // .env is already loaded by dotenvy in bootstrap
        let path = std::env::var("SETTINGS_PATH")
            .ok()
            .filter(|v| !v.trim().is_empty())
            .unwrap_or_else(|| "app/settings.toml".to_string());

        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read settings file at '{path}'"))?;

        let mut tree: toml::Value = if content.trim().is_empty() {
            toml::Value::Table(toml::map::Map::new())
        } else {
            toml::from_str(&content).with_context(|| format!("Invalid TOML in {path}"))?
        };

        // Overlay env vars on every leaf
        overlay_env_vars(&mut tree, &[]);

        // Deserialize typed framework sections
        let app: AppSettings = deserialize_section(&tree, "app")?;
        let server: ServerSettings = deserialize_section(&tree, "server")?;
        let db: DbSettingsRaw = deserialize_section(&tree, "database")?;
        let redis_raw: RedisSettingsRaw = deserialize_section(&tree, "redis")?;
        let s3: S3Settings = deserialize_section(&tree, "s3")?;
        let worker: WorkerSettings = deserialize_section(&tree, "worker")?;
        let middleware: MiddlewareSettings = deserialize_section(&tree, "middleware")?;
        let cors: CorsSettings = deserialize_section(&tree, "cors")?;
        let mail: MailSettings = deserialize_section(&tree, "mail")?;
        let http_log: HttpLogSettings = deserialize_section(&tree, "http_log")?;
        let realtime_raw: RealtimeSettingsRaw = deserialize_section(&tree, "realtime")?;
        let auth_raw: AuthSettingsRaw = deserialize_section(&tree, "auth")?;
        let languages: LanguagesSettings = deserialize_section(&tree, "languages")?;

        // Build Redis settings (resolve URL from parts if needed)
        let redis = RedisSettings {
            url: redis_raw.url,
            prefix: resolve_redis_prefix(&app.name, &app.env, &redis_raw.cache_prefix),
        };

        // Build CDN settings
        let cdn = CdnSettings {
            base_url: if s3.public_url.is_empty() {
                None
            } else {
                Some(s3.public_url.clone())
            },
        };

        // Build auth settings
        let auth = AuthSettings {
            default_guard: auth_raw.default,
            guards: auth_raw.guards,
        };

        // Build realtime settings
        let realtime = RealtimeSettings {
            enabled: realtime_raw.enabled,
            host: realtime_raw.host,
            port: realtime_raw.port,
            heartbeat_secs: realtime_raw.heartbeat_secs,
            presence_ttl_secs: realtime_raw.presence_ttl_secs,
            max_connections: realtime_raw.max_connections as usize,
            max_message_bytes: realtime_raw.max_message_bytes as usize,
            max_frame_bytes: realtime_raw.max_frame_bytes as usize,
            max_messages_per_sec: realtime_raw.max_messages_per_sec,
            send_queue_capacity: realtime_raw.send_queue_capacity as usize,
            require_auth: realtime_raw.require_auth,
            checkpoint_enabled: realtime_raw.checkpoint_enabled,
            checkpoint_ttl_secs: realtime_raw.checkpoint_ttl_secs,
            delivery_mode: RealtimeDeliveryMode::parse(&realtime_raw.delivery_mode)?,
            stream_max_len: realtime_raw.stream_max_len as usize,
            stream_retention_secs: realtime_raw.stream_retention_secs,
            replay_limit_default: realtime_raw.replay_limit_default as usize,
            replay_limit_max: realtime_raw.replay_limit_max as usize,
            replay_gap_alert_threshold: realtime_raw.replay_gap_alert_threshold,
            replay_gap_alert_window_secs: realtime_raw.replay_gap_alert_window_secs,
            channels: realtime_raw.channels,
        };

        // Build i18n settings (requires special handling for &'static str + UtcOffset)
        let tz_str = &app.timezone;
        let supported_locales: &'static [&'static str] = if languages.supported.is_empty() {
            &["en", "zh"]
        } else {
            let leaked: Vec<&'static str> = languages
                .supported
                .into_iter()
                .map(|s| Box::leak(s.into_boxed_str()) as &str)
                .collect();
            Box::leak(leaked.into_boxed_slice())
        };
        let default_locale: &'static str = if languages.default.is_empty() {
            "en"
        } else {
            Box::leak(languages.default.into_boxed_str())
        };
        let i18n = core_i18n::config::I18nSettings {
            default_locale,
            supported_locales,
            default_timezone: core_i18n::config::I18nSettings::parse_utc_offset(tz_str)
                .context("Invalid [app].timezone; expected format like +08:00")?,
            default_timezone_str: tz_str.clone(),
        };

        // Validation
        if app.key.is_empty() {
            return Err(anyhow!(
                "[app].key is required. Generate one with: openssl rand -base64 32 and prefix with 'base64:'"
            ));
        }
        if db.url.is_empty() {
            return Err(anyhow!("[database].url is required"));
        }
        if cors.supports_credentials
            && cors.allowed_origins.len() == 1
            && cors.allowed_origins[0] == "*"
        {
            return Err(anyhow!(
                "[cors] supports_credentials = true requires explicit allowed_origins (cannot use [\"*\"])"
            ));
        }

        Ok(Self {
            app,
            server,
            realtime,
            db: DbSettings {
                url: db.url,
                max_connections: db.max_connections,
                connect_timeout: Duration::from_secs(db.connect_timeout_secs),
                snowflake_node_id: db.snowflake_node_id,
                sql_profiler_enabled: db.sql_profiler_enabled,
                sql_profiler_retention_days: db.sql_profiler_retention_days,
            },
            redis,
            s3,
            cdn,
            worker,
            i18n,
            middleware,
            cors,
            auth,
            mail,
            http_log,
            tree: Arc::new(tree),
        })
    }

    /// Deserialize any TOML section into a custom struct.
    /// Returns `T::default()` if the section doesn't exist.
    pub fn section<T: serde::de::DeserializeOwned + Default>(&self, name: &str) -> Result<T> {
        match self.tree.get(name) {
            Some(val) => val
                .clone()
                .try_into()
                .with_context(|| format!("Invalid [{name}] section in settings")),
            None => Ok(T::default()),
        }
    }
}

// ─── Framework Section Structs ──────────────────────────────────────────────

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct AppSettings {
    pub name: String,
    pub env: String,
    pub key: String,
    pub debug: bool,
    pub timezone: String,
    pub log_level: String,
    pub cookie_secure: String,
    pub default_per_page: u64,
    pub datatable_unknown_filter_mode: String,
    pub datatable_export_link_ttl_secs: u64,
    pub enable_framework_docs: bool,
    pub framework_docs_path: String,
    pub framework_docs_dist_dir: String,
    pub enable_openapi_docs: bool,
    pub openapi_docs_path: String,
    pub openapi_json_path: String,
    pub paths_migrations_dir: String,
    pub paths_seeders_dir: String,
    pub paths_public: String,
    pub paths_i18n_dir: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            name: "starter".into(),
            env: "local".into(),
            key: String::new(),
            debug: false,
            timezone: "+08:00".into(),
            log_level: "info".into(),
            cookie_secure: String::new(),
            default_per_page: 30,
            datatable_unknown_filter_mode: "ignore".into(),
            datatable_export_link_ttl_secs: 604_800,
            enable_framework_docs: false,
            framework_docs_path: "/framework-documentation".into(),
            framework_docs_dist_dir: "/public/framework-documentation".into(),
            enable_openapi_docs: true,
            openapi_docs_path: "/openapi".into(),
            openapi_json_path: "/openapi.json".into(),
            paths_migrations_dir: "migrations".into(),
            paths_seeders_dir: "app/src/seeds".into(),
            paths_public: "public".into(),
            paths_i18n_dir: "i18n".into(),
        }
    }
}

impl AppSettings {
    pub fn is_production(&self) -> bool {
        matches!(self.env.as_str(), "production" | "prod")
    }

    pub fn cookie_secure(&self) -> bool {
        if !self.cookie_secure.is_empty() {
            return matches!(
                self.cookie_secure.to_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            );
        }
        // Auto: secure unless local/dev/test
        !matches!(self.env.as_str(), "local" | "dev" | "development" | "test")
    }

    pub fn datatable_unknown_filter_mode(&self) -> DataTableUnknownFilterMode {
        DataTableUnknownFilterMode::parse(&self.datatable_unknown_filter_mode)
            .unwrap_or(DataTableUnknownFilterMode::Ignore)
    }
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
            "ignore" | "0" | "false" | "" => Ok(Self::Ignore),
            "warn" | "warning" => Ok(Self::Warn),
            "error" | "strict" | "1" | "true" => Ok(Self::Error),
            _ => Err(anyhow!(
                "Invalid datatable_unknown_filter_mode (expected ignore|warn|error)"
            )),
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct ServerSettings {
    pub host: String,
    pub port: u16,
}

impl Default for ServerSettings {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".into(),
            port: 3000,
        }
    }
}

impl ServerSettings {
    pub fn bind_addr(&self) -> Result<SocketAddr> {
        format!("{}:{}", self.host, self.port)
            .parse()
            .context("Invalid [server] host/port")
    }
}

// Intermediate raw struct for TOML deserialization
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
struct RealtimeSettingsRaw {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    pub heartbeat_secs: u64,
    pub presence_ttl_secs: u64,
    pub max_connections: u64,
    pub max_message_bytes: u64,
    pub max_frame_bytes: u64,
    pub max_messages_per_sec: u32,
    pub send_queue_capacity: u64,
    pub require_auth: bool,
    pub checkpoint_enabled: bool,
    pub checkpoint_ttl_secs: u64,
    pub delivery_mode: String,
    pub stream_max_len: u64,
    pub stream_retention_secs: u64,
    pub replay_limit_default: u64,
    pub replay_limit_max: u64,
    pub replay_gap_alert_threshold: u64,
    pub replay_gap_alert_window_secs: u64,
    #[serde(default)]
    pub channels: std::collections::HashMap<String, RealtimeChannelConfig>,
}

impl Default for RealtimeSettingsRaw {
    fn default() -> Self {
        Self {
            enabled: true,
            host: "0.0.0.0".into(),
            port: 3010,
            heartbeat_secs: 20,
            presence_ttl_secs: 60,
            max_connections: 10000,
            max_message_bytes: 65536,
            max_frame_bytes: 65536,
            max_messages_per_sec: 150,
            send_queue_capacity: 1024,
            require_auth: true,
            checkpoint_enabled: false,
            checkpoint_ttl_secs: 2_592_000,
            delivery_mode: "at_most_once".into(),
            stream_max_len: 100_000,
            stream_retention_secs: 0,
            replay_limit_default: 200,
            replay_limit_max: 1000,
            replay_gap_alert_threshold: 100,
            replay_gap_alert_window_secs: 60,
            channels: std::collections::HashMap::new(),
        }
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
        format!("{}:{}", self.host, self.port)
            .parse()
            .context("Invalid [realtime] host/port")
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
            "at_most_once" | "" => Ok(Self::AtMostOnce),
            "durable" => Ok(Self::Durable),
            _ => Err(anyhow!(
                "Invalid realtime delivery_mode (expected at_most_once|durable)"
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

/// Database settings (intermediate for TOML, converted to final DbSettings with Duration).
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
struct DbSettingsRaw {
    pub url: String,
    pub max_connections: u32,
    pub connect_timeout_secs: u64,
    pub snowflake_node_id: u16,
    pub sql_profiler_enabled: bool,
    pub sql_profiler_retention_days: u64,
}

impl Default for DbSettingsRaw {
    fn default() -> Self {
        Self {
            url: String::new(),
            max_connections: 10,
            connect_timeout_secs: 5,
            snowflake_node_id: 1,
            sql_profiler_enabled: false,
            sql_profiler_retention_days: 7,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DbSettings {
    pub url: String,
    pub max_connections: u32,
    pub connect_timeout: Duration,
    pub snowflake_node_id: u16,
    pub sql_profiler_enabled: bool,
    pub sql_profiler_retention_days: u64,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
struct RedisSettingsRaw {
    pub url: String,
    pub cache_prefix: String,
}

impl Default for RedisSettingsRaw {
    fn default() -> Self {
        Self {
            url: "redis://127.0.0.1:6379/0".into(),
            cache_prefix: String::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RedisSettings {
    pub url: String,
    pub prefix: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct S3Settings {
    pub endpoint: String,
    pub region: String,
    pub bucket: String,
    pub access_key: String,
    pub secret_key: String,
    pub force_path_style: bool,
    pub public_url: String,
}

impl Default for S3Settings {
    fn default() -> Self {
        Self {
            endpoint: String::new(),
            region: "auto".into(),
            bucket: String::new(),
            access_key: String::new(),
            secret_key: String::new(),
            force_path_style: false,
            public_url: String::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CdnSettings {
    pub base_url: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct WorkerSettings {
    pub enabled: bool,
    pub concurrency: usize,
    pub sweep_interval: u64,
}

impl Default for WorkerSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            concurrency: 10,
            sweep_interval: 30,
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct MiddlewareSettings {
    pub rate_limit_per_second: u32,
    pub rate_limit_burst: u32,
    pub timeout_secs: u64,
    pub body_limit_mb: usize,
}

impl Default for MiddlewareSettings {
    fn default() -> Self {
        Self {
            rate_limit_per_second: 10,
            rate_limit_burst: 100,
            timeout_secs: 30,
            body_limit_mb: 10,
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct CorsSettings {
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<String>,
    pub allowed_headers: Vec<String>,
    pub exposed_headers: Vec<String>,
    pub max_age: u64,
    pub supports_credentials: bool,
}

impl Default for CorsSettings {
    fn default() -> Self {
        Self {
            allowed_origins: vec!["*".to_string()],
            allowed_methods: vec!["*".to_string()],
            allowed_headers: vec!["*".to_string()],
            exposed_headers: vec![],
            max_age: 0,
            supports_credentials: false,
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
struct AuthSettingsRaw {
    pub default: String,
    #[serde(default)]
    pub guards: std::collections::HashMap<String, GuardConfig>,
}

impl Default for AuthSettingsRaw {
    fn default() -> Self {
        Self {
            default: "web".into(),
            guards: std::collections::HashMap::new(),
        }
    }
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
    pub provider: String,
    pub ttl_min: u64,
    pub refresh_ttl_days: u64,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct MailSettings {
    pub enable: bool,
    pub driver: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub from_address: String,
}

impl Default for MailSettings {
    fn default() -> Self {
        Self {
            enable: false,
            driver: "log".into(),
            host: "smtp.mailtrap.io".into(),
            port: 2525,
            username: String::new(),
            password: String::new(),
            from_address: "hello@example.com".into(),
        }
    }
}

impl MailSettings {
    pub fn username_opt(&self) -> Option<&str> {
        if self.username.is_empty() {
            None
        } else {
            Some(&self.username)
        }
    }

    pub fn password_opt(&self) -> Option<&str> {
        if self.password.is_empty() {
            None
        } else {
            Some(&self.password)
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct HttpLogSettings {
    pub webhook_enabled: bool,
    pub webhook_paths: String,
    pub client_enabled: bool,
    pub retention_days: u64,
}

impl Default for HttpLogSettings {
    fn default() -> Self {
        Self {
            webhook_enabled: false,
            webhook_paths: "/wh/,/webhook/".into(),
            client_enabled: false,
            retention_days: 7,
        }
    }
}

impl HttpLogSettings {
    pub fn webhook_path_list(&self) -> Vec<String> {
        self.webhook_paths
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
struct LanguagesSettings {
    pub default: String,
    pub supported: Vec<String>,
}

impl Default for LanguagesSettings {
    fn default() -> Self {
        Self {
            default: "en".into(),
            supported: vec!["en".into(), "zh".into()],
        }
    }
}

// ─── Helpers ────────────────────────────────────────────────────────────────

fn default_true() -> bool {
    true
}

/// Deserialize a TOML section into a typed struct.
/// Returns `T::default()` if the section doesn't exist.
fn deserialize_section<T: serde::de::DeserializeOwned + Default>(
    tree: &toml::Value,
    section: &str,
) -> Result<T> {
    match tree.get(section) {
        Some(val) => val
            .clone()
            .try_into()
            .with_context(|| format!("Invalid [{section}] section in settings")),
        None => Ok(T::default()),
    }
}

/// Recursively walk a TOML tree and overlay matching env vars.
///
/// Convention: `[section].field` → `SECTION_FIELD` env var.
/// Nested: `[section.sub].field` → `SECTION_SUB_FIELD`.
fn overlay_env_vars(value: &mut toml::Value, path: &[String]) {
    match value {
        toml::Value::Table(map) => {
            for key in map.keys().cloned().collect::<Vec<_>>() {
                let mut new_path = path.to_vec();
                new_path.push(key.clone());
                if let Some(val) = map.get_mut(&key) {
                    overlay_env_vars(val, &new_path);
                }
            }
        }
        toml::Value::Array(_) => {
            // Arrays can be overridden as comma-separated env vars
            let env_key = path.join("_").to_uppercase();
            if let Ok(env_val) = std::env::var(&env_key) {
                let items: Vec<toml::Value> = env_val
                    .split(',')
                    .map(|s| toml::Value::String(s.trim().to_string()))
                    .filter(|v| v.as_str() != Some(""))
                    .collect();
                *value = toml::Value::Array(items);
            }
        }
        leaf => {
            let env_key = path.join("_").to_uppercase();
            if let Ok(env_val) = std::env::var(&env_key) {
                *leaf = coerce_to_toml(leaf, &env_val);
            }
        }
    }
}

/// Coerce an env var string to the same TOML type as the existing value.
fn coerce_to_toml(existing: &toml::Value, env_val: &str) -> toml::Value {
    match existing {
        toml::Value::Boolean(_) => toml::Value::Boolean(matches!(
            env_val.to_lowercase().as_str(),
            "1" | "true" | "yes" | "y" | "on"
        )),
        toml::Value::Integer(_) => env_val
            .parse::<i64>()
            .map(toml::Value::Integer)
            .unwrap_or_else(|_| toml::Value::String(env_val.to_string())),
        toml::Value::Float(_) => env_val
            .parse::<f64>()
            .map(toml::Value::Float)
            .unwrap_or_else(|_| toml::Value::String(env_val.to_string())),
        _ => toml::Value::String(env_val.to_string()),
    }
}

fn resolve_redis_prefix(app_name: &str, app_env: &str, explicit: &str) -> Option<String> {
    let explicit = normalize_prefix_token(explicit);
    if !explicit.is_empty() {
        return Some(explicit);
    }

    let app_name = normalize_prefix_token(app_name);
    let app_env = normalize_prefix_token(app_env);
    let parts: Vec<_> = [app_name, app_env]
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect();

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
        .filter(|ch| ch.is_ascii_alphanumeric() || *ch == '_')
        .collect::<String>()
        .trim_matches('_')
        .to_string()
}
