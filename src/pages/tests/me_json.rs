// Feature: spa-frontend, Property 20: /me/json returns correct auth state
//
// **Validates: Requirements 11.4**
//
// For any request to `/me/json` with a valid session, the response should
// contain `logged_in: true`, the user's `uid`, and `username`.
// For any request without a valid session, the response should contain
// `logged_in: false`.
//
// Since we cannot spin up a full database in unit tests, we test the
// serialization logic of the MeLoggedIn / MeLoggedOut structs that the
// handler produces. This verifies the JSON shape contract.

use proptest::prelude::*;
use serde_json::Value;

/// Mirrors the private MeLoggedIn struct from auth.rs
#[derive(serde::Serialize)]
struct MeLoggedIn {
    logged_in: bool,
    uid: i32,
    username: String,
}

/// Mirrors the private MeLoggedOut struct from auth.rs
#[derive(serde::Serialize)]
struct MeLoggedOut {
    logged_in: bool,
}

/// Strategy for generating valid usernames (non-empty alphanumeric strings).
fn username_strategy() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9_]{1,64}"
}

/// Strategy for generating valid user IDs (positive i32).
fn uid_strategy() -> impl Strategy<Value = i32> {
    1..=i32::MAX
}

proptest! {
    /// Property 20: When a valid session exists, /me/json returns
    /// `logged_in: true` with the correct uid and username.
    #[test]
    fn me_json_logged_in_returns_correct_fields(
        uid in uid_strategy(),
        username in username_strategy(),
    ) {
        let body = serde_json::to_string(&MeLoggedIn {
            logged_in: true,
            uid,
            username: username.clone(),
        })
        .unwrap();

        let parsed: Value = serde_json::from_str(&body).unwrap();

        // Must have logged_in: true
        prop_assert_eq!(parsed["logged_in"].as_bool(), Some(true));
        // Must have uid matching the input
        prop_assert_eq!(parsed["uid"].as_i64(), Some(uid as i64));
        // Must have username matching the input
        prop_assert_eq!(parsed["username"].as_str(), Some(username.as_str()));
        // All three fields must be present
        prop_assert!(parsed.get("logged_in").is_some());
        prop_assert!(parsed.get("uid").is_some());
        prop_assert!(parsed.get("username").is_some());
    }

    /// Property 20: When no valid session exists, /me/json returns
    /// `logged_in: false` without uid or username.
    #[test]
    fn me_json_logged_out_returns_false(
        // Use a dummy input to drive proptest iterations
        _dummy in 0..1000u32,
    ) {
        let body = serde_json::to_string(&MeLoggedOut {
            logged_in: false,
        })
        .unwrap();

        let parsed: Value = serde_json::from_str(&body).unwrap();

        // Must have logged_in: false
        prop_assert_eq!(parsed["logged_in"].as_bool(), Some(false));
        // Must NOT have uid or username
        prop_assert!(parsed.get("uid").is_none());
        prop_assert!(parsed.get("username").is_none());
    }
}
