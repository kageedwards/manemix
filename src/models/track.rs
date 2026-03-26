use serde::Serialize;
use sqlx::PgPool;

use crate::text::{format_time, md5_hex, format_bbcode};
use super::art::Art;
use super::user::User;

// ---------------------------------------------------------------------------
// Core Track (mirrors Track in the original)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Track {
    pub id: i32,
    pub title: String,
    pub artist: User,
    pub visible: bool,
    pub date: String,
    pub timestamp: String,
}

#[derive(Debug, Serialize)]
pub struct TrackContext {
    pub tid: i32,
    pub title: String,
    pub uid: i32,
    pub username: String,
    pub is_visible: bool,
    pub is_hidden: bool,
    pub date: String,
    pub timestamp: String,
    pub day: String,
    pub has_art: bool,
}

impl Track {
    pub async fn by_id(pool: &PgPool, tid: i32) -> Option<Self> {
        let row = sqlx::query_as::<_, TrackRow>(
            "SELECT tracks.id, tracks.title, tracks.user_id, users.name, tracks.visible, \
             tracks.date::text, extract(epoch from tracks.date)::text as ts \
             FROM tracks, users \
             WHERE tracks.id = $1 AND tracks.user_id = users.id"
        )
        .bind(tid)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()?;

        Some(row.into())
    }

    pub fn context(&self, manemix_dir: &str) -> TrackContext {
        TrackContext {
            tid: self.id,
            title: self.title.clone(),
            uid: self.artist.id,
            username: self.artist.name.clone(),
            is_visible: self.visible,
            is_hidden: !self.visible,
            date: self.date.clone(),
            timestamp: self.timestamp.clone(),
            day: format_time(&self.date, "%B %-d, %Y"),
            has_art: Art::exists(manemix_dir, self.id),
        }
    }

    pub fn url(&self) -> String {
        format!("/track/{}", self.id)
    }
}

// ---------------------------------------------------------------------------
// Extended Track (mirrors ExtendedTrack)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct ExtendedTrack {
    pub track: Track,
    pub email: String,
    pub notes: String,
    pub airable: bool,
    pub license: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ExtendedTrackContext {
    #[serde(flatten)]
    pub base: TrackContext,
    pub email_md5: String,
    pub notes: String,
    pub notes_html: String,
    pub has_notes: bool,
    pub license: String,
    pub has_license: bool,
    pub tags: Vec<String>,
    pub has_tags: bool,
    pub is_copyright: bool,
    pub license_key: String,
    pub airable: bool,
}

#[derive(sqlx::FromRow)]
struct ExtendedTrackRow {
    id: i32,
    title: String,
    user_id: i32,
    name: String,
    email: String,
    visible: bool,
    date: Option<String>,
    ts: Option<String>,
    notes: String,
    airable: bool,
    license: String,
    tagstr: Option<String>,
}

impl ExtendedTrack {
    pub async fn by_id(pool: &PgPool, tid: i32) -> Option<Self> {
        let row = sqlx::query_as::<_, ExtendedTrackRow>(
            "SELECT tracks.id, tracks.title, tracks.user_id, users.name, users.email, \
             tracks.visible, tracks.date::text, extract(epoch from tracks.date)::text as ts, \
             tracks.notes, tracks.airable, tracks.license, \
             array_to_string(tracks.tags, ',') as tagstr \
             FROM tracks, users \
             WHERE tracks.id = $1 AND tracks.user_id = users.id"
        )
        .bind(tid)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()?;

        let tags: Vec<String> = row.tagstr
            .unwrap_or_default()
            .split(',')
            .filter(|s| !s.is_empty())
            .map(String::from)
            .collect();

        Some(ExtendedTrack {
            track: Track {
                id: row.id,
                title: row.title,
                artist: User::from((row.user_id, row.name)),
                visible: row.visible,
                date: row.date.unwrap_or_default(),
                timestamp: row.ts.unwrap_or_default(),
            },
            email: row.email,
            notes: row.notes,
            airable: row.airable,
            license: row.license,
            tags,
        })
    }

    pub fn context(&self, manemix_dir: &str) -> ExtendedTrackContext {
        let license_key = match self.license.as_str() {
            "Copyright" => "copyright",
            "CC BY" => "cc_by",
            "CC BY-NC" => "cc_by_nc",
            "CC BY-SA" => "cc_by_sa",
            "CC BY-ND" => "cc_by_nd",
            "CC BY-NC-SA" => "cc_by_nc_sa",
            "CC BY-NC-ND" => "cc_by_nc_nd",
            "Public domain" => "public",
            _ => "custom",
        };

        ExtendedTrackContext {
            base: self.track.context(manemix_dir),
            email_md5: md5_hex(&self.email),
            notes: self.notes.clone(),
            notes_html: format_bbcode(&self.notes),
            has_notes: !self.notes.is_empty(),
            license: self.license.clone(),
            has_license: !self.license.is_empty(),
            tags: self.tags.clone(),
            has_tags: !self.tags.is_empty(),
            is_copyright: self.license == "Copyright",
            license_key: license_key.into(),
            airable: self.airable,
        }
    }
}

