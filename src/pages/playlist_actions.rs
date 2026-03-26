use axum::extract::{Path, State};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::Form;
use serde::Deserialize;

use crate::AppState;
use crate::models::playlist::Playlist;
use crate::models::session::Session;
use crate::session::RequiredSession;

#[derive(Deserialize)]
pub struct NonceForm {
    nonce: Option<String>,
    #[serde(flatten)]
    extra: std::collections::HashMap<String, String>,
}

// POST /playlist/new
pub async fn create(
    State(state): State<AppState>,
    RequiredSession(sess): RequiredSession,
    Form(form): Form<NonceForm>,
) -> Response {
    let name = form.extra.get("name").cloned().unwrap_or_default();
    if name.is_empty() || form.nonce.as_deref() != Some(&sess.nonce) {
        return Redirect::to(&format!("/user/{}", sess.user.id)).into_response();
    }
    Session::new_nonce(&state.db, &sess.sid).await;

    let pid: Option<i32> = sqlx::query_scalar(
        "INSERT INTO playlists (user_id, name, description, track_ids) \
         VALUES ($1, $2, '', ARRAY[]::int[]) RETURNING id"
    )
    .bind(sess.user.id)
    .bind(&name)
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten();

    match pid {
        Some(pid) => {
            tracing::info!("New playlist: {} ({})", name, pid);
            Redirect::to(&format!("/playlist/{pid}?firstrun=1")).into_response()
        }
        None => Redirect::to(&format!("/user/{}", sess.user.id)).into_response(),
    }
}

// POST /playlist/:id/edit
pub async fn edit(
    State(state): State<AppState>,
    RequiredSession(sess): RequiredSession,
    Path(id): Path<i32>,
    Form(form): Form<NonceForm>,
) -> Response {
    if let Some(p) = Playlist::by_id(&state.db, id).await {
        if p.author.id == sess.user.id && form.nonce.as_deref() == Some(&sess.nonce) {
            Session::new_nonce(&state.db, &sess.sid).await;
            let name = form.extra.get("name").cloned().unwrap_or_default();
            let desc = form.extra.get("desc").cloned().unwrap_or_default();
            if !name.is_empty() {
                let _ = sqlx::query("UPDATE playlists SET name = $1, description = $2 WHERE id = $3")
                    .bind(&name).bind(&desc).bind(id).execute(&state.db).await;
            }
        }
    }
    Redirect::to(&format!("/playlist/{id}")).into_response()
}

// POST /playlist/:id/delete
pub async fn delete_page(
    State(state): State<AppState>,
    RequiredSession(sess): RequiredSession,
    Path(id): Path<i32>,
) -> Response {
    let p = match Playlist::by_id(&state.db, id).await {
        Some(p) if p.author.id == sess.user.id => p,
        _ => return Redirect::to(&format!("/playlist/{id}")).into_response(),
    };
    let new_nonce = Session::new_nonce(&state.db, &sess.sid).await;
    let mut ctx = tera::Context::new();
    ctx.insert("title", "Playlist deletion");
    ctx.insert("has_title", &true);
    ctx.insert("manemix_url", &state.base_url);
    ctx.insert("what", &p.name);
    ctx.insert("cancel_url", &format!("/playlist/{id}"));
    ctx.insert("nonce", &new_nonce);
    ctx.insert("delete_url", &format!("/playlist/{id}/delete"));
    let body = state.tera.render("html/delete.tpl", &ctx).unwrap_or_default();
    let mut sess = sess;
    sess.nonce = new_nonce;
    Html(super::home::wrap_page(&state, &ctx, &body, Some(&sess), &sess.theme)).into_response()
}

#[derive(Deserialize)]
pub struct DeleteForm {
    confirm: Option<String>,
    nonce: Option<String>,
}

pub async fn delete_submit(
    State(state): State<AppState>,
    RequiredSession(sess): RequiredSession,
    Path(id): Path<i32>,
    Form(form): Form<DeleteForm>,
) -> Response {
    if let Some(p) = Playlist::by_id(&state.db, id).await {
        if p.author.id == sess.user.id
            && form.confirm.as_deref() == Some("Delete")
            && form.nonce.as_deref() == Some(&sess.nonce)
        {
            Session::new_nonce(&state.db, &sess.sid).await;
            tracing::info!("Deleting playlist: {} ({})", p.name, id);
            let _ = sqlx::query("DELETE FROM user_features WHERE type = 'playlist' AND ref = $1")
                .bind(id).execute(&state.db).await;
            let _ = sqlx::query("DELETE FROM playlists WHERE id = $1")
                .bind(id).execute(&state.db).await;
            return Redirect::to(&format!("/user/{}", sess.user.id)).into_response();
        }
    }
    Redirect::to(&format!("/playlist/{id}")).into_response()
}

