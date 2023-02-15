use rustis::client::Client;
use rustis::commands::HashCommands;
use serde::Serialize;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

pub async fn is_cached(name: String, conn: &mut Client) -> Option<String> {
    let current: u64 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let index: Result<HashMap<String, String>, rustis::Error> = conn.hgetall(name).await;
    match index {
        Ok(hash) => {
            if hash.is_empty() {
                return None;
            }

            let expiration: u64 = hash["expiration"].parse().unwrap();
            if current < expiration {
                return Some(hash["json"].clone());
            } else {
                return None;
            }
        }
        Err(err) => {
            println!("{:?}", err);
            return None;
        }
    }
}

pub async fn store_cache<T>(name: String, conn: &mut Client, info: &T)
where
    T: Borrow<T>,
    T: Serialize,
{
    let json = serde_json::to_string(&info).unwrap();
    let expiration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + 3600;
    let mut keys = HashMap::new();
    keys.insert("json", json);
    keys.insert("expiration", expiration.to_string());
    let _ = conn.hset(name, keys).await;
}
