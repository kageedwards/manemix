use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use serde::Deserialize;

use crate::AppState;
use crate::models::audio::Audio;
use crate::models::event;
use crate::models::playlist::Playlist;
use crate::models::track::{ExtendedTrack, Track};
use crate::session::OptionalSession;
use super::home::wrap_page;

/// GET /track/:tid — HTML track page
pub async fn show(
    State(state): State<AppState>,
    OptionalSession(sess, theme): OptionalSession,
    headers: axum::http::HeaderMap,
    Path(tid): Path<i32>,
) -> Response {
    let t = match ExtendedTrack::by_id(&state.db, tid).await {
        Some(t) => t,
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    // Hide unpublished tracks from non-owners
    if !t.track.visible {
        let is_owner = sess.as_ref().map(|s| s.user.id == t.track.artist.id).unwrap_or(false);
        if !is_owner {
            return StatusCode::NOT_FOUND.into_response();
        }
    }

    let audio = Audio::new(&t.track, &state.manemix_dir);
    let events = event::for_track(&state.db, tid).await;

    // Push view stat
    let mut redis = state.redis.clone();
    let remote = headers.get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or("").trim().to_string())
        .unwrap_or_else(|| "127.0.0.1".into());
    crate::models::stat::push(&mut redis, "trackView", t.track.artist.id, tid, &remote, "").await;

    let mut ctx = tera::Context::new();
    ctx.insert("title", &t.track.title);
    ctx.insert("has_title", &true);
    ctx.insert("manemix_url", &state.base_url);
    ctx.insert("track", &t.context(&state.manemix_dir));
    ctx.insert("audio", &audio.status());
    ctx.insert("events", &events.iter().map(|e| e.context()).collect::<Vec<_>>());
    ctx.insert("has_oembed", &true);
    ctx.insert("tid", &tid);

    // Check if current user is the track owner, favorited, has playlists
    if let Some(ref s) = sess {
        let is_owner = s.user.id == t.track.artist.id;
        ctx.insert("is_owner", &is_owner);
        ctx.insert("nonce", &s.nonce);

        let is_fav: bool = sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM favorites WHERE type = 'track' AND ref = $1 AND user_id = $2)"
        ).bind(tid).bind(s.user.id).fetch_one(&state.db).await.unwrap_or(false);
        ctx.insert("is_favorite", &is_fav);

        let playlists = Playlist::for_user(&state.db, s.user.id).await;
        ctx.insert("has_playlists", &!playlists.is_empty());
        ctx.insert("user_playlists", &playlists.iter().map(|p| p.context()).collect::<Vec<_>>());
    }

    let body = state.tera.render("html/track.tpl", &ctx).unwrap_or_default();
    Html(wrap_page(&state, &ctx, &body, sess.as_ref(), &theme)).into_response()
}

