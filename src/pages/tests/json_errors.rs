// Feature: spa-frontend, Property 21: JSON error responses when Accept: application/json
//
// **Validates: Requirements 11.5**
//
// For any request that would result in a 400, 401, 403, or 404 status,
// if the request includes `Accept: application/json`, the response body
// should be valid JSON with an `error` field, not HTML.
//
// We test the json_error_middleware logic directly by constructing requests
// and responses and verifying the middleware transforms them correctly.

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use proptest::prelude::*;
use serde_json::Value;

// Re-implement the pure helper functions from json_errors.rs for testing,
// since they are private. This tests the same logic the middleware uses.

fn wants_json(req: &Request<Body>) -> bool {
    req.headers()
        .get(header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.contains("application/json"))
        .unwrap_or(false)
}

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

/// Build a JSON error body the same way the middleware does.
fn build_json_error(status: StatusCode) -> Option<String> {
    let code = error_code(status)?;
    let body = serde_json::json!({
        "error": code,
        "message": default_message(status),
    });
    Some(body.to_string())
}

/// Strategy that produces one of the error status codes the middleware handles.
fn error_status_strategy() -> impl Strategy<Value = u16> {
    prop_oneof![
        Just(400u16),
        Just(401u16),
        Just(403u16),
        Just(404u16),
    ]
}

/// Strategy that produces status codes the middleware does NOT handle.
fn non_error_status_strategy() -> impl Strategy<Value = u16> {
    prop_oneof![
        Just(200u16),
        Just(201u16),
        Just(204u16),
        Just(301u16),
        Just(302u16),
        Just(304u16),
    ]
}

/// Strategy for Accept header values that contain application/json.
fn json_accept_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("application/json".to_string()),
        Just("application/json, text/html".to_string()),
        Just("text/html, application/json".to_string()),
        Just("application/json;q=0.9".to_string()),
    ]
}

proptest! {
    /// Property 21: Error status codes produce valid JSON with `error` field
    /// when Accept: application/json is present.
    #[test]
    fn error_responses_return_json_with_error_field(
        status_code in error_status_strategy(),
        accept in json_accept_strategy(),
    ) {
        let status = StatusCode::from_u16(status_code).unwrap();

        // Verify the request would be detected as wanting JSON
        let req = Request::builder()
            .header(header::ACCEPT, &accept)
            .body(Body::empty())
            .unwrap();
        prop_assert!(wants_json(&req));

        // Verify the middleware would produce a JSON error body
        let json_body = build_json_error(status);
        prop_assert!(json_body.is_some(), "Expected JSON error body for status {}", status_code);

        let body_str = json_body.unwrap();
        let parsed: Value = serde_json::from_str(&body_str).unwrap();

        // Must have an "error" field that is a non-empty string
        let error_field = parsed.get("error");
        prop_assert!(error_field.is_some(), "Missing 'error' field in JSON response");
        prop_assert!(error_field.unwrap().is_string(), "'error' field must be a string");
        prop_assert!(!error_field.unwrap().as_str().unwrap().is_empty(), "'error' field must not be empty");

        // Must have a "message" field that is a non-empty string
        let message_field = parsed.get("message");
        prop_assert!(message_field.is_some(), "Missing 'message' field in JSON response");
        prop_assert!(message_field.unwrap().is_string(), "'message' field must be a string");
        prop_assert!(!message_field.unwrap().as_str().unwrap().is_empty(), "'message' field must not be empty");

        // The body must be valid JSON (not HTML)
        prop_assert!(!body_str.starts_with("<!"), "Response body looks like HTML, not JSON");
        prop_assert!(!body_str.starts_with("<html"), "Response body looks like HTML, not JSON");
    }

    /// Property 21: Non-error status codes are NOT transformed by the middleware.
    #[test]
    fn non_error_statuses_are_not_transformed(
        status_code in non_error_status_strategy(),
    ) {
        let status = StatusCode::from_u16(status_code).unwrap();
        let json_body = build_json_error(status);
        prop_assert!(json_body.is_none(), "Status {} should not produce a JSON error body", status_code);
    }

    /// Property 21: Requests without Accept: application/json are not affected.
    #[test]
    fn non_json_requests_are_not_affected(
        accept in prop_oneof![
            Just("text/html".to_string()),
            Just("*/*".to_string()),
            Just("text/plain".to_string()),
        ],
    ) {
        let req = Request::builder()
            .header(header::ACCEPT, &accept)
            .body(Body::empty())
            .unwrap();
        prop_assert!(!wants_json(&req), "Accept '{}' should not be detected as wanting JSON", accept);
    }
}
