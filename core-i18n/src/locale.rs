//! Locale storage using task-local variables.

use std::sync::OnceLock;
use tokio::task_local;

task_local! {
    static CURRENT_LOCALE: &'static str;
}

/// Default locale, set once at startup.
static DEFAULT_LOCALE: OnceLock<&'static str> = OnceLock::new();
/// Supported locales, set once at startup.
static SUPPORTED_LOCALES: OnceLock<&'static [&'static str]> = OnceLock::new();

/// Set the default locale (call once at app startup).
pub fn set_default_locale(locale: &'static str) {
    let _ = DEFAULT_LOCALE.set(locale);
}

/// Set supported locales (call once at app startup).
pub fn set_supported_locales(locales: &'static [&'static str]) {
    let _ = SUPPORTED_LOCALES.set(locales);
}

/// Run an async block with a specific locale.
pub async fn scope_locale<F, Fut, T>(locale: &'static str, f: F) -> T
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = T>,
{
    CURRENT_LOCALE.scope(locale, f()).await
}

/// Get the current locale (from task-local or default).
pub fn current_locale() -> &'static str {
    CURRENT_LOCALE
        .try_with(|l| *l)
        .unwrap_or_else(|_| DEFAULT_LOCALE.get().copied().unwrap_or("en"))
}

/// Get all supported locales.
pub fn supported_locales() -> &'static [&'static str] {
    SUPPORTED_LOCALES.get().copied().unwrap_or(&["en"])
}

/// Match a raw locale value to one configured supported locale.
///
/// Accepts exact locale (e.g. `zh`) or language tag (`zh-CN` -> `zh`).
pub fn match_supported_locale(raw: &str) -> Option<&'static str> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    for &locale in supported_locales() {
        if locale.eq_ignore_ascii_case(trimmed) {
            return Some(locale);
        }
    }

    let base = trimmed.split(['-', '_']).next().unwrap_or_default().trim();
    if base.is_empty() {
        return None;
    }

    supported_locales()
        .iter()
        .copied()
        .find(|locale| locale.eq_ignore_ascii_case(base))
}
