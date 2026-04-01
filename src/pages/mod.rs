pub mod home;
mod statics;
mod track;
mod tracks;
mod user;
mod users;
mod playlist;
mod favorites;
mod art;
mod oembed;
mod stats;
mod feed;
mod auth;
mod account;
mod password;
mod social;
mod track_actions;
mod playlist_actions;
mod feature_actions;
mod track_api;
mod playlist_api;
mod album_api;
pub mod queue;
mod ticker;
mod events_api;

#[cfg(test)]
mod tests;

use axum::{Router, routing::{get, post}};
use crate::AppState;

/// All routes, mirroring the original's callback table in main.cpp.
pub fn routes() -> Router<AppState> {
    Router::new()
        // Static pages
        .route("/", get(home::home))
        .route("/faq", get(statics::faq))
        .route("/thanks", get(statics::thanks))
        .route("/api", get(statics::api))
        .route("/credits", get(statics::credits_redirect))

        // Auth
        .route("/login", get(auth::login_page).post(auth::login_submit))
        .route("/logout", get(auth::logout))
        .route("/register", get(auth::register_page).post(auth::register_submit))
        .route("/me/json", get(auth::me_json))

        // Account management
        .route("/account", get(account::account_page).post(account::account_submit))
        .route("/account/delete", get(account::delete_page).post(account::delete_submit))
        .route("/account/theme", post(account::theme_submit))
        .route("/account/reset", get(password::reset_page).post(password::reset_submit))
        .route("/account/license", get(track_actions::account_license_page).post(track_actions::account_license_submit))

        // Tracks (read)
        .route("/track/{tid}", get(track::show))
        .route("/track/{tid}/json", get(track::json))
        .route("/track/{tid}/status", get(track::status))
        .route("/track/{tid}/embed", get(track::embed))
        .route("/track/{tid}/mp3", get(track::download_mp3))
        .route("/track/{tid}/vorbis", get(track::download_vorbis))
        .route("/track/{tid}/aac", get(track::download_aac))
        .route("/track/{tid}/opus", get(track::download_opus))
        .route("/track/{tid}/original", get(track::download_original))

        // Track actions (write)
        .route("/track/new", post(track_actions::upload_new))
        .route("/track/{tid}/upload", post(track_actions::upload_replace))
        .route("/track/{tid}/rename", post(track_actions::rename))
        .route("/track/{tid}/tags", post(track_actions::tags))
        .route("/track/{tid}/notes", post(track_actions::notes))
        .route("/track/{tid}/flags", post(track_actions::flags))
        .route("/track/{tid}/publish", post(track_actions::publish))
        .route("/track/{tid}/report", post(track_actions::report))
        .route("/track/{tid}/delete", get(track_actions::delete_page).post(track_actions::delete_submit))
        .route("/track/{tid}/license", get(track_actions::license_page).post(track_actions::license_submit))
        .route("/track/{tid}/played", post(track_actions::played))

        // Art (read + write)
        .route("/track/{tid}/art", get(art::full))
        .route("/track/{tid}/art/medium", get(art::medium))
        .route("/track/{tid}/art/thumb", get(art::thumb))
        .route("/track/{tid}/art/upload", post(track_actions::art_upload))
        .route("/track/{tid}/art/delete", post(track_actions::art_delete))

        // Track lists
        .route("/tracks/latest", get(tracks::latest))
        .route("/tracks/featured", get(tracks::featured))
        .route("/tracks/random", get(tracks::random))
        .route("/tracks/search", get(tracks::search))
        .route("/tracks/tag/{tag}", get(tracks::by_tag))

        // Track list JSON
        .route("/tracks/latest/json", get(tracks::latest_json))
        .route("/tracks/featured/json", get(tracks::featured_json))
        .route("/tracks/random/json", get(tracks::random_json))
        .route("/tracks/search/json", get(tracks::search_json))
        .route("/tracks/search/exact/json", get(tracks::exact_search_json))
        .route("/tracks/all/json", get(tracks::all_json))

        // Feeds
        .route("/tracks/latest/atom", get(feed::latest_atom))
        .route("/tracks/featured/atom", get(feed::featured_atom))

        // Social actions
        .route("/track/{tid}/favorite", post(social::favorite_track))
        .route("/track/{tid}/unfavorite", post(social::unfavorite_track))
        .route("/track/{tid}/comment", post(social::comment_track))
        .route("/track/{tid}/feature", post(feature_actions::feature_track))
        .route("/user/{uid}/follow", post(social::follow_user))
        .route("/user/{uid}/unfollow", post(social::unfollow_user))
        .route("/user/{uid}/comment", post(social::comment_user))
        .route("/user/{uid}/defeature", post(feature_actions::defeature))

        // Users
        .route("/user/{uid}", get(user::show))
        .route("/user/{uid}/json", get(user::json))
        .route("/artists", get(users::artists))
        .route("/artists/json", get(users::artists_json))
        .route("/users/search", get(users::search))
        .route("/users/search/json", get(users::search_json))

        // Favorites
        .route("/user/{uid}/favorites", get(favorites::show))
        .route("/user/{uid}/favorites/json", get(favorites::json))

        // Playlists (read)
        .route("/playlist/{id}", get(playlist::show))
        .route("/playlist/{id}/json", get(playlist::json))

        // Playlist actions (write)
        .route("/playlist/new", post(playlist_actions::create))
        .route("/playlist/{id}/edit", post(playlist_actions::edit))
        .route("/playlist/{id}/delete", get(playlist_actions::delete_page).post(playlist_actions::delete_submit))
        .route("/playlist/{id}/remove", post(playlist_actions::remove_track))
        .route("/playlist/{id}/move", post(playlist_actions::move_track))
        .route("/playlist/{id}/feature", post(feature_actions::feature_playlist))
        .route("/track/{tid}/playlist", post(playlist_actions::add_track))

        // Stats
        .route("/track/{tid}/stats", get(stats::track_stats))
        .route("/user/{uid}/stats", get(stats::user_stats))

        // oEmbed
        .route("/oembed", get(oembed::oembed))

        // --- SPA API (v1) ---
        // Dedicated prefix for the SPA frontend. These are aliases of the
        // existing endpoints above so the legacy frontend keeps working.
        .nest("/api/v1", api_v1_routes())
}

