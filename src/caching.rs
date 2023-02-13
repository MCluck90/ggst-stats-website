use redis::Commands;
use redis::Connection;
use serde::{Deserialize, Serialize};
use sqlx::{pool::PoolConnection, Postgres};
use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::BTreeMap;
pub fn is_cached(name: &'static str, conn: &Connection) -> Option<String> {
    None
}

pub fn store_cache<T: Serialize>(mut name: &'static str, conn: &mut Connection, info: T) {
    let mut json = serde_json::to_string(&info).unwrap();
    let expiration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + 3600;
    let string_expiration = format!("expiration {}", expiration);
    let string_json = format!("json {}", json);
    let mut hash = BTreeMap::new();
    hash.insert(String::from("json"), json);
    hash.insert(String::from("expiration"), expiration.to_string());
    let _: () = redis::cmd("HSET")
        .arg(format!("name"))
        .arg(hash)
        .query(conn)
        .expect("failed to execute HSET");
}
