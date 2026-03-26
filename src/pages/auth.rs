use axum::extract::{Query, State};
use axum::http::{HeaderMap, HeaderValue};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::Form;
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::models::session::Session;
use crate::session::OptionalSession;
use crate::text::valid_email;
use super::home::wrap_page;

fn remote_addr_from_headers(headers: &HeaderMap) -> String {
    headers.get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or("").trim().to_string())
        .unwrap_or_else(|| "127.0.0.1".into())
}

// ---------------------------------------------------------------------------
// Login
// ---------------------------------------------------------------------------

#[derive(Deserialize, Default)]
pub struct LoginForm {
    email: Option<String>,
    pw: Option<String>,
    redirect: Option<String>,
}

#[derive(Deserialize)]
pub struct RedirectQuery {
    redirect: Option<String>,
}

pub async fn login_page(
    State(state): State<AppState>,
    OptionalSession(sess, _theme): OptionalSession,
    Query(q): Query<RedirectQuery>,
) -> Response {
    if sess.is_some() {
        return Redirect::to("/").into_response();
    }
    Html(render_login(&state, None, "", q.redirect.as_deref())).into_response()
}

pub async fn login_submit(
    State(state): State<AppState>,
    OptionalSession(sess, _theme): OptionalSession,
    headers: HeaderMap,
    Form(form): Form<LoginForm>,
) -> Response {
    if sess.is_some() {
        return Redirect::to("/").into_response();
    }

    let email = form.email.unwrap_or_default();
    let pw = form.pw.unwrap_or_default();
    let redir = form.redirect.unwrap_or_default();

    if email.is_empty() {
        return Html(render_login(&state, None, "Please specify an email address.", Some(&redir))).into_response();
    }
    if pw.is_empty() {
        return Html(render_login(&state, None, "Please specify a password.", Some(&redir))).into_response();
    }
    if !email.contains('@') || !email.contains('.') {
        return Html(render_login(&state, None, "That doesn't look like an email address.", Some(&redir))).into_response();
    }

    let remote = remote_addr_from_headers(&headers);
    match Session::authenticate(&state.db, &email, &pw).await {
        Some((uid, _name)) => {
            if let Some(sid) = Session::create(&state.db, uid, &remote).await {
                let target = if redir.starts_with('/') { redir } else { format!("/user/{uid}") };
                let mut resp = Redirect::to(&target).into_response();
                let cookie = format!("sid={sid}; Max-Age=2592000; Path=/; HttpOnly");
                resp.headers_mut().insert("Set-Cookie", HeaderValue::from_str(&cookie).unwrap());
                resp
            } else {
                Html(render_login(&state, None, "Something went wrong. Please try again.", Some(&redir))).into_response()
            }
        }
        None => {
            Html(render_login(&state, None, "Sorry, wrong credentials.", Some(&redir))).into_response()
        }
    }
}

pub async fn logout(
    State(state): State<AppState>,
    OptionalSession(sess, _theme): OptionalSession,
    Query(q): Query<RedirectQuery>,
) -> Response {
    if let Some(s) = sess {
        Session::destroy(&state.db, &s.sid).await;
    }
    let target = q.redirect.filter(|r| r.starts_with('/')).unwrap_or_else(|| "/".into());
    let mut resp = Redirect::to(&target).into_response();
    resp.headers_mut().insert("Set-Cookie", HeaderValue::from_static("sid=; Max-Age=0; Path=/; HttpOnly"));
    resp
}

fn render_login(state: &AppState, sess: Option<&Session>, error: &str, redirect: Option<&str>) -> String {
    let mut ctx = tera::Context::new();
    ctx.insert("title", "Login");
    ctx.insert("has_title", &true);
    ctx.insert("manemix_url", &state.base_url);
    if !error.is_empty() {
        ctx.insert("error", error);
        ctx.insert("has_error", &true);
    }
    if let Some(r) = redirect {
        ctx.insert("redirect", r);
    }
    let body = state.tera.render("html/login.tpl", &ctx).unwrap_or_default();
    wrap_page(state, &ctx, &body, sess, "auto")
}

// ---------------------------------------------------------------------------
// Registration
// ---------------------------------------------------------------------------

#[derive(Deserialize, Default)]
pub struct RegisterForm {
    name: Option<String>,
    email: Option<String>,
    pw: Option<String>,
    pwconf: Option<String>,
}

