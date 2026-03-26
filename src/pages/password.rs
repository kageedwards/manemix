use axum::extract::{Query, State};
use axum::http::{HeaderMap, HeaderValue};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::Form;
use serde::Deserialize;

use crate::AppState;
use crate::models::account::Account;
use crate::models::session::{Session, random_string};
use crate::session::OptionalSession;
use super::home::wrap_page;

fn remote_addr_from_headers(headers: &HeaderMap) -> String {
    headers.get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or("").trim().to_string())
        .unwrap_or_else(|| "127.0.0.1".into())
}

#[derive(Deserialize)]
pub struct ResetQuery {
    token: Option<String>,
}

#[derive(Deserialize)]
pub struct ResetForm {
    email: Option<String>,
}

pub async fn reset_page(
    State(state): State<AppState>,
    OptionalSession(sess, _theme): OptionalSession,
    headers: HeaderMap,
    Query(q): Query<ResetQuery>,
) -> Response {
    if sess.is_some() {
        return Redirect::to("/account").into_response();
    }

    // Coming from an email with a token
    if let Some(token) = q.token.filter(|t| !t.is_empty()) {
        let remote = remote_addr_from_headers(&headers);
        return handle_token(&state, &token, &remote).await;
    }

    Html(render_reset(&state, "", "")).into_response()
}

pub async fn reset_submit(
    State(state): State<AppState>,
    OptionalSession(sess, _theme): OptionalSession,
    Form(form): Form<ResetForm>,
) -> Response {
    if sess.is_some() {
        return Redirect::to("/account").into_response();
    }

    let email = form.email.unwrap_or_default();
    if email.is_empty() {
        return Html(render_reset(&state, "Please enter your email address.", "")).into_response();
    }

    let uid: Option<i32> = sqlx::query_scalar(
        "SELECT id FROM users WHERE lower(email) = lower($1)"
    )
    .bind(&email)
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten();

    match uid {
        Some(uid) => {
            let token = random_string(32);
            let _ = sqlx::query("INSERT INTO resets (user_id, token) VALUES ($1, $2)")
                .bind(uid).bind(&token).execute(&state.db).await;

            // Send email
            let reset_url = format!("{}/account/reset?token={}", state.base_url, token);
            let mail_body = format!(
                "Hi.\n\n\
                 Someone, hopefully you, requested a password reset on your account at Manemix. \
                 If you want to reset your password, please click this link: {}\n\
                 If you didn't request it, please ignore this email.\n\n\
                 Cheers.",
                reset_url
            );
            crate::models::mail::send(&email, "Resetting your password on Manemix", &mail_body);

            Html(render_reset(&state, "",
                "A reset link has been sent to your email address."
            )).into_response()
        }
        None => {
            Html(render_reset(&state,
                "Sorry, we couldn't find any account with this email address.",
                ""
            )).into_response()
        }
    }
}

async fn handle_token(state: &AppState, token: &str, remote_addr: &str) -> Response {
    let uid: Option<i32> = sqlx::query_scalar(
        "SELECT user_id FROM resets WHERE token = $1"
    )
    .bind(token)
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten();

    match uid {
        Some(uid) => {
            let _ = sqlx::query("DELETE FROM resets WHERE token = $1")
                .bind(token).execute(&state.db).await;

            let new_pw = random_string(16);
            let hash = bcrypt::hash(&new_pw, bcrypt::DEFAULT_COST).unwrap();
            let _ = sqlx::query("UPDATE users SET password = $1 WHERE id = $2")
                .bind(&hash).bind(uid).execute(&state.db).await;

            if let Some(sid) = Session::create(&state.db, uid, remote_addr).await {
                let a = Account::by_id(&state.db, uid).await;
                let mut ctx = tera::Context::new();
                ctx.insert("title", "Your account");
                ctx.insert("has_title", &true);
                ctx.insert("manemix_url", &state.base_url);
                ctx.insert("message", &format!("Your password has been reset to {}. You can now change it below.", new_pw));
                ctx.insert("has_message", &true);
                ctx.insert("old_password", &new_pw);
                if let Some(ref a) = a {
                    ctx.insert("account", &a.context());
                }
                let body = state.tera.render("html/account.tpl", &ctx).unwrap_or_default();
                let page = wrap_page(state, &ctx, &body, None, "auto");
                let mut resp = Html(page).into_response();
                let cookie = format!("sid={sid}; Max-Age=2592000; Path=/; HttpOnly");
                resp.headers_mut().insert("Set-Cookie", HeaderValue::from_str(&cookie).unwrap());
                resp
            } else {
                Redirect::to("/login").into_response()
            }
        }
        None => {
            Html(render_reset(state, "Sorry, looks like your token has expired.", "")).into_response()
        }
    }
}

fn render_reset(state: &AppState, error: &str, message: &str) -> String {
    let mut ctx = tera::Context::new();
    ctx.insert("title", "Password reset");
    ctx.insert("has_title", &true);
    ctx.insert("manemix_url", &state.base_url);
    if !error.is_empty() {
        ctx.insert("error", error);
        ctx.insert("has_error", &true);
    }
    if !message.is_empty() {
        ctx.insert("message", message);
        ctx.insert("has_message", &true);
    }
    let body = state.tera.render("html/password-reset.tpl", &ctx).unwrap_or_default();
    wrap_page(state, &ctx, &body, None, "auto")
}
