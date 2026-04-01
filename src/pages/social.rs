use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Json, Redirect, Response};
use bytes::Bytes;
use serde::Deserialize;

use crate::AppState;
use crate::models::event;
use crate::models::session::Session;
use crate::models::track::Track;
use crate::models::user::User;
use crate::session::RequiredSession;

// ---------------------------------------------------------------------------
// Helpers for dual JSON / form handling
// ---------------------------------------------------------------------------

/// Returns `true` when the Content-Type header indicates JSON.
fn is_json_content(headers: &HeaderMap) -> bool {
    headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.contains("application/json"))
        .unwrap_or(false)
}

/// Returns `true` when the Accept header indicates JSON.
fn wants_json(headers: &HeaderMap) -> bool {
    headers
        .get("accept")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.contains("application/json"))
        .unwrap_or(false)
}

/// Return either a JSON `{ "ok": true }` or a redirect, depending on Accept header.
fn ok_or_redirect(headers: &HeaderMap, redirect_url: &str) -> Response {
    if wants_json(headers) {
        Json(serde_json::json!({ "ok": true })).into_response()
    } else {
        Redirect::to(redirect_url).into_response()
    }
}

/// Return either a JSON error or a redirect, depending on Accept header.
fn err_or_redirect(headers: &HeaderMap, redirect_url: &str) -> Response {
    if wants_json(headers) {
        (
            axum::http::StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "bad_request", "message": "Invalid request" })),
        )
            .into_response()
    } else {
        Redirect::to(redirect_url).into_response()
    }
}

// ---------------------------------------------------------------------------
// Nonce form (legacy form-encoded)
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct NonceForm {
    nonce: Option<String>,
}

// ---------------------------------------------------------------------------
// Favorite / Unfavorite track
// ---------------------------------------------------------------------------

pub async fn favorite_track(
    State(state): State<AppState>,
    RequiredSession(sess): RequiredSession,
    Path(tid): Path<i32>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let t = match Track::by_id(&state.db, tid).await {
        Some(t) => t,
        None => return err_or_redirect(&headers, "/"),
    };

    let is_json = is_json_content(&headers);

    // For JSON requests, skip nonce check (session cookie is CSRF protection).
    // For form requests, validate nonce as before.
    let nonce_ok = if is_json {
        true
    } else {
        let form: NonceForm = serde_urlencoded::from_bytes(&body).unwrap_or(NonceForm { nonce: None });
        form.nonce.as_deref() == Some(&sess.nonce)
    };

    if nonce_ok {
        Session::new_nonce(&state.db, &sess.sid).await;
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM favorites WHERE type = 'track' AND ref = $1 AND user_id = $2)",
        )
        .bind(tid)
        .bind(sess.user.id)
        .fetch_one(&state.db)
        .await
        .unwrap_or(true);

        if !exists {
            let _ = sqlx::query(
                "INSERT INTO favorites (user_id, type, ref) VALUES ($1, 'track'::favorite_type, $2)",
            )
            .bind(sess.user.id)
            .bind(tid)
            .execute(&state.db)
            .await;

            event::push_event(&state.db, "favorite", &sess.user, &t.artist, Some(&t)).await;
        }
    }
    ok_or_redirect(&headers, &format!("/track/{tid}"))
}

pub async fn unfavorite_track(
    State(state): State<AppState>,
    RequiredSession(sess): RequiredSession,
    Path(tid): Path<i32>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let is_json = is_json_content(&headers);

    let nonce_ok = if is_json {
        true
    } else {
        let form: NonceForm = serde_urlencoded::from_bytes(&body).unwrap_or(NonceForm { nonce: None });
        form.nonce.as_deref() == Some(&sess.nonce)
    };

    if nonce_ok {
        Session::new_nonce(&state.db, &sess.sid).await;
        let _ = sqlx::query(
            "DELETE FROM favorites WHERE type = 'track' AND ref = $1 AND user_id = $2",
        )
        .bind(tid)
        .bind(sess.user.id)
        .execute(&state.db)
        .await;
        let _ = sqlx::query(
            "DELETE FROM events WHERE type = 'favorite' AND source_id = $1 AND track_id = $2",
        )
        .bind(sess.user.id)
        .bind(tid)
        .execute(&state.db)
        .await;
    }
    ok_or_redirect(&headers, &format!("/track/{tid}"))
}

