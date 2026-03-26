use axum::extract::State;
use axum::response::Html;

use crate::AppState;
use crate::models::session::Session;
use crate::models::track;
use crate::session::OptionalSession;

pub async fn home(
    State(state): State<AppState>,
    OptionalSession(sess, theme): OptionalSession,
) -> Html<String> {
    let featured = track::featured(&state.db, 15).await;
    let latest = track::latest(&state.db, 5, 0).await;
    let random = track::random(&state.db, 5).await;

    let mut ctx = tera::Context::new();
    ctx.insert("title", "");
    ctx.insert("manemix_url", &state.base_url);
    ctx.insert("feed_url", "/tracks/latest/atom");
    ctx.insert("featured_tracks", &featured.iter().map(|t| t.context(&state.manemix_dir)).collect::<Vec<_>>());
    ctx.insert("latest_tracks", &latest.iter().map(|t| t.context(&state.manemix_dir)).collect::<Vec<_>>());
    ctx.insert("random_tracks", &random.iter().map(|t| t.context(&state.manemix_dir)).collect::<Vec<_>>());

    let body = state.tera.render("html/home.tpl", &ctx).unwrap_or_default();
    Html(wrap_page(&state, &ctx, &body, sess.as_ref(), &theme))
}

/// Render the body into the page shell, mirroring Document::generate().
/// Injects session/nonce context for the page chrome (login/logout links, CSRF).
pub fn wrap_page(state: &AppState, ctx: &tera::Context, body: &str, sess: Option<&Session>, theme: &str) -> String {
    let mut page_ctx = ctx.clone();
    page_ctx.insert("body", body);
    page_ctx.insert("manemix_url", &state.base_url);
    page_ctx.insert("theme", theme);
    if let Some(s) = sess {
        page_ctx.insert("logged_in", &true);
        page_ctx.insert("session_uid", &s.user.id);
        page_ctx.insert("session_username", &s.user.name);
        page_ctx.insert("nonce", &s.nonce);
    } else {
        page_ctx.insert("logged_in", &false);
    }
    state.tera.render("html/page.tpl", &page_ctx).unwrap_or_default()
}
