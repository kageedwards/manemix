//! Standalone tag updater for use by transcode.sh.
//! Usage: manemix-updatetags TRACK_ID
//!
//! Reads MANEMIX_DIR and MANEMIX_POSTGRES from the environment,
//! fetches the track's title and artist from the database,
//! then writes ID3/Vorbis/MP4 tags to all transcoded files.

use lofty::prelude::*;
use lofty::config::WriteOptions;
use std::path::PathBuf;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} TRACK_ID", args[0]);
        std::process::exit(1);
    }

    let tid: i32 = args[1].parse().unwrap_or_else(|_| {
        eprintln!("Invalid track ID");
        std::process::exit(1);
    });

    let manemix_dir = std::env::var("MANEMIX_DIR")
        .unwrap_or_else(|_| "/var/lib/manemix".into());

    let pg_url = std::env::var("MANEMIX_POSTGRES")
        .unwrap_or_else(|_| "postgres://localhost/manemix".into());

    let pool = sqlx::PgPool::connect(&pg_url).await.unwrap_or_else(|e| {
        eprintln!("DB connection failed: {e}");
        std::process::exit(1);
    });

    let row: Option<(String, String)> = sqlx::query_as(
        "SELECT tracks.title, users.name FROM tracks, users \
         WHERE tracks.id = $1 AND tracks.user_id = users.id"
    )
    .bind(tid)
    .fetch_optional(&pool)
    .await
    .unwrap_or(None);

    let (title, artist) = match row {
        Some(r) => r,
        None => {
            eprintln!("Track {tid} not found");
            std::process::exit(1);
        }
    };

    let base = PathBuf::from(format!("{manemix_dir}/tracks"));
    let paths = [
        base.join(format!("{tid}.mp3")),
        base.join(format!("{tid}.ogg")),
        base.join(format!("{tid}.m4a")),
        base.join(format!("{tid}.opus")),
    ];

    for path in &paths {
        if !path.exists() { continue; }
        match lofty::read_from_path(path) {
            Ok(mut tagged) => {
                if let Some(tag) = tagged.primary_tag_mut() {
                    tag.set_title(title.clone());
                    tag.set_artist(artist.clone());
                    if let Err(e) = tagged.save_to_path(path, WriteOptions::default()) {
                        eprintln!("Failed to write tags to {}: {e}", path.display());
                    }
                }
            }
            Err(e) => eprintln!("Failed to read {}: {e}", path.display()),
        }
    }
}
