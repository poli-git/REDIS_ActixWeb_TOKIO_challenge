use storage::connections::cache::Cache;
use storage::connections::db::establish_connection;
use storage::error::StorageError;

/// Get a Redis cache connection (async).
pub async fn get_cache() -> Cache {
    Cache::new()
        .await
        .map_err(|e| {
            log::error!("Failed to connect to Redis: {}", e);
            StorageError::from(e)
        })
        .unwrap()
}

/// Get a PostgreSQL pooled connection (async).
pub async fn get_db_connection() -> Option<storage::connections::db::PgPooledConnection> {
    let pool = establish_connection().await;
    match pool.get() {
        Ok(conn) => Some(conn),
        Err(e) => {
            log::error!("Failed to get DB connection: {}", e);
            None
        }
    }
}
