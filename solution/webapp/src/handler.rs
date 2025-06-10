use redis::AsyncCommands;
use redis::aio::MultiplexedConnection;
use std::collections::{HashSet, HashMap};
use chrono::{NaiveDateTime, TimeZone, Utc};
use log::error;

const ROOT_KEY: &str = "base_event";

pub struct FilterQuery {
    pub starts_at: NaiveDateTime,
    pub ends_at: NaiveDateTime,
}

impl Cache {
    pub async fn get_all_events(
        &self,
        filter_query: FilterQuery,
    ) -> Result<Vec<ProviderABaseEvent>, CacheError> {
        let mut conn = self.conn.clone();
        let start_timestamp = filter_query.starts_at.and_utc().timestamp();
        let end_timestamp = filter_query.ends_at.and_utc().timestamp();

        let mut pipe = redis::pipe();
        pipe.cmd("ZRANGEBYSCORE")
            .arg("event_start")
            .arg(start_timestamp)
            .arg("+inf");
        pipe.cmd("ZRANGEBYSCORE")
            .arg("event_end")
            .arg("-inf")
            .arg(end_timestamp);

        let (start_event_ids, end_event_ids): (Vec<String>, Vec<String>) = pipe
            .query_async(&mut conn)
            .await
            .map_err(|e| {
                error!("Error getting events in Redis: {}", e);
                CacheError::Error(format!("Redis error: {}", e))
            })?;

        // Intersection as in Python
        let event_ids: HashSet<_> = start_event_ids.into_iter().collect();
        let event_ids: HashSet<_> = event_ids
            .intersection(&end_event_ids.into_iter().collect())
            .cloned()
            .collect();

        let mut base_events = HashMap::new();

        for event_id in event_ids {
            let base_ids = self.get_base_ids(&event_id).await?;
            let base_event_data_list = self.get_base_event_data(&base_ids).await?;
            for base_event_data in base_event_data_list {
                if let Ok(base_event) = serde_json::from_slice::<ProviderABaseEvent>(&base_event_data) {
                    base_events.insert(base_event.base_event_id.clone(), base_event);
                }
            }
        }

        Ok(base_events.into_values().collect())
    }

    pub async fn get_base_ids(&self, event_id: &str) -> Result<HashSet<String>, CacheError> {
        let mut conn = self.conn.clone();
        let mut base_ids = HashSet::new();
        let pattern = format!("{}:*:{}", ROOT_KEY, event_id);

        let mut iter: redis::aio::AsyncIter<String> = conn.scan_match(pattern).await?;
        while let Some(base_id) = iter.next_item().await {
            if let Some(id) = base_id.split(':').nth(1) {
                base_ids.insert(id.to_string());
            }
        }
        Ok(base_ids)
    }

    pub async fn get_base_event_data(
        &self,
        base_ids: &HashSet<String>,
    ) -> Result<Vec<Vec<u8>>, CacheError> {
        let mut conn = self.conn.clone();
        let mut result = Vec::new();
        for base_id in base_ids {
            let key = format!("{}:{}", ROOT_KEY, base_id);
            if let Some(data): Option<Vec<u8>> = conn.get(&key).await.map_err(|e| {
                CacheError::Error(format!("Redis get error: {}", e))
            })? {
                result.push(data);
            }
        }
        Ok(result)
    }
}

// Example struct for ProviderABaseEvent
#[derive(Serialize, Deserialize, Clone)]
pub struct ProviderABaseEvent {
    pub base_event_id: String,
    // ... other fields ...
}