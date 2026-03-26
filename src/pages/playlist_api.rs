//! Dedicated JSON API endpoints for SPA playlist management.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;

use crate::AppState;
use crate::models::playlist::Playlist;
use crate::session::{OptionalSession, RequiredSession};

fn ok() -> Response { Json(serde_json::json!({ "ok": true })).into_response() }
fn err(status: StatusCode, msg: &str) -> Response {
    (status, Json(serde_json::json!({ "error": msg }))).into_response()
}

/// GET /api/v1/playlists — browse public playlists
pub async fn list_public(State(state): State<AppState>) -> Response {
    let playlists = Playlist::public_all(&state.db).await;
    Json(playlists.iter().map(|p| p.context()).collect::<Vec<_>>()).into_response()
}

/// GET /api/v1/playlist/{id} — get playlist (respects visibility)
pub async fn get_playlist(
    State(state): State<AppState>,
    OptionalSession(sess, _theme): OptionalSession,
    Path(id): Path<i32>,
) -> Response {
    let p = match Playlist::by_id(&state.db, id).await {
        Some(p) if !p.is_album => p,
        _ => return err(StatusCode::NOT_FOUND, "Playlist not found"),
    };
    // Private playlists only visible to owner
    if !p.is_public {
        let is_owner = sess.as_ref().map(|s| s.user.id == p.author.id).unwrap_or(false);
        if !is_owner {
            return err(StatusCode::NOT_FOUND, "Playlist not found");
        }
    }
    let tracks = p.tracks(&state.db).await;
    Json(serde_json::json!({
        "playlist": p.context(),
        "tracks": tracks.iter().map(|t| t.context(&state.manemix_dir)).collect::<Vec<_>>(),
    })).into_response()
}

#[derive(Deserialize)]
pub struct CreateBody {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub public: bool,
}

/// POST /api/v1/playlist/new
pub async fn create(
    State(state): State<AppState>,
    RequiredSession(sess): RequiredSession,
    Json(body): Json<CreateBody>,
) -> Response {
    if body.name.is_empty() {
        return err(StatusCode::BAD_REQUEST, "Name cannot be empty");
    }
    let id: Option<i32> = sqlx::query_scalar(
        "INSERT INTO playlists (user_id, name, description, track_ids, public, is_album) \
         VALUES ($1, $2, $3, ARRAY[]::int[], $4, false) RETURNING id"
    )
    .bind(sess.user.id)
    .bind(&body.name)
    .bind(&body.description)
    .bind(body.public)
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten();

    match id {
        Some(id) => Json(serde_json::json!({ "ok": true, "id": id })).into_response(),
        None => err(StatusCode::INTERNAL_SERVER_ERROR, "Failed to create playlist"),
    }
}

#[derive(Deserialize)]
pub struct EditBody {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub public: Option<bool>,
}

/// POST /api/v1/playlist/{id}/edit
pub async fn edit(
    State(state): State<AppState>,
    RequiredSession(sess): RequiredSession,
    Path(id): Path<i32>,
    Json(body): Json<EditBody>,
) -> Response {
    match Playlist::by_id(&state.db, id).await {
        Some(p) if !p.is_album && p.author.id == sess.user.id => {},
        _ => return err(StatusCode::FORBIDDEN, "Not your playlist"),
    };
    if let Some(name) = &body.name {
        let _ = sqlx::query("UPDATE playlists SET name = $1 WHERE id = $2")
            .bind(name).bind(id).execute(&state.db).await;
    }
    if let Some(desc) = &body.description {
        let _ = sqlx::query("UPDATE playlists SET description = $1 WHERE id = $2")
            .bind(desc).bind(id).execute(&state.db).await;
    }
    if let Some(public) = body.public {
        let _ = sqlx::query("UPDATE playlists SET public = $1 WHERE id = $2")
            .bind(public).bind(id).execute(&state.db).await;
    }
    ok()
}

/// POST /api/v1/playlist/{id}/delete
pub async fn delete(
    State(state): State<AppState>,
    RequiredSession(sess): RequiredSession,
    Path(id): Path<i32>,
) -> Response {
    match Playlist::by_id(&state.db, id).await {
        Some(p) if !p.is_album && p.author.id == sess.user.id => {},
        _ => return err(StatusCode::FORBIDDEN, "Not your playlist"),
    };
    let _ = sqlx::query("DELETE FROM user_features WHERE type = 'playlist' AND ref = $1").bind(id).execute(&state.db).await;
    let _ = sqlx::query("DELETE FROM playlists WHERE id = $1").bind(id).execute(&state.db).await;
    ok()
}

#[derive(Deserialize)]
pub struct AddTrackBody {
    pub track_id: i32,
}

/// POST /api/v1/playlist/{id}/add
pub async fn add_track(
    State(state): State<AppState>,
    RequiredSession(sess): RequiredSession,
    Path(id): Path<i32>,
    Json(body): Json<AddTrackBody>,
) -> Response {
    match Playlist::by_id(&state.db, id).await {
        Some(p) if !p.is_album && p.author.id == sess.user.id => {},
        _ => return err(StatusCode::FORBIDDEN, "Not your playlist"),
    };
    let _ = sqlx::query("UPDATE playlists SET track_ids = array_append(track_ids, $1) WHERE id = $2")
        .bind(body.track_id).bind(id).execute(&state.db).await;
    ok()
}

#[derive(Deserialize)]
pub struct RemoveTrackBody {
    pub track_id: i32,
}

/// POST /api/v1/playlist/{id}/remove
pub async fn remove_track(
    State(state): State<AppState>,
    RequiredSession(sess): RequiredSession,
    Path(id): Path<i32>,
    Json(body): Json<RemoveTrackBody>,
) -> Response {
    match Playlist::by_id(&state.db, id).await {
        Some(p) if !p.is_album && p.author.id == sess.user.id => {},
        _ => return err(StatusCode::FORBIDDEN, "Not your playlist"),
    };
    let _ = sqlx::query("UPDATE playlists SET track_ids = array_remove(track_ids, $1) WHERE id = $2")
        .bind(body.track_id).bind(id).execute(&state.db).await;
    ok()
}
