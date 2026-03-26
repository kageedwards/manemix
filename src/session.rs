/// Axum extractor for the current session (from cookie).
/// Use `OptionalSession` when the route works for both logged-in and anonymous users.
/// Use `RequiredSession` when the route requires authentication.

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::response::{IntoResponse, Redirect, Response};

use crate::AppState;
use crate::models::session::Session;

/// Optional session — always succeeds extraction, session may be None.
#[derive(Debug, Clone)]
pub struct OptionalSession(pub Option<Session>, pub String);

impl<S> FromRequestParts<S> for OptionalSession
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);
        let sid = extract_sid_from_cookie(parts);
        let cookie_theme = extract_theme_from_cookie(parts);
        if let Some(sid) = sid {
            let remote = extract_remote_addr(parts);
            let session = Session::from_sid(&app_state.db, &sid, &remote).await;
            let theme = session.as_ref().map(|s| s.theme.clone()).unwrap_or(cookie_theme);
            Ok(OptionalSession(session, theme))
        } else {
            Ok(OptionalSession(None, cookie_theme))
        }
    }
}

/// Required session — rejects with redirect to /login if not authenticated.
#[derive(Debug, Clone)]
pub struct RequiredSession(pub Session);

impl<S> FromRequestParts<S> for RequiredSession
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);
        let sid = extract_sid_from_cookie(parts);
        if let Some(sid) = sid {
            let remote = extract_remote_addr(parts);
            if let Some(session) = Session::from_sid(&app_state.db, &sid, &remote).await {
                return Ok(RequiredSession(session));
            }
        }
        // Return JSON 401 for API requests, redirect for HTML requests
        let wants_json = parts.headers.get("accept")
            .and_then(|v| v.to_str().ok())
            .map(|v| v.contains("application/json"))
            .unwrap_or(false);
        if wants_json {
            Err((
                axum::http::StatusCode::UNAUTHORIZED,
                axum::Json(serde_json::json!({ "error": "unauthorized", "message": "Not logged in" })),
            ).into_response())
        } else {
            let path = parts.uri.path();
            Err(Redirect::to(&format!("/login?redirect={path}")).into_response())
        }
    }
}

/// Trait to extract AppState from the router state.
pub trait FromRef<T> {
    fn from_ref(input: &T) -> Self;
}

impl FromRef<AppState> for AppState {
    fn from_ref(input: &AppState) -> Self {
        input.clone()
    }
}

fn extract_sid_from_cookie(parts: &Parts) -> Option<String> {
    let cookie_header = parts.headers.get("cookie")?.to_str().ok()?;
    for pair in cookie_header.split(';') {
        let pair = pair.trim();
        if let Some(val) = pair.strip_prefix("sid=") {
            let val = val.trim();
            if !val.is_empty() {
                return Some(val.to_string());
            }
        }
    }
    None
}

fn extract_theme_from_cookie(parts: &Parts) -> String {
    let cookie_header = parts.headers.get("cookie")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    for pair in cookie_header.split(';') {
        let pair = pair.trim();
        if let Some(val) = pair.strip_prefix("theme=") {
            let val = val.trim();
            if matches!(val, "light" | "dark" | "auto") {
                return val.to_string();
            }
        }
    }
    "auto".to_string()
}

fn extract_remote_addr(parts: &Parts) -> String {
    // Try X-Forwarded-For first (behind nginx), then fall back to peer addr
    parts.headers.get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or("").trim().to_string())
        .unwrap_or_else(|| {
            parts.extensions.get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
                .map(|ci| ci.0.ip().to_string())
                .unwrap_or_else(|| "127.0.0.1".into())
        })
}
