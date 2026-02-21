use time::Duration;
use tower_cookies::{cookie::SameSite, Cookie, Cookies};

fn sanitize_guard_for_cookie(guard: &str) -> String {
    let value = guard
        .trim()
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect::<String>();

    if value.is_empty() {
        "default".to_string()
    } else {
        value
    }
}

pub fn guard_refresh_cookie_name(guard: &str) -> String {
    format!("rf_{}_refresh_token", sanitize_guard_for_cookie(guard))
}

/// Sets a highly secure authentication cookie (access_token).
/// - HttpOnly: YES (Prevent XSS)
/// - Secure: YES (Prevent Sniffing)
/// - SameSite: Lax (Prevent CSRF)
pub fn set_auth(cookies: &Cookies, token: &str, ttl: Duration) {
    let mut c = Cookie::new("access_token", token.to_string());
    c.set_http_only(true);
    c.set_secure(true);
    c.set_same_site(SameSite::Lax);
    c.set_path("/");

    // time::Duration -> tower_cookies::cookie::time::Duration (via i64 seconds)
    let secs = ttl.whole_seconds();
    c.set_max_age(tower_cookies::cookie::time::Duration::seconds(secs));

    cookies.add(c);
}

/// Sets a standard secure cookie (not strictly for auth, but still server-side only).
/// Useful for server-side preferences or flash messages.
pub fn set_standard(cookies: &Cookies, name: &str, val: &str, ttl: Duration) {
    let mut c = Cookie::new(name.to_string(), val.to_string());
    c.set_http_only(true);
    c.set_secure(true);
    c.set_same_site(SameSite::Lax);
    c.set_path("/");

    let secs = ttl.whole_seconds();
    c.set_max_age(tower_cookies::cookie::time::Duration::seconds(secs));

    cookies.add(c);
}

/// Sets a public cookie that JavaScript CAN read.
/// Use this for UI preferences (theme, language) that the frontend needs to access.
pub fn set_public(cookies: &Cookies, name: &str, val: &str, ttl: Duration) {
    let mut c = Cookie::new(name.to_string(), val.to_string());
    c.set_http_only(false); // <--- JS accessible
    c.set_secure(true);
    c.set_same_site(SameSite::Lax);
    c.set_path("/");

    let secs = ttl.whole_seconds();
    c.set_max_age(tower_cookies::cookie::time::Duration::seconds(secs));

    cookies.add(c);
}

/// Removes a cookie by name
pub fn remove(cookies: &Cookies, name: &str) {
    let mut c = Cookie::new(name.to_string(), "");
    c.set_path("/");
    cookies.remove(c);
}

pub fn set_guard_refresh(
    cookies: &Cookies,
    guard: &str,
    refresh_token: &str,
    ttl: Duration,
    path: &str,
) {
    let mut cookie = Cookie::new(guard_refresh_cookie_name(guard), refresh_token.to_string());
    cookie.set_http_only(true);
    cookie.set_secure(true);
    cookie.set_same_site(SameSite::Lax);
    cookie.set_path(path.to_string());
    cookie.set_max_age(tower_cookies::cookie::time::Duration::seconds(
        ttl.whole_seconds(),
    ));
    cookies.add(cookie);
}

pub fn remove_guard_refresh(cookies: &Cookies, guard: &str, path: &str) {
    let mut cookie = Cookie::new(guard_refresh_cookie_name(guard), "");
    cookie.set_path(path.to_string());
    cookies.remove(cookie);
}
