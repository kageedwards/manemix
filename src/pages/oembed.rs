use axum::extract::{Query, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;

use crate::AppState;
use crate::models::track::Track;

#[derive(Deserialize)]
pub struct OembedQuery {
    url: Option<String>,
    format: Option<String>,
    maxwidth: Option<i32>,
}

pub async fn oembed(
    State(state): State<AppState>,
    Query(q): Query<OembedQuery>,
) -> Response {
    let url = q.url.unwrap_or_default();

    // Parse track ID from URL, mirroring the original's drop-based parsing
    let tid = extract_track_id(&url);
    if tid == 0 {
        return StatusCode::NOT_FOUND.into_response();
    }

    let t = match Track::by_id(&state.db, tid).await {
        Some(t) => t,
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    let width = q.maxwidth.unwrap_or(150).max(1);

    let embed_html = format!(
        r#"<iframe width="{}px" height="150px" frameborder="0" src="{}/track/{}/embed"><a href="{}/track/{}">{}</a> by <a href="{}/user/{}">{}</a></iframe>"#,
        width, state.base_url, tid, state.base_url, tid, t.title,
        state.base_url, t.artist.id, t.artist.name
    );

    let mut ctx = tera::Context::new();
    ctx.insert("manemix_url", &state.base_url);
    ctx.insert("track", &t.context(&state.manemix_dir));
    ctx.insert("width", &width);
    ctx.insert("tid", &tid);
    ctx.insert("embed_html", &embed_html);

    let (template, content_type) = if q.format.as_deref() == Some("xml") {
        ("oembed/xml.tpl", "text/xml")
    } else {
        ("oembed/json.tpl", "application/json")
    };

    let body = state.tera.render(template, &ctx).unwrap_or_default();

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", HeaderValue::from_str(content_type).unwrap());
    (headers, body).into_response()
}

fn extract_track_id(url: &str) -> i32 {
    // Strip protocol and domain to get /track/123
    let path = url
        .trim_start_matches("http://")
        .trim_start_matches("https://")
        .trim_start_matches("www.")
        .strip_prefix("manemix.org")
        .unwrap_or("");

    if let Some(rest) = path.strip_prefix("/track/") {
        rest.split('/').next()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0)
    } else {
        0
    }
}
