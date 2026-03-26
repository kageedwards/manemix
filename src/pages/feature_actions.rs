use axum::extract::{Path, State};
use axum::response::{IntoResponse, Redirect, Response};
use axum::Form;
use serde::Deserialize;

use crate::AppState;
use crate::models::playlist::Playlist;
use crate::models::session::Session;
use crate::models::track::Track;
use crate::session::RequiredSession;

#[derive(Deserialize)]
pub struct NonceForm {
    nonce: Option<String>,
}

// POST /track/:tid/feature
pub async fn feature_track(
    State(state): State<AppState>,
    RequiredSession(sess): RequiredSession,
    Path(tid): Path<i32>,
    Form(form): Form<NonceForm>,
) -> Response {
    let _t = match Track::by_id(&state.db, tid).await {
        Some(t) if t.artist.id == sess.user.id => t,
        _ => return Redirect::to(&format!("/track/{tid}")).into_response(),
    };
    if form.nonce.as_deref() == Some(&sess.nonce) {
        Session::new_nonce(&state.db, &sess.sid).await;
        upsert_feature(&state, sess.user.id, tid, "track").await;
        return Redirect::to(&format!("/user/{}", sess.user.id)).into_response();
    }
    Redirect::to(&format!("/track/{tid}")).into_response()
}

// POST /playlist/:id/feature
pub async fn feature_playlist(
    State(state): State<AppState>,
    RequiredSession(sess): RequiredSession,
    Path(id): Path<i32>,
    Form(form): Form<NonceForm>,
) -> Response {
    let _p = match Playlist::by_id(&state.db, id).await {
        Some(p) if p.author.id == sess.user.id => p,
        _ => return Redirect::to(&format!("/playlist/{id}")).into_response(),
    };
    if form.nonce.as_deref() == Some(&sess.nonce) {
        Session::new_nonce(&state.db, &sess.sid).await;
        upsert_feature(&state, sess.user.id, id, "playlist").await;
        return Redirect::to(&format!("/user/{}", sess.user.id)).into_response();
    }
    Redirect::to(&format!("/playlist/{id}")).into_response()
}

// POST /user/:uid/defeature
pub async fn defeature(
    State(state): State<AppState>,
    RequiredSession(sess): RequiredSession,
    Path(uid): Path<i32>,
    Form(form): Form<NonceForm>,
) -> Response {
    if uid == sess.user.id && form.nonce.as_deref() == Some(&sess.nonce) {
        Session::new_nonce(&state.db, &sess.sid).await;
        let _ = sqlx::query("DELETE FROM user_features WHERE user_id = $1")
            .bind(uid).execute(&state.db).await;
    }
    Redirect::to(&format!("/user/{uid}")).into_response()
}

async fn upsert_feature(state: &AppState, uid: i32, ref_id: i32, feat_type: &str) {
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM user_features WHERE user_id = $1)"
    ).bind(uid).fetch_one(&state.db).await.unwrap_or(false);

    if exists {
        let _ = sqlx::query(
            "UPDATE user_features SET ref = $1, type = $2::feature_type WHERE user_id = $3"
        ).bind(ref_id).bind(feat_type).bind(uid).execute(&state.db).await;
    } else {
        let _ = sqlx::query(
            "INSERT INTO user_features (ref, type, user_id) VALUES ($1, $2::feature_type, $3)"
        ).bind(ref_id).bind(feat_type).bind(uid).execute(&state.db).await;
    }
}
