use serde::Serialize;
use sqlx::PgPool;

use crate::text::{format_time, fuzzy_time, format_bbcode};
use super::user::User;

#[derive(Debug, Clone)]
pub struct Event {
    pub id: i32,
    pub event_type: EventType,
    pub source: User,
    pub target: User,
    pub track_id: Option<i32>,
    pub track_title: Option<String>,
    pub message: String,
    pub date: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EventType { Publish, Comment, TrackComment, UserComment, Favorite, Follow }

#[derive(Debug, Serialize)]
pub struct EventContext {
    pub event_id: i32,
    pub utc_date: String,
    pub fuzzy_time: String,
    pub is_publish: bool,
    pub is_comment: bool,
    pub is_track_comment: bool,
    pub is_user_comment: bool,
    pub is_favorite: bool,
    pub is_follow: bool,
    pub source_uid: i32,
    pub source_name: String,
    pub target_uid: i32,
    pub target_name: String,
    pub has_track: bool,
    pub tid: i32,
    pub track_title: String,
    pub message: String,
    pub message_html: String,
}

impl Event {
    pub fn context(&self) -> EventContext {
        EventContext {
            event_id: self.id,
            utc_date: format_time(&self.date, "%F %R"),
            fuzzy_time: fuzzy_time(&self.date),
            is_publish: self.event_type == EventType::Publish,
            is_comment: matches!(self.event_type, EventType::Comment | EventType::TrackComment | EventType::UserComment),
            is_track_comment: self.event_type == EventType::TrackComment,
            is_user_comment: self.event_type == EventType::UserComment,
            is_favorite: self.event_type == EventType::Favorite,
            is_follow: self.event_type == EventType::Follow,
            source_uid: self.source.id,
            source_name: self.source.name.clone(),
            target_uid: self.target.id,
            target_name: self.target.name.clone(),
            has_track: self.track_id.is_some() && self.track_id != Some(0),
            tid: self.track_id.unwrap_or(0),
            track_title: self.track_title.clone().unwrap_or_default(),
            message: self.message.clone(),
            message_html: format_bbcode(&self.message),
        }
    }
}

pub async fn for_user(pool: &PgPool, uid: i32, limit: i64) -> Vec<Event> {
    fetch_events(pool,
        "WHERE (source_id = $1 OR (target_id = $1 AND type IN ('comment', 'track_comment', 'user_comment'))) \
         AND (track_id IS NULL OR track_id NOT IN (SELECT id FROM tracks WHERE visible = false))",
        Some(uid), limit
    ).await
}

pub async fn for_track(pool: &PgPool, tid: i32) -> Vec<Event> {
    fetch_events(pool,
        "WHERE (type = 'track_comment' AND track_id = $1) \
         OR (type = 'comment' AND track_id = $1)",
        Some(tid), 0
    ).await
}

/// Global recent events (excludes hidden tracks), for the homepage activity feed.
pub async fn recent(pool: &PgPool, limit: i64) -> Vec<Event> {
    fetch_events(pool,
        "WHERE (track_id IS NULL OR track_id NOT IN (SELECT id FROM tracks WHERE visible = false))",
        None, limit
    ).await
}

async fn fetch_events(pool: &PgPool, cond: &str, bind: Option<i32>, limit: i64) -> Vec<Event> {
    let limit_clause = if limit > 0 { format!(" LIMIT {limit}") } else { String::new() };
    let sql = format!(
        "SELECT type, source_id, source_name, target_id, target_name, \
         track_id, track_title, message, date AT TIME ZONE 'UTC' as date, id \
         FROM events {cond} ORDER BY date DESC{limit_clause}"
    );

    let rows = if let Some(id) = bind {
        sqlx::query_as::<_, EventRow>(&sql)
            .bind(id)
            .fetch_all(pool)
            .await
    } else {
        sqlx::query_as::<_, EventRow>(&sql)
            .fetch_all(pool)
            .await
    };

    rows.unwrap_or_default().into_iter().map(|r| {
        let event_type = match r.r#type.as_str() {
            "publish" => EventType::Publish,
            "track_comment" => EventType::TrackComment,
            "user_comment" => EventType::UserComment,
            "comment" => EventType::Comment,
            "favorite" => EventType::Favorite,
            _ => EventType::Follow,
        };
        Event {
            id: r.id,
            event_type,
            source: User::from((r.source_id.unwrap_or(0), r.source_name.unwrap_or_default())),
            target: User::from((r.target_id.unwrap_or(0), r.target_name.unwrap_or_default())),
            track_id: r.track_id,
            track_title: r.track_title,
            message: r.message,
            date: r.date.unwrap_or_default(),
        }
    }).collect()
}

#[derive(sqlx::FromRow)]
struct EventRow {
    r#type: String,
    source_id: Option<i32>,
    source_name: Option<String>,
    target_id: Option<i32>,
    target_name: Option<String>,
    track_id: Option<i32>,
    track_title: Option<String>,
    message: String,
    date: Option<String>,
    id: i32,
}

/// Insert an event (favorite, follow, publish).
pub async fn push_event(pool: &PgPool, event_type: &str, source: &User, target: &User, track: Option<&crate::models::track::Track>) {
    let tid = track.map(|t| t.id).unwrap_or(0);
    let ttitle = track.map(|t| t.title.as_str()).unwrap_or("");
    let _ = sqlx::query(
        "INSERT INTO events (type, target_id, target_name, source_id, source_name, track_id, track_title, message, date) \
         VALUES ($1::event_type, $2, $3, $4, $5, $6, $7, '', now())"
    )
    .bind(event_type)
    .bind(target.id)
    .bind(&target.name)
    .bind(source.id)
    .bind(&source.name)
    .bind(tid)
    .bind(ttitle)
    .execute(pool)
    .await;
}

/// Insert a track comment event.
pub async fn push_track_comment(pool: &PgPool, source: &User, target: &User, track: &crate::models::track::Track, message: &str) {
    let _ = sqlx::query(
        "INSERT INTO events (type, target_id, target_name, source_id, source_name, track_id, track_title, message, date) \
         VALUES ('track_comment'::event_type, $1, $2, $3, $4, $5, $6, $7, now())"
    )
    .bind(target.id)
    .bind(&target.name)
    .bind(source.id)
    .bind(&source.name)
    .bind(track.id)
    .bind(&track.title)
    .bind(message)
    .execute(pool)
    .await;
}

/// Insert a user-page comment event.
pub async fn push_user_comment(pool: &PgPool, source: &User, target: &User, message: &str) {
    let _ = sqlx::query(
        "INSERT INTO events (type, target_id, target_name, source_id, source_name, track_id, track_title, message, date) \
         VALUES ('user_comment'::event_type, $1, $2, $3, $4, 0, '', $5, now())"
    )
    .bind(target.id)
    .bind(&target.name)
    .bind(source.id)
    .bind(&source.name)
    .bind(message)
    .execute(pool)
    .await;
}
