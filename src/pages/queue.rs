//! Playback queue endpoint: given a playback context and the current track,
//! return the next chunk of tracks so the player can keep going.

use axum::extract::State;
use axum::http::{HeaderMap, HeaderValue};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;

use crate::AppState;
use crate::models::track::{self, Track, TrackContext, TrackRow};
use crate::models::playlist::Playlist;

/// How many tracks to return per chunk.
const CHUNK_SIZE: i64 = 20;

/// The playback context descriptor sent by the SPA.
#[derive(Debug, Deserialize)]
pub struct NextRequest {
    /// Current track id the player is on.
    pub current_tid: i32,
    /// Context kind: "latest", "featured", "random", "search", "tag",
    /// "playlist", "user", "favorites"
    pub context: String,
    /// Extra parameter depending on context:
    ///   search  → the query string
    ///   tag     → the tag name
    ///   playlist → playlist id (as string)
    ///   user    → user id (as string)
    ///   favorites → user id (as string)
    #[serde(default)]
    pub param: Option<String>,
}

/// Response: a chunk of tracks that come after `current_tid` in the given
/// context, wrapping around to the beginning when the end is reached.
pub async fn next_tracks(
    State(state): State<AppState>,
    Json(body): Json<NextRequest>,
) -> Response {
    let all = fetch_context_tracks(&state, &body.context, &body.param).await;

    if all.is_empty() {
        return json_response(&[]);
    }

    // Find where current_tid sits in the ordered list.
    let pos = all.iter().position(|t| t.id == body.current_tid);

    // Build the chunk starting after current_tid, wrapping around.
    let start = match pos {
        Some(i) => i + 1,
        // Track not found in context (maybe deleted) — start from the top.
        None => 0,
    };

    let len = all.len();
    let chunk: Vec<TrackContext> = (0..CHUNK_SIZE as usize)
        .map(|offset| &all[(start + offset) % len])
        .map(|t| t.context(&state.manemix_dir))
        .collect();

    json_response(&chunk)
}

async fn fetch_context_tracks(state: &AppState, context: &str, param: &Option<String>) -> Vec<Track> {
    match context {
        "latest" => track::latest(&state.db, 200, 0).await,
        "featured" => track::featured(&state.db, 200).await,
        "random" => track::random(&state.db, 200).await,
        "search" => {
            let q = param.as_deref().unwrap_or("");
            track::search(&state.db, q).await
        }
        "tag" => {
            let tag = param.as_deref().unwrap_or("");
            track::by_tag(&state.db, tag).await
        }
        "playlist" => {
            let id: i32 = param.as_deref().and_then(|s| s.parse().ok()).unwrap_or(0);
            match Playlist::by_id(&state.db, id).await {
                Some(pl) => pl.tracks(&state.db).await,
                None => vec![],
            }
        }
        "user" => {
            let uid: i32 = param.as_deref().and_then(|s| s.parse().ok()).unwrap_or(0);
            track::by_user(&state.db, uid, false).await
        }
        "favorites" => {
            let uid: i32 = param.as_deref().and_then(|s| s.parse().ok()).unwrap_or(0);
            fetch_favorites(&state, uid).await
        }
        _ => vec![],
    }
}

async fn fetch_favorites(state: &AppState, uid: i32) -> Vec<Track> {
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

fn json_response(items: &[TrackContext]) -> Response {
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", HeaderValue::from_static("application/json"));
    (headers, serde_json::to_string(items).unwrap_or_default()).into_response()
}
