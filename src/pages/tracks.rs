use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, HeaderValue};
use axum::response::{Html, IntoResponse, Response};
use serde::Deserialize;

use crate::AppState;
use crate::models::session::Session;
use crate::models::track;
use crate::session::OptionalSession;
use super::home::wrap_page;

#[derive(Deserialize)]
pub struct PageQuery {
    p: Option<i64>,
    q: Option<String>,
    artist: Option<String>,
    track: Option<String>,
    per_page: Option<i64>,
    page: Option<i64>,
}

// --- HTML endpoints ---

pub async fn latest(State(state): State<AppState>, OptionalSession(sess, theme): OptionalSession, Query(q): Query<PageQuery>) -> Html<String> {
    let page = q.p.unwrap_or(1).max(1);
    let mut tracks = track::latest(&state.db, 16, 15 * (page - 1)).await;
    let has_next = tracks.len() == 16;
    if has_next { tracks.pop(); }

    render_list(&state, "Latest tracks", &tracks, page > 1, has_next, page, sess.as_ref(), &theme)
}

pub async fn featured(State(state): State<AppState>, OptionalSession(sess, theme): OptionalSession) -> Html<String> {
    let tracks = track::featured(&state.db, 15).await;
    render_list(&state, "Featured tracks", &tracks, false, false, 1, sess.as_ref(), &theme)
}

pub async fn random(State(state): State<AppState>, OptionalSession(sess, theme): OptionalSession) -> Html<String> {
    let tracks = track::random(&state.db, 15).await;
    render_list(&state, "Random tracks", &tracks, false, false, 1, sess.as_ref(), &theme)
}

pub async fn search(State(state): State<AppState>, OptionalSession(sess, theme): OptionalSession, Query(q): Query<PageQuery>) -> Html<String> {
    let query = q.q.unwrap_or_default();
    let tracks = track::search(&state.db, &query).await;
    let mut ctx = tera::Context::new();
    ctx.insert("title", &query);
    ctx.insert("has_title", &true);
    ctx.insert("search", &query);
    ctx.insert("manemix_url", &state.base_url);
    ctx.insert("tracks", &tracks.iter().map(|t| t.context(&state.manemix_dir)).collect::<Vec<_>>());
    ctx.insert("has_prev", &false);
    ctx.insert("has_next", &false);
    let body = state.tera.render("html/tracklist-page.tpl", &ctx).unwrap_or_default();
    Html(wrap_page(&state, &ctx, &body, sess.as_ref(), &theme))
}

pub async fn by_tag(State(state): State<AppState>, OptionalSession(sess, theme): OptionalSession, Path(tag): Path<String>) -> Html<String> {
    let tracks = track::by_tag(&state.db, &tag).await;
    render_list(&state, &tag, &tracks, false, false, 1, sess.as_ref(), &theme)
}

// --- JSON endpoints ---

pub async fn latest_json(State(state): State<AppState>) -> Response {
    json_array(&state, track::latest(&state.db, 50, 0).await)
}

pub async fn featured_json(State(state): State<AppState>) -> Response {
    json_array(&state, track::featured(&state.db, 50).await)
}

pub async fn random_json(State(state): State<AppState>) -> Response {
    json_array(&state, track::random(&state.db, 50).await)
}

pub async fn search_json(State(state): State<AppState>, Query(q): Query<PageQuery>) -> Response {
    let query = q.q.unwrap_or_default();
    json_array(&state, track::search(&state.db, &query).await)
}

pub async fn exact_search_json(State(state): State<AppState>, Query(q): Query<PageQuery>) -> Response {
    let artist = q.artist.unwrap_or_default();
    let title = q.track.unwrap_or_default();
    if artist.is_empty() || title.is_empty() {
        return json_array(&state, vec![]);
    }
    json_array(&state, track::exact_search(&state.db, &artist, &title).await)
}

pub async fn all_json(State(state): State<AppState>, Query(q): Query<PageQuery>) -> Response {
    let per_page = q.per_page.unwrap_or(50);
    let page = q.page.unwrap_or(1).max(1);
    let tracks = track::latest(&state.db, per_page, per_page * (page - 1)).await;
    json_array(&state, tracks)
}

// --- Helpers ---

fn render_list(
    state: &AppState,
    title: &str,
    tracks: &[track::Track],
    has_prev: bool,
    has_next: bool,
    page: i64,
    sess: Option<&Session>,
    theme: &str,
) -> Html<String> {
    let mut ctx = tera::Context::new();
    ctx.insert("title", title);
    ctx.insert("has_title", &true);
    ctx.insert("manemix_url", &state.base_url);
    ctx.insert("tracks", &tracks.iter().map(|t| t.context(&state.manemix_dir)).collect::<Vec<_>>());
    ctx.insert("has_prev", &has_prev);
    ctx.insert("has_next", &has_next);
    ctx.insert("prev", &(page - 1));
    ctx.insert("next", &(page + 1));
    let body = state.tera.render("html/tracklist-page.tpl", &ctx).unwrap_or_default();
    Html(wrap_page(state, &ctx, &body, sess, theme))
}

fn json_array(state: &AppState, tracks: Vec<track::Track>) -> Response {
    let items: Vec<_> = tracks.iter().map(|t| t.context(&state.manemix_dir)).collect();
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", HeaderValue::from_static("application/json"));
    (headers, serde_json::to_string(&items).unwrap_or_default()).into_response()
}
