use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use serde::Deserialize;

use crate::AppState;
use crate::models::playlist::Playlist;
use crate::session::OptionalSession;
use super::home::wrap_page;

#[derive(Deserialize)]
pub struct PlaylistQuery {
    firstrun: Option<String>,
}

pub async fn show(
    State(state): State<AppState>,
    OptionalSession(sess, theme): OptionalSession,
    Path(id): Path<i32>,
    Query(q): Query<PlaylistQuery>,
) -> Response {
    let p = match Playlist::by_id(&state.db, id).await {
        Some(p) => p,
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    let tracks = p.tracks(&state.db).await;
    let is_owner = sess.as_ref().map(|s| s.user.id == p.author.id).unwrap_or(false);

    let mut ctx = tera::Context::new();
    ctx.insert("title", &p.name);
    ctx.insert("has_title", &true);
    ctx.insert("manemix_url", &state.base_url);
    ctx.insert("playlist", &p.context());
    ctx.insert("tracks", &tracks.iter().map(|t| t.context(&state.manemix_dir)).collect::<Vec<_>>());
    ctx.insert("is_owner", &is_owner);
    ctx.insert("firstrun", &q.firstrun.is_some());
    if let Some(ref s) = sess {
        ctx.insert("nonce", &s.nonce);
    }

    let body = state.tera.render("html/playlist.tpl", &ctx).unwrap_or_default();
    Html(wrap_page(&state, &ctx, &body, sess.as_ref(), &theme)).into_response()
}

pub async fn json(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Response {
    let p = match Playlist::by_id(&state.db, id).await {
        Some(p) => p,
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    let tracks = p.tracks(&state.db).await;

    let body = serde_json::json!({
        "playlist": p.context(),
        "tracks": tracks.iter().map(|t| t.context(&state.manemix_dir)).collect::<Vec<_>>(),
    });

    axum::Json(body).into_response()
}
