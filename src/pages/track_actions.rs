use axum::extract::{Multipart, Path, State};
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Json, Redirect, Response};
use axum::Form;
use serde::Deserialize;
use std::io::Write;

use crate::AppState;
use crate::models::audio::Audio;
use crate::models::session::Session;
use crate::models::track::Track;
use crate::session::RequiredSession;

/// Returns `true` when the Accept header indicates JSON.
fn wants_json(headers: &HeaderMap) -> bool {
    headers.get("accept").and_then(|v| v.to_str().ok()).map(|v| v.contains("application/json")).unwrap_or(false)
}

/// Returns JSON `{ "ok": true }` when Accept: application/json, otherwise a redirect.
fn ok_or_redirect(headers: &HeaderMap, url: &str) -> Response {
    if wants_json(headers) {
        Json(serde_json::json!({ "ok": true })).into_response()
    } else {
        Redirect::to(url).into_response()
    }
}

/// Check nonce: skip for JSON requests (session cookie is CSRF protection), require for form submissions.
fn nonce_ok(headers: &HeaderMap, form_nonce: Option<&str>, session_nonce: &str) -> bool {
    if wants_json(headers) {
        return true; // SPA uses session cookie for CSRF
    }
    form_nonce == Some(session_nonce)
}

// ---------------------------------------------------------------------------
// Upload (POST /track/new and POST /track/:tid/upload)
// ---------------------------------------------------------------------------

pub async fn upload_new(
    State(state): State<AppState>,
    RequiredSession(sess): RequiredSession,
    headers: axum::http::HeaderMap,
    mut multipart: Multipart,
) -> Response {
    let mut file_data: Vec<u8> = Vec::new();
    let mut filename = String::new();

    while let Ok(Some(field)) = multipart.next_field().await {
        if field.name() == Some("qqfile") {
            filename = field.file_name().unwrap_or("upload").to_string();
            file_data = field.bytes().await.unwrap_or_default().to_vec();
        }
    }

    if file_data.is_empty() {
        return Redirect::to(&format!("/user/{}", sess.user.id)).into_response();
    }

    let title = extract_title_from_filename(&filename);

    // Get user's default license
    let license: String = sqlx::query_scalar("SELECT license FROM users WHERE id = $1")
        .bind(sess.user.id)
        .fetch_one(&state.db)
        .await
        .unwrap_or_else(|_| "Copyright".into());

    let tid: Option<i32> = sqlx::query_scalar(
        "INSERT INTO tracks (user_id, title, date, license) VALUES ($1, $2, now(), $3) RETURNING id"
    )
    .bind(sess.user.id)
    .bind(&title)
    .bind(&license)
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten();

    let is_ajax = headers.get("x-requested-with")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.eq_ignore_ascii_case("xmlhttprequest"))
        .unwrap_or(false);

    match tid {
        Some(tid) => {
            save_and_transcode(&state, tid, &file_data, &filename);
            tracing::info!("Track uploaded: {} ({})", title, tid);
            if is_ajax {
                Json(serde_json::json!({"success": true, "tid": tid, "title": title})).into_response()
            } else {
                Redirect::to(&format!("/track/{tid}")).into_response()
            }
        }
        None => {
            if is_ajax {
                Json(serde_json::json!({"success": false})).into_response()
            } else {
                Redirect::to(&format!("/user/{}", sess.user.id)).into_response()
            }
        }
    }
}

pub async fn upload_replace(
    State(state): State<AppState>,
    RequiredSession(sess): RequiredSession,
    Path(tid): Path<i32>,
    mut multipart: Multipart,
) -> Response {
    let t = match Track::by_id(&state.db, tid).await {
        Some(t) if t.artist.id == sess.user.id => t,
        _ => return Redirect::to("/").into_response(),
    };

    let mut file_data: Vec<u8> = Vec::new();
    let mut filename = String::new();

    while let Ok(Some(field)) = multipart.next_field().await {
        if field.name() == Some("qqfile") {
            filename = field.file_name().unwrap_or("upload").to_string();
            file_data = field.bytes().await.unwrap_or_default().to_vec();
        }
    }

    if file_data.is_empty() {
        return Redirect::to(&format!("/track/{tid}")).into_response();
    }

    save_and_transcode(&state, tid, &file_data, &filename);
    Json(serde_json::json!({"success": true, "tid": tid, "title": t.title})).into_response()
}

