use crate::config::I18nSettings;

pub mod config;
pub mod locale;
pub mod middleware;
pub mod translator;

pub use locale::{
    current_locale, match_supported_locale, scope_locale, set_default_locale,
    set_supported_locales, supported_locales,
};
pub use translator::{t, t_args, t_for_locale};

pub fn init(settings: &I18nSettings) {
    set_default_locale(settings.default_locale);
    set_supported_locales(settings.supported_locales);
    translator::warmup();

    // Set process-wide timezone (if supported by system/lib)
    // Note: Rust's `time` crate generally relies on system or env, but we can set `TZ` env var here.
    std::env::set_var("TZ", &settings.default_timezone_str);

    // Check if effective offset matches (best effort warning)
    if let Ok(now) = time::OffsetDateTime::now_local() {
        if now.offset() != settings.default_timezone {
            tracing::warn!(
                "TZ={} set, but local offset is {}; expected {}",
                settings.default_timezone_str,
                now.offset(),
                settings.default_timezone
            );
        }
    } else {
        tracing::warn!(
            "Failed to read local time after setting TZ={}",
            settings.default_timezone_str
        );
    }
}
