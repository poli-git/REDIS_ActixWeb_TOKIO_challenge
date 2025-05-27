use crate::errors::Error;
use crate::pool::{pooled_connection, PooledConn};
pub use diesel::{
    pg::PgConnection,
    r2d2::{self, ConnectionManager, Pool, PooledConnection},
    Connection,
};

/// Retrieve a pooled connection from the pool
pub fn connect() -> Result<PooledConn, Error> {
    pooled_connection()
}

/// Hits the DB with a noop query to test connectivity
pub fn is_healthy(connect: &PooledConn) -> bool {
    connect.execute("SELECT 1").map(|_| ()).is_ok()
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn it_checks_health() {
        let healthy = is_healthy(&connect().unwrap());
        assert!(healthy);
    }
}
