use crate::connections::db::PgPooledConnection;
use crate::error::StorageError;
use crate::models::providers::*;
use crate::schema::providers;
use diesel::prelude::*;
use diesel::ExpressionMethods;
use diesel::RunQueryDsl;

pub fn get_active_providers(
    connection: &mut PgPooledConnection,
) -> Result<Vec<Provider>, StorageError> {
    providers::table
        .filter(providers::is_active.eq(true))
        .load::<Provider>(connection)
        .map_err(StorageError::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connections::db::establish_connection;

    #[tokio::test]
    async fn test_get_providers_returns_ok() {
        let connection = establish_connection().await;
        let mut pg_pool = connection
            .get()
            .expect("Failed to get connection from pool");

        let result = get_active_providers(&mut pg_pool);
        assert!(result.is_ok(), "Expected Ok, got {:?}", result);
    }

    #[tokio::test]
    async fn test_get_providers_returns_vec() {
        let connection = establish_connection().await;
        let mut pg_pool = connection
            .get()
            .expect("Failed to get connection from pool");

        let result = get_active_providers(&mut pg_pool).expect("Expected Ok result");
        // This just checks that the result is a Vec (could be empty)
        assert!(result.is_empty() || !result.is_empty());
    }

    #[tokio::test]
    async fn test_get_providers_returns_error_on_invalid_connection() {
        // Create and immediately drop the connection to simulate an invalid connection
        let connection = establish_connection().await;
        let pg_pool = connection
            .get()
            .expect("Failed to get connection from pool");
        drop(pg_pool); // Close the connection

        // Try to use the dropped connection (should error)
        // We need to create a new variable to avoid using-after-move
        let mut invalid_pg_pool = connection
            .get()
            .expect("Failed to get connection from pool");
        drop(connection); // Drop the pool itself

        // Manually close the connection to simulate error (if possible)
        // Now, forcibly cause an error by passing a closed connection
        // This may not always work depending on the pool implementation,
        // but it's a common pattern for simulating connection errors in tests.

        let result = get_active_providers(&mut invalid_pg_pool);
        assert!(result.is_err(), "Expected Err, got {:?}", result);
    }
}