/// GET /track/:tid/json
pub async fn json(
    State(state): State<AppState>,
    OptionalSession(sess, _theme): OptionalSession,
    Path(tid): Path<i32>,
) -> Response {
    let t = match ExtendedTrack::by_id(&state.db, tid).await {
        Some(t) => t,
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    // Hide unpublished tracks from non-owners
    if !t.track.visible {
        let is_owner = sess.as_ref().map(|s| s.user.id == t.track.artist.id).unwrap_or(false);
        if !is_owner {
            return StatusCode::NOT_FOUND.into_response();
        }
    }

    let events = event::for_track(&state.db, tid).await;
    let events_ctx: Vec<_> = events.iter().map(|e| e.context()).collect();

    let mut track_json = serde_json::to_value(&t.context(&state.manemix_dir)).unwrap_or_default();
    if let Some(obj) = track_json.as_object_mut() {
        obj.insert("events".into(), serde_json::to_value(&events_ctx).unwrap_or_default());
    }

    axum::Json(track_json).into_response()
}

/// GET /track/:tid/embed
pub async fn embed(
    State(state): State<AppState>,
    Path(tid): Path<i32>,
) -> Html<String> {
    let mut ctx = tera::Context::new();
    ctx.insert("manemix_url", &state.base_url);

    if let Some(t) = Track::by_id(&state.db, tid).await {
        ctx.insert("found", &true);
        ctx.insert("track", &t.context(&state.manemix_dir));
    } else {
        ctx.insert("found", &false);
    }

    Html(state.tera.render("html/player-embed.tpl", &ctx).unwrap_or_default())
}
/// GET /track/:tid/status — lightweight JSON audio status for polling
pub async fn status(
    State(state): State<AppState>,
    Path(tid): Path<i32>,
) -> Response {
    let t = match Track::by_id(&state.db, tid).await {
        Some(t) => t,
        None => return StatusCode::NOT_FOUND.into_response(),
    };
    let audio = Audio::new(&t, &state.manemix_dir);
    axum::Json(audio.status()).into_response()
}

#[derive(Deserialize, Default)]
pub struct StreamQuery {
    stream: Option<String>,
}

// Per-format download wrappers
pub async fn download_mp3(s: State<AppState>, p: Path<i32>, q: Query<StreamQuery>) -> Response { download_fmt(s, p, "mp3", q.stream.is_some()).await }
pub async fn download_vorbis(s: State<AppState>, p: Path<i32>, q: Query<StreamQuery>) -> Response { download_fmt(s, p, "vorbis", q.stream.is_some()).await }
pub async fn download_aac(s: State<AppState>, p: Path<i32>, q: Query<StreamQuery>) -> Response { download_fmt(s, p, "aac", q.stream.is_some()).await }
pub async fn download_opus(s: State<AppState>, p: Path<i32>, q: Query<StreamQuery>) -> Response { download_fmt(s, p, "opus", q.stream.is_some()).await }
pub async fn download_original(s: State<AppState>, p: Path<i32>, q: Query<StreamQuery>) -> Response { download_fmt(s, p, "original", q.stream.is_some()).await }

async fn download_fmt(
    State(state): State<AppState>,
    Path(tid): Path<i32>,
    fmt: &str,
    stream: bool,
) -> Response {
    let t = match Track::by_id(&state.db, tid).await {
        Some(t) => t,
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    let audio = Audio::new(&t, &state.manemix_dir);
    let base_name = format!("{} - {}", t.artist.name, t.title);

    let (accel_path, filename, mime) = match fmt {
        "mp3" => (format!("tracks/{}.mp3", tid), format!("{base_name}.mp3"), "audio/mpeg"),
        "vorbis" => (format!("tracks/{}.ogg", tid), format!("{base_name}.ogg"), "audio/ogg"),
        "aac" => (format!("tracks/{}.m4a", tid), format!("{base_name}.m4a"), "audio/aac"),
        "opus" => (format!("tracks/{}.opus", tid), format!("{base_name}.opus"), "audio/ogg"),
        "original" => {
            match audio.original_path() {
                Some(p) => {
                    let ext = p.extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap_or_default();
                    let fname = p.file_name().unwrap_or_default().to_string_lossy().to_string();
                    (format!("tracks/{fname}"), format!("{base_name}{ext}"), "application/octet-stream")
                }
                None => return StatusCode::NOT_FOUND.into_response(),
            }
        }
        _ => return StatusCode::NOT_FOUND.into_response(),
    };

    let disposition = if stream { "inline" } else { "attachment" };

    // Push download stat (only for non-stream requests)
    if !stream {
        let mut redis = state.redis.clone();
        crate::models::stat::push(&mut redis, "trackDownload", t.artist.id, tid, "", "").await;
    }

    let mut headers = HeaderMap::new();
    headers.insert("X-Accel-Redirect", HeaderValue::from_str(&format!("/downloads/{accel_path}")).unwrap());
    headers.insert("Content-Disposition", HeaderValue::from_str(&format!("{disposition}; filename=\"{filename}\"")).unwrap());
    headers.insert("Content-Type", HeaderValue::from_str(mime).unwrap());
    (headers, "").into_response()
}
