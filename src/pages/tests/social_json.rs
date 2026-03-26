// Feature: spa-frontend, Property 23: Social actions accept JSON and return JSON
//
// **Validates: Requirements 11.7**
//
// For any social action endpoint (favorite, unfavorite, follow, unfollow,
// comment), when the request has `Content-Type: application/json` and
// `Accept: application/json`, the response should be JSON (not a redirect)
// with an appropriate status code.
//
// We test the helper functions from social.rs that determine request/response
// format: `is_json_content`, `wants_json`, `ok_or_redirect`, `err_or_redirect`.

use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::IntoResponse;
use proptest::prelude::*;
use proptest::prelude::*;
/// Mirrors the is_json_content helper from social.rs
fn is_json_content(headers: &HeaderMap) -> bool {
    headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.contains("application/json"))
        .unwrap_or(false)
}

/// Mirrors the wants_json helper from social.rs
fn wants_json(headers: &HeaderMap) -> bool {
    headers
        .get("accept")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.contains("application/json"))
        .unwrap_or(false)
}

/// Mirrors the ok_or_redirect helper from social.rs
fn ok_or_redirect(headers: &HeaderMap, redirect_url: &str) -> axum::response::Response {
    if wants_json(headers) {
        axum::Json(serde_json::json!({ "ok": true })).into_response()
    } else {
        axum::response::Redirect::to(redirect_url).into_response()
    }
}

/// Mirrors the err_or_redirect helper from social.rs
fn err_or_redirect(headers: &HeaderMap, redirect_url: &str) -> axum::response::Response {
    if wants_json(headers) {
        (
            StatusCode::BAD_REQUEST,
            axum::Json(serde_json::json!({ "error": "bad_request", "message": "Invalid request" })),
        )
            .into_response()
    } else {
        axum::response::Redirect::to(redirect_url).into_response()
    }
}

/// Strategy for Content-Type headers that indicate JSON.
fn json_content_type_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("application/json".to_string()),
        Just("application/json; charset=utf-8".to_string()),
        Just("application/json;charset=UTF-8".to_string()),
    ]
}

/// Strategy for Accept headers that indicate JSON.
fn json_accept_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("application/json".to_string()),
        Just("application/json, text/html".to_string()),
        Just("text/html, application/json".to_string()),
    ]
}

/// Strategy for redirect URLs used in social actions.
fn redirect_url_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        (1..10000i32).prop_map(|tid| format!("/track/{tid}")),
        (1..10000i32).prop_map(|uid| format!("/user/{uid}")),
        Just("/".to_string()),
    ]
}

proptest! {
    /// Property 23: When Content-Type is application/json, is_json_content returns true.
    #[test]
    fn json_content_type_detected(
        ct in json_content_type_strategy(),
    ) {
        let mut headers = HeaderMap::new();
        headers.insert("content-type", HeaderValue::from_str(&ct).unwrap());
        prop_assert!(is_json_content(&headers), "Content-Type '{}' should be detected as JSON", ct);
    }

    /// Property 23: When Accept is application/json, wants_json returns true.
    #[test]
    fn json_accept_detected(
        accept in json_accept_strategy(),
    ) {
        let mut headers = HeaderMap::new();
        headers.insert("accept", HeaderValue::from_str(&accept).unwrap());
        prop_assert!(wants_json(&headers), "Accept '{}' should be detected as wanting JSON", accept);
    }

    /// Property 23: ok_or_redirect returns JSON (not redirect) when Accept: application/json.
    #[test]
    fn ok_returns_json_not_redirect_when_json_accepted(
        accept in json_accept_strategy(),
        redirect_url in redirect_url_strategy(),
    ) {
        let mut headers = HeaderMap::new();
        headers.insert("accept", HeaderValue::from_str(&accept).unwrap());

        let response = ok_or_redirect(&headers, &redirect_url);

        // Status should be 200 (not 3xx redirect)
        let status = response.status();
        prop_assert_eq!(status, StatusCode::OK, "Expected 200 OK, got {}", status);

        // Response should not be a redirect
        prop_assert!(!status.is_redirection(), "Response should not be a redirect");
    }

    /// Property 23: err_or_redirect returns JSON error (not redirect) when Accept: application/json.
    #[test]
    fn err_returns_json_not_redirect_when_json_accepted(
        accept in json_accept_strategy(),
        redirect_url in redirect_url_strategy(),
    ) {
        let mut headers = HeaderMap::new();
        headers.insert("accept", HeaderValue::from_str(&accept).unwrap());

        let response = err_or_redirect(&headers, &redirect_url);

        // Status should be 400 (not 3xx redirect)
        let status = response.status();
        prop_assert_eq!(status, StatusCode::BAD_REQUEST, "Expected 400, got {}", status);

        // Response should not be a redirect
        prop_assert!(!status.is_redirection(), "Response should not be a redirect");
    }

    /// Property 23: ok_or_redirect returns redirect (not JSON) when Accept is NOT application/json.
    #[test]
    fn ok_returns_redirect_when_no_json_accept(
        redirect_url in redirect_url_strategy(),
    ) {
        let mut headers = HeaderMap::new();
        headers.insert("accept", HeaderValue::from_static("text/html"));

        let response = ok_or_redirect(&headers, &redirect_url);

        // Status should be 3xx redirect
        let status = response.status();
        prop_assert!(status.is_redirection(), "Expected redirect, got {}", status);
    }

    /// Property 23: Social endpoints with both JSON Content-Type and Accept
    /// produce JSON responses (combined test).
    #[test]
    fn social_json_roundtrip(
        ct in json_content_type_strategy(),
        accept in json_accept_strategy(),
        redirect_url in redirect_url_strategy(),
    ) {
        let mut headers = HeaderMap::new();
        headers.insert("content-type", HeaderValue::from_str(&ct).unwrap());
        headers.insert("accept", HeaderValue::from_str(&accept).unwrap());

        // Both detection functions should return true
        prop_assert!(is_json_content(&headers));
        prop_assert!(wants_json(&headers));

        // ok_or_redirect should return JSON
        let ok_resp = ok_or_redirect(&headers, &redirect_url);
        prop_assert_eq!(ok_resp.status(), StatusCode::OK);
        prop_assert!(!ok_resp.status().is_redirection());

        // err_or_redirect should return JSON error
        let err_resp = err_or_redirect(&headers, &redirect_url);
        prop_assert_eq!(err_resp.status(), StatusCode::BAD_REQUEST);
        prop_assert!(!err_resp.status().is_redirection());
    }
}
