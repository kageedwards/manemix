use axum::extract::{Path, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};

use crate::AppState;
use crate::models::stat;
use crate::models::track::Track;
use crate::models::user::User;

/// GET /track/:tid/stats
pub async fn track_stats(
    State(state): State<AppState>,
    Path(tid): Path<i32>,
) -> Response {
    let t = match Track::by_id(&state.db, tid).await {
        Some(t) => t,
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    let mut redis = state.redis.clone();
    let stats = stat::track_stats(&mut redis, t.artist.id, tid, true).await;

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", HeaderValue::from_static("application/json"));
    (headers, serde_json::to_string(&stats).unwrap_or_default()).into_response()
}

/// GET /user/:uid/stats
pub async fn user_stats(
    State(state): State<AppState>,
    Path(uid): Path<i32>,
) -> Response {
    let _u = match User::by_id(&state.db, uid).await {
        Some(u) => u,
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    let mut redis = state.redis.clone();
    let stats = stat::track_stats(&mut redis, uid, 0, false).await;

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", HeaderValue::from_static("application/json"));
    (headers, serde_json::to_string(&stats).unwrap_or_default()).into_response()
}