// ---------------------------------------------------------------------------
// Track metadata actions
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct NonceForm {
    nonce: Option<String>,
    #[serde(flatten)]
    extra: std::collections::HashMap<String, String>,
}

pub async fn rename(
    State(state): State<AppState>,
    RequiredSession(sess): RequiredSession,
    Path(tid): Path<i32>,
    headers: HeaderMap,
    Form(form): Form<NonceForm>,
) -> Response {
    let t = match Track::by_id(&state.db, tid).await {
        Some(t) if t.artist.id == sess.user.id => t,
        _ => return Redirect::to(&format!("/track/{tid}")).into_response(),
    };
    if nonce_ok(&headers, form.nonce.as_deref(), &sess.nonce) {
        Session::new_nonce(&state.db, &sess.sid).await;
        if let Some(title) = form.extra.get("title").filter(|t| !t.is_empty()) {
            let _ = sqlx::query("UPDATE tracks SET title = $1 WHERE id = $2")
                .bind(title).bind(tid).execute(&state.db).await;
            tracing::info!("Renaming track {} from \"{}\" to \"{}\"", tid, t.title, title);
            let t2 = Track { title: title.clone(), ..t };
            Audio::new(&t2, &state.manemix_dir).update_tags();
        }
    }
    ok_or_redirect(&headers, &format!("/track/{tid}"))
}

pub async fn tags(
    State(state): State<AppState>,
    RequiredSession(sess): RequiredSession,
    Path(tid): Path<i32>,
    headers: HeaderMap,
    Form(form): Form<NonceForm>,
) -> Response {
    if let Some(t) = Track::by_id(&state.db, tid).await {
        if t.artist.id == sess.user.id && nonce_ok(&headers, form.nonce.as_deref(), &sess.nonce) {
            Session::new_nonce(&state.db, &sess.sid).await;
            if let Some(tags_str) = form.extra.get("tags") {
                let _ = sqlx::query(
                    "UPDATE tracks SET tags = regexp_split_to_array(lower($1), E' *, *') WHERE id = $2"
                ).bind(tags_str).bind(tid).execute(&state.db).await;
            }
        }
    }
    ok_or_redirect(&headers, &format!("/track/{tid}"))
}

pub async fn notes(
    State(state): State<AppState>,
    RequiredSession(sess): RequiredSession,
    Path(tid): Path<i32>,
    headers: HeaderMap,
    Form(form): Form<NonceForm>,
) -> Response {
    if let Some(t) = Track::by_id(&state.db, tid).await {
        if t.artist.id == sess.user.id && nonce_ok(&headers, form.nonce.as_deref(), &sess.nonce) {
            Session::new_nonce(&state.db, &sess.sid).await;
            let notes_val = form.extra.get("notes").cloned().unwrap_or_default();
            let _ = sqlx::query("UPDATE tracks SET notes = $1 WHERE id = $2")
                .bind(&notes_val).bind(tid).execute(&state.db).await;
        }
    }
    ok_or_redirect(&headers, &format!("/track/{tid}"))
}

pub async fn flags(
    State(state): State<AppState>,
    RequiredSession(sess): RequiredSession,
    Path(tid): Path<i32>,
    headers: HeaderMap,
    Form(form): Form<NonceForm>,
) -> Response {
    if let Some(t) = Track::by_id(&state.db, tid).await {
        if t.artist.id == sess.user.id && nonce_ok(&headers, form.nonce.as_deref(), &sess.nonce) {
            Session::new_nonce(&state.db, &sess.sid).await;
            let airable = form.extra.contains_key("airable");
            let _ = sqlx::query("UPDATE tracks SET airable = $1 WHERE id = $2")
                .bind(airable).bind(tid).execute(&state.db).await;
        }
    }
    ok_or_redirect(&headers, &format!("/track/{tid}"))
}

