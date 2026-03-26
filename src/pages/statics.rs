use axum::extract::State;
use axum::response::{Html, Redirect};

use crate::AppState;
use super::home::wrap_page;

pub async fn faq(State(state): State<AppState>) -> Html<String> {
    render_static(&state, "html/faq.tpl", "FAQ")
}

pub async fn thanks(State(state): State<AppState>) -> Html<String> {
    render_static(&state, "html/thanks.tpl", "Thanks")
}

pub async fn api(State(state): State<AppState>) -> Html<String> {
    render_static(&state, "html/api.tpl", "API")
}

pub async fn credits_redirect() -> Redirect {
    Redirect::permanent("/thanks")
}

fn render_static(state: &AppState, template: &str, title: &str) -> Html<String> {
    let mut ctx = tera::Context::new();
    ctx.insert("title", title);
    ctx.insert("has_title", &true);
    ctx.insert("manemix_url", &state.base_url);
    let body = state.tera.render(template, &ctx).unwrap_or_default();
    Html(wrap_page(state, &ctx, &body, None, "auto"))
}
