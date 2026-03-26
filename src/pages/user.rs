use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use serde::Deserialize;

use crate::AppState;
use crate::models::account::Account;
use crate::models::event;
use crate::models::feature::Feature;
use crate::models::playlist::Playlist;
use crate::models::track;
use crate::session::OptionalSession;
use super::home::wrap_page;

#[derive(Deserialize)]
pub struct UserQuery {
    welcome: Option<String>,
}

/// GET /user/:uid — HTML user page
pub async fn show(
    State(state): State<AppState>,
    OptionalSession(sess, theme): OptionalSession,
    Path(uid): Path<i32>,
    Query(q): Query<UserQuery>,
) -> Response {
    let u = match Account::by_id(&state.db, uid).await {
        Some(u) => u,
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    let is_self = sess.as_ref().map(|s| s.user.id == uid).unwrap_or(false);
    let tracks = track::by_user(&state.db, uid, is_self).await;
    let playlists = Playlist::for_user(&state.db, uid).await;
    let events = event::for_user(&state.db, uid, 12).await;
    let feature = Feature::for_user(&state.db, uid, &state.manemix_dir).await;

    let mut ctx = tera::Context::new();
    ctx.insert("title", &u.user.name);
    ctx.insert("has_title", &true);
    ctx.insert("manemix_url", &state.base_url);
    ctx.insert("account", &u.context());
    ctx.insert("is_self", &is_self);
    ctx.insert("welcome", &q.welcome.is_some());
    ctx.insert("tracks", &tracks.iter().map(|t| t.context(&state.manemix_dir)).collect::<Vec<_>>());

    // Check if logged-in user follows this user
    if let Some(ref s) = sess {
        ctx.insert("nonce", &s.nonce);
        if !is_self {
            let is_followed: bool = sqlx::query_scalar(
                "SELECT EXISTS (SELECT 1 FROM favorites WHERE type = 'artist' AND ref = $1 AND user_id = $2)"
            ).bind(uid).bind(s.user.id).fetch_one(&state.db).await.unwrap_or(false);
            ctx.insert("is_followed", &is_followed);
        }
    }
    ctx.insert("playlists", &playlists.iter().map(|p| p.context()).collect::<Vec<_>>());
    ctx.insert("has_playlists", &!playlists.is_empty());
    ctx.insert("events", &events.iter().map(|e| e.context()).collect::<Vec<_>>());
    ctx.insert("feature", &feature.context(&state.manemix_dir));

    let body = state.tera.render("html/user.tpl", &ctx).unwrap_or_default();
    Html(wrap_page(&state, &ctx, &body, sess.as_ref(), &theme)).into_response()
}

/// GET /user/:uid/json
pub async fn json(
    State(state): State<AppState>,
    OptionalSession(sess, _theme): OptionalSession,
    Path(uid): Path<i32>,
) -> Response {
    let u = match Account::by_id(&state.db, uid).await {
        Some(u) => u,
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    let is_self = sess.as_ref().map(|s| s.user.id == uid).unwrap_or(false);
    let tracks = track::by_user(&state.db, uid, is_self).await;
    let all_playlists = Playlist::for_user(&state.db, uid).await;
    let playlists: Vec<_> = all_playlists.iter().filter(|p| !p.is_album).collect();
    let albums: Vec<_> = all_playlists.iter().filter(|p| p.is_album).collect();
    let events = event::for_user(&state.db, uid, 12).await;

    let body = serde_json::json!({
        "uid": u.user.id,
        "username": u.user.name,
        "email_md5": u.context().email_md5,
        "about": u.about,
        "about_html": crate::text::format_bbcode(&u.about),
        "has_about": !u.about.is_empty(),
        "num_favs": u.num_favs,
        "has_favs": u.num_favs > 0,
        "num_followers": u.num_followers,
        "has_followers": u.num_followers > 0,
        "tracks": tracks.iter().map(|t| t.context(&state.manemix_dir)).collect::<Vec<_>>(),
        "playlists": playlists.iter().map(|p| p.context()).collect::<Vec<_>>(),
        "albums": albums.iter().map(|a| a.context()).collect::<Vec<_>>(),
        "events": events.iter().map(|e| e.context()).collect::<Vec<_>>(),
    });

    axum::Json(body).into_response()
}