pub async fn report(
    State(state): State<AppState>,
    Path(tid): Path<i32>,
) -> Response {
    if let Some(t) = Track::by_id(&state.db, tid).await {
        let path = format!("{}/reports", state.manemix_dir);
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
            let _ = writeln!(f, "{} {} - {} {}", t.artist.id, t.artist.name, t.id, t.title);
        }
    }
    Redirect::to(&format!("/track/{tid}")).into_response()
}

// ---------------------------------------------------------------------------
// License
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct LicenseForm {
    nonce: Option<String>,
    license: Option<String>,
    #[serde(rename = "custom-license")]
    custom_license: Option<String>,
    mkdefault: Option<String>,
    retro: Option<String>,
}

pub async fn license_page(
    State(state): State<AppState>,
    RequiredSession(sess): RequiredSession,
    Path(tid): Path<i32>,
) -> axum::response::Html<String> {
    let new_nonce = Session::new_nonce(&state.db, &sess.sid).await;
    let mut ctx = tera::Context::new();
    ctx.insert("title", "License selection");
    ctx.insert("has_title", &true);
    ctx.insert("manemix_url", &state.base_url);
    ctx.insert("nonce", &new_nonce);
    ctx.insert("tid", &tid);
    if let Some(t) = crate::models::track::ExtendedTrack::by_id(&state.db, tid).await {
        ctx.insert("has_track", &true);
        ctx.insert("track_title", &t.track.title);
        ctx.insert("track_url", &format!("/track/{tid}"));
        ctx.insert("current_license", &t.license);
    }
    let body = state.tera.render("html/license.tpl", &ctx).unwrap_or_default();
    let mut sess = sess;
    sess.nonce = new_nonce;
    axum::response::Html(super::home::wrap_page(&state, &ctx, &body, Some(&sess), &sess.theme))
}

pub async fn license_submit(
    State(state): State<AppState>,
    RequiredSession(sess): RequiredSession,
    Path(tid): Path<i32>,
    headers: HeaderMap,
    Form(form): Form<LicenseForm>,
) -> Response {
    if !nonce_ok(&headers, form.nonce.as_deref(), &sess.nonce) {
        return ok_or_redirect(&headers, &format!("/track/{tid}/license"));
    }
    Session::new_nonce(&state.db, &sess.sid).await;

    let mut license = form.license.unwrap_or_default();
    if license == "custom" {
        license = form.custom_license.unwrap_or_default();
    }
    if license.is_empty() {
        return ok_or_redirect(&headers, &format!("/track/{tid}/license"));
    }

    let mkdefault = form.mkdefault.is_some();
    let retro = form.retro.is_some();

    if mkdefault {
        let _ = sqlx::query("UPDATE users SET license = $1 WHERE id = $2")
            .bind(&license).bind(sess.user.id).execute(&state.db).await;
    }
    if retro {
        let _ = sqlx::query("UPDATE tracks SET license = $1 WHERE user_id = $2")
            .bind(&license).bind(sess.user.id).execute(&state.db).await;
    } else {
        let _ = sqlx::query("UPDATE tracks SET license = $1 WHERE id = $2")
            .bind(&license).bind(tid).execute(&state.db).await;
    }

    ok_or_redirect(&headers, &format!("/track/{tid}"))
}

pub async fn account_license_page(
    State(state): State<AppState>,
    RequiredSession(sess): RequiredSession,
) -> axum::response::Html<String> {
    let new_nonce = Session::new_nonce(&state.db, &sess.sid).await;
    let license: String = sqlx::query_scalar("SELECT license FROM users WHERE id = $1")
        .bind(sess.user.id).fetch_one(&state.db).await.unwrap_or_default();
    let mut ctx = tera::Context::new();
    ctx.insert("title", "License selection");
    ctx.insert("has_title", &true);
    ctx.insert("manemix_url", &state.base_url);
    ctx.insert("nonce", &new_nonce);
    ctx.insert("has_track", &false);
    ctx.insert("current_license", &license);
    let body = state.tera.render("html/license.tpl", &ctx).unwrap_or_default();
    let mut sess = sess;
    sess.nonce = new_nonce;
    axum::response::Html(super::home::wrap_page(&state, &ctx, &body, Some(&sess), &sess.theme))
}

