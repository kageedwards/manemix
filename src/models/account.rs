use serde::Serialize;
use sqlx::{PgPool, FromRow};

use crate::text::{md5_hex, format_bbcode};
use super::user::User;

#[derive(Debug, Clone)]
pub struct Account {
    pub user: User,
    pub email: String,
    pub about: String,
    pub notify: bool,
    pub license: String,
    pub theme: String,
    pub num_favs: i64,
    pub num_followers: i64,
}

/// Template-ready context, mirrors Account::fill(Dict*).
#[derive(Debug, Serialize)]
pub struct AccountContext {
    pub uid: i32,
    pub username: String,
    pub email: String,
    pub email_md5: String,
    pub about: String,
    pub about_html: String,
    pub has_about: bool,
    pub license: String,
    pub theme: String,
    pub num_favs: i64,
    pub has_favs: bool,
    pub num_followers: i64,
    pub has_followers: bool,
    pub followers_plural: bool,
    pub is_self: bool,
    pub notify: bool,
}

#[derive(FromRow)]
struct AccountRow {
    name: String,
    email: String,
    about: String,
    notify: bool,
    license: String,
    theme: String,
}

impl Account {
    pub async fn by_id(pool: &PgPool, uid: i32) -> Option<Self> {
        let row = sqlx::query_as::<_, AccountRow>(
            "SELECT name, email, about, notify, license, theme FROM users WHERE id = $1"
        )
        .bind(uid)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()?;

        let num_favs: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM favorites, tracks \
             WHERE tracks.visible = TRUE AND tracks.id = favorites.ref \
             AND favorites.type = 'track' AND favorites.user_id = $1"
        )
        .bind(uid)
        .fetch_one(pool)
        .await
        .unwrap_or(0);

        let num_followers: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM favorites \
             WHERE favorites.ref = $1 AND favorites.type = 'artist'"
        )
        .bind(uid)
        .fetch_one(pool)
        .await
        .unwrap_or(0);

        Some(Account {
            user: User { id: uid, name: row.name },
            email: row.email,
            about: row.about,
            notify: row.notify,
            license: row.license,
            theme: row.theme,
            num_favs,
            num_followers,
        })
    }

    pub fn context(&self) -> AccountContext {
        AccountContext {
            uid: self.user.id,
            username: self.user.name.clone(),
            email: self.email.clone(),
            email_md5: md5_hex(&self.email),
            about: self.about.clone(),
            about_html: format_bbcode(&self.about),
            has_about: !self.about.is_empty(),
            license: self.license.clone(),
            theme: self.theme.clone(),
            num_favs: self.num_favs,
            has_favs: self.num_favs > 0,
            num_followers: self.num_followers,
            has_followers: self.num_followers > 0,
            followers_plural: self.num_followers > 1,
            is_self: false,
            notify: self.notify,
        }
    }
}
