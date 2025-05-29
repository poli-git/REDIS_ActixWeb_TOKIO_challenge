use crate::config::CONFIG;
use crate::errors::StorageError;
use dotenv::dotenv;

pub use diesel::{
    pg::PgConnection,
    r2d2::{self, Builder, ConnectionManager, Pool, PooledConnection},
};

pub type Conn = PgConnection;
pub type PgPool = Pool<ConnectionManager<Conn>>;
pub type PooledConn = PooledConnection<ConnectionManager<Conn>>;

/// Initialize the collection pool
/// WARNING: Calling this more than once will introduce connection leaks
pub fn init_pool<T: Into<String>>(database_url: T) -> Result<PgPool, StorageError> {
    let builder = Builder::new().max_size(CONFIG.database_pool_max_size);
    let manager = ConnectionManager::<Conn>::new(database_url.into());
    let pool = builder.build(manager)?;
    Ok(pool)
}

/// Get the connection pool, initialized with config data
/// WARNING: Calling this more than once will introduce connection leaks
pub fn get_pool() -> Result<PgPool, StorageError> {
    dotenv().ok();
    let pool = init_pool(CONFIG.database_url.to_string())?;
    Ok(pool)
}

/// Get the pooled connection for use in other crates
/// WARNING: Calling this more than once will introduce connection leaks
pub fn pooled_connection() -> Result<PooledConn, StorageError> {
    let pool = get_pool()?;
    let pooled_connection = pool.get()?;
    Ok(pooled_connection)
}

/// Get the connection pool by url
/// WARNING: Calling this more than once will introduce connection leaks
pub fn get_pool_by_url(url: String) -> Result<PgPool, StorageError> {
    let pool = init_pool(url)?;
    Ok(pool)
}

/// Get the pooled connection for use in other crates and tests
/// WARNING: Calling this more than once will introduce connection leaks
pub fn create_pooled_connection_by_url(database_url: String) -> Result<PooledConn, StorageError> {
    let pool = get_pool_by_url(database_url)?;
    let pooled_connection = pool.get()?;

    Ok(pooled_connection)
}
