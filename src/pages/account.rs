use axum::extract::State;
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::Form;
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::models::account::Account;
use crate::models::session::Session;
use crate::session::RequiredSession;
use crate::text::valid_email;
use super::home::wrap_page;

#[derive(Deserialize, Default)]
pub struct AccountForm {
    nonce: Option<String>,
    name: Option<String>,
    email: Option<String>,
    about: Option<String>,
    notify: Option<String>,
    theme: Option<String>,
    oldpw: Option<String>,
    newpw: Option<String>,
    newpwconf: Option<String>,
}

pub async fn account_page(
    State(state): State<AppState>,
    RequiredSession(sess): RequiredSession,
) -> Html<String> {
    let a = Account::by_id(&state.db, sess.user.id).await.unwrap();
    Html(render_account(&state, &a, &sess, "", ""))
}

/// GET /api/v1/account — JSON account data for the SPA
#[derive(Serialize)]
struct AccountJson {
    username: String,
    email: String,
    about: String,
    notify: bool,
    license: String,
    theme: String,
}

pub async fn account_json(
    State(state): State<AppState>,
    RequiredSession(sess): RequiredSession,
) -> Response {
    match Account::by_id(&state.db, sess.user.id).await {
        Some(a) => axum::Json(AccountJson {
            username: a.user.name,
            email: a.email,
            about: a.about,
            notify: a.notify,
            license: a.license,
            theme: a.theme,
        }).into_response(),
        None => {
            let body = serde_json::json!({"error": "Account not found"});
            (axum::http::StatusCode::NOT_FOUND, axum::Json(body)).into_response()
        }
    }
}

pub async fn account_submit(
    State(state): State<AppState>,
    RequiredSession(sess): RequiredSession,
    Form(form): Form<AccountForm>,
) -> Response {
    let a = match Account::by_id(&state.db, sess.user.id).await {
        Some(a) => a,
        None => return Redirect::to("/").into_response(),
    };

    // CSRF check
    if form.nonce.as_deref() != Some(&sess.nonce) {
        return Html(render_account(&state, &a, &sess, "", "")).into_response();
    }

    // Regenerate nonce
    let new_nonce = Session::new_nonce(&state.db, &sess.sid).await;
    let mut sess = sess;
    sess.nonce = new_nonce;

    match apply_account_changes(&state, &a, &form).await {
        Ok(_) => {
            let updated = Account::by_id(&state.db, sess.user.id).await.unwrap();
            Html(render_account(&state, &updated, &sess, "", "Changes applied.")).into_response()
        }
        Err(msg) => Html(render_account(&state, &a, &sess, &msg, "")).into_response(),
    }
}

/// POST /api/v1/account — JSON account update for the SPA (no nonce required)
pub async fn account_submit_json(
    State(state): State<AppState>,
    RequiredSession(sess): RequiredSession,
    Form(form): Form<AccountForm>,
) -> Response {
    let a = match Account::by_id(&state.db, sess.user.id).await {
        Some(a) => a,
        None => {
            let body = serde_json::json!({"error": "Account not found"});
            return (axum::http::StatusCode::NOT_FOUND, axum::Json(body)).into_response();
        }
    };

    match apply_account_changes(&state, &a, &form).await {
        Ok(_) => axum::Json(serde_json::json!({"ok": true})).into_response(),
        Err(msg) => {
            let body = serde_json::json!({"error": msg});
            (axum::http::StatusCode::BAD_REQUEST, axum::Json(body)).into_response()
        }
    }
}

