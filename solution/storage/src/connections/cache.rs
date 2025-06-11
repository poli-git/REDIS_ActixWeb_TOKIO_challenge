use crate::error::{CacheError, CacheResult};
use chrono::NaiveDateTime;
use dotenv::dotenv;
use futures::{stream, StreamExt};
use log::{error, info};
use redis::Client;
use redis::FromRedisValue;
use redis::Pipeline;
use redis::{aio::MultiplexedConnection, AsyncCommands};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::env;

pub struct Cache {
    pub(super) conn: MultiplexedConnection,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProviderABaseEvent {
    pub id: String,
    pub title: String,
    pub sell_mode: String,
    pub plan: Plan,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Plan {
    #[serde(rename = "plan_start_date")]
    pub plan_start_date: String,
    #[serde(rename = "plan_end_date")]
    pub plan_end_date: String,
    #[serde(rename = "plan_id")]
    pub plan_id: String,
    #[serde(rename = "sell_from")]
    pub sell_from: String,
    #[serde(rename = "sell_to")]
    pub sell_to: String,
    #[serde(rename = "sold_out")]
    pub sold_out: bool,
    #[serde(rename = "zone")]
    pub zones: Vec<Zone>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Zone {
    #[serde(rename = "zone_id")]
    pub zone_id: String,
    #[serde(rename = "capacity")]
    pub capacity: String,
    #[serde(rename = "price")]
    pub price: String,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "numbered")]
    pub numbered: bool,
}

pub struct FilterQuery {
    pub starts_at: NaiveDateTime,
    pub ends_at: NaiveDateTime,
}

const ROOT_KEY: &str = "plan";

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

    /// Get from sorted Set plans that start after start_timestamp and plans that end before end_timestamp.
    pub async fn get_matched_plans(
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

        // Computational Intersection of start and end event IDs
        if start_event_ids.is_empty() || end_event_ids.is_empty() {
            return Ok(Vec::new());
        }
        // Create a HashSet for start_event_ids for efficient lookup
        let start_event_ids: HashSet<_> = start_event_ids
            .into_iter()
            .map(|id| id.to_string())
            .collect();
        // Create a HashSet for end_event_ids for efficient lookup
        let end_event_ids: HashSet<_> =
            end_event_ids.into_iter().map(|id| id.to_string()).collect();
        // Find the intersection of start and end event IDs
        // This will give us the event IDs that are present in both sets
        if start_event_ids.is_empty() || end_event_ids.is_empty() {
            return Ok(Vec::new());
        }
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

            let key = format!("{}:{}:{}:{}", ROOT_KEY, "*", base_id, plan_id);

            let scan_result = self.get_keys_matching_pattern(&key).await.map_err(|e| {
                error!("Error scanning for plans in Redis: {}", e);
                CacheError::Error(format!("Redis scan error: {}", e))
            })?;
            if scan_result.is_empty() {
                error!("No plans found for base ID: {}", base_id);
                continue;
            }

            // Get plans stored in Redis for the given base_id and plan_id
            // Iterate over the results and deserialize each plan
            for result in scan_result {
                if result.trim().is_empty() {
                    error!("Plan string from Redis is empty for key: {}", key);
                    continue;
                }
                let result_clone = result.clone();
                let plan = self.get(result).await.map_err(|e| {
                    error!("Error getting plan from Redis: {}", e);
                    CacheError::Error(format!("Redis get error: {}", e))
                })?;
                if plan.trim().is_empty() {
                    error!("Plan string is empty for key: {}", result_clone);
                    continue;
                }

                // Remove "@" from all field names before deserialization
                let plan_json = plan
                    .replace("\"@plan_start_date\"", "\"plan_start_date\"")
                    .replace("\"@plan_end_date\"", "\"plan_end_date\"")
                    .replace("\"@plan_id\"", "\"plan_id\"")
                    .replace("\"@sell_from\"", "\"sell_from\"")
                    .replace("\"@sell_to\"", "\"sell_to\"")
                    .replace("\"@sold_out\"", "\"sold_out\"")
                    .replace("\"@zone_id\"", "\"zone_id\"")
                    .replace("\"@capacity\"", "\"capacity\"")
                    .replace("\"@price\"", "\"price\"")
                    .replace("\"@name\"", "\"name\"")
                    .replace("\"@numbered\"", "\"numbered\"");

                let plan: ProviderABaseEvent = serde_json::from_str(&plan_json).map_err(|e| {
                    error!("Error deserializing plan: {} | raw value: {}", e, plan_json);
                    CacheError::Error(format!("Deserialization error: {}", e))
                })?;

                // Insert the plan into the base_events map
                base_events
                    .entry(base_id_clone.clone())
                    .or_insert_with(Vec::new)
                    .push(plan);
            }
        }

        Ok(base_events.into_values().flatten().collect())
    }

    /// Scan for all keys matching the given pattern and return them as Vec<String>
    pub async fn get_keys_matching_pattern(
        &self,
        pattern: &str,
    ) -> Result<Vec<String>, CacheError> {
        let mut conn = self.conn.clone();
        let mut keys = Vec::new();
        let mut iter: redis::AsyncIter<String> = conn
            .scan_match(pattern)
            .await
            .map_err(|_| CacheError::CannotScan(pattern.to_string()))?;

        let mut seen = HashSet::new();
        while let Some(key) = iter.next_item().await {
            if seen.insert(key.clone()) {
                keys.push(key);
            }
        }
        Ok(keys)
    }

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
        let keys: Vec<String> = cache.get_keys_matching_pattern(&pattern).await.unwrap();
        assert_eq!(keys.len(), 20);
    }
     #[tokio::test]
    async fn it_caches_plan_dates() {
        let cache = get_cache().await;
        let event_base_id = "event_base_1".to_string();
        let event_plan_id = "event_plan_1".to_string();
        let start_date = NaiveDateTime::from_timestamp(1_700_000_000, 0);
        let end_date = NaiveDateTime::from_timestamp(1_800_000_000, 0);

        cache
            .cache_plan_dates(
                event_base_id.clone(),
                event_plan_id.clone(),
                start_date,
                end_date,
            )
            .await
            .unwrap();

        let plans = cache
            .get_matched_plans(start_date, end_date)
            .await
            .unwrap();

        assert!(!plans.is_empty());
    }

    #[tokio::test]
    async fn it_checks_health() {
        let cache = get_cache().await;
        let healthy = is_healthy(&cache).await;
        assert!(healthy);
    }

    #[tokio::test]
    async fn it_handles_empty_plan() {
        let cache = get_cache().await;
        let key = test_key();
        let result = cache.get(key.clone()).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), format!("Key not found: {}", key));
    }

    #[tokio::test]
    async fn it_handles_invalid_plan_format() {
        let cache = get_cache().await;
        let key = test_key();
        cache.set(key.clone(), "invalid_plan_format".to_string()).await.unwrap();
        let result = cache.get(key).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Deserialization error"));
    }
}