/// JSON + action endpoints under `/api/v1/` for the SPA frontend.
/// Keeps the proxy config trivial: forward everything under `/api/v1/` to the backend.
fn api_v1_routes() -> Router<AppState> {
    Router::new()
        // Auth
        .route("/me", get(auth::me_json))
        .route("/login", post(auth::login_submit))
        .route("/logout", get(auth::logout))
        .route("/register", post(auth::register_submit))

        // Track (read)
        .route("/track/{tid}", get(track::json))
        .route("/track/{tid}/status", get(track::status))

        // Track streaming / downloads
        .route("/track/{tid}/mp3", get(track::download_mp3))
        .route("/track/{tid}/vorbis", get(track::download_vorbis))
        .route("/track/{tid}/aac", get(track::download_aac))
        .route("/track/{tid}/opus", get(track::download_opus))
        .route("/track/{tid}/original", get(track::download_original))

        // Track art
        .route("/track/{tid}/art", get(art::full))
        .route("/track/{tid}/art/medium", get(art::medium))
        .route("/track/{tid}/art/thumb", get(art::thumb))

        // Track actions (dedicated JSON API)
        .route("/track/new", post(track_actions::upload_new))
        .route("/track/{tid}/upload", post(track_actions::upload_replace))
        .route("/track/{tid}/rename", post(track_api::rename))
        .route("/track/{tid}/tags", post(track_api::tags))
        .route("/track/{tid}/notes", post(track_api::notes))
        .route("/track/{tid}/flags", post(track_actions::flags))
        .route("/track/{tid}/publish", post(track_api::publish))
        .route("/track/{tid}/delete", post(track_api::delete))
        .route("/track/{tid}/license", post(track_api::license))
        .route("/track/{tid}/played", post(track_actions::played))
        .route("/track/{tid}/art/upload", post(track_actions::art_upload))
        .route("/track/{tid}/art/delete", post(track_api::art_delete))

        // Social actions
        .route("/track/{tid}/favorite", post(social::favorite_track))
        .route("/track/{tid}/unfavorite", post(social::unfavorite_track))
        .route("/track/{tid}/comment", post(social::comment_track_json))
        .route("/track/{tid}/feature", post(feature_actions::feature_track))
        .route("/user/{uid}/follow", post(social::follow_user))
        .route("/user/{uid}/unfollow", post(social::unfollow_user))
        .route("/user/{uid}/comment", post(social::comment_user_json))

        // Track lists
        .route("/tracks/latest", get(tracks::latest_json))
        .route("/tracks/featured", get(tracks::featured_json))
        .route("/tracks/random", get(tracks::random_json))
        .route("/tracks/search", get(tracks::search_json))
        .route("/tracks/search/exact", get(tracks::exact_search_json))
        .route("/tracks/all", get(tracks::all_json))
        .route("/tracks/tag/{tag}", get(tracks::by_tag))

        // Users
        .route("/user/{uid}", get(user::json))
        .route("/user/{uid}/favorites", get(favorites::json))
        .route("/artists", get(users::artists_json))
        .route("/users/search", get(users::search_json))

        // Playlists (dedicated JSON API)
        .route("/playlists", get(playlist_api::list_public))
        .route("/playlist/{id}", get(playlist_api::get_playlist))
        .route("/playlist/new", post(playlist_api::create))
        .route("/playlist/{id}/edit", post(playlist_api::edit))
        .route("/playlist/{id}/delete", post(playlist_api::delete))
        .route("/playlist/{id}/add", post(playlist_api::add_track))
        .route("/playlist/{id}/remove", post(playlist_api::remove_track))

        // Albums (dedicated JSON API)
        .route("/albums", get(album_api::list_public))
        .route("/album/{id}", get(album_api::get_album))
        .route("/album/new", post(album_api::create))
        .route("/album/{id}/edit", post(album_api::edit))
        .route("/album/{id}/publish", post(album_api::publish))
        .route("/album/{id}/unpublish", post(album_api::unpublish))
        .route("/album/{id}/delete", post(album_api::delete))
        .route("/album/{id}/add", post(album_api::add_track))
        .route("/album/{id}/remove", post(album_api::remove_track))

        // Account
        .route("/account", get(account::account_json).post(account::account_submit_json))
        .route("/account/delete", post(account::delete_submit))
        .route("/account/theme", post(account::theme_submit))
        .route("/account/license", post(track_actions::account_license_submit))

        // Stats
        .route("/track/{tid}/stats", get(stats::track_stats))
        .route("/user/{uid}/stats", get(stats::user_stats))

        // Playback queue
        .route("/queue/next", post(queue::next_tracks))

        // Ticker & activity
        .route("/ticker", get(ticker::ticker_json))
        .route("/events/recent", get(events_api::recent_json))

        // oEmbed
        .route("/oembed", get(oembed::oembed))
}