// ---------------------------------------------------------------------------
// Follow / Unfollow artist
// ---------------------------------------------------------------------------

pub async fn follow_user(
    State(state): State<AppState>,
    RequiredSession(sess): RequiredSession,
    Path(uid): Path<i32>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let u = match User::by_id(&state.db, uid).await {
        Some(u) => u,
        None => return err_or_redirect(&headers, "/"),
    };
    if uid == sess.user.id {
        return ok_or_redirect(&headers, &format!("/user/{uid}"));
    }

    let is_json = is_json_content(&headers);

    let nonce_ok = if is_json {
        true
    } else {
        let form: NonceForm = serde_urlencoded::from_bytes(&body).unwrap_or(NonceForm { nonce: None });
        form.nonce.as_deref() == Some(&sess.nonce)
    };

    if nonce_ok {
        Session::new_nonce(&state.db, &sess.sid).await;
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM favorites WHERE type = 'artist' AND ref = $1 AND user_id = $2)",
        )
        .bind(uid)
        .bind(sess.user.id)
        .fetch_one(&state.db)
        .await
        .unwrap_or(true);

        if !exists {
            let _ = sqlx::query(
                "INSERT INTO favorites (user_id, type, ref) VALUES ($1, 'artist'::favorite_type, $2)",
            )
            .bind(sess.user.id)
            .bind(uid)
            .execute(&state.db)
            .await;

            event::push_event(&state.db, "follow", &sess.user, &u, None).await;
        }
    }
    ok_or_redirect(&headers, &format!("/user/{uid}"))
}

pub async fn unfollow_user(
    State(state): State<AppState>,
    RequiredSession(sess): RequiredSession,
    Path(uid): Path<i32>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let is_json = is_json_content(&headers);

    let nonce_ok = if is_json {
        true
    } else {
        let form: NonceForm = serde_urlencoded::from_bytes(&body).unwrap_or(NonceForm { nonce: None });
        form.nonce.as_deref() == Some(&sess.nonce)
    };

    if nonce_ok {
        Session::new_nonce(&state.db, &sess.sid).await;
        let _ = sqlx::query(
            "DELETE FROM favorites WHERE type = 'artist' AND ref = $1 AND user_id = $2",
        )
        .bind(uid)
        .bind(sess.user.id)
        .execute(&state.db)
        .await;
        let _ = sqlx::query(
            "DELETE FROM events WHERE type = 'follow' AND source_id = $1 AND target_id = $2",
        )
        .bind(sess.user.id)
        .bind(uid)
        .execute(&state.db)
        .await;
    }
    ok_or_redirect(&headers, &format!("/user/{uid}"))
}

// ---------------------------------------------------------------------------
// Comments
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct CommentForm {
    nonce: Option<String>,
    name: Option<String>,
    msg: Option<String>,
    url: Option<String>, // honeypot
}

#[derive(Deserialize)]
pub struct CommentJson {
    msg: Option<String>,
    name: Option<String>,
}