/// Shared validation and update logic for account changes.
async fn apply_account_changes(
    state: &AppState,
    a: &Account,
    form: &AccountForm,
) -> Result<(), String> {
    let name = form.name.clone().unwrap_or_default();
    let email = form.email.clone().unwrap_or_default();

    // Validate name change
    let final_name = if !name.is_empty() && name != a.user.name {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM users WHERE lower(name) = lower($1))"
        ).bind(&name).fetch_one(&state.db).await.unwrap_or(true);
        if exists {
            return Err("Name already in use.".into());
        }
        name
    } else {
        a.user.name.clone()
    };

    // Validate email change
    let final_email = if !email.is_empty() && email != a.email {
        if !valid_email(&email) {
            return Err("Invalid email address.".into());
        }
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM users WHERE lower(email) = lower($1))"
        ).bind(&email).fetch_one(&state.db).await.unwrap_or(true);
        if exists {
            return Err("Email already in use.".into());
        }
        email
    } else {
        a.email.clone()
    };

    // Password change
    let oldpw = form.oldpw.clone().unwrap_or_default();
    let newpw = form.newpw.clone().unwrap_or_default();
    let newpwconf = form.newpwconf.clone().unwrap_or_default();
    if !oldpw.is_empty() && !newpw.is_empty() {
        if newpw != newpwconf {
            return Err("Passwords mismatch.".into());
        }
        if Session::authenticate(&state.db, &a.email, &oldpw).await.is_none() {
            return Err("Wrong password.".into());
        }
        let hash = bcrypt::hash(&newpw, bcrypt::DEFAULT_COST).unwrap();
        let _ = sqlx::query("UPDATE users SET password = $1 WHERE id = $2")
            .bind(&hash).bind(a.user.id).execute(&state.db).await;
    }

    // Apply changes
    let notify = !form.notify.clone().unwrap_or_default().is_empty();
    let about = form.about.clone().unwrap_or_default();
    let theme = match form.theme.as_deref() {
        Some("light") => "light",
        Some("dark") => "dark",
        _ => "auto",
    };
    let _ = sqlx::query(
        "UPDATE users SET name = $1, email = $2, about = $3, notify = $4, theme = $5 WHERE id = $6"
    )
    .bind(&final_name)
    .bind(&final_email)
    .bind(&about)
    .bind(notify)
    .bind(theme)
    .bind(a.user.id)
    .execute(&state.db)
    .await;

    Ok(())
}

pub async fn delete_page(
    State(state): State<AppState>,
    RequiredSession(sess): RequiredSession,
) -> Html<String> {
    let new_nonce = Session::new_nonce(&state.db, &sess.sid).await;
    let mut ctx = tera::Context::new();
    ctx.insert("title", "Account deletion");
    ctx.insert("has_title", &true);
    ctx.insert("manemix_url", &state.base_url);
    ctx.insert("cancel_url", "/account");
    ctx.insert("nonce", &new_nonce);
    let body = state.tera.render("html/delete-account.tpl", &ctx).unwrap_or_default();
    let mut sess = sess;
    sess.nonce = new_nonce;
    Html(wrap_page(&state, &ctx, &body, Some(&sess), &sess.theme))
}

#[derive(Deserialize)]
pub struct DeleteForm {
    confirm: Option<String>,
    nonce: Option<String>,
}

