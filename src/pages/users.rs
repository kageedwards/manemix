use axum::extract::{Query, State};
use axum::http::{HeaderMap, HeaderValue};
use axum::response::{Html, IntoResponse, Response};
use serde::Deserialize;
use sqlx::{PgPool, FromRow};

use crate::AppState;
use crate::models::account::Account;
use super::home::wrap_page;

#[derive(Deserialize)]
pub struct PageQuery {
    p: Option<i64>,
    q: Option<String>,
}

#[derive(FromRow)]
struct UserRow {
    id: i32,
    name: String,
    email: String,
    about: String,
}

/// Lightweight account list query (mirrors AccountList in the original).
async fn fetch_artists(pool: &PgPool, limit: i64, offset: i64) -> Vec<Account> {
    sqlx::query_as::<_, UserRow>(
        "SELECT users.id, users.name, users.email, users.about \
         FROM users WHERE EXISTS \
         (SELECT 1 FROM tracks WHERE user_id = users.id AND visible = true) \
         ORDER BY name ASC LIMIT $1 OFFSET $2"
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(|r| Account {
        user: crate::models::user::User { id: r.id, name: r.name },
        email: r.email,
        about: r.about,
        notify: false,
        license: String::new(),
        theme: "auto".into(),
        num_favs: 0,
        num_followers: 0,
    })
    .collect()
}

async fn search_users(pool: &PgPool, q: &str) -> Vec<Account> {
    let pattern = format!("%{q}%");
    sqlx::query_as::<_, UserRow>(
        "SELECT id, name, email, about FROM users WHERE name ILIKE $1 ORDER BY name ASC"
    )
    .bind(&pattern)
    .fetch_all(pool)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(|r| Account {
        user: crate::models::user::User { id: r.id, name: r.name },
        email: r.email,
        about: r.about,
        notify: false,
        license: String::new(),
        theme: "auto".into(),
        num_favs: 0,
        num_followers: 0,
    })
    .collect()
}

pub async fn artists(State(state): State<AppState>, Query(q): Query<PageQuery>) -> Html<String> {
    let page = q.p.unwrap_or(1).max(1);
    let mut users = fetch_artists(&state.db, 21, 20 * (page - 1)).await;
    let has_next = users.len() == 21;
    if has_next { users.pop(); }

    let mut ctx = tera::Context::new();
    ctx.insert("title", "Artists");
    ctx.insert("has_title", &true);
    ctx.insert("manemix_url", &state.base_url);
    ctx.insert("users", &users.iter().map(|u| u.context()).collect::<Vec<_>>());
    ctx.insert("has_prev", &(page > 1));
    ctx.insert("has_next", &has_next);
    ctx.insert("prev", &(page - 1));
    ctx.insert("next", &(page + 1));
    let body = state.tera.render("html/userlist-page.tpl", &ctx).unwrap_or_default();
    Html(wrap_page(&state, &ctx, &body, None, "auto"))
}

pub async fn search(State(state): State<AppState>, Query(q): Query<PageQuery>) -> Html<String> {
    let query = q.q.unwrap_or_default();
    let users = search_users(&state.db, &query).await;

    let mut ctx = tera::Context::new();
    ctx.insert("title", &query);
    ctx.insert("has_title", &true);
    ctx.insert("manemix_url", &state.base_url);
    ctx.insert("users", &users.iter().map(|u| u.context()).collect::<Vec<_>>());
    ctx.insert("has_prev", &false);
    ctx.insert("has_next", &false);
    let body = state.tera.render("html/userlist-page.tpl", &ctx).unwrap_or_default();
    Html(wrap_page(&state, &ctx, &body, None, "auto"))
}

pub async fn artists_json(State(state): State<AppState>) -> Response {
    let users = fetch_artists(&state.db, 2000, 0).await;
    let items: Vec<_> = users.iter().map(|u| u.context()).collect();
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", HeaderValue::from_static("application/json"));
    (headers, serde_json::to_string(&items).unwrap_or_default()).into_response()
}

pub async fn search_json(State(state): State<AppState>, Query(q): Query<PageQuery>) -> Response {
    let query = q.q.unwrap_or_default();
    let users = search_users(&state.db, &query).await;
    let items: Vec<_> = users.iter().map(|u| u.context()).collect();
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", HeaderValue::from_static("application/json"));
    (headers, serde_json::to_string(&items).unwrap_or_default()).into_response()
}
