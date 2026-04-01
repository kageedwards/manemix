use axum::extract::State;
use axum::response::{IntoResponse, Response};

use crate::AppState;
use crate::models::ticker;

/// GET /api/v1/ticker — active ticker items as JSON
pub async fn ticker_json(State(state): State<AppState>) -> Response {
    let items = ticker::active(&state.db).await;
    axum::Json(items).into_response()
}
