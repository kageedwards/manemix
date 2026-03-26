use axum::extract::{Path, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};

use crate::AppState;
use crate::models::art::Art;
use crate::models::track::Track;

/// Serve art via X-Accel-Redirect, mirroring the original's nginx internal redirect.
pub async fn full(State(state): State<AppState>, Path(tid): Path<i32>) -> Response {
    serve_art(&state, tid, "full").await
}

pub async fn medium(State(state): State<AppState>, Path(tid): Path<i32>) -> Response {
    serve_art(&state, tid, "medium").await
}

pub async fn thumb(State(state): State<AppState>, Path(tid): Path<i32>) -> Response {
    serve_art(&state, tid, "thumb").await
}

async fn serve_art(state: &AppState, tid: i32, size: &str) -> Response {
    let t = match Track::by_id(&state.db, tid).await {
        Some(t) => t,
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    let art = match Art::new(&state.manemix_dir, tid) {
        Some(a) => a,
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    let base = format!("{} - {}", t.artist.name, t.title);
    let (accel, filename) = match size {
        "medium" => {
            // Fall back to full-size if no medium thumbnail (small images, GIFs)
            if art.medium_path().exists() {
                (format!("art/medium/{tid}.jpg"), format!("{base}.medium.jpg"))
            } else {
                (format!("art/{tid}"), base.clone())
            }
        }
        "thumb" => {
            if art.thumb_path().exists() {
                (format!("art/thumb/{tid}.png"), format!("{base}.thumb.png"))
            } else {
                (format!("art/{tid}"), base.clone())
            }
        }
        _ => (format!("art/{tid}"), base),
    };

    let mut headers = HeaderMap::new();
    headers.insert("X-Accel-Redirect", HeaderValue::from_str(&format!("/downloads/{accel}")).unwrap());
    headers.insert("Content-Disposition", HeaderValue::from_str(&format!("inline; filename=\"{filename}\"")).unwrap());
    headers.insert("Content-Type", HeaderValue::from_static(art.mime()));
    (headers, "").into_response()
}
