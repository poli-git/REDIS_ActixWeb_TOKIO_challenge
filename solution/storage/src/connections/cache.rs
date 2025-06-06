use crate::error::{CacheError, CacheResult};
use dotenv::dotenv;
use futures::{stream, StreamExt};
use redis::Client;
use redis::FromRedisValue;
use redis::{aio::MultiplexedConnection, pipe, AsyncCommands, ToRedisArgs};
use std::env;

pub struct Cache {
    pub(super) conn: MultiplexedConnection,
}

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

    /// Get a value by key from redis.
    pub async fn get(&self, key: String) -> CacheResult<String> {
        let mut conn = self.conn.clone();
        conn.get(key.clone())
            .await
            .map_err(|_| CacheError::NotFound(key))
    }

    /// Get a u64 value by key from redis.
    pub async fn get_as_u64(&self, key: String) -> CacheResult<u64> {
        let mut conn = self.conn.clone();
        conn.get(key.clone())
            .await
            .map_err(|_| CacheError::NotFound(key.clone()))
    }

    /// Get a specific value by a key formated like:
    /// format!("idempotency-key:{val:?}", val=idempotency_key.to_string());
    /// format!("accounts:{val:}", val=account_id.to_string());
    pub async fn hash_get(&self, query: String, value: String) -> CacheResult<String> {
        let mut conn = self.conn.clone();
        conn.hget(query.clone(), value)
            .await
            .map_err(|_| CacheError::NotFound(query))
    }

    /// set a key/value pair in redis
    pub async fn set(&self, key: String, value: String) -> CacheResult<()> {
        let mut conn = self.conn.clone();
        conn.set(key.clone(), value)
            .await
            .map_err(|_| CacheError::CannotSet(key))
    }

    /// Set a key to a provided value if it doesn't already exist.
    /// Return Ok() for either condition.
    /// TODO: consider returning u8 or bool to denote a successful set
    pub async fn set_nx(
        &self,
        key: &str,
        value: impl ToRedisArgs + Send + Sync,
    ) -> CacheResult<()> {
        let mut conn = self.conn.clone();
        conn.set_nx(key, value)
            .await
            .map_err(|_| CacheError::CannotSetNx(key.into()))
    }

    /// Set a key to a provided value with expiration of a key
    pub async fn set_ex(
        &self,
        key: &str,
        value: impl ToRedisArgs + Send + Sync,
        seconds: usize,
    ) -> CacheResult<()> {
        let mut conn = self.conn.clone();
        conn.set_ex(key, value, seconds)
            .await
            .map_err(|_| CacheError::CannotSetEx(key.into()))
    }

    /// Return current value of a key then overwrite that value.
    pub async fn getset(
        &self,
        key: &str,
        value: impl ToRedisArgs + Send + Sync,
    ) -> CacheResult<String> {
        let mut conn = self.conn.clone();
        conn.getset(key, value)
            .await
            .map_err(|_| CacheError::CannotGetSet(key.into()))
    }

    /// Delete a value by key from redis
    /// redis-rs returns Ok() even if the key does not exist
    pub async fn delete(&self, key: String) -> CacheResult<()> {
        let mut conn = self.conn.clone();
        conn.del(key.clone())
            .await
            .map_err(|_| CacheError::CannotDelete(key))
    }

    /// Determine if a key exists in the store
    /// redis-rs returns Ok() even if the key does not exist
    pub async fn exists(&self, key: String) -> CacheResult<bool> {
        let mut conn = self.conn.clone();
        let exists: i32 = conn
            .exists(key.clone())
            .await
            .map_err(|_| CacheError::CannotExists(key))?;
        Ok(exists == 1)
    }

    /// Sets expiration for a cache entry at given key.
    pub async fn expire(&self, key: String, seconds: usize) -> CacheResult<()> {
        let mut conn = self.conn.clone();
        conn.expire(key.clone(), seconds)
            .await
            .map_err(|_| CacheError::CannotExpire(key))?;
        Ok(())
    }

    /// Increment a value stored at the given key. If the value does not exist, it will be set to the delta.
    pub async fn increment(&self, key: &str, delta: u64) -> CacheResult<u64> {
        let mut conn = self.conn.clone();
        let incremented = conn
            .incr(key, delta)
            .await
            .map_err(|e| CacheError::CannotIncrement(key.to_string(), e.to_string()))?;
        Ok(incremented)
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

    /// Get a record and also delete it within a single transaction.
    pub async fn get_delete(&self, key: &str) -> CacheResult<String> {
        let mut conn = self.conn.clone();
        // This is the equivelant of sending MULTI, GET, DEL, EXEC in sequence.
        let (value, _): (String, isize) = pipe()
            .atomic()
            .get(key)
            .del(key)
            .query_async(&mut conn)
            .await
            .map_err(|_| CacheError::CannotGetDelete(key.to_string()))?;

        Ok(value)
    }

    /// Set a key as being watched.
    pub async fn watch(&self, key: &str) -> CacheResult<()> {
        let mut conn = self.conn.clone();
        redis::cmd("WATCH")
            .arg(key)
            .query_async(&mut conn)
            .await
            .map_err(|_| CacheError::CannotWatch(key.to_string()))
    }

    /// Remove the watch on a key.
    pub async fn unwatch(&self, key: &str) -> CacheResult<()> {
        let mut conn = self.conn.clone();
        redis::cmd("UNWATCH")
            .arg(key)
            .query_async(&mut conn)
            .await
            .map_err(|_| CacheError::CannotUnwatch(key.to_string()))
    }

    /// Adds an item to an ordered set at the given key
    pub async fn zadd(&self, key: String, value: String, score: i64) -> CacheResult<()> {
        let mut conn = self.conn.clone();
        conn.zadd(&key, value, score)
            .await
            .map_err(|_| CacheError::CannotZadd(key))?;
        Ok(())
    }

    /// Removes an item from an ordered set matching the given key and value.
    pub async fn zrem(&self, key: String, value: String) -> CacheResult<()> {
        let mut conn = self.conn.clone();
        conn.zrem(&key, value)
            .await
            .map_err(|_| CacheError::CannotZrem(key))?;
        Ok(())
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

    /// Returns the sorted set cardinality (number of elements) of the sorted set stored at key, or 0 if key does not exist.
    pub async fn zcard(&self, key: String) -> CacheResult<u64> {
        let mut conn = self.conn.clone();
        let results = conn.zcard(&key).await.map_err(|er| {
            log::error!("Error to execute zcard operation: {}", er);
            CacheError::CannotZcard(key)
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

    /// Renames key to new_key. It returns an error when key does not exist.
    pub async fn rename(&self, key: String, new_key: String) -> CacheResult<()> {
        let mut conn = self.conn.clone();
        conn.rename(key.clone(), new_key.clone())
            .await
            .map_err(|_| CacheError::CannotRename(key, new_key))?;
        Ok(())
    }
}

/// Queries the redis PING command to determine health
#[allow(clippy::redundant_pattern_matching)] // FIXME
pub async fn is_healthy(cache: &Cache) -> bool {
    let mut conn = cache.conn.clone();
    // The redis command PING should return PONG, but we don't care what's returned as long as we get something back.
    match redis::cmd("PING")
        .query_async::<MultiplexedConnection, String>(&mut conn)
        .await
    {
        Ok(_) => true,
        Err(_) => false,
    }
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
    async fn it_getset() {
        let cache = get_cache().await;
        let key = test_key();

        let value = "9";
        cache.set(key.clone(), value.into()).await.unwrap();

        let updated_value = "4";
        let get_previous_value = cache.getset(&key.clone(), updated_value).await.unwrap();

        let get_updated_value = cache.get(key).await.unwrap();

        assert_eq!(get_previous_value, value);
        assert_eq!(get_updated_value, updated_value);
    }

    #[tokio::test]
    async fn it_sets_nx() {
        let cache = get_cache().await;
        let key = test_key();
        let value_1 = "1";
        let value_2 = "2";

        // validate that it sets a new value
        cache.set_nx(&key.clone(), value_1.clone()).await.unwrap();
        let get_value_1 = cache.get(key.clone()).await.unwrap();
        assert_eq!(get_value_1, value_1);

        // validate that values don't overwrite
        cache.set_nx(&key.clone(), value_2.clone()).await.unwrap();
        let get_value_1 = cache.get(key).await.unwrap();
        assert_eq!(get_value_1, value_1);
    }

    #[tokio::test]
    async fn it_sets_ex() {
        let cache = get_cache().await;
        let key = test_key();
        let value = "131178";
        let seconds = 600;
        cache
            .set_ex(&key.clone(), value.clone(), seconds)
            .await
            .unwrap();
        let get_value = cache.get(key).await.unwrap();
        assert_eq!(value, get_value);
    }

    #[tokio::test]
    async fn it_increments_a_value() {
        let cache = get_cache().await;
        let key = test_key();
        let incremented = cache.increment(&key, 1).await.unwrap();
        assert_eq!(incremented, 1);

        let incremented = cache.increment(&key, 2).await.unwrap();
        assert_eq!(incremented, 3);
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

    #[tokio::test]
    async fn it_get_deletes() {
        let cache = get_cache().await;
        let key = test_key();
        let value = String::from("test");

        cache.set(key.clone(), value.clone()).await.unwrap();

        let get = cache.get_delete(&key).await.unwrap();
        assert_eq!(get, value);

        let _deleted = cache.get(key).await.unwrap_err();
    }

    #[tokio::test]
    async fn it_zadds() {
        let cache = get_cache().await;
        let key = test_key();
        let value = String::from("test");
        cache.zadd(key.clone(), value.clone(), 1).await.unwrap();

        let fetched = cache.zrange_by_score(key, 0, 2).await.unwrap();
        assert_eq!(fetched.first().unwrap(), &value);
    }

    #[tokio::test]
    async fn it_zrems() {
        let cache = get_cache().await;
        let key = test_key();
        let value = String::from("test");
        cache.zadd(key.clone(), value.clone(), 1).await.unwrap();
        let fetched = cache.zrange_by_score(key.clone(), 0, 2).await.unwrap();
        assert_eq!(fetched.first().unwrap(), &value);

        cache.zrem(key.clone(), value).await.unwrap();

        let fetched = cache.zrange_by_score(key.clone(), 0, 2).await.unwrap();
        assert!(fetched.first().is_none());
    }

    #[tokio::test]
    async fn it_zrange_by_scores() {
        let cache = get_cache().await;
        let key = test_key();

        cache
            .zadd(key.clone(), String::from("one"), 1)
            .await
            .unwrap();
        cache
            .zadd(key.clone(), String::from("two"), 2)
            .await
            .unwrap();
        cache
            .zadd(key.clone(), String::from("three"), 3)
            .await
            .unwrap();
        cache
            .zadd(key.clone(), String::from("four"), 4)
            .await
            .unwrap();
        cache
            .zadd(key.clone(), String::from("five"), 5)
            .await
            .unwrap();

        let fetched = cache.zrange_by_score(key.clone(), 2, 4).await.unwrap();
        assert_eq!(fetched.len(), 3);
        assert_eq!(fetched.first().unwrap(), &String::from("two"));
        assert_eq!(fetched.last().unwrap(), &String::from("four"));
    }

    #[tokio::test]
    async fn it_zrange() {
        let cache = get_cache().await;
        let key = test_key();

        cache
            .zadd(key.clone(), String::from("one"), 1)
            .await
            .unwrap();
        cache
            .zadd(key.clone(), String::from("two"), 2)
            .await
            .unwrap();
        cache
            .zadd(key.clone(), String::from("three"), 3)
            .await
            .unwrap();
        cache
            .zadd(key.clone(), String::from("four"), 4)
            .await
            .unwrap();
        cache
            .zadd(key.clone(), String::from("five"), 5)
            .await
            .unwrap();

        let fetched = cache.zrange(key.clone(), 2, 4).await.unwrap();
        assert_eq!(fetched.len(), 3);
        assert_eq!(fetched.first().unwrap(), &String::from("three"));
        assert_eq!(fetched.last().unwrap(), &String::from("five"));
    }

    #[tokio::test]
    async fn it_zcard() {
        let cache = get_cache().await;
        let key = test_key();

        cache
            .zadd(key.clone(), String::from("one"), 1)
            .await
            .unwrap();
        cache
            .zadd(key.clone(), String::from("two"), 2)
            .await
            .unwrap();
        cache
            .zadd(key.clone(), String::from("three"), 3)
            .await
            .unwrap();

        let mut fetched: u64 = cache.zcard(key.clone()).await.unwrap();
        assert_eq!(fetched, 3);

        cache
            .zadd(key.clone(), String::from("four"), 4)
            .await
            .unwrap();
        cache
            .zadd(key.clone(), String::from("five"), 5)
            .await
            .unwrap();

        fetched = cache.zcard(key.clone()).await.unwrap();
        assert_eq!(fetched, 5);
    }

    #[tokio::test]
    async fn it_mgets() {
        let cache = get_cache().await;
        let keys = vec![test_key(), test_key(), test_key()];

        for key in keys.clone() {
            cache
                .set(key.clone(), format!("{}-value", key))
                .await
                .unwrap();
        }

        let values = cache.mget(keys.clone()).await.unwrap();

        values
            .iter()
            .zip(keys.iter())
            .for_each(|(value, key)| assert_eq!(value, &format!("{}-value", key)))
    }

    #[tokio::test]
    async fn it_zscan() {
        let cache = get_cache().await;
        let key = test_key();

        // generate records to test.
        for i in 0..1999 {
            cache.zadd(key.clone(), i.to_string(), i).await.unwrap()
        }

        let results = cache.zscan(key.clone(), 0, 100).await.unwrap();
        let mut index = results.0;

        while index != 0 {
            let results = cache.zscan(key.clone(), index, 100).await.unwrap();
            index = results.0;
        }
        // All records were properly scanned
        assert_eq!(index, 0);
    }

    #[tokio::test]
    async fn it_renames() {
        let cache = get_cache().await;
        let key = test_key();

        let value = "1311";
        cache.set(key.clone(), value.into()).await.unwrap();

        let updated_key = test_key();

        cache
            .rename(key.clone(), updated_key.clone())
            .await
            .unwrap();
        let get_value = cache.get(updated_key).await.unwrap();

        assert_eq!(get_value, value);
    }
}
