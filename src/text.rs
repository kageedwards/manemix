use chrono::{NaiveDateTime, Utc};
use regex::Regex;
use std::sync::LazyLock;

/// Format a Postgres timestamp string into a display string.
pub fn format_time(date: &str, fmt: &str) -> String {
    NaiveDateTime::parse_from_str(date, "%Y-%m-%d %H:%M:%S")
        .map(|dt| dt.format(fmt).to_string())
        .unwrap_or_default()
}

/// "3 hours ago", "2 days ago", etc.
pub fn fuzzy_time(date: &str) -> String {
    let dt = match NaiveDateTime::parse_from_str(date, "%Y-%m-%d %H:%M:%S") {
        Ok(dt) => dt,
        Err(_) => return String::new(),
    };
    let secs = (Utc::now().naive_utc() - dt).num_seconds();
    match secs {
        s if s < 0 => "In the future".into(),
        s if s < 5 => "A few seconds ago".into(),
        s if s < 60 => "Less than a minute ago".into(),
        s if s < 120 => "A minute ago".into(),
        s if s < 10800 => {
            let mins = (s / 60) % 60;
            let hours = s / 3600;
            let h = if hours == 0 {
                String::new()
            } else if hours == 1 {
                "An hour, ".into()
            } else {
                format!("{hours} hours, ")
            };
            let plural = if mins <= 1 { "" } else { "s" };
            format!("{h}{mins} minute{plural} ago")
        }
        s if s < 172800 => format!("{} hours ago", s / 3600),
        s if s < 5184000 => format!("{} days ago", s / 86400),
        s if s < 62208000 => format!("{} months ago", s / 2592000),
        s => format!("{} years ago", s / 31104000),
    }
}

/// BBCode-lite formatter: [b], [i], [u], newlines, auto-link URLs.
/// Replaces the ctemplate x-format modifier.
pub fn format_bbcode(input: &str) -> String {
    static RE_B: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\[b\](.*?)\[/b\]").unwrap());
    static RE_I: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\[i\](.*?)\[/i\]").unwrap());
    static RE_U: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\[u\](.*?)\[/u\]").unwrap());
    static RE_URL: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(https?|ftp)://[\w\-_]+(\.[\w\-_]+)([\w\-.,@?!^=%&:/~+#]*[\w\-@?!^=%&/~+#])?")
            .unwrap()
    });

    let escaped = html_escape(input);
    let s = escaped.replace('\n', "<br />");
    let s = RE_B.replace_all(&s, "<b>$1</b>");
    let s = RE_I.replace_all(&s, "<i>$1</i>");
    let s = RE_U.replace_all(&s, "<u>$1</u>");
    let s = RE_URL.replace_all(&s, r#"<a href="$0" rel="nofollow">$0</a>"#);
    s.into_owned()
}

/// HTML-escape a string (replaces ctemplate's pre_escape).
pub fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Encode each character as an HTML numeric entity (email obfuscation).
/// Replaces the ctemplate x-email modifier.
pub fn email_escape(s: &str) -> String {
    s.bytes().map(|b| format!("&#{};", b)).collect()
}

/// Sanitize a string for use as an IRC nickname.
/// Replaces the ctemplate x-irc modifier.
pub fn irc_escape(s: &str) -> String {
    const ACCEPTED: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789|-[]{}\\`^_";
    s.chars()
        .take(20)
        .filter(|c| c.is_ascii())
        .map(|c| if ACCEPTED.contains(c) { c } else { '_' })
        .collect()
}

/// MD5 hex digest (for Gravatar).
pub fn md5_hex(s: &str) -> String {
    use md5::{Md5, Digest};
    hex::encode(Md5::digest(s.as_bytes()))
}

/// Basic email validation (matches the original's check).
pub fn valid_email(s: &str) -> bool {
    s.contains('@') && s.contains('.') && s.len() > 5
}
