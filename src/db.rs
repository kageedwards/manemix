/// Thin query helpers mirroring the original's DB::query interface.
/// Unlike the original, all queries are parameterized.

use sqlx::PgPool;
use sqlx::postgres::PgRow;

pub type Result<T> = std::result::Result<T, sqlx::Error>;

/// Fetch all rows for a simple query with no parameters.
pub async fn query(pool: &PgPool, sql: &str) -> Result<Vec<PgRow>> {
    sqlx::query(sql).fetch_all(pool).await
}

/// Check that the critical tables exist.
pub async fn health_check(pool: &PgPool) -> Result<bool> {
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM pg_tables WHERE schemaname = 'public' \
         AND tablename = ANY(ARRAY['users', 'tracks', 'sessions'])"
    )
    .fetch_one(pool)
    .await?;
    Ok(row.0 == 3)
}
