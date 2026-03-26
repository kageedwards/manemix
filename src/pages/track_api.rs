//! Dedicated JSON API endpoints for the SPA track editor.
//! No nonces, no redirects — just JSON request/response.

use axum::extract::{Multipart, Path, State};
use axum::response::{IntoResponse, Response};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

use crate::AppState;
use crate::models::audio::Audio;
use crate::models::track::Track;
use crate::session::RequiredSession;

fn ok() -> Response {
    Json(serde_json::json!({ "ok": true })).into_response()
}

fn err(status: StatusCode, msg: &str) -> Response {
    (status, Json(serde_json::json!({ "error": msg }))).into_response()
}

#[derive(Deserialize)]
pub struct RenameBody {
    pub title: String,
}

pub async fn rename(
    State(state): State<AppState>,
    RequiredSession(sess): RequiredSession,
    Path(tid): Path<i32>,
    Json(body): Json<RenameBody>,
) -> Response {
    let t = match Track::by_id(&state.db, tid).await {
        Some(t) if t.artist.id == sess.user.id => t,
        _ => return err(StatusCode::FORBIDDEN, "Not your track"),
    };
    if body.title.is_empty() {
        return err(StatusCode::BAD_REQUEST, "Title cannot be empty");
    }
    let _ = sqlx::query("UPDATE tracks SET title = $1 WHERE id = $2")
        .bind(&body.title).bind(tid).execute(&state.db).await;
    let t2 = Track { title: body.title, ..t };
    Audio::new(&t2, &state.manemix_dir).update_tags();
    ok()
}

#[derive(Deserialize)]
pub struct TagsBody {
    pub tags: String,
}

pub async fn tags(
    State(state): State<AppState>,
    RequiredSession(sess): RequiredSession,
    Path(tid): Path<i32>,
    Json(body): Json<TagsBody>,
) -> Response {
    match Track::by_id(&state.db, tid).await {
        Some(t) if t.artist.id == sess.user.id => {},
        _ => return err(StatusCode::FORBIDDEN, "Not your track"),
    };
    let _ = sqlx::query(
        "UPDATE tracks SET tags = regexp_split_to_array(lower($1), E' *, *') WHERE id = $2"
    ).bind(&body.tags).bind(tid).execute(&state.db).await;
    ok()
}

#[derive(Deserialize)]
pub struct NotesBody {
    pub notes: String,
}

pub async fn notes(
    State(state): State<AppState>,
    RequiredSession(sess): RequiredSession,
    Path(tid): Path<i32>,
    Json(body): Json<NotesBody>,
) -> Response {
    match Track::by_id(&state.db, tid).await {
        Some(t) if t.artist.id == sess.user.id => {},
        _ => return err(StatusCode::FORBIDDEN, "Not your track"),
    };
    let _ = sqlx::query("UPDATE tracks SET notes = $1 WHERE id = $2")
        .bind(&body.notes).bind(tid).execute(&state.db).await;
    ok()
}

#[derive(Deserialize)]
pub struct LicenseBody {
    pub license: String,
    #[serde(default)]
    pub mkdefault: bool,
    #[serde(default)]
    pub retro: bool,
}

pub async fn license(
    State(state): State<AppState>,
    RequiredSession(sess): RequiredSession,
    Path(tid): Path<i32>,
    Json(body): Json<LicenseBody>,
) -> Response {
    match Track::by_id(&state.db, tid).await {
        Some(t) if t.artist.id == sess.user.id => {},
        _ => return err(StatusCode::FORBIDDEN, "Not your track"),
    };
    if body.license.is_empty() {
        return err(StatusCode::BAD_REQUEST, "License cannot be empty");
    }
    if body.mkdefault {
        let _ = sqlx::query("UPDATE users SET license = $1 WHERE id = $2")
            .bind(&body.license).bind(sess.user.id).execute(&state.db).await;
    }
    if body.retro {
        let _ = sqlx::query("UPDATE tracks SET license = $1 WHERE user_id = $2")
            .bind(&body.license).bind(sess.user.id).execute(&state.db).await;
    } else {
        let _ = sqlx::query("UPDATE tracks SET license = $1 WHERE id = $2")
            .bind(&body.license).bind(tid).execute(&state.db).await;
    }
    ok()
}

pub async fn publish(
    State(state): State<AppState>,
    RequiredSession(sess): RequiredSession,
    Path(tid): Path<i32>,
) -> Response {
    let t = match Track::by_id(&state.db, tid).await {
        Some(t) if t.artist.id == sess.user.id && !t.visible => t,
        _ => return err(StatusCode::FORBIDDEN, "Cannot publish"),
    };
    let _ = sqlx::query("UPDATE tracks SET visible = 't', date = now() WHERE id = $1")
        .bind(tid).execute(&state.db).await;
    crate::models::event::push_event(&state.db, "publish", &sess.user, &t.artist, Some(&t)).await;
    ok()
}

pub async fn delete(
    State(state): State<AppState>,
    RequiredSession(sess): RequiredSession,
    Path(tid): Path<i32>,
) -> Response {
    let t = match Track::by_id(&state.db, tid).await {
        Some(t) if t.artist.id == sess.user.id => t,
        _ => return err(StatusCode::FORBIDDEN, "Not your track"),
    };
    let _ = sqlx::query("DELETE FROM favorites WHERE type = 'track' AND ref = $1").bind(tid).execute(&state.db).await;
    let _ = sqlx::query("DELETE FROM featured_tracks WHERE track_id = $1").bind(tid).execute(&state.db).await;
    let _ = sqlx::query("DELETE FROM user_features WHERE type = 'track' AND ref = $1").bind(tid).execute(&state.db).await;
    let _ = sqlx::query("DELETE FROM events WHERE track_id = $1").bind(tid).execute(&state.db).await;
    let _ = sqlx::query("UPDATE playlists SET track_ids = array_remove(track_ids, $1)").bind(tid).execute(&state.db).await;
    let _ = sqlx::query("DELETE FROM tracks WHERE id = $1").bind(tid).execute(&state.db).await;
    Audio::new(&t, &state.manemix_dir).unlink();
    let _ = std::fs::remove_file(format!("{}/art/{}", state.manemix_dir, tid));
    let _ = std::fs::remove_file(format!("{}/art/medium/{}.jpg", state.manemix_dir, tid));
    let _ = std::fs::remove_file(format!("{}/art/thumb/{}.png", state.manemix_dir, tid));
    ok()
}

pub async fn art_delete(
    State(state): State<AppState>,
    RequiredSession(sess): RequiredSession,
    Path(tid): Path<i32>,
) -> Response {
    match Track::by_id(&state.db, tid).await {
        Some(t) if t.artist.id == sess.user.id => {},
        _ => return err(StatusCode::FORBIDDEN, "Not your track"),
    };
    let _ = std::fs::remove_file(format!("{}/art/{}", state.manemix_dir, tid));
    let _ = std::fs::remove_file(format!("{}/art/medium/{}.jpg", state.manemix_dir, tid));
    let _ = std::fs::remove_file(format!("{}/art/thumb/{}.png", state.manemix_dir, tid));
    ok()
}
