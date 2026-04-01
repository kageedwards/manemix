use axum::extract::State;
use axum::response::{IntoResponse, Response};

use crate::AppState;
use crate::models::event;

/// GET /api/v1/events/recent — latest global events as JSON
pub async fn recent_json(State(state): State<AppState>) -> Response {
    let events = event::recent(&state.db, 15).await;
    let ctx: Vec<_> = events.iter().map(|e| e.context()).collect();
    axum::Json(ctx).into_response()
}
