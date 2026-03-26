use axum::extract::State;
use axum::http::{HeaderMap, HeaderValue};
use axum::response::{IntoResponse, Response};

use crate::AppState;
use crate::models::track;

pub async fn latest_atom(State(state): State<AppState>) -> Response {
    let tracks = track::latest(&state.db, 50, 0).await;

    let mut ctx = tera::Context::new();
    ctx.insert("manemix_url", &state.base_url);
    ctx.insert("what", "Latest tracks");
    ctx.insert("feed_url", "/tracks/latest/atom");
    ctx.insert("updated", &tracks.first().map(|t| t.date.as_str()).unwrap_or("1970-01-01 01:00:00+00"));
    ctx.insert("tracks", &tracks.iter().map(|t| t.context(&state.manemix_dir)).collect::<Vec<_>>());

    render_atom(&state, &ctx)
}

pub async fn featured_atom(State(state): State<AppState>) -> Response {
    let tracks = track::featured(&state.db, 50).await;
    let now = chrono::Utc::now().format("%F 01:00:00+00").to_string();

    let mut ctx = tera::Context::new();
    ctx.insert("manemix_url", &state.base_url);
    ctx.insert("what", "Featured tracks");
    ctx.insert("feed_url", "/tracks/featured/atom");
    ctx.insert("updated", &now);
    ctx.insert("tracks", &tracks.iter().map(|t| t.context(&state.manemix_dir)).collect::<Vec<_>>());

    render_atom(&state, &ctx)
}

fn render_atom(state: &AppState, ctx: &tera::Context) -> Response {
    let body = state.tera.render("atom-feed.tpl", ctx).unwrap_or_default();
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", HeaderValue::from_static("application/atom+xml; charset=UTF-8"));
    (headers, body).into_response()
}