// POST /track/:tid/playlist — add track to playlist
pub async fn add_track(
    State(state): State<AppState>,
    RequiredSession(sess): RequiredSession,
    Path(tid): Path<i32>,
    Form(form): Form<NonceForm>,
) -> Response {
    let playlist_id: i32 = form.extra.get("playlist")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    if let Some(p) = Playlist::by_id(&state.db, playlist_id).await {
        if p.author.id == sess.user.id && form.nonce.as_deref() == Some(&sess.nonce) {
            Session::new_nonce(&state.db, &sess.sid).await;
            if let Some(t) = crate::models::track::Track::by_id(&state.db, tid).await {
                if t.visible {
                    let _ = sqlx::query(
                        "UPDATE playlists SET track_ids = track_ids || $1 WHERE id = $2"
                    ).bind(tid).bind(playlist_id).execute(&state.db).await;
                }
            }
        }
        return Redirect::to(&format!("/playlist/{playlist_id}")).into_response();
    }
    Redirect::to(&format!("/track/{tid}")).into_response()
}

// POST /playlist/:id/remove
pub async fn remove_track(
    State(state): State<AppState>,
    RequiredSession(sess): RequiredSession,
    Path(id): Path<i32>,
    Form(form): Form<NonceForm>,
) -> Response {
    if let Some(p) = Playlist::by_id(&state.db, id).await {
        if p.author.id == sess.user.id && form.nonce.as_deref() == Some(&sess.nonce) {
            Session::new_nonce(&state.db, &sess.sid).await;
            let pos: i32 = form.extra.get("item").and_then(|s| s.parse().ok()).unwrap_or(-1);
            if pos >= 0 {
                // Remove element at position by reconstructing the array
                let _ = sqlx::query(
                    "UPDATE playlists SET track_ids = ( \
                     SELECT array_agg(tid) FROM ( \
                       SELECT unnest(track_ids) AS tid, row_number() OVER () - 1 AS pos \
                       FROM playlists WHERE id = $1 \
                     ) sub WHERE pos != $2 \
                   ) WHERE id = $1"
                ).bind(id).bind(pos).execute(&state.db).await;
            }
        }
    }
    Redirect::to(&format!("/playlist/{id}#tracks")).into_response()
}

// POST /playlist/:id/move
pub async fn move_track(
    State(state): State<AppState>,
    RequiredSession(sess): RequiredSession,
    Path(id): Path<i32>,
    Form(form): Form<NonceForm>,
) -> Response {
    if let Some(p) = Playlist::by_id(&state.db, id).await {
        if p.author.id == sess.user.id && form.nonce.as_deref() == Some(&sess.nonce) {
            Session::new_nonce(&state.db, &sess.sid).await;
            let item: i32 = form.extra.get("item").and_then(|s| s.parse().ok()).unwrap_or(-1);
            let dir = form.extra.get("dir").cloned().unwrap_or_default();
            let swap_with = if dir == "up" { item - 1 } else { item + 1 };
            if item >= 0 && swap_with >= 0 {
                // Swap two positions in the array
                let _ = sqlx::query(
                    "UPDATE playlists SET track_ids = ( \
                     SELECT array_agg(CASE \
                       WHEN pos = $2 THEN (SELECT track_ids[$3+1] FROM playlists WHERE id = $1) \
                       WHEN pos = $3 THEN (SELECT track_ids[$2+1] FROM playlists WHERE id = $1) \
                       ELSE tid END) \
                     FROM (SELECT unnest(track_ids) AS tid, row_number() OVER () - 1 AS pos \
                           FROM playlists WHERE id = $1) sub \
                   ) WHERE id = $1"
                ).bind(id).bind(item).bind(swap_with).execute(&state.db).await;
            }
        }
    }
    Redirect::to(&format!("/playlist/{id}#tracks")).into_response()
}