pub async fn account_license_submit(
    State(state): State<AppState>,
    RequiredSession(sess): RequiredSession,
    Form(form): Form<LicenseForm>,
) -> Response {
    if form.nonce.as_deref() != Some(&sess.nonce) {
        return Redirect::to("/account/license").into_response();
    }
    Session::new_nonce(&state.db, &sess.sid).await;

    let mut license = form.license.unwrap_or_default();
    if license == "custom" {
        license = form.custom_license.unwrap_or_default();
    }
    if license.is_empty() {
        return Redirect::to("/account/license").into_response();
    }

    let _ = sqlx::query("UPDATE users SET license = $1 WHERE id = $2")
        .bind(&license).bind(sess.user.id).execute(&state.db).await;

    if form.retro.is_some() {
        let _ = sqlx::query("UPDATE tracks SET license = $1 WHERE user_id = $2")
            .bind(&license).bind(sess.user.id).execute(&state.db).await;
    }

    Redirect::to("/account").into_response()
}

// ---------------------------------------------------------------------------
// Art upload / delete
// ---------------------------------------------------------------------------

pub async fn art_upload(
    State(state): State<AppState>,
    RequiredSession(sess): RequiredSession,
    Path(tid): Path<i32>,
    mut multipart: Multipart,
) -> Response {
    let _t = match Track::by_id(&state.db, tid).await {
        Some(t) if t.artist.id == sess.user.id => t,
        _ => return Redirect::to(&format!("/track/{tid}")).into_response(),
    };

    let mut file_data: Vec<u8> = Vec::new();
    while let Ok(Some(field)) = multipart.next_field().await {
        if field.name() == Some("file") {
            file_data = field.bytes().await.unwrap_or_default().to_vec();
        }
    }

    if !file_data.is_empty() {
        let art_dir = format!("{}/art", state.manemix_dir);
        let _ = std::fs::create_dir_all(&art_dir);
        let _ = std::fs::create_dir_all(format!("{}/medium", art_dir));
        let _ = std::fs::create_dir_all(format!("{}/thumb", art_dir));
        let art_path = format!("{}/{}", art_dir, tid);
        if let Ok(mut f) = std::fs::File::create(&art_path) {
            let _ = f.write_all(&file_data);
        }
        if let Some(art) = crate::models::art::Art::new(&state.manemix_dir, tid) {
            let _ = art.make_thumbs();
        }
    }

    Redirect::to(&format!("/track/{tid}")).into_response()
}

pub async fn art_delete(
    State(state): State<AppState>,
    RequiredSession(sess): RequiredSession,
    Path(tid): Path<i32>,
    headers: HeaderMap,
    Form(form): Form<NonceForm>,
) -> Response {
    if let Some(t) = Track::by_id(&state.db, tid).await {
        if t.artist.id == sess.user.id && nonce_ok(&headers, form.nonce.as_deref(), &sess.nonce) {
            Session::new_nonce(&state.db, &sess.sid).await;
            let _ = std::fs::remove_file(format!("{}/art/{}", state.manemix_dir, tid));
            let _ = std::fs::remove_file(format!("{}/art/medium/{}.jpg", state.manemix_dir, tid));
            let _ = std::fs::remove_file(format!("{}/art/thumb/{}.png", state.manemix_dir, tid));
        }
    }
    ok_or_redirect(&headers, &format!("/track/{tid}"))
}

// ---------------------------------------------------------------------------
// Track played beacon (for stats)
// ---------------------------------------------------------------------------