pub async fn comment_track(
    State(state): State<AppState>,
    crate::session::OptionalSession(sess, _theme): crate::session::OptionalSession,
    Path(tid): Path<i32>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let t = match Track::by_id(&state.db, tid).await {
        Some(t) => t,
        None => return err_or_redirect(&headers, "/"),
    };

    let is_json = is_json_content(&headers);

    let (msg, source_name, is_bot) = if is_json {
        let parsed: CommentJson = match serde_json::from_slice(&body) {
            Ok(p) => p,
            Err(e) => {
                tracing::warn!("comment_track: JSON parse error: {e}, body len={}", body.len());
                return (
                    axum::http::StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({ "error": "parse_error", "message": format!("Invalid JSON: {e}") })),
                ).into_response();
            }
        };
        (parsed.msg.unwrap_or_default(), parsed.name, false)
    } else {
        let form: CommentForm = serde_urlencoded::from_bytes(&body).unwrap_or(CommentForm {
            nonce: None,
            name: None,
            msg: None,
            url: None,
        });

        // CSRF check for logged-in users (form path only)
        if let Some(ref s) = sess {
            if form.nonce.as_deref() != Some(&s.nonce) {
                return err_or_redirect(&headers, &format!("/track/{tid}"));
            }
            Session::new_nonce(&state.db, &s.sid).await;
        }

        let bot = form.url.map(|u| !u.is_empty()).unwrap_or(false);
        (form.msg.unwrap_or_default(), form.name, bot)
    };

    // Spam detection
    let is_spam = msg.contains("<a") || msg.contains("[url");

    if msg.is_empty() || is_bot || is_spam {
        if is_json {
            let reason = if msg.is_empty() { "Message cannot be empty" } else { "Message rejected" };
            return (
                axum::http::StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "bad_request", "message": reason })),
            ).into_response();
        }
        return err_or_redirect(&headers, &format!("/track/{tid}"));
    }

    // For JSON path with logged-in user, rotate nonce
    if is_json {
        if let Some(ref s) = sess {
            Session::new_nonce(&state.db, &s.sid).await;
        }
    }

    let source = sess
        .as_ref()
        .map(|s| s.user.clone())
        .unwrap_or_else(|| User {
            id: 0,
            name: source_name.unwrap_or_default(),
        });

    event::push_track_comment(&state.db, &source, &t.artist, &t, &msg).await;

    // Email notification
    if let Some(acct) = crate::models::account::Account::by_id(&state.db, t.artist.id).await {
        if acct.notify && acct.user.id != source.id {
            let mail_body = format!(
                "{} posted a comment on {}:\n\n{}\n\n\
                 You can view it here: {}/track/{}\n\
                 Disable notifications: {}/account",
                source.name, t.title, msg, state.base_url, tid, state.base_url
            );
            crate::models::mail::send(&acct.email, "Manemix comment notification", &mail_body);
        }
    }

    ok_or_redirect(&headers, &format!("/track/{tid}"))
}

pub async fn comment_user(
    State(state): State<AppState>,
    crate::session::OptionalSession(sess, _theme): crate::session::OptionalSession,
    Path(uid): Path<i32>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let u = match User::by_id(&state.db, uid).await {
        Some(u) => u,
        None => return err_or_redirect(&headers, "/"),
    };

    let is_json = is_json_content(&headers);

    let (msg, source_name, is_bot) = if is_json {
        let parsed: CommentJson = match serde_json::from_slice(&body) {
            Ok(p) => p,
            Err(e) => {
                tracing::warn!("comment_user: JSON parse error: {e}, body len={}", body.len());
                return (
                    axum::http::StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({ "error": "parse_error", "message": format!("Invalid JSON: {e}") })),
                ).into_response();
            }
        };
        (parsed.msg.unwrap_or_default(), parsed.name, false)
    } else {
        let form: CommentForm = serde_urlencoded::from_bytes(&body).unwrap_or(CommentForm {
            nonce: None,
            name: None,
            msg: None,
            url: None,
        });

        if let Some(ref s) = sess {
            if form.nonce.as_deref() != Some(&s.nonce) {
                return err_or_redirect(&headers, &format!("/user/{uid}"));
            }
            Session::new_nonce(&state.db, &s.sid).await;
        }

        let bot = form.url.map(|u| !u.is_empty()).unwrap_or(false);
        (form.msg.unwrap_or_default(), form.name, bot)
    };

    let is_spam = msg.contains("<a") || msg.contains("[url");

    if msg.is_empty() || is_bot || is_spam {
        if is_json {
            let reason = if msg.is_empty() { "Message cannot be empty" } else { "Message rejected" };
            return (
                axum::http::StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "bad_request", "message": reason })),
            ).into_response();
        }
        return err_or_redirect(&headers, &format!("/user/{uid}"));
    }

    // For JSON path with logged-in user, rotate nonce
    if is_json {
        if let Some(ref s) = sess {
            Session::new_nonce(&state.db, &s.sid).await;
        }
    }

    let source = sess
        .as_ref()
        .map(|s| s.user.clone())
        .unwrap_or_else(|| User {
            id: 0,
            name: source_name.unwrap_or_default(),
        });

    event::push_user_comment(&state.db, &source, &u, &msg).await;

    // Email notification
    if let Some(acct) = crate::models::account::Account::by_id(&state.db, uid).await {
        if acct.notify && acct.user.id != source.id {
            let mail_body = format!(
                "{} posted a comment on your user page:\n\n{}\n\n\
                 You can view it here: {}/user/{}\n\
                 Disable notifications: {}/account",
                source.name, msg, state.base_url, uid, state.base_url
            );
            crate::models::mail::send(&acct.email, "Manemix comment notification", &mail_body);
        }
    }

    ok_or_redirect(&headers, &format!("/user/{uid}"))
}