pub async fn register_page(
    State(state): State<AppState>,
    OptionalSession(sess, _theme): OptionalSession,
) -> Response {
    if sess.is_some() {
        return Redirect::to("/").into_response();
    }
    Html(render_register(&state, "", "", "")).into_response()
}

pub async fn register_submit(
    State(state): State<AppState>,
    OptionalSession(sess, _theme): OptionalSession,
    headers: HeaderMap,
    Form(form): Form<RegisterForm>,
) -> Response {
    if sess.is_some() {
        return Redirect::to("/").into_response();
    }

    let name = form.name.unwrap_or_default();
    let email = form.email.unwrap_or_default();
    let pw = form.pw.unwrap_or_default();
    let pwconf = form.pwconf.unwrap_or_default();

    if name.is_empty() {
        return Html(render_register(&state, "Please specify a display name.", &name, &email)).into_response();
    }
    if email.is_empty() {
        return Html(render_register(&state, "Please specify an email address.", &name, &email)).into_response();
    }
    if !valid_email(&email) {
        return Html(render_register(&state, "Invalid email address.", &name, &email)).into_response();
    }
    if pw.is_empty() {
        return Html(render_register(&state, "Please specify a password.", &name, &email)).into_response();
    }
    if pw != pwconf {
        return Html(render_register(&state, "Passwords mismatch.", &name, &email)).into_response();
    }

    // Check for duplicate name/email
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM users WHERE lower(name) = lower($1) OR lower(email) = lower($2))"
    )
    .bind(&name)
    .bind(&email)
    .fetch_one(&state.db)
    .await
    .unwrap_or(true);

    if exists {
        return Html(render_register(&state, "Sorry, name or email already in use.", &name, &email)).into_response();
    }

    // Hash password with bcrypt
    let hash = bcrypt::hash(&pw, bcrypt::DEFAULT_COST).unwrap();

    let uid: Option<i32> = sqlx::query_scalar(
        "INSERT INTO users (name, password, email, registration, last_login) \
         VALUES ($1, $2, $3, now(), now()) RETURNING id"
    )
    .bind(&name)
    .bind(&hash)
    .bind(&email)
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten();

    match uid {
        Some(uid) => {
            tracing::info!("New user: {} ({})", name, uid);
            let remote = remote_addr_from_headers(&headers);
            if let Some(sid) = Session::create(&state.db, uid, &remote).await {
                let mut resp = Redirect::to(&format!("/user/{uid}?welcome=1")).into_response();
                let cookie = format!("sid={sid}; Max-Age=2592000; Path=/; HttpOnly");
                resp.headers_mut().insert("Set-Cookie", HeaderValue::from_str(&cookie).unwrap());
                resp
            } else {
                Redirect::to("/login").into_response()
            }
        }
        None => {
            Html(render_register(&state, "Something went wrong. Please try again.", &name, &email)).into_response()
        }
    }
}

fn render_register(state: &AppState, error: &str, name: &str, email: &str) -> String {
    let mut ctx = tera::Context::new();
    ctx.insert("title", "Register");
    ctx.insert("has_title", &true);
    ctx.insert("manemix_url", &state.base_url);
    if !error.is_empty() {
        ctx.insert("error", error);
        ctx.insert("has_error", &true);
    }
    ctx.insert("name", name);
    ctx.insert("email", email);
    let body = state.tera.render("html/registration.tpl", &ctx).unwrap_or_default();
    wrap_page(state, &ctx, &body, None, "auto")
}

// ---------------------------------------------------------------------------
// /me/json — SPA auth state endpoint
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct MeLoggedIn {
    logged_in: bool,
    uid: i32,
    username: String,
}

#[derive(Serialize)]
struct MeLoggedOut {
    logged_in: bool,
}

/// GET /me/json — returns current auth state for the SPA
pub async fn me_json(
    OptionalSession(sess, _theme): OptionalSession,
) -> Response {
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", HeaderValue::from_static("application/json"));

    match sess {
        Some(s) => {
            let body = serde_json::to_string(&MeLoggedIn {
                logged_in: true,
                uid: s.user.id,
                username: s.user.name.clone(),
            }).unwrap();
            (headers, body).into_response()
        }
        None => {
            let body = serde_json::to_string(&MeLoggedOut {
                logged_in: false,
            }).unwrap();
            (headers, body).into_response()
        }
    }
}
