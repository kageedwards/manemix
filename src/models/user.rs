use serde::Serialize;
use sqlx::{PgPool, FromRow};

#[derive(Debug, Clone, Default, Serialize, FromRow)]
pub struct User {
    pub id: i32,
    pub name: String,
}

/// Template-ready context, mirrors the original User::fill(Dict*).
#[derive(Debug, Serialize)]
pub struct UserContext {
    pub uid: i32,
    pub username: String,
    pub is_self: bool,
}

impl User {
    pub async fn by_id(pool: &PgPool, uid: i32) -> Option<Self> {
        sqlx::query_as::<_, Self>("SELECT id, name FROM users WHERE id = $1")
            .bind(uid)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten()
    }

    pub fn context(&self) -> UserContext {
        UserContext {
            uid: self.id,
            username: self.name.clone(),
            is_self: false, // read-only site, no sessions
        }
    }

    pub fn url(&self) -> String {
        format!("/user/{}", self.id)
    }
}

impl From<(i32, String)> for User {
    fn from((id, name): (i32, String)) -> Self {
        Self { id, name }
    }
}
