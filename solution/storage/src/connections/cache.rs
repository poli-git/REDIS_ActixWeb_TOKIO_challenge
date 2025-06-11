use crate::error::{CacheError, CacheResult};
use chrono::NaiveDateTime;
use dotenv::dotenv;
use futures::{stream, StreamExt};
use log::error;
use redis::Client;
use redis::FromRedisValue;
use redis::Pipeline;
use redis::{aio::MultiplexedConnection, AsyncCommands, ToRedisArgs};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::env;

pub struct Cache {
    pub(super) conn: MultiplexedConnection,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ProviderABaseEvent {
    pub base_event_id: String,
    pub event_id: String,
}

pub struct FilterQuery {
    pub starts_at: NaiveDateTime,
    pub ends_at: NaiveDateTime,
}

const ROOT_KEY: &str = "base_plan";

/// Async Cache implementation for redis
impl Cache {
    pub async fn new() -> Result<Self, CacheError> {
        dotenv().ok();

        let redis_url = env::var("REDIS_URI").expect("REDIS_URI must be set");
        Self::with_url(redis_url.to_string()).await
    }

    pub async fn with_url(redis_url: String) -> CacheResult<Self> {
        let client = Client::open(redis_url.as_str())?;
        let conn = client.get_multiplexed_async_connection().await?;

        Ok(Self { conn })
    }

    // Returns a Redis async pipeline and a cloned connection for use in async contexts.
    async fn pipeline(&self) -> (Pipeline, redis::aio::MultiplexedConnection) {
        (redis::pipe(), self.conn.clone())
    }

    pub async fn cache_plan_dates(
        &self,
        event_base_id: String,
        event_plan_id: String,
        start_date: chrono::NaiveDateTime,
        end_date: chrono::NaiveDateTime,
    ) -> Result<(), CacheError> {
        let (mut pipe, mut conn) = self.pipeline().await;
        pipe.cmd("ZADD")
            .arg("start_date")
            .arg(start_date.and_utc().timestamp())
            .arg(format!(
                "{}:{}",
                event_base_id.clone(),
                event_plan_id.clone()
            ));

        pipe.cmd("ZADD")
            .arg("end_date")
            .arg(end_date.and_utc().timestamp())
            .arg(format!(
                "{}:{}",
                event_base_id.clone(),
                event_plan_id.clone()
            ));

        pipe.query_async::<_, ()>(&mut conn)
            .await
            .map_err(|e| CacheError::Error(format!("Failed to cache plan dates: {}", e)))
    }

    /// Get from sorted Set events start after start_timestamp and events end before end_timestamp.
    pub async fn get_matched_events(
        &self,
        start_timestamp: NaiveDateTime,
        end_timestamp: NaiveDateTime,
    ) -> Result<Vec<ProviderABaseEvent>, CacheError> {
        let (mut pipe, mut conn) = self.pipeline().await;

        pipe.cmd("ZRANGEBYSCORE")
            .arg("start_date")
            .arg(start_timestamp.and_utc().timestamp())
            .arg("+inf");
        pipe.cmd("ZRANGEBYSCORE")
            .arg("end_date")
            .arg("-inf")
            .arg(end_timestamp.and_utc().timestamp());

        let (start_event_ids, end_event_ids): (Vec<String>, Vec<String>) =
            pipe.query_async(&mut conn).await.map_err(|e| {
                error!("Error getting events in Redis: {}", e);
                CacheError::Error(format!("Redis error: {}", e))
            })?;

        // Computational Intersection instead of ZINTERSTORE to avoid blocking the Redis server
        let matched_event_ids: HashSet<_> = start_event_ids.into_iter().collect();
        let matched_event_ids: HashSet<_> = matched_event_ids
            .intersection(&end_event_ids.into_iter().collect())
            .cloned()
            .collect();

        let mut base_events = HashMap::new();

        for event_id in matched_event_ids {
            let parts: Vec<&str> = event_id.split(':').collect();
            if parts.len() != 2 {
                error!("Invalid event ID format: {}", event_id);
                continue;
            }
            let base_id = parts[0].to_string();
            let plan_id = parts[1].to_string();
            let base_id_clone = base_id.clone();

            base_events
                .entry(base_id)
                .or_insert_with(|| ProviderABaseEvent {
                    base_event_id: base_id_clone,
                    event_id: plan_id,
                    // Initialize other fields as needed
                });
        }

        Ok(base_events.into_values().collect())
    }