// ---------------------------------------------------------------------------
// Dedicated JSON comment handlers for the SPA API
// ---------------------------------------------------------------------------

/// POST /api/v1/track/:tid/comment — JSON-only
pub async fn comment_track_json(
    State(state): State<AppState>,
    crate::session::OptionalSession(sess, _theme): crate::session::OptionalSession,
    Path(tid): Path<i32>,
    Json(body): Json<CommentJson>,
) -> Response {
    let t = match Track::by_id(&state.db, tid).await {
        Some(t) => t,
        None => return (
            axum::http::StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "not_found" })),
        ).into_response(),
    };

    let msg = body.msg.unwrap_or_default();
    if msg.is_empty() {
        return (
            axum::http::StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "bad_request", "message": "Message cannot be empty" })),
        ).into_response();
    }
    if msg.contains("<a") || msg.contains("[url") {
        return (
            axum::http::StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "bad_request", "message": "Message rejected" })),
        ).into_response();
    }

    if let Some(ref s) = sess {
        Session::new_nonce(&state.db, &s.sid).await;
    }

    let source = sess
        .as_ref()
        .map(|s| s.user.clone())
        .unwrap_or_else(|| User { id: 0, name: body.name.unwrap_or_default() });

    event::push_track_comment(&state.db, &source, &t.artist, &t, &msg).await;

    if let Some(acct) = crate::models::account::Account::by_id(&state.db, t.artist.id).await {
        if acct.notify && acct.user.id != source.id {
            let mail_body = format!(
                "{} posted a comment on {}:\n\n{}\n\n\
                 You can view it here: {}/track/{}\n\
                 Disable notifications: {}/account",
                source.name, t.title, msg, state.base_url, tid, state.base_url
            );
            crate::models::mail::send(&acct.email, "Manemix comment notification", &mail_body);
        }
    }

    Json(serde_json::json!({ "ok": true })).into_response()
}

/// POST /api/v1/user/:uid/comment — JSON-only
pub async fn comment_user_json(
    State(state): State<AppState>,
    crate::session::OptionalSession(sess, _theme): crate::session::OptionalSession,
    Path(uid): Path<i32>,
    Json(body): Json<CommentJson>,
) -> Response {
    let u = match User::by_id(&state.db, uid).await {
        Some(u) => u,
        None => return (
            axum::http::StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "not_found" })),
        ).into_response(),
    };

    let msg = body.msg.unwrap_or_default();
    if msg.is_empty() {
        return (
            axum::http::StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "bad_request", "message": "Message cannot be empty" })),
        ).into_response();
    }
    if msg.contains("<a") || msg.contains("[url") {
        return (
            axum::http::StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "bad_request", "message": "Message rejected" })),
        ).into_response();
    }

    if let Some(ref s) = sess {
        Session::new_nonce(&state.db, &s.sid).await;
    }

    let source = sess
        .as_ref()
        .map(|s| s.user.clone())
        .unwrap_or_else(|| User { id: 0, name: body.name.unwrap_or_default() });

    event::push_user_comment(&state.db, &source, &u, &msg).await;

    if let Some(acct) = crate::models::account::Account::by_id(&state.db, uid).await {
        if acct.notify && acct.user.id != source.id {
            let mail_body = format!(
                "{} posted a comment on your user page:\n\n{}\n\n\
                 You can view it here: {}/user/{}\n\
                 Disable notifications: {}/account",
                source.name, msg, state.base_url, uid, state.base_url
            );
            crate::models::mail::send(&acct.email, "Manemix comment notification", &mail_body);
        }
    }

    Json(serde_json::json!({ "ok": true })).into_response()
}
