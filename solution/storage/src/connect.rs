use crate::errors::{StorageError, StorageResult};
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use r2d2::PooledConnection;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;
pub type DbConnection = PooledConnection<ConnectionManager<PgConnection>>;

pub fn init_pool(database_url: &str) -> StorageResult<DbPool> {
    let manager = ConnectionManager::new(database_url);
    let pool = Pool::builder()
        .build(manager)
        .map_err(|e| StorageError::DatabaseConnectionError(e.to_string()))?;

    Ok(pool)
}