pub async fn played(
    State(state): State<AppState>,
    Path(tid): Path<i32>,
) -> Response {
    if let Some(t) = Track::by_id(&state.db, tid).await {
        let mut redis = state.redis.clone();
        crate::models::stat::push(&mut redis, "trackPlay", t.artist.id, tid, "", "").await;
    }
    axum::Json(serde_json::json!({})).into_response()
}

// ---------------------------------------------------------------------------
// Publish (POST /track/:tid/publish)
// ---------------------------------------------------------------------------

pub async fn publish(
    State(state): State<AppState>,
    RequiredSession(sess): RequiredSession,
    Path(tid): Path<i32>,
    headers: HeaderMap,
    Form(form): Form<NonceForm>,
) -> Response {
    let t = match Track::by_id(&state.db, tid).await {
        Some(t) if t.artist.id == sess.user.id && !t.visible => t,
        _ => return Redirect::to(&format!("/track/{tid}")).into_response(),
    };
    if nonce_ok(&headers, form.nonce.as_deref(), &sess.nonce) {
        Session::new_nonce(&state.db, &sess.sid).await;
        let _ = sqlx::query("UPDATE tracks SET visible = 't', date = now() WHERE id = $1")
            .bind(tid).execute(&state.db).await;

        crate::models::event::push_event(&state.db, "publish", &sess.user, &t.artist, Some(&t)).await;

        // Notify followers
        let followers: Vec<(String,)> = sqlx::query_as(
            "SELECT users.email FROM favorites, users \
             WHERE favorites.type = 'artist' AND favorites.ref = $1 \
             AND users.id = favorites.user_id AND users.notify = true"
        ).bind(sess.user.id).fetch_all(&state.db).await.unwrap_or_default();

        for (email,) in followers {
            let mail_body = format!(
                "{} just published a new track: {}\n\
                 Listen to it here: {}/track/{}\n\n\
                 You're receiving this email because you're following {} on Manemix.\n\
                 If you don't want to receive these notifications anymore, go to {}/user/{} and click \"Stop following\".",
                sess.user.name, t.title, state.base_url, tid,
                sess.user.name, state.base_url, sess.user.id
            );
            crate::models::mail::send(&email, &format!("Manemix notification: {} - {}", sess.user.name, t.title), &mail_body);
        }
    }
    ok_or_redirect(&headers, &format!("/track/{tid}"))
}

// ---------------------------------------------------------------------------
// Delete track (GET/POST /track/:tid/delete)
// ---------------------------------------------------------------------------

pub async fn delete_page(
    State(state): State<AppState>,
    RequiredSession(sess): RequiredSession,
    Path(tid): Path<i32>,
) -> Response {
    let t = match Track::by_id(&state.db, tid).await {
        Some(t) if t.artist.id == sess.user.id => t,
        _ => return Redirect::to(&format!("/track/{tid}")).into_response(),
    };
    let new_nonce = Session::new_nonce(&state.db, &sess.sid).await;
    let mut ctx = tera::Context::new();
    ctx.insert("title", "Track deletion");
    ctx.insert("has_title", &true);
    ctx.insert("manemix_url", &state.base_url);
    ctx.insert("what", &t.title);
    ctx.insert("cancel_url", &format!("/track/{tid}"));
    ctx.insert("nonce", &new_nonce);
    ctx.insert("delete_url", &format!("/track/{tid}/delete"));
    let body = state.tera.render("html/delete.tpl", &ctx).unwrap_or_default();
    let mut sess = sess;
    sess.nonce = new_nonce;
    axum::response::Html(super::home::wrap_page(&state, &ctx, &body, Some(&sess), &sess.theme)).into_response()
}

#[derive(Deserialize)]
pub struct DeleteForm {
    confirm: Option<String>,
    nonce: Option<String>,
}

