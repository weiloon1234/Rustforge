export function Configuration() {
    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">Configuration</h1>
                <p className="text-xl text-gray-500">
                    Tiered configuration system (Environment Variables + TOML).
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                {/* Overview */}
                <p>
                    The framework uses a two-tier configuration system combining
                    environment variables for secrets/runtime and TOML for structured
                    application settings.
                </p>

                {/* Environment Variables */}
                <h2>Environment Variables (.env)</h2>
                <p>
                    System-level settings are loaded from <code>.env</code> file. Copy{' '}
                    <code>.env.example</code> to get started.
                </p>

                <h3>Complete .env.example Reference</h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-bash">{`# ----------------------------
# App
# ----------------------------
APP_NAME=foundation
APP_ENV=local                   # local | staging | production
APP_KEY=base64:...              # 32-byte encryption key
APP_TIMEZONE=+08:00             # Fixed offset in ±HH:MM format
RUST_LOG=info                   # trace | debug | info | warn | error
ENABLE_FRAMEWORK_DOCS=true
FRAMEWORK_DOCS_PATH=/framework-documentation
ENABLE_OPENAPI_DOCS=true
OPENAPI_DOCS_PATH=/openapi
OPENAPI_JSON_PATH=/openapi.json
DEFAULT_PER_PAGE=30             # Default pagination limit
DATATABLE_UNKNOWN_FILTER_MODE=ignore  # ignore | warn | error

# Server bind
SERVER_HOST=0.0.0.0
SERVER_PORT=3000

# Realtime WebSocket
REALTIME_ENABLED=true
REALTIME_HOST=0.0.0.0
REALTIME_PORT=3010
REALTIME_HEARTBEAT_SECS=20
REALTIME_PRESENCE_TTL_SECS=60
REALTIME_MAX_CONNECTIONS=10000
REALTIME_MAX_MESSAGE_BYTES=65536
REALTIME_MAX_FRAME_BYTES=65536
REALTIME_MAX_MESSAGES_PER_SEC=150
REALTIME_SEND_QUEUE_CAPACITY=1024
REALTIME_REQUIRE_AUTH=true
REALTIME_DELIVERY_MODE=at_most_once  # at_most_once | durable
REALTIME_STREAM_MAX_LEN=100000
REALTIME_STREAM_RETENTION_SECS=0
REALTIME_REPLAY_LIMIT_DEFAULT=200
REALTIME_REPLAY_LIMIT_MAX=1000
REALTIME_REPLAY_GAP_ALERT_THRESHOLD=100
REALTIME_REPLAY_GAP_ALERT_WINDOW_SECS=60
REALTIME_CHECKPOINT_ENABLED=false
REALTIME_CHECKPOINT_TTL_SECS=2592000

# ----------------------------
# Worker & Jobs
# ----------------------------
RUN_WORKER=false                # Enable background job processing
WORKER_CONCURRENCY=10           # Number of concurrent jobs
WORKER_SWEEP_INTERVAL=30        # Seconds between queue sweeps

# ----------------------------
# Database (Postgres)
# ----------------------------
DATABASE_URL=postgres://...
DB_MAX_CONNECTIONS=10           # Connection pool size
DB_CONNECT_TIMEOUT_SECS=5       # Connection timeout

# ----------------------------
# Redis
# ----------------------------
# REDIS_URL=redis://127.0.0.1/  # Optional override
REDIS_HOST=127.0.0.1
REDIS_PORT=6379
REDIS_PASSWORD=
REDIS_DB=0
# Optional override. Leave empty to auto-derive "{APP_NAME}_{APP_ENV}".
REDIS_CACHE_PREFIX=

# ----------------------------
# Object Storage (S3/R2/MinIO)
# ----------------------------
S3_ENDPOINT=                    # Empty for AWS S3
S3_REGION=auto
S3_BUCKET=
S3_ACCESS_KEY=
S3_SECRET_KEY=
S3_FORCE_PATH_STYLE=false       # true for MinIO
S3_URL=                         # Public CDN base URL

# ----------------------------
# Middleware Limits
# ----------------------------
MW_TIMEOUT_SECS=30
MW_BODY_LIMIT_MB=10             # Max request body size
MW_RATE_LIMIT_PER_SEC=2         # Requests per second per IP
MW_RATE_LIMIT_BURST=60          # Burst allowance

# ----------------------------
# Mailer
# ----------------------------
MAIL_ENABLE=false
MAIL_DRIVER=log                 # smtp | log
MAIL_HOST=smtp.mailtrap.io
MAIL_PORT=2525
MAIL_USERNAME=
MAIL_PASSWORD=
MAIL_FROM_ADDRESS=hello@example.com

# ----------------------------
# HTTP Traffic Logging
# ----------------------------
HTTP_LOG_WEBHOOK_ENABLED=false
HTTP_LOG_WEBHOOK_PATHS=/wh/,/webhook/  # Prefix match, comma-separated
HTTP_LOG_CLIENT_ENABLED=false
HTTP_LOG_RETENTION_DAYS=7       # Auto-cleanup retention`}</code>
                </pre>

                {/* Environment Variables Table */}
                <h3 className="mt-8">Complete Variable Guide</h3>
                <div className="overflow-x-auto border rounded-lg">
                    <table className="min-w-full divide-y divide-gray-200 text-sm">
                        <thead className="bg-gray-50">
                            <tr>
                                <th className="px-3 py-2 text-left font-medium text-gray-900">Variable</th>
                                <th className="px-3 py-2 text-left font-medium text-gray-900">Default</th>
                                <th className="px-3 py-2 text-left font-medium text-gray-900">Description</th>
                            </tr>
                        </thead>
                        <tbody className="divide-y divide-gray-200 bg-white">
                            {/* App */}
                            <tr>
                                <td className="px-3 py-2 font-mono text-blue-600">APP_ENV</td>
                                <td className="px-3 py-2 text-gray-500">local</td>
                                <td className="px-3 py-2 text-gray-700">Runtime mode (local/staging/production)</td>
                            </tr>
                            <tr>
                                <td className="px-3 py-2 font-mono text-blue-600">APP_KEY</td>
                                <td className="px-3 py-2 text-red-500 font-bold">Required</td>
                                <td className="px-3 py-2 text-gray-700">32-byte Base64 key for encryption</td>
                            </tr>
                             <tr>
                                <td className="px-3 py-2 font-mono text-blue-600">DEFAULT_PER_PAGE</td>
                                <td className="px-3 py-2 text-gray-500">30</td>
                                <td className="px-3 py-2 text-gray-700">Default pagination limit</td>
                            </tr>
                            <tr>
                                <td className="px-3 py-2 font-mono text-blue-600">ENABLE_FRAMEWORK_DOCS</td>
                                <td className="px-3 py-2 text-gray-500">false</td>
                                <td className="px-3 py-2 text-gray-700">Enable built-in framework docs router</td>
                            </tr>
                            <tr>
                                <td className="px-3 py-2 font-mono text-blue-600">FRAMEWORK_DOCS_PATH</td>
                                <td className="px-3 py-2 text-gray-500">/framework-documentation</td>
                                <td className="px-3 py-2 text-gray-700">Mount path for framework docs frontend</td>
                            </tr>
                            <tr>
                                <td className="px-3 py-2 font-mono text-blue-600">ENABLE_OPENAPI_DOCS</td>
                                <td className="px-3 py-2 text-gray-500">false</td>
                                <td className="px-3 py-2 text-gray-700">Enable OpenAPI JSON + UI routes in api-server</td>
                            </tr>
                            <tr>
                                <td className="px-3 py-2 font-mono text-blue-600">OPENAPI_DOCS_PATH</td>
                                <td className="px-3 py-2 text-gray-500">/openapi</td>
                                <td className="px-3 py-2 text-gray-700">Mount path for OpenAPI HTML docs UI (Redoc)</td>
                            </tr>
                            <tr>
                                <td className="px-3 py-2 font-mono text-blue-600">OPENAPI_JSON_PATH</td>
                                <td className="px-3 py-2 text-gray-500">/openapi.json</td>
                                <td className="px-3 py-2 text-gray-700">Route path for generated OpenAPI JSON specification</td>
                            </tr>
                            <tr>
                                <td className="px-3 py-2 font-mono text-blue-600">DATATABLE_UNKNOWN_FILTER_MODE</td>
                                <td className="px-3 py-2 text-gray-500">ignore</td>
                                <td className="px-3 py-2 text-gray-700">Default unknown f-* behavior (ignore/warn/error)</td>
                            </tr>

                            {/* Database */}
                            <tr>
                                <td className="px-3 py-2 font-mono text-blue-600">DATABASE_URL</td>
                                <td className="px-3 py-2 text-red-500 font-bold">Required</td>
                                <td className="px-3 py-2 text-gray-700">Postgres connection string</td>
                            </tr>
                            <tr>
                                <td className="px-3 py-2 font-mono text-blue-600">REALTIME_ENABLED</td>
                                <td className="px-3 py-2 text-gray-500">true</td>
                                <td className="px-3 py-2 text-gray-700">Global realtime websocket toggle</td>
                            </tr>
                            <tr>
                                <td className="px-3 py-2 font-mono text-blue-600">REALTIME_REQUIRE_AUTH</td>
                                <td className="px-3 py-2 text-gray-500">true</td>
                                <td className="px-3 py-2 text-gray-700">Require auth payload handshake before subscriptions</td>
                            </tr>
                            <tr>
                                <td className="px-3 py-2 font-mono text-blue-600">REALTIME_DELIVERY_MODE</td>
                                <td className="px-3 py-2 text-gray-500">at_most_once</td>
                                <td className="px-3 py-2 text-gray-700">Delivery mode (`at_most_once` or stream-backed `durable` replay)</td>
                            </tr>
                            <tr>
                                <td className="px-3 py-2 font-mono text-blue-600">REALTIME_REPLAY_LIMIT_MAX</td>
                                <td className="px-3 py-2 text-gray-500">1000</td>
                                <td className="px-3 py-2 text-gray-700">Upper clamp for replay batch requested by clients</td>
                            </tr>
                            <tr>
                                <td className="px-3 py-2 font-mono text-blue-600">REALTIME_MAX_MESSAGE_BYTES</td>
                                <td className="px-3 py-2 text-gray-500">65536</td>
                                <td className="px-3 py-2 text-gray-700">Max text message payload size accepted per connection</td>
                            </tr>
                            <tr>
                                <td className="px-3 py-2 font-mono text-blue-600">REALTIME_MAX_FRAME_BYTES</td>
                                <td className="px-3 py-2 text-gray-500">65536</td>
                                <td className="px-3 py-2 text-gray-700">WebSocket frame payload upper bound for ingress parsing</td>
                            </tr>
                            <tr>
                                <td className="px-3 py-2 font-mono text-blue-600">REALTIME_MAX_MESSAGES_PER_SEC</td>
                                <td className="px-3 py-2 text-gray-500">150</td>
                                <td className="px-3 py-2 text-gray-700">Inbound message rate limit per websocket connection</td>
                            </tr>
                            <tr>
                                <td className="px-3 py-2 font-mono text-blue-600">REALTIME_STREAM_RETENTION_SECS</td>
                                <td className="px-3 py-2 text-gray-500">0</td>
                                <td className="px-3 py-2 text-gray-700">Durable stream retention window in seconds (0 disables time-window trim)</td>
                            </tr>
                            <tr>
                                <td className="px-3 py-2 font-mono text-blue-600">REALTIME_REPLAY_GAP_ALERT_THRESHOLD</td>
                                <td className="px-3 py-2 text-gray-500">100</td>
                                <td className="px-3 py-2 text-gray-700">Replay gap count threshold to trigger SLO alert per window</td>
                            </tr>
                            <tr>
                                <td className="px-3 py-2 font-mono text-blue-600">REALTIME_REPLAY_GAP_ALERT_WINDOW_SECS</td>
                                <td className="px-3 py-2 text-gray-500">60</td>
                                <td className="px-3 py-2 text-gray-700">Replay gap SLO evaluation window size in seconds</td>
                            </tr>
                            <tr>
                                <td className="px-3 py-2 font-mono text-blue-600">REALTIME_SEND_QUEUE_CAPACITY</td>
                                <td className="px-3 py-2 text-gray-500">1024</td>
                                <td className="px-3 py-2 text-gray-700">Bounded outbound queue size per connection before slow-consumer disconnect</td>
                            </tr>
                            <tr>
                                <td className="px-3 py-2 font-mono text-blue-600">REALTIME_CHECKPOINT_ENABLED</td>
                                <td className="px-3 py-2 text-gray-500">false</td>
                                <td className="px-3 py-2 text-gray-700">Enable Redis-backed per-subject replay checkpoint updates via `ack`</td>
                            </tr>
                            <tr>
                                <td className="px-3 py-2 font-mono text-blue-600">REALTIME_CHECKPOINT_TTL_SECS</td>
                                <td className="px-3 py-2 text-gray-500">2592000</td>
                                <td className="px-3 py-2 text-gray-700">Checkpoint expiration in seconds when checkpoint mode is enabled</td>
                            </tr>

                            {/* Worker */}
                            <tr>
                                <td className="px-3 py-2 font-mono text-blue-600">RUN_WORKER</td>
                                <td className="px-3 py-2 text-gray-500">false</td>
                                <td className="px-3 py-2 text-gray-700">Enable background job processing in this instance</td>
                            </tr>

                            {/* Middleware */}
                            <tr>
                                <td className="px-3 py-2 font-mono text-blue-600">MW_BODY_LIMIT_MB</td>
                                <td className="px-3 py-2 text-gray-500">10</td>
                                <td className="px-3 py-2 text-gray-700">Max upload size in Megabytes</td>
                            </tr>
                            <tr>
                                <td className="px-3 py-2 font-mono text-blue-600">MW_RATE_LIMIT_PER_SEC</td>
                                <td className="px-3 py-2 text-gray-500">2</td>
                                <td className="px-3 py-2 text-gray-700">Requests per second per IP</td>
                            </tr>

                            {/* Redis */}
                            <tr>
                                <td className="px-3 py-2 font-mono text-blue-600">REDIS_URL</td>
                                <td className="px-3 py-2 text-gray-500">None</td>
                                <td className="px-3 py-2 text-gray-700">Full connection string (overrides separate host/port)</td>
                            </tr>
                            <tr>
                                <td className="px-3 py-2 font-mono text-blue-600">REDIS_CACHE_PREFIX</td>
                                <td className="px-3 py-2 text-gray-500">(auto)</td>
                                <td className="px-3 py-2 text-gray-700">
                                    Leave empty to auto-derive <code>{'{APP_NAME}_{APP_ENV}'}</code>, or set explicit prefix to isolate keys.
                                </td>
                            </tr>
                            
                             {/* S3 */}
                            <tr>
                                <td className="px-3 py-2 font-mono text-blue-600">S3_BUCKET</td>
                                <td className="px-3 py-2 text-gray-500">None</td>
                                <td className="px-3 py-2 text-gray-700">Object storage bucket name</td>
                            </tr>
                            <tr>
                                <td className="px-3 py-2 font-mono text-blue-600">S3_URL</td>
                                <td className="px-3 py-2 text-gray-500">None</td>
                                <td className="px-3 py-2 text-gray-700">Public CDN URL for uploaded assets</td>
                            </tr>

                            {/* Logging */}
                             <tr>
                                <td className="px-3 py-2 font-mono text-blue-600">HTTP_LOG_WEBHOOK_ENABLED</td>
                                <td className="px-3 py-2 text-gray-500">false</td>
                                <td className="px-3 py-2 text-gray-700">Log incoming webhooks to DB</td>
                            </tr>
                             <tr>
                                <td className="px-3 py-2 font-mono text-blue-600">HTTP_LOG_WEBHOOK_PATHS</td>
                                <td className="px-3 py-2 text-gray-500">/wh/,/webhook/</td>
                                <td className="px-3 py-2 text-gray-700">Paths to log (prefix match)</td>
                            </tr>
                             <tr>
                                <td className="px-3 py-2 font-mono text-blue-600">HTTP_LOG_RETENTION_DAYS</td>
                                <td className="px-3 py-2 text-gray-500">7</td>
                                <td className="px-3 py-2 text-gray-700">Days to keep http logs</td>
                            </tr>
                        </tbody>
                    </table>
                </div>

                {/* TOML Config */}
                <h2 className="mt-10">App Configuration (TOML)</h2>
                <p>
                    Structured application config is stored in{' '}
                    <code>app/configs.toml</code>.
                </p>

                <h3>Complete configs.toml Reference</h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-toml">{`# ----------------------------
# Internationalization (I18n)
# ----------------------------
[languages]
default = "en"                  # Default locale
supported = ["en", "zh"]        # Available languages
timezone = "+00:00"             # Optional fallback timezone metadata

# ----------------------------
# Authentication Guards
# ----------------------------
[auth]
default = "web"                 # Default guard name (example only)

[auth.guards.web]
provider = "account"            # Provider key (app-defined)
ttl_min = 15                    # Access token lifetime (minutes)
refresh_ttl_days = 7            # App-level refresh policy (days)

[auth.guards.partner_api]
provider = "partner_account"
ttl_min = 60
refresh_ttl_days = 30

# ----------------------------
# Realtime Channel Policies
# ----------------------------
[realtime.channels.public_feed]
enabled = true
presence_enabled = false

[realtime.channels.admin_notifications]
enabled = true
guard = "admin"
presence_enabled = true
max_message_bytes = 32768
max_frame_bytes = 32768

# ----------------------------
# CORS (mirrors Laravel config/cors.php)
# ----------------------------
[cors]
allowed_origins = ["*"]         # ["*"] or explicit origins
allowed_methods = ["*"]         # ["*"] or ["GET","POST",...]
allowed_headers = ["*"]         # ["*"] or explicit headers
exposed_headers = []
max_age = 0                     # Preflight cache (seconds)
supports_credentials = false    # true requires explicit origins`}</code>
                </pre>
                <p className="text-sm text-gray-600 mt-3">
                    Guard names/providers are application choices. The framework does not define
                    fixed predefined roles.
                </p>
                <p className="text-sm text-gray-600">
                    `user` / `admin` here are naming examples only.
                </p>
                <p className="text-sm text-gray-600">
                    Note: refresh token endpoints/rotation are application workflows; the framework
                    only provides guard config + token primitives.
                </p>
                <p className="text-sm text-gray-600">
                    Realtime channel config is declarative in TOML, while realtime runtime knobs
                    are environment variables.
                </p>

                {/* Usage Example */}
                <h3 className="mt-8">Usage in Code</h3>
                <p>
                    Access configuration values via the <code>Settings</code> struct:
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`use core_config::Settings;

let settings = Settings::load()?;

// Environment variables
println!("App: {}", settings.app.name);
println!("Port: {}", settings.server.port);

// TOML config
println!("Default locale: {}", settings.i18n.default_locale);`}</code>
                </pre>

                <div className="bg-blue-50 border-l-4 border-blue-400 p-4 mt-6">
                    <p className="text-sm text-blue-700">
                        <strong>Pro Tip:</strong> Environment variables always take precedence
                        over TOML values.
                    </p>
                </div>
            </div>
        </div>
    )
}
