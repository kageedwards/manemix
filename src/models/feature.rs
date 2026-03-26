use serde::Serialize;
use sqlx::{PgPool, FromRow};

use super::playlist::Playlist;
use super::track::{Track, TrackContext};

/// User-featured content. Mirrors src/userfeature/feature.{h,cpp}.

pub enum Feature {
    None,
    Track(Vec<Track>),
    Playlist(Playlist, Vec<Track>),
}

#[derive(Debug, Serialize)]
pub struct FeatureContext {
    pub has_featured: bool,
    pub featured_title: Option<String>,
    pub featured_link: Option<String>,
    pub tracks: Vec<TrackContext>,
}

#[derive(FromRow)]
struct FeatureRow {
    r#type: Option<String>,
    r#ref: i32,
}

impl Feature {
    pub async fn for_user(pool: &PgPool, uid: i32, _manemix_dir: &str) -> Self {
        let row = sqlx::query_as::<_, FeatureRow>(
            "SELECT type::text, ref FROM user_features WHERE user_id = $1"
        )
        .bind(uid)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();

        let row = match row {
            Some(r) => r,
            None => return Feature::None,
        };

        let feat_type = row.r#type.unwrap_or_default();
        let feat_ref = row.r#ref;

        if feat_type == "track" {
            if let Some(t) = Track::by_id(pool, feat_ref).await {
                return Feature::Track(vec![t]);
            }
        } else if feat_type == "playlist" {
            if let Some(p) = Playlist::by_id(pool, feat_ref).await {
                let tracks = p.tracks(pool).await;
                return Feature::Playlist(p, tracks);
            }
        }

        Feature::None
    }

    pub fn context(&self, manemix_dir: &str) -> FeatureContext {
        match self {
            Feature::None => FeatureContext {
                has_featured: false,
                featured_title: None,
                featured_link: None,
                tracks: vec![],
            },
            Feature::Track(tracks) => FeatureContext {
                has_featured: true,
                featured_title: None,
                featured_link: None,
                tracks: tracks.iter().map(|t| t.context(manemix_dir)).collect(),
            },
            Feature::Playlist(p, tracks) => FeatureContext {
                has_featured: true,
                featured_title: Some(p.name.clone()),
                featured_link: Some(format!("/playlist/{}", p.id)),
                tracks: tracks.iter().map(|t| t.context(manemix_dir)).collect(),
            },
        }
    }
}
