mod db;
mod json_errors;
mod models;
mod pages;
mod render;
mod session;
mod text;

use axum::{Router, extract::DefaultBodyLimit};
use axum::http::{HeaderValue, Method};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

pub type AppState = Arc<State>;

pub struct State {
    pub db: sqlx::PgPool,
    pub redis: redis::aio::MultiplexedConnection,
    pub tera: tera::Tera,
    pub manemix_dir: String,
    pub base_url: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let manemix_dir = std::env::var("MANEMIX_DIR")
        .unwrap_or_else(|_| "/var/lib/manemix".into());

    // Ensure required data directories exist
    for sub in ["tracks", "art", "art/medium", "art/thumb", "tmp"] {
        let _ = std::fs::create_dir_all(format!("{manemix_dir}/{sub}"));
    }

    let pg_url = std::env::var("MANEMIX_POSTGRES")
        .unwrap_or_else(|_| "postgres://localhost/manemix".into());

    let redis_url = std::env::var("MANEMIX_REDIS")
        .unwrap_or_else(|_| "redis://127.0.0.1/".into());

    let templates_dir = std::env::var("MANEMIX_TEMPLATES")
        .unwrap_or_else(|_| "templates/**/*".into());

    let base_url = std::env::var("MANEMIX_URL")
        .unwrap_or_else(|_| "http://localhost:8642".into());

    let db = sqlx::postgres::PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(std::time::Duration::from_secs(30))
        .connect(&pg_url)
        .await
        .expect("couldn't connect to PostgreSQL");

    let redis_client = redis::Client::open(redis_url)
        .expect("invalid redis URL");
    let redis = redis_client.get_multiplexed_tokio_connection()
        .await
        .expect("couldn't connect to Redis");

    let mut tera = tera::Tera::new(&templates_dir)
        .expect("couldn't load templates");
    render::register_filters(&mut tera);

    let state: AppState = Arc::new(State {
        db,
        redis,
        tera,
        manemix_dir,
        base_url,
    });

    let spa_origin = std::env::var("MANEMIX_SPA_ORIGIN")
        .unwrap_or_else(|_| "http://localhost:5173".into());

    let cors = CorsLayer::new()
        .allow_origin(spa_origin.parse::<HeaderValue>().expect("invalid SPA origin"))
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::ACCEPT,
            axum::http::header::AUTHORIZATION,
        ])
        .allow_credentials(true);

    let app = Router::new()
        .merge(pages::routes())
        .nest_service("/static", ServeDir::new("static"))
        .layer(DefaultBodyLimit::max(100 * 1024 * 1024)) // 100MB, matches nginx
        .layer(axum::middleware::from_fn(json_errors::json_error_middleware))
        .layer(cors)
        .with_state(state);

    let bind = std::env::var("MANEMIX_BIND")
        .unwrap_or_else(|_| "0.0.0.0:8100".into());

    let listener = tokio::net::TcpListener::bind(&bind)
        .await
        .expect("couldn't bind");

    tracing::info!("listening on {}", bind);
    axum::serve(listener, app).await.unwrap();
}