pub async fn delete_submit(
    State(state): State<AppState>,
    RequiredSession(sess): RequiredSession,
    Path(tid): Path<i32>,
    headers: HeaderMap,
    Form(form): Form<DeleteForm>,
) -> Response {
    if let Some(t) = Track::by_id(&state.db, tid).await {
        if t.artist.id == sess.user.id
            && form.confirm.as_deref() == Some("Delete")
            && nonce_ok(&headers, form.nonce.as_deref(), &sess.nonce)
        {
            Session::new_nonce(&state.db, &sess.sid).await;
            let _ = sqlx::query("DELETE FROM favorites WHERE type = 'track' AND ref = $1").bind(tid).execute(&state.db).await;
            let _ = sqlx::query("DELETE FROM featured_tracks WHERE track_id = $1").bind(tid).execute(&state.db).await;
            let _ = sqlx::query("DELETE FROM user_features WHERE type = 'track' AND ref = $1").bind(tid).execute(&state.db).await;
            let _ = sqlx::query("DELETE FROM events WHERE track_id = $1").bind(tid).execute(&state.db).await;
            let _ = sqlx::query(
                "UPDATE playlists SET track_ids = array_remove(track_ids, $1)"
            ).bind(tid).execute(&state.db).await;
            let _ = sqlx::query("DELETE FROM tracks WHERE id = $1").bind(tid).execute(&state.db).await;
            Audio::new(&t, &state.manemix_dir).unlink();
            let _ = std::fs::remove_file(format!("{}/art/{}", state.manemix_dir, tid));
            let _ = std::fs::remove_file(format!("{}/art/medium/{}.jpg", state.manemix_dir, tid));
            let _ = std::fs::remove_file(format!("{}/art/thumb/{}.png", state.manemix_dir, tid));
            tracing::info!("Deleted track: {} ({})", t.title, tid);
            return ok_or_redirect(&headers, &format!("/user/{}", sess.user.id));
        }
    }
    ok_or_redirect(&headers, &format!("/track/{tid}"))
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn extract_title_from_filename(filename: &str) -> String {
    let stem = std::path::Path::new(filename)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "Untitled".into());
    if stem.is_empty() { "Untitled".into() } else { stem }
}

fn save_and_transcode(state: &AppState, tid: i32, data: &[u8], filename: &str) {
    let ext = std::path::Path::new(filename)
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_else(|| ".derp".into());

    // Ensure required directories exist
    let tmp_dir = format!("{}/tmp", state.manemix_dir);
    let tracks_dir = format!("{}/tracks", state.manemix_dir);
    let _ = std::fs::create_dir_all(&tmp_dir);
    let _ = std::fs::create_dir_all(&tracks_dir);

    let tmp_path = format!("{}/eqb{}{}", tmp_dir, tid, ext);
    if let Ok(mut f) = std::fs::File::create(&tmp_path) {
        let _ = f.write_all(data);
        tracing::info!("Saved upload to {} ({} bytes)", tmp_path, data.len());
    } else {
        tracing::error!("Failed to create temp file: {}", tmp_path);
        return;
    }

    let manemix_dir = state.manemix_dir.clone();
    let tid_str = tid.to_string();

    let script = std::env::var("MANEMIX_TRANSCODE").unwrap_or_else(|_| {
        if let Ok(exe) = std::env::current_exe() {
            let sibling = exe.parent().unwrap_or(std::path::Path::new(".")).join("transcode.sh");
            if sibling.exists() {
                return sibling.to_string_lossy().into_owned();
            }
        }
        "transcode.sh".into()
    });

    std::thread::spawn(move || {
        tracing::info!("Starting transcode: {} {}", script, tmp_path);
        match std::process::Command::new(&script)
            .arg(&tid_str)
            .arg(&tmp_path)
            .env("MANEMIX_DIR", &manemix_dir)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
        {
            Ok(child) => {
                match child.wait_with_output() {
                    Ok(output) if !output.status.success() => {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        tracing::error!("transcode.sh exited with {} for track {}\nstdout: {}\nstderr: {}", output.status, tid_str, stdout, stderr);
                    }
                    Ok(_) => {
                        tracing::info!("Transcoding complete for track {tid_str}");
                    }
                    Err(e) => {
                        tracing::error!("transcode.sh wait error for track {tid_str}: {e}");
                    }
                }
            }
            Err(e) => {
                tracing::error!("Failed to spawn transcode.sh: {e}");
            }
        }
    });
}
