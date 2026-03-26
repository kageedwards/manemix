use rand::Rng;
use sqlx::{PgPool, FromRow};

use super::user::User;

#[derive(Debug, Clone)]
pub struct Session {
    pub user: User,
    pub sid: String,
    pub nonce: String,
    pub theme: String,
}

#[derive(FromRow)]
struct SessionRow {
    id: i32,
    name: String,
    nonce: String,
    theme: String,
}

impl Session {
    /// Look up a session by cookie SID + host. Returns None if expired or invalid.
    pub async fn from_sid(pool: &PgPool, sid: &str, remote_addr: &str) -> Option<Self> {
        let row = sqlx::query_as::<_, SessionRow>(
            "SELECT users.id, users.name, sessions.nonce, users.theme \
             FROM sessions, users \
             WHERE sessions.sid = $1 AND sessions.host = $2::inet \
             AND users.id = sessions.user_id"
        )
        .bind(sid)
        .bind(remote_addr)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()?;

        Some(Session {
            user: User { id: row.id, name: row.name },
            sid: sid.to_string(),
            nonce: row.nonce,
            theme: row.theme,
        })
    }

    /// Create a new session for a user. Returns the SID.
    pub async fn create(pool: &PgPool, uid: i32, remote_addr: &str) -> Option<String> {
        let sid = random_string(64);
        sqlx::query(
            "INSERT INTO sessions (user_id, sid, host, date) VALUES ($1, $2, $3::inet, now())"
        )
        .bind(uid)
        .bind(&sid)
        .bind(remote_addr)
        .execute(pool)
        .await
        .ok()?;

        sqlx::query("UPDATE users SET last_login = now() WHERE id = $1")
            .bind(uid)
            .execute(pool)
            .await
            .ok();

        Some(sid)
    }

    /// Destroy a session.
    pub async fn destroy(pool: &PgPool, sid: &str) {
        let _ = sqlx::query("DELETE FROM sessions WHERE sid = $1")
            .bind(sid)
            .execute(pool)
            .await;
    }

    /// Generate and store a new nonce for CSRF protection.
    pub async fn new_nonce(pool: &PgPool, sid: &str) -> String {
        let nonce = random_string(16);
        let _ = sqlx::query("UPDATE sessions SET nonce = $1 WHERE sid = $2")
            .bind(&nonce)
            .bind(sid)
            .execute(pool)
            .await;
        nonce
    }

    /// Verify email/password, return user id + name on success.
    pub async fn authenticate(pool: &PgPool, email: &str, password: &str) -> Option<(i32, String)> {
        // Fetch the bcrypt hash from the DB
        let row = sqlx::query_as::<_, (i32, String, String)>(
            "SELECT id, name, password FROM users WHERE lower(email) = lower($1)"
        )
        .bind(email)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()?;

        // Verify with bcrypt
        if bcrypt::verify(password, &row.2).unwrap_or(false) {
            Some((row.0, row.1))
        } else {
            None
        }
    }
}

pub fn random_string(len: usize) -> String {
    const POOL: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_-";
    let mut rng = rand::thread_rng();
    (0..len).map(|_| POOL[rng.gen_range(0..POOL.len())] as char).collect()
}
