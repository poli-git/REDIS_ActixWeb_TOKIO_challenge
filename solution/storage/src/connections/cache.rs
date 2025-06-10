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
    // ... other fields ...
}

pub struct FilterQuery {
    pub starts_at: NaiveDateTime,
    pub ends_at: NaiveDateTime,
}

const ROOT_KEY: &str = "base_event";

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

        let matched_event_ids: HashSet<_> = start_event_ids.into_iter().collect();
        let matched_event_ids: HashSet<_> = matched_event_ids
            .intersection(&end_event_ids.into_iter().collect())
            .cloned()
            .collect();

        let mut base_events = HashMap::new();

        for event_id in matched_event_ids {
            let base_ids = self.get_base_ids(&event_id).await?;
            let base_event_data_list = self.get_base_event_data(&base_ids).await?;
            for base_event_data in base_event_data_list {
                if let Ok(base_event) =
                    serde_json::from_slice::<ProviderABaseEvent>(&base_event_data)
                {
                    base_events.insert(base_event.base_event_id.clone(), base_event);
                }
            }
        }

        Ok(base_events.into_values().collect())
    }

    async fn get_base_ids(&self, event_id: &str) -> Result<HashSet<String>, CacheError> {
        let mut conn = self.conn.clone();
        let mut base_ids = HashSet::new();
        let pattern = format!("{}:*:{}", ROOT_KEY, event_id);

        let mut iter = conn.scan_match::<String, String>(pattern).await?;
        while let Some(base_id) = iter.next_item().await {
            if let Some(id) = base_id.split(':').nth(1) {
                base_ids.insert(id.to_string());
            }
        }
        Ok(base_ids)
    }
    async fn get_base_event_data(
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
    }

    /// Get a value by key from redis.
    pub async fn get(&self, key: String) -> CacheResult<String> {
        let mut conn = self.conn.clone();
        conn.get(key.clone())
            .await
            .map_err(|_| CacheError::NotFound(key))
    }

    /// set a key/value pair in redis
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

    /// Returns the specified range of elements in the sorted set stored at specific `key`.
    ///
    /// The order of elements is from the lowest to the highest score. Elements with the same score are ordered lexicographically.
    ///
    /// The `min` and `max` arguments represent zero-based indexes, where 0 is the first element, 1 is the next element, and so on.
    ///
    /// These arguments specify an inclusive range, so for example, ZRANGE myzset 0 1 will return both the first and the second element of the sorted set.
    /// These arguments can also be negative numbers indicating offsets from the end of the sorted set, with -1 being the last element of the sorted set, -2 the penultimate element, and so on.
    pub async fn zrange(&self, key: String, min: isize, max: isize) -> CacheResult<Vec<String>> {
        let mut conn = self.conn.clone();
        let results = conn.zrange(&key, min, max).await.map_err(|er| {
            log::error!("{}", er);
            CacheError::CannotZrange(key)
        })?;
        Ok(results)
    }

    /// Gets all items in a ordered set contained at the given key that are within the score range.
    ///
    /// For `min` and `max` values, passed in numbers are treated inclusively. It is possible to specify exclusive values by passing in a string prefixed with `(`, such as `"(3"`.
    /// Additionally, negative or positive infinity may be specified as `"-inf"` and `"+inf"`.
    pub async fn zrange_by_score(
        &self,
        key: String,
        min: impl ToRedisArgs + Send + Sync,
        max: impl ToRedisArgs + Send + Sync,
    ) -> CacheResult<Vec<String>> {
        let mut conn = self.conn.clone();
        let results = conn.zrangebyscore(&key, min, max).await.map_err(|er| {
            log::error!("{}", er);
            CacheError::CannotZrangeByScore(key)
        })?;
        Ok(results)
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

    /// Iterates elements of Sorted Set types and their associated scores
    ///
    /// ZSCAN array of elements contain two elements, a member and its associated score, for every returned element of the sorted set.
    /// [COUNT] option is the amount of work that should be done at every call in order to retrieve elements from the collection.
    pub async fn zscan(
        &self,
        key: String,
        cursor: u64,
        count: u64,
    ) -> CacheResult<(u64, Vec<(String, u64)>)> {
        let mut conn = self.conn.clone();
        redis::cmd("ZSCAN")
            .arg(key)
            .arg(cursor)
            .arg("COUNT")
            .arg(count)
            .query_async::<MultiplexedConnection, (u64, Vec<(String, u64)>)>(&mut conn)
            .await
            .map_err(|e| CacheError::CannotZscan(e.to_string()))
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
