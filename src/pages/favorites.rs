use axum::extract::{Path, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::{Html, IntoResponse, Response};

use crate::AppState;
use crate::models::track::{Track, TrackRow};
use crate::models::user::User;
use super::home::wrap_page;

async fn favorite_tracks(state: &AppState, uid: i32) -> Vec<Track> {
    sqlx::query_as::<_, TrackRow>(
        "SELECT tracks.id, tracks.title, tracks.user_id, users.name, \
         tracks.visible, tracks.date::text, extract(epoch from tracks.date)::text as ts \
         FROM tracks, users, favorites \
         WHERE tracks.user_id = users.id AND favorites.type = 'track' \
         AND favorites.ref = tracks.id AND favorites.user_id = $1 \
         ORDER BY users.name ASC"
    )
    .bind(uid)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(Into::into)
    .collect()
}

pub async fn show(
    State(state): State<AppState>,
    Path(uid): Path<i32>,
) -> Response {
    let _u = match User::by_id(&state.db, uid).await {
        Some(u) => u,
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    let tracks = favorite_tracks(&state, uid).await;
    let title = format!("{} - Favorite tracks", _u.name);

    let mut ctx = tera::Context::new();
    ctx.insert("title", &title);
    ctx.insert("has_title", &true);
    ctx.insert("manemix_url", &state.base_url);
    ctx.insert("tracks", &tracks.iter().map(|t| t.context(&state.manemix_dir)).collect::<Vec<_>>());
    ctx.insert("has_prev", &false);
    ctx.insert("has_next", &false);

    let body = state.tera.render("html/tracklist-page.tpl", &ctx).unwrap_or_default();
    Html(wrap_page(&state, &ctx, &body, None, "auto")).into_response()
}

pub async fn json(
    State(state): State<AppState>,
    Path(uid): Path<i32>,
) -> Response {
    if User::by_id(&state.db, uid).await.is_none() {
        return StatusCode::NOT_FOUND.into_response();
    }

    let tracks = favorite_tracks(&state, uid).await;
    let items: Vec<_> = tracks.iter().map(|t| t.context(&state.manemix_dir)).collect();

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", HeaderValue::from_static("application/json"));
    (headers, serde_json::to_string(&items).unwrap_or_default()).into_response()
}
