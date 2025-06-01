use redis::{Client, Connection, RedisError};
use std::env;

/// Connects to a Redis server using the REDIS_URL environment variable.
pub fn connect() -> Result<Connection, RedisError> {
    let redis_url = env::var("REDIS_URL").expect("REDIS_URL must be set");
    let client = Client::open(redis_url)?;
    client.get_connection()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redis_connect() {
        let result = connect();
        assert!(
            result.is_ok(),
            "Failed to connect to Redis: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_redis_set_and_get() {
        let mut conn = connect().expect("Failed to connect to Redis");
        let key = "test_key";
        let value = "test_value";

        let set_result: redis::RedisResult<()> =
            redis::cmd("SET").arg(key).arg(value).query(&mut conn);
        assert!(
            set_result.is_ok(),
            "Failed to set key in Redis: {:?}",
            set_result.err()
        );

        let get_result: redis::RedisResult<String> = redis::cmd("GET").arg(key).query(&mut conn);
        assert!(
            get_result.is_ok(),
            "Failed to get key from Redis: {:?}",
            get_result.err()
        );
        assert_eq!(get_result.unwrap(), value);
    }

    #[test]
    fn test_redis_del() {
        let mut conn = connect().expect("Failed to connect to Redis");
        let key = "test_del_key";
        let value = "to_delete";

        let _: () = redis::cmd("SET")
            .arg(key)
            .arg(value)
            .query(&mut conn)
            .expect("Failed to set key");
        let del_result: redis::RedisResult<i32> = redis::cmd("DEL").arg(key).query(&mut conn);
        assert!(
            del_result.is_ok(),
            "Failed to delete key from Redis: {:?}",
            del_result.err()
        );
        assert_eq!(del_result.unwrap(), 1);
    }

    #[test]
    fn test_redis_connection_error() {
        // Try to connect to an invalid URL to force an error
        let client = redis::Client::open("redis://invalid:6379");
        assert!(client.is_err(), "Expected error for invalid Redis URL");
    }
}
