pub use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initialize logging. Returns a WorkerGuard that must be held directly by `main`.
/// When the guard is dropped (on shutdown), logs are flushed.
pub fn init() -> WorkerGuard {
    // 1. Env Filter (controls verbosity via RUST_LOG)
    // Default to info if unset.
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    // 2. File Appender (Daily rolling in ./logs)
    let file_appender = tracing_appender::rolling::daily("./logs", "app.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    // 3. Layers
    // Stdout layer (human readable)
    let stdout_layer = fmt::layer().with_target(true).with_level(true).compact();

    // File layer (JSON might be better for tools, or plain text)
    // Using plain text for now as requested "simple complete work".
    // We suppress ANSI colors for file logs.
    let file_layer = fmt::layer()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_target(true)
        .with_level(true)
        .compact();

    // 4. Register
    tracing_subscriber::registry()
        .with(env_filter)
        .with(stdout_layer)
        .with(file_layer)
        .init();

    // 5. Global Panic Hook
    // Captures panics outside of Axum requests (e.g. background tasks, startup)
    // and logs them to the file/stdout.
    let prev_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let payload = panic_info.payload();
        let details = if let Some(s) = payload.downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = payload.downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown panic".to_string()
        };

        let location = if let Some(location) = panic_info.location() {
            format!(
                "{}:{}:{}",
                location.file(),
                location.line(),
                location.column()
            )
        } else {
            "unknown location".to_string()
        };

        tracing::error!(target: "panic", "Application panic at {}: {}", location, details);

        // Continue with default hook (prints to stderr and likely aborts)
        prev_hook(panic_info);
    }));

    guard
}