// ---------------------------------------------------------------------------
// Track lists (mirrors TrackList / Tracks namespace)
// ---------------------------------------------------------------------------

pub async fn latest(pool: &PgPool, n: i64, offset: i64) -> Vec<Track> {
    fetch_list(pool,
        "SELECT tracks.id, tracks.title, tracks.user_id, users.name, \
         tracks.visible, tracks.date::text, extract(epoch from tracks.date)::text as ts \
         FROM tracks, users WHERE tracks.user_id = users.id AND tracks.visible = true \
         ORDER BY date DESC LIMIT $1 OFFSET $2",
        n, offset
    ).await
}

pub async fn featured(pool: &PgPool, n: i64) -> Vec<Track> {
    fetch_list(pool,
        "SELECT tracks.id, tracks.title, tracks.user_id, users.name, \
         tracks.visible, tracks.date::text, extract(epoch from tracks.date)::text as ts \
         FROM tracks, users, featured_tracks \
         WHERE tracks.user_id = users.id AND featured_tracks.track_id = tracks.id \
         ORDER BY featured_tracks.date DESC LIMIT $1 OFFSET $2",
        n, 0
    ).await
}

pub async fn random(pool: &PgPool, n: i64) -> Vec<Track> {
    fetch_list(pool,
        "SELECT tracks.id, tracks.title, tracks.user_id, users.name, \
         tracks.visible, tracks.date::text, extract(epoch from tracks.date)::text as ts \
         FROM tracks, users WHERE tracks.user_id = users.id AND tracks.visible = true \
         ORDER BY random() LIMIT $1 OFFSET $2",
        n, 0
    ).await
}

pub async fn by_user(pool: &PgPool, uid: i32, all: bool) -> Vec<Track> {
    let vis = if all { "" } else { " AND tracks.visible = true" };
    let sql = format!(
        "SELECT tracks.id, tracks.title, tracks.user_id, users.name, \
         tracks.visible, tracks.date::text, extract(epoch from tracks.date)::text as ts \
         FROM tracks, users WHERE tracks.user_id = users.id \
         AND users.id = $1{vis} ORDER BY date DESC"
    );
    sqlx::query_as::<_, TrackRow>(&sql)
        .bind(uid)
        .fetch_all(pool)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(Into::into)
        .collect()
}

pub async fn search(pool: &PgPool, q: &str) -> Vec<Track> {
    if q.is_empty() {
        return vec![];
    }
    let pattern = format!("%{q}%");
    sqlx::query_as::<_, TrackRow>(
        "SELECT tracks.id, tracks.title, tracks.user_id, users.name, \
         tracks.visible, tracks.date::text, extract(epoch from tracks.date)::text as ts \
         FROM tracks, users WHERE tracks.user_id = users.id AND tracks.visible = true \
         AND (tracks.title ILIKE $1 OR users.name ILIKE $1)"
    )
    .bind(&pattern)
    .fetch_all(pool)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(Into::into)
    .collect()
}

pub async fn exact_search(pool: &PgPool, artist: &str, title: &str) -> Vec<Track> {
    sqlx::query_as::<_, TrackRow>(
        "SELECT tracks.id, tracks.title, tracks.user_id, users.name, \
         tracks.visible, tracks.date::text, extract(epoch from tracks.date)::text as ts \
         FROM tracks, users WHERE tracks.user_id = users.id AND tracks.visible = true \
         AND users.name = $1 AND tracks.title = $2"
    )
    .bind(artist)
    .bind(title)
    .fetch_all(pool)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(Into::into)
    .collect()
}

pub async fn by_tag(pool: &PgPool, tag: &str) -> Vec<Track> {
    sqlx::query_as::<_, TrackRow>(
        "SELECT tracks.id, tracks.title, tracks.user_id, users.name, \
         tracks.visible, tracks.date::text, extract(epoch from tracks.date)::text as ts \
         FROM tracks, users WHERE tracks.user_id = users.id AND tracks.visible = true \
         AND $1 = ANY(tracks.tags) ORDER BY tracks.date DESC"
    )
    .bind(tag)
    .fetch_all(pool)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(Into::into)
    .collect()
}

// Row type for sqlx::FromRow — pub so other modules can reuse it
#[derive(sqlx::FromRow)]
pub struct TrackRow {
    id: i32,
    title: String,
    user_id: i32,
    name: String,
    visible: bool,
    date: Option<String>,
    ts: Option<String>,
}

impl From<TrackRow> for Track {
    fn from(r: TrackRow) -> Self {
        Track {
            id: r.id,
            title: r.title,
            artist: User::from((r.user_id, r.name)),
            visible: r.visible,
            date: r.date.unwrap_or_default(),
            timestamp: r.ts.unwrap_or_default(),
        }
    }
}

async fn fetch_list(pool: &PgPool, sql: &str, limit: i64, offset: i64) -> Vec<Track> {
    sqlx::query_as::<_, TrackRow>(sql)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(Into::into)
        .collect()
}
