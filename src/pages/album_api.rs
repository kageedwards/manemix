//! Dedicated JSON API endpoints for SPA album management.
//! Albums are playlists with is_album=true. Only the author's own tracks
//! can be added, and publishing an album publishes all contained tracks.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;

use crate::AppState;
use crate::models::playlist::Playlist;
use crate::models::track::Track;
use crate::session::{OptionalSession, RequiredSession};

fn ok() -> Response { Json(serde_json::json!({ "ok": true })).into_response() }
fn err(status: StatusCode, msg: &str) -> Response {
    (status, Json(serde_json::json!({ "error": msg }))).into_response()
}

/// GET /api/v1/albums — browse public albums
pub async fn list_public(State(state): State<AppState>) -> Response {
    let albums = Playlist::public_albums(&state.db).await;
    Json(albums.iter().map(|a| a.context()).collect::<Vec<_>>()).into_response()
}

/// GET /api/v1/album/{id} — get album (respects visibility)
pub async fn get_album(
    State(state): State<AppState>,
    OptionalSession(sess, _theme): OptionalSession,
    Path(id): Path<i32>,
) -> Response {
    let a = match Playlist::by_id(&state.db, id).await {
        Some(a) if a.is_album => a,
        _ => return err(StatusCode::NOT_FOUND, "Album not found"),
    };
    if !a.is_public {
        let is_owner = sess.as_ref().map(|s| s.user.id == a.author.id).unwrap_or(false);
        if !is_owner {
            return err(StatusCode::NOT_FOUND, "Album not found");
        }
    }
    let tracks = a.tracks(&state.db).await;
    Json(serde_json::json!({
        "playlist": a.context(),
        "tracks": tracks.iter().map(|t| t.context(&state.manemix_dir)).collect::<Vec<_>>(),
    })).into_response()
}

#[derive(Deserialize)]
pub struct CreateBody {
    pub name: String,
    #[serde(default)]
    pub description: String,
}

/// POST /api/v1/album/new — create a new album (always starts private)
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
         VALUES ($1, $2, $3, ARRAY[]::int[], false, true) RETURNING id"
    )
    .bind(sess.user.id)
    .bind(&body.name)
    .bind(&body.description)
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten();

    match id {
        Some(id) => Json(serde_json::json!({ "ok": true, "id": id })).into_response(),
        None => err(StatusCode::INTERNAL_SERVER_ERROR, "Failed to create album"),
    }
}

#[derive(Deserialize)]
pub struct EditBody {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

/// POST /api/v1/album/{id}/edit — edit album metadata (not visibility)
pub async fn edit(
    State(state): State<AppState>,
    RequiredSession(sess): RequiredSession,
    Path(id): Path<i32>,
    Json(body): Json<EditBody>,
) -> Response {
    match Playlist::by_id(&state.db, id).await {
        Some(a) if a.is_album && a.author.id == sess.user.id => {},
        _ => return err(StatusCode::FORBIDDEN, "Not your album"),
    };
    if let Some(name) = &body.name {
        let _ = sqlx::query("UPDATE playlists SET name = $1 WHERE id = $2")
            .bind(name).bind(id).execute(&state.db).await;
    }
    if let Some(desc) = &body.description {
        let _ = sqlx::query("UPDATE playlists SET description = $1 WHERE id = $2")
            .bind(desc).bind(id).execute(&state.db).await;
    }
    ok()
}

/// POST /api/v1/album/{id}/publish — publish album + all contained tracks
pub async fn publish(
    State(state): State<AppState>,
    RequiredSession(sess): RequiredSession,
    Path(id): Path<i32>,
) -> Response {
    let a = match Playlist::by_id(&state.db, id).await {
        Some(a) if a.is_album && a.author.id == sess.user.id => a,
        _ => return err(StatusCode::FORBIDDEN, "Not your album"),
    };
    // Albums must contain at least one track to be published
    let tracks = a.tracks(&state.db).await;
    if tracks.is_empty() {
        return err(StatusCode::BAD_REQUEST, "Cannot publish an album with no tracks");
    }
    // Make album public
    let _ = sqlx::query("UPDATE playlists SET public = true WHERE id = $1")
        .bind(id).execute(&state.db).await;
    // Publish all contained tracks that aren't already visible
    for t in &tracks {
        if !t.visible {
            let _ = sqlx::query("UPDATE tracks SET visible = 't', date = now() WHERE id = $1")
                .bind(t.id).execute(&state.db).await;
            crate::models::event::push_event(&state.db, "publish", &sess.user, &t.artist, Some(t)).await;
        }
    }
    ok()
}

/// POST /api/v1/album/{id}/unpublish — make album private again
pub async fn unpublish(
    State(state): State<AppState>,
    RequiredSession(sess): RequiredSession,
    Path(id): Path<i32>,
) -> Response {
    match Playlist::by_id(&state.db, id).await {
        Some(a) if a.is_album && a.author.id == sess.user.id => {},
        _ => return err(StatusCode::FORBIDDEN, "Not your album"),
    };
    let _ = sqlx::query("UPDATE playlists SET public = false WHERE id = $1")
        .bind(id).execute(&state.db).await;
    ok()
}

/// POST /api/v1/album/{id}/delete
pub async fn delete(
    State(state): State<AppState>,
    RequiredSession(sess): RequiredSession,
    Path(id): Path<i32>,
) -> Response {
    match Playlist::by_id(&state.db, id).await {
        Some(a) if a.is_album && a.author.id == sess.user.id => {},
        _ => return err(StatusCode::FORBIDDEN, "Not your album"),
    };
    let _ = sqlx::query("DELETE FROM user_features WHERE type = 'playlist' AND ref = $1").bind(id).execute(&state.db).await;
    let _ = sqlx::query("DELETE FROM playlists WHERE id = $1").bind(id).execute(&state.db).await;
    ok()
}

#[derive(Deserialize)]
pub struct AddTrackBody {
    pub track_id: i32,
}

/// POST /api/v1/album/{id}/add — add track (must be author's own track)
pub async fn add_track(
    State(state): State<AppState>,
    RequiredSession(sess): RequiredSession,
    Path(id): Path<i32>,
    Json(body): Json<AddTrackBody>,
) -> Response {
    match Playlist::by_id(&state.db, id).await {
        Some(a) if a.is_album && a.author.id == sess.user.id => {},
        _ => return err(StatusCode::FORBIDDEN, "Not your album"),
    };
    // Albums can only contain the author's own tracks
    match Track::by_id(&state.db, body.track_id).await {
        Some(t) if t.artist.id == sess.user.id => {},
        _ => return err(StatusCode::BAD_REQUEST, "You can only add your own tracks to an album"),
    };
    let _ = sqlx::query("UPDATE playlists SET track_ids = array_append(track_ids, $1) WHERE id = $2")
        .bind(body.track_id).bind(id).execute(&state.db).await;
    ok()
}

#[derive(Deserialize)]
pub struct RemoveTrackBody {
    pub track_id: i32,
}

/// POST /api/v1/album/{id}/remove
pub async fn remove_track(
    State(state): State<AppState>,
    RequiredSession(sess): RequiredSession,
    Path(id): Path<i32>,
    Json(body): Json<RemoveTrackBody>,
) -> Response {
    match Playlist::by_id(&state.db, id).await {
        Some(a) if a.is_album && a.author.id == sess.user.id => {},
        _ => return err(StatusCode::FORBIDDEN, "Not your album"),
    };
    let _ = sqlx::query("UPDATE playlists SET track_ids = array_remove(track_ids, $1) WHERE id = $2")
        .bind(body.track_id).bind(id).execute(&state.db).await;
    ok()
}
