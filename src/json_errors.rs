//! Middleware that converts error responses to JSON when the client sends
//! `Accept: application/json`.
//!
//! For 400, 401, 403, 404, and 500 status codes the body is replaced with:
//! ```json
//! { "error": "<code>", "message": "<human-readable>" }
//! ```

use axum::{
    body::Body,
    http::{header, Request, Response, StatusCode},
    middleware::Next,
};

/// Returns `true` when the Accept header contains `application/json`.
fn wants_json(req: &Request<Body>) -> bool {
    req.headers()
        .get(header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.contains("application/json"))
        .unwrap_or(false)
}

/// Map a status code to the short error code string used in the JSON body.
fn error_code(status: StatusCode) -> Option<&'static str> {
    match status {
        StatusCode::BAD_REQUEST => Some("bad_request"),
        StatusCode::UNAUTHORIZED => Some("unauthorized"),
        StatusCode::FORBIDDEN => Some("forbidden"),
        StatusCode::NOT_FOUND => Some("not_found"),
        StatusCode::INTERNAL_SERVER_ERROR => Some("internal_error"),
        _ => None,
    }
}

/// Human-readable default message for each handled status.
fn default_message(status: StatusCode) -> &'static str {
    match status {
        StatusCode::BAD_REQUEST => "Bad request",
        StatusCode::UNAUTHORIZED => "Unauthorized",
        StatusCode::FORBIDDEN => "Forbidden",
        StatusCode::NOT_FOUND => "Not found",
        StatusCode::INTERNAL_SERVER_ERROR => "Internal server error",
        _ => "Error",
    }
}

/// Axum middleware function.
///
/// Must be used with `axum::middleware::from_fn`.
pub async fn json_error_middleware(
    req: Request<Body>,
    next: Next,
) -> Response<Body> {
    let json = wants_json(&req);
    let mut response = next.run(req).await;

    if !json {
        return response;
    }

    let status = response.status();
    let code = match error_code(status) {
        Some(c) => c,
        None => return response,
    };

    // Check if the response already has a JSON content-type — if so, the
    // handler already produced a proper JSON error body and we should not
    // overwrite it.
    let already_json = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.contains("application/json"))
        .unwrap_or(false);

    if already_json {
        return response;
    }

    // Build the JSON error body.
    let body = serde_json::json!({
        "error": code,
        "message": default_message(status),
    });

    // Replace the body and set the content-type header.
    *response.body_mut() = Body::from(body.to_string());
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("application/json"),
    );

    response
}