pub async fn delete_submit(
    State(state): State<AppState>,
    RequiredSession(sess): RequiredSession,
    Form(form): Form<DeleteForm>,
) -> Response {
    if form.confirm.as_deref() != Some("Delete") || form.nonce.as_deref() != Some(&sess.nonce) {
        return Redirect::to("/account").into_response();
    }
    // Delete all user data (cascade)
    let uid = sess.user.id;
    let _ = sqlx::query("DELETE FROM sessions WHERE user_id = $1").bind(uid).execute(&state.db).await;
    let _ = sqlx::query("DELETE FROM favorites WHERE user_id = $1").bind(uid).execute(&state.db).await;
    let _ = sqlx::query("DELETE FROM favorites WHERE type = 'artist' AND ref = $1").bind(uid).execute(&state.db).await;
    // Remove favorites referencing user's tracks
    let _ = sqlx::query("DELETE FROM favorites WHERE type = 'track' AND ref IN (SELECT id FROM tracks WHERE user_id = $1)").bind(uid).execute(&state.db).await;
    let _ = sqlx::query("DELETE FROM resets WHERE user_id = $1").bind(uid).execute(&state.db).await;
    let _ = sqlx::query("DELETE FROM events WHERE source_id = $1 OR target_id = $1").bind(uid).execute(&state.db).await;
    let _ = sqlx::query("DELETE FROM user_features WHERE user_id = $1").bind(uid).execute(&state.db).await;
    // Remove user_features referencing user's tracks/playlists
    let _ = sqlx::query("DELETE FROM user_features WHERE type = 'track' AND ref IN (SELECT id FROM tracks WHERE user_id = $1)").bind(uid).execute(&state.db).await;
    let _ = sqlx::query("DELETE FROM user_features WHERE type = 'playlist' AND ref IN (SELECT id FROM playlists WHERE user_id = $1)").bind(uid).execute(&state.db).await;
    // Remove user's tracks from other people's playlists
    let _ = sqlx::query(
        "UPDATE playlists SET track_ids = ( \
         SELECT coalesce(array_agg(tid), ARRAY[]::int[]) FROM unnest(track_ids) AS tid \
         WHERE tid NOT IN (SELECT id FROM tracks WHERE user_id = $1) \
        ) WHERE track_ids && (SELECT array_agg(id) FROM tracks WHERE user_id = $1)"
    ).bind(uid).execute(&state.db).await;
    let _ = sqlx::query("DELETE FROM playlists WHERE user_id = $1").bind(uid).execute(&state.db).await;
    // Delete tracks (and their associated data)
    let _ = sqlx::query("DELETE FROM featured_tracks WHERE track_id IN (SELECT id FROM tracks WHERE user_id = $1)").bind(uid).execute(&state.db).await;
    let _ = sqlx::query("DELETE FROM events WHERE track_id IN (SELECT id FROM tracks WHERE user_id = $1)").bind(uid).execute(&state.db).await;
    let _ = sqlx::query("DELETE FROM tracks WHERE user_id = $1").bind(uid).execute(&state.db).await;
    let _ = sqlx::query("DELETE FROM users WHERE id = $1").bind(uid).execute(&state.db).await;

    let mut resp = Redirect::to("/").into_response();
    resp.headers_mut().insert("Set-Cookie", axum::http::HeaderValue::from_static("sid=; Max-Age=0; Path=/; HttpOnly"));
    resp
}

#[derive(Deserialize)]
pub struct ThemeForm {
    theme: Option<String>,
}

/// POST /account/theme — AJAX theme toggle for logged-in users
pub async fn theme_submit(
    State(state): State<AppState>,
    RequiredSession(sess): RequiredSession,
    Form(form): Form<ThemeForm>,
) -> Response {
    let theme = match form.theme.as_deref() {
        Some("light") => "light",
        Some("dark") => "dark",
        _ => "auto",
    };
    let _ = sqlx::query("UPDATE users SET theme = $1 WHERE id = $2")
        .bind(theme)
        .bind(sess.user.id)
        .execute(&state.db)
        .await;
    axum::Json(serde_json::json!({"ok": true})).into_response()
}

fn render_account(state: &AppState, a: &Account, sess: &Session, error: &str, message: &str) -> String {
    let mut ctx = tera::Context::new();
    ctx.insert("title", "Your account");
    ctx.insert("has_title", &true);
    ctx.insert("manemix_url", &state.base_url);
    ctx.insert("account", &a.context());
    if !error.is_empty() {
        ctx.insert("error", error);
        ctx.insert("has_error", &true);
    }
    if !message.is_empty() {
        ctx.insert("message", message);
        ctx.insert("has_message", &true);
    }
    ctx.insert("nonce", &sess.nonce);
    let body = state.tera.render("html/account.tpl", &ctx).unwrap_or_default();
    wrap_page(state, &ctx, &body, Some(sess), &sess.theme)
}
