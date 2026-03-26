//! Admin CLI for Manemix.
//! Usage:
//!   manemix-admin feature <TRACK_ID>       — Feature a track immediately
//!   manemix-admin autofeature              — Rotate featured track (for cron)
//!   manemix-admin fqueue <TRACK_ID>        — Queue a track for future featuring
//!   manemix-admin dumptracks [--all] [--id] [--airable]
//!   manemix-admin stats                    — Print DB summary

use std::env;
use std::fs;
use std::io::Write;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        usage();
        std::process::exit(1);
    }

    match args[1].as_str() {
        "feature" => {
            if args.len() != 3 { die("Usage: manemix-admin feature TRACK_ID"); }
            let tid: i32 = parse_tid(&args[2]);
            let pool = connect().await;
            feature(&pool, tid).await;
        }
        "autofeature" => {
            let pool = connect().await;
            let manemix_dir = env::var("MANEMIX_DIR").unwrap_or_else(|_| "/var/lib/manemix".into());
            autofeature(&pool, &manemix_dir).await;
        }
        "fqueue" => {
            if args.len() != 3 { die("Usage: manemix-admin fqueue TRACK_ID"); }
            let tid: i32 = parse_tid(&args[2]);
            let pool = connect().await;
            let manemix_dir = env::var("MANEMIX_DIR").unwrap_or_else(|_| "/var/lib/manemix".into());
            fqueue(&pool, &manemix_dir, tid).await;
        }
        "dumptracks" => {
            let pool = connect().await;
            let all = args.iter().any(|a| a == "--all" || a == "-a");
            let id_only = args.iter().any(|a| a == "--id" || a == "-i");
            let airable = args.iter().any(|a| a == "--airable");
            dumptracks(&pool, all, id_only, airable).await;
        }
        "stats" => {
            let pool = connect().await;
            stats(&pool).await;
        }
        _ => { usage(); std::process::exit(1); }
    }
}

fn usage() {
    eprintln!("Usage: manemix-admin <command> [args]");
    eprintln!("Commands:");
    eprintln!("  feature <TRACK_ID>       Feature a track immediately");
    eprintln!("  autofeature              Rotate featured track (cron)");
    eprintln!("  fqueue <TRACK_ID>        Queue a track for featuring");
    eprintln!("  dumptracks [opts]        Dump track list (--all --id --airable)");
    eprintln!("  stats                    Print DB summary");
}

fn die(msg: &str) -> ! {
    eprintln!("{msg}");
    std::process::exit(1);
}

fn parse_tid(s: &str) -> i32 {
    s.parse().unwrap_or_else(|_| die("Invalid track ID"))
}

async fn connect() -> sqlx::PgPool {
    let url = env::var("MANEMIX_POSTGRES")
        .unwrap_or_else(|_| "postgres://localhost/manemix".into());
    sqlx::PgPool::connect(&url).await.unwrap_or_else(|e| die(&format!("DB: {e}")))
}

// ---- feature: insert a track into featured_tracks immediately ----

async fn feature(pool: &sqlx::PgPool, tid: i32) {
    let r = sqlx::query("INSERT INTO featured_tracks (track_id, date) VALUES ($1, now())")
        .bind(tid)
        .execute(pool)
        .await;
    match r {
        Ok(_) => println!("Featured track {tid}"),
        Err(e) => die(&format!("Failed to feature track {tid}: {e}")),
    }
}

// ---- autofeature: pop from queue or rotate a random featured track ----

async fn autofeature(pool: &sqlx::PgPool, manemix_dir: &str) {
    let queue_path = format!("{manemix_dir}/feature.queue");

    // Try to pop from queue first
    if let Ok(contents) = fs::read_to_string(&queue_path) {
        let mut lines: Vec<&str> = contents.lines().collect();
        if let Some(first) = lines.first() {
            if let Some(tid_str) = first.split_whitespace().next() {
                if let Ok(tid) = tid_str.parse::<i32>() {
                    let _ = sqlx::query("INSERT INTO featured_tracks (track_id, date) VALUES ($1, now())")
                        .bind(tid).execute(pool).await;
                    println!("Featured queued track {tid}");
                    lines.remove(0);
                    let _ = fs::write(&queue_path, lines.join("\n"));
                    return;
                }
            }
        }
    }

    // No queue — rotate a random existing featured track
    let r = sqlx::query(
        "WITH feature_to_push AS ( \
            SELECT track_id, featured_tracks.date \
            FROM tracks JOIN featured_tracks ON id = track_id \
            ORDER BY random() LIMIT 1 \
        ) \
        UPDATE featured_tracks SET date = now() \
        FROM feature_to_push \
        WHERE featured_tracks.date = feature_to_push.date \
        AND featured_tracks.track_id = feature_to_push.track_id"
    ).execute(pool).await;

    match r {
        Ok(res) => {
            if res.rows_affected() > 0 {
                println!("Rotated a random featured track");
            } else {
                println!("No featured tracks to rotate");
            }
        }
        Err(e) => die(&format!("autofeature failed: {e}")),
    }
}

