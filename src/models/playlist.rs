use serde::Serialize;
use sqlx::{PgPool, FromRow};

use super::track::Track;
use super::user::User;
use crate::text::{md5_hex, format_bbcode};

#[derive(Debug, Clone)]
pub struct Playlist {
    pub id: i32,
    pub author: User,
    pub author_email: String,
    pub name: String,
    pub length: i32,
    pub description: String,
    pub is_public: bool,
    pub is_album: bool,
}

#[derive(Debug, Serialize)]
pub struct PlaylistContext {
    pub playlist_id: i32,
    pub playlist_name: String,
    pub playlist_url: String,
    pub playlist_track_count: i32,
    pub track_count: String,
    pub uid: i32,
    pub username: String,
    pub email_md5: String,
    pub description: String,
    pub description_html: String,
    pub has_description: bool,
    pub is_public: bool,
    pub is_album: bool,
}

#[derive(FromRow)]
struct PlaylistFullRow {
    uid: i32,
    name: String,
    email: String,
    pname: Option<String>,
    len: Option<i32>,
    descr: Option<String>,
    is_public: bool,
    is_album: bool,
}

#[derive(FromRow)]
struct PlaylistShortRow {
    id: i32,
    name: Option<String>,
    len: Option<i32>,
    is_album: bool,
    description: Option<String>,
    is_public: bool,
}

impl Playlist {
    pub async fn by_id(pool: &PgPool, id: i32) -> Option<Self> {
        let row = sqlx::query_as::<_, PlaylistFullRow>(
            "SELECT users.id as uid, users.name, users.email, \
             playlists.name as pname, array_length(playlists.track_ids, 1) as len, \
             playlists.description as descr, playlists.public as is_public, \
             playlists.is_album \
             FROM playlists, users \
             WHERE users.id = playlists.user_id AND playlists.id = $1"
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()?;

        Some(Playlist {
            id,
            author: User::from((row.uid, row.name)),
            author_email: row.email,
            name: row.pname.unwrap_or_default(),
            length: row.len.unwrap_or(0),
            description: row.descr.unwrap_or_default(),
            is_public: row.is_public,
            is_album: row.is_album,
        })
    }

    pub async fn for_user(pool: &PgPool, uid: i32) -> Vec<Playlist> {
        sqlx::query_as::<_, PlaylistShortRow>(
            "SELECT id, name, array_length(track_ids, 1) as len, description, public as is_public, is_album \
             FROM playlists WHERE user_id = $1 ORDER BY name ASC"
        )
        .bind(uid)
        .fetch_all(pool)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|r| Playlist {
            id: r.id,
            author: User::default(),
            author_email: String::new(),
            name: r.name.unwrap_or_default(),
            length: r.len.unwrap_or(0),
            description: r.description.unwrap_or_default(),
            is_public: r.is_public,
            is_album: r.is_album,
        })
        .collect()
    }

    /// Public playlists from all users, for the browse page (excludes albums).
    pub async fn public_all(pool: &PgPool) -> Vec<Playlist> {
        #[derive(FromRow)]
        struct Row { id: i32, uid: i32, uname: String, email: String, pname: Option<String>, len: Option<i32>, descr: Option<String> }
        sqlx::query_as::<_, Row>(
            "SELECT playlists.id, users.id as uid, users.name as uname, users.email, \
             playlists.name as pname, array_length(playlists.track_ids, 1) as len, \
             playlists.description as descr \
             FROM playlists, users \
             WHERE playlists.public = true AND playlists.is_album = false \
             AND users.id = playlists.user_id \
             AND array_length(playlists.track_ids, 1) > 0 \
             ORDER BY playlists.id DESC"
        )
        .fetch_all(pool)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|r| Playlist {
            id: r.id,
            author: User::from((r.uid, r.uname)),
            author_email: r.email,
            name: r.pname.unwrap_or_default(),
            length: r.len.unwrap_or(0),
            description: r.descr.unwrap_or_default(),
            is_public: true,
            is_album: false,
        })
        .collect()
    }

    /// Public albums from all users, for the albums browse page.
    pub async fn public_albums(pool: &PgPool) -> Vec<Playlist> {
        #[derive(FromRow)]
        struct Row { id: i32, uid: i32, uname: String, email: String, pname: Option<String>, len: Option<i32>, descr: Option<String> }
        sqlx::query_as::<_, Row>(
            "SELECT playlists.id, users.id as uid, users.name as uname, users.email, \
             playlists.name as pname, array_length(playlists.track_ids, 1) as len, \
             playlists.description as descr \
             FROM playlists, users \
             WHERE playlists.public = true AND playlists.is_album = true \
             AND users.id = playlists.user_id \
             AND array_length(playlists.track_ids, 1) > 0 \
             ORDER BY playlists.id DESC"
        )
        .fetch_all(pool)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|r| Playlist {
            id: r.id,
            author: User::from((r.uid, r.uname)),
            author_email: r.email,
            name: r.pname.unwrap_or_default(),
            length: r.len.unwrap_or(0),
            description: r.descr.unwrap_or_default(),
            is_public: true,
            is_album: true,
        })
        .collect()
    }

    pub async fn tracks(&self, pool: &PgPool) -> Vec<Track> {
        let sql = "WITH playlist AS \
             (SELECT unnest AS track_id, row_number() OVER () AS pos \
              FROM unnest(coalesce((SELECT track_ids FROM playlists WHERE id = $1)))) \
             SELECT tracks.id, tracks.title, tracks.user_id, users.name, \
             tracks.visible, tracks.date::text, extract(epoch from tracks.date)::text as ts \
             FROM tracks, users, playlist \
             WHERE tracks.user_id = users.id AND tracks.id = playlist.track_id \
             ORDER BY playlist.pos";
        sqlx::query_as::<_, super::track::TrackRow>(sql)
            .bind(self.id)
            .fetch_all(pool)
            .await
            .unwrap_or_default()
            .into_iter()
            .map(Into::into)
            .collect()
    }

    pub fn context(&self) -> PlaylistContext {
        let plural = if self.length != 1 { "s" } else { "" };
        PlaylistContext {
            playlist_id: self.id,
            playlist_name: self.name.clone(),
            playlist_url: if self.is_album {
                format!("/album/{}", self.id)
            } else {
                format!("/playlist/{}", self.id)
            },
            playlist_track_count: self.length,
            track_count: format!("{} track{}", self.length, plural),
            uid: self.author.id,
            username: self.author.name.clone(),
            email_md5: md5_hex(&self.author_email),
            description: self.description.clone(),
            description_html: format_bbcode(&self.description),
            has_description: !self.description.is_empty(),
            is_public: self.is_public,
            is_album: self.is_album,
        }
    }
}
