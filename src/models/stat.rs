use std::collections::HashMap;
use redis::AsyncCommands;
use serde::Serialize;

/// Track/user statistics via Redis. Mirrors src/stat/*.cpp.

#[derive(Debug, Default, Serialize)]
pub struct Stats {
    pub totals: HashMap<String, i64>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub unique_totals: HashMap<String, i64>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub days: HashMap<String, HashMap<String, i64>>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub unique_days: HashMap<String, HashMap<String, i64>>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub referrers: HashMap<String, i64>,
}

pub async fn get(
    redis: &mut redis::aio::MultiplexedConnection,
    stat_type: &str,
    uid: i32,
    tid: i32,
    unique: bool,
) -> i64 {
    let entity = if tid > 0 { format!("track:{tid}") } else { format!("user:{uid}") };
    let suffix = if unique { ":unique" } else { "" };
    let key = format!("stat:{entity}:{stat_type}{suffix}");
    redis.get::<_, Option<i64>>(&key).await.unwrap_or(None).unwrap_or(0)
}

pub async fn get_days(
    redis: &mut redis::aio::MultiplexedConnection,
    stat_type: &str,
    tid: i32,
    unique: bool,
) -> HashMap<String, i64> {
    let suffix = if unique { ":unique" } else { "" };
    let key = format!("stat:track:{tid}:{stat_type}:daily{suffix}");
    redis.hgetall::<_, HashMap<String, i64>>(&key)
        .await
        .unwrap_or_default()
}

pub async fn get_referrers(
    redis: &mut redis::aio::MultiplexedConnection,
    tid: i32,
    limit: isize,
) -> HashMap<String, i64> {
    let key = format!("stat:track:{tid}:referrers");
    let pairs: Vec<(String, i64)> = redis
        .zrevrange_withscores(&key, 0, limit)
        .await
        .unwrap_or_default();
    pairs.into_iter().collect()
}

pub async fn push(
    redis: &mut redis::aio::MultiplexedConnection,
    stat_type: &str,
    uid: i32,
    tid: i32,
    remote_addr: &str,
    referrer: &str,
) -> i64 {
    let _: Option<i64> = redis
        .incr(format!("stat:user:{uid}:{stat_type}"), 1i64)
        .await
        .ok();

    if tid <= 0 { return 0; }

    let count: i64 = redis
        .incr(format!("stat:track:{tid}:{stat_type}"), 1i64)
        .await
        .unwrap_or(0);

    let day = chrono::Utc::now().format("%F").to_string();

    let _: Option<i64> = redis
        .hincr(format!("stat:track:{tid}:{stat_type}:daily"), &day, 1i64)
        .await
        .ok();

    // Unique tracking via SETNX
    let seen_key = format!("stat:track:{tid}:{stat_type}:seen:{day}:{remote_addr}");
    let was_set: bool = redis.set_nx(&seen_key, 1i64).await.unwrap_or(false);
    if was_set {
        let _: () = redis.expire(&seen_key, 86400).await.unwrap_or(());
        let _: Option<i64> = redis.incr(format!("stat:track:{tid}:{stat_type}:unique"), 1i64).await.ok();
        let _: Option<i64> = redis.hincr(format!("stat:track:{tid}:{stat_type}:daily:unique"), &day, 1i64).await.ok();
        let _: Option<f64> = redis.zincr(format!("stat:track:{tid}:referrers"), referrer, 1f64).await.ok();
    }

    count
}

/// Build a full Stats object for a track, mirroring the original's fillMeasurement calls.
pub async fn track_stats(
    redis: &mut redis::aio::MultiplexedConnection,
    uid: i32,
    tid: i32,
    include_daily: bool,
) -> Stats {
    let types = ["trackView", "trackPlay", "trackDownload"];
    let mut stats = Stats::default();

    for t in &types {
        stats.totals.insert(t.to_string(), get(redis, t, uid, tid, false).await);
        stats.unique_totals.insert(t.to_string(), get(redis, t, uid, tid, true).await);

        if include_daily {
            stats.days.insert(t.to_string(), get_days(redis, t, tid, false).await);
            stats.unique_days.insert(t.to_string(), get_days(redis, t, tid, true).await);
        }
    }

    if include_daily {
        stats.referrers = get_referrers(redis, tid, 20).await;
    }

    stats
}