// ---- fqueue: queue a track for future featuring ----

async fn fqueue(pool: &sqlx::PgPool, manemix_dir: &str, tid: i32) {
    // Check if already featured
    let already: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM featured_tracks WHERE track_id = $1)"
    ).bind(tid).fetch_one(pool).await.unwrap_or(false);
    if already { die(&format!("{tid}: already featured")); }

    // Check track exists and is visible
    let row: Option<(String, bool)> = sqlx::query_as(
        "SELECT title, visible FROM tracks WHERE id = $1"
    ).bind(tid).fetch_optional(pool).await.unwrap_or(None);
    let (title, visible) = match row {
        Some(r) => r,
        None => die(&format!("{tid}: track not found")),
    };
    if !visible { die(&format!("{tid}: track not published")); }

    let queue_path = format!("{manemix_dir}/feature.queue");

    // Check if already queued
    if let Ok(contents) = fs::read_to_string(&queue_path) {
        for line in contents.lines() {
            if let Some(id_str) = line.split_whitespace().next() {
                if id_str.parse::<i32>().ok() == Some(tid) {
                    die(&format!("{tid}: already queued"));
                }
            }
        }
    }

    let mut f = fs::OpenOptions::new().create(true).append(true)
        .open(&queue_path).unwrap_or_else(|e| die(&format!("Cannot open queue: {e}")));
    writeln!(f, "{tid} {title}").unwrap_or_else(|e| die(&format!("Write failed: {e}")));
    println!("queued '{title}'");
}

// ---- dumptracks: list tracks to stdout ----

async fn dumptracks(pool: &sqlx::PgPool, all: bool, id_only: bool, airable: bool) {
    if id_only {
        let mut sql = "SELECT id FROM tracks".to_string();
        if !all {
            sql.push_str(" WHERE visible = true");
            if airable { sql.push_str(" AND airable = true"); }
        }
        sql.push_str(" ORDER BY id ASC");
        let rows: Vec<(i32,)> = sqlx::query_as(&sql).fetch_all(pool).await.unwrap_or_default();
        for (id,) in rows { println!("{id}"); }
    } else {
        let mut sql = "SELECT tracks.user_id, users.name, tracks.id, tracks.title \
                        FROM tracks, users WHERE users.id = tracks.user_id".to_string();
        if !all {
            sql.push_str(" AND visible = true");
            if airable { sql.push_str(" AND airable = true"); }
        }
        sql.push_str(" ORDER BY tracks.id ASC");
        let rows: Vec<(i32, String, i32, String)> = sqlx::query_as(&sql)
            .fetch_all(pool).await.unwrap_or_default();
        for (uid, name, tid, title) in rows {
            let name = name.replace('\\', "\\\\").replace('-', "\\-")
                .replace('\n', "").replace('\r', "");
            println!("{uid} {name} - {tid} {title}");
        }
    }
}

// ---- stats: print DB summary ----

async fn stats(pool: &sqlx::PgPool) {
    let total: i64 = sqlx::query_scalar("SELECT count(*) FROM tracks")
        .fetch_one(pool).await.unwrap_or(0);
    let hidden: i64 = sqlx::query_scalar("SELECT count(*) FROM tracks WHERE visible = false")
        .fetch_one(pool).await.unwrap_or(0);
    let playlists: i64 = sqlx::query_scalar("SELECT count(*) FROM playlists")
        .fetch_one(pool).await.unwrap_or(0);
    let users: i64 = sqlx::query_scalar("SELECT count(*) FROM users")
        .fetch_one(pool).await.unwrap_or(0);
    let artists: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM users WHERE EXISTS \
         (SELECT 1 FROM tracks WHERE user_id = users.id AND visible = true)"
    ).fetch_one(pool).await.unwrap_or(0);
    let last_user: String = sqlx::query_scalar(
        "SELECT name FROM users ORDER BY registration DESC LIMIT 1"
    ).fetch_one(pool).await.unwrap_or_else(|_| "(none)".into());

    println!("{total} tracks ({hidden} hidden)");
    println!("{playlists} playlists");
    println!("{users} users");
    println!("{artists} artists");
    println!("Last user: {last_user}");
}
