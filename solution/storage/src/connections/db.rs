use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PoolError, PooledConnection};
use dotenv::dotenv;
use std::env;

pub type PgPool = Pool<ConnectionManager<PgConnection>>;
pub type PgPooledConnection = PooledConnection<ConnectionManager<PgConnection>>;

//Connects to Postgres and call init pool
pub fn establish_connection() -> PgPool {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    init_pool(&database_url).expect("Failed to create pool")
}

//Creates a default R2D2 Postgres DB Pool
fn init_pool(database_url: &str) -> Result<PgPool, PoolError> {
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::builder().build(manager)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_establish_connection() {
        let pool = establish_connection();
        let conn = pool.get();
        assert!(
            conn.is_ok(),
            "Failed to get connection from pool: {:?}",
            conn.err()
        );
    }

    #[test]
    fn test_init_pool() {
        let database_url = "postgres://user:password@localhost/test_db";
        let pool = init_pool(database_url);
        assert!(pool.is_ok(), "Failed to create pool: {:?}", pool.err());
    }

    #[test]
    fn test_init_pool_invalid_url() {
        let database_url = "invalid_url";
        let pool = init_pool(database_url);
        assert!(pool.is_err(), "Expected error for invalid URL, got Ok");
    }

    #[test]
    fn test_init_pool_empty_url() {
        let database_url = "";
        let pool = init_pool(database_url);
        assert!(pool.is_err(), "Expected error for empty URL, got Ok");
    }

    #[test]
    fn test_init_pool_missing_env_var() {
        env::remove_var("DATABASE_URL");
        let pool = init_pool("");
        assert!(
            pool.is_err(),
            "Expected error for missing DATABASE_URL, got Ok"
        );
    }
}