    /* async fn get_base_event_data(
        &self,
        base_ids: &HashSet<String>,
    ) -> Result<Vec<Vec<u8>>, CacheError> {
        let mut conn = self.conn.clone();
        let mut result = Vec::new();
        for base_id in base_ids {
            let key = format!("{}:{}", ROOT_KEY, base_id);
            if let Some(data_bytes) = conn
                .get(&key)
                .await
                .map_err(|e| CacheError::Error(format!("Redis get error: {}", e)))?
            {
                result.push(data_bytes);
            }
        }
        Ok(result)
    } */

    /// Get a value by key from redis.
    pub async fn get(&self, key: String) -> CacheResult<String> {
        let mut conn = self.conn.clone();
        conn.get(key.clone())
            .await
            .map_err(|_| CacheError::NotFound(key))
    }

    /// Set a key/value pair in redis
    pub async fn set(&self, key: String, value: String) -> CacheResult<()> {
        let mut conn = self.conn.clone();
        conn.set(key.clone(), value)
            .await
            .map_err(|_| CacheError::CannotSet(key))
    }

    /// Scan for any keys that match the given pattern.
    pub async fn scan_match_all(&self, pattern: &str) -> CacheResult<Vec<String>> {
        let mut conn = self.conn.clone();
        let keys = conn
            .scan_match(pattern)
            .await
            .map_err(|_| CacheError::CannotScan(pattern.to_string()))?;
        let keys = stream::unfold(keys, |mut keys| async move {
            let next = keys.next_item().await;
            next.map(|key| (key, keys))
        })
        .collect()
        .await;
        Ok(keys)
    }

    /// Get multiple values from redis. Returns a vec of resulting values.
    /// This does not error if a key does not have a corresponding value,
    /// the resulting value is simply omitted from the returned vec.
    pub async fn mget(&self, keys: Vec<String>) -> CacheResult<Vec<String>> {
        let mut conn = self.conn.clone();

        // Get result from redis as a raw redis Value.
        let result = conn
            .get(keys.clone())
            .await
            .map_err(|e| CacheError::CannotMget(e.to_string()))?;

        // Attempt to convert the result to a vec.
        Vec::<String>::from_redis_value(&result)
            // If that fails, attempt to convert it to a string and wrap it in a vec.
            .or_else(|_| String::from_redis_value(&result).map(|single_value| vec![single_value]))
            .map_err(|e| CacheError::CannotMget(e.to_string()))
    }
}

/// Queries the redis PING command to determine health
pub async fn is_healthy(cache: &Cache) -> bool {
    let mut conn = cache.conn.clone();
    redis::cmd("PING")
        .query_async::<MultiplexedConnection, String>(&mut conn)
        .await
        .is_ok()
}

#[cfg(test)]
pub mod tests {
    use super::*;

    pub fn test_key() -> String {
        format!("test_key_{}", uuid::Uuid::new_v4())
    }

    pub async fn get_cache() -> Cache {
        Cache::new().await.unwrap()
    }

    #[tokio::test]
    async fn it_gets_and_sets_a_value() {
        let cache = get_cache().await;
        let key = test_key();
        let value = "1";
        cache.set(key.clone(), value.into()).await.unwrap();
        let get_value = cache.get(key).await.unwrap();

        assert_eq!(value, get_value);
    }

    #[tokio::test]
    async fn it_scans_records() {
        let cache = get_cache().await;
        let key = test_key();

        // generate records unique to test.
        for i in 0..20 {
            cache
                .set(format!("{}.{}", key, i), i.to_string())
                .await
                .unwrap()
        }

        let pattern = format!("{}.*", key);
        let keys: Vec<String> = cache.scan_match_all(&pattern).await.unwrap();
        assert_eq!(keys.len(), 20);
    }
}
