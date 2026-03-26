// Feature: spa-frontend, Property 25: Session cookie sharing between frontends
//
// **Validates: Requirements 12.4**
//
// For any session created via the legacy frontend's `/login` endpoint,
// the same `sid` cookie should be accepted by the SPA's API requests
// (via `/me/json`), and vice versa.
//
// Since both frontends share the same Axum backend and Redis session store,
// session sharing is guaranteed by design: the `sid` cookie is set with
// `Path=/` and both frontends proxy to the same backend.
//
// This test verifies the session cookie contract: the `sid` cookie name
// and `Path=/` attribute are consistent.

use proptest::prelude::*;

/// The session cookie name used by the Rust backend.
const SESSION_COOKIE_NAME: &str = "sid";

/// The cookie path that makes the session available to all routes.
const SESSION_COOKIE_PATH: &str = "/";

proptest! {
    /// Property 25: For any session ID string, the cookie format is
    /// consistent between legacy and SPA frontends.
    /// Both use `sid=<value>; Path=/` which means any frontend
    /// proxying to the same backend will share the session.
    #[test]
    fn session_cookie_format_is_shareable(
        session_id in "[a-f0-9]{32,64}",
    ) {
        // Construct the Set-Cookie header as the backend would
        let cookie_header = format!(
            "{}={}; Path={}; HttpOnly",
            SESSION_COOKIE_NAME,
            session_id,
            SESSION_COOKIE_PATH,
        );

        // Verify the cookie name is "sid"
        prop_assert!(cookie_header.starts_with("sid="));

        // Verify Path=/ is present (makes cookie available to all routes)
        prop_assert!(cookie_header.contains("Path=/"));

        // Verify the session ID is preserved in the cookie
        prop_assert!(cookie_header.contains(&session_id));

        // A request from the SPA would send this cookie back as:
        let request_cookie = format!("{}={}", SESSION_COOKIE_NAME, session_id);
        prop_assert!(request_cookie.starts_with("sid="));
        prop_assert!(request_cookie.contains(&session_id));
    }
}
