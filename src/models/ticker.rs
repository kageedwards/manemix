use serde::Serialize;
use sqlx::PgPool;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct TickerItem {
    pub id: i32,
    pub title: String,
    pub url: String,
}

/// Return all non-expired ticker items, newest first.
pub async fn active(pool: &PgPool) -> Vec<TickerItem> {
    sqlx::query_as::<_, TickerItem>(
        "SELECT id, title, url FROM ticker WHERE expire > now() ORDER BY id DESC"
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default()
}
