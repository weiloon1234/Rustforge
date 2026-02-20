use time::Duration;
use tower_cookies::{cookie::SameSite, Cookie, Cookies};

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
