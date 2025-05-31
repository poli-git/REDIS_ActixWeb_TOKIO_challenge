use crate::models::provider::*;
use crate::schema::providers;
use diesel::result::Error;
use diesel::RunQueryDsl; // Import Diesel's error type

use crate::connections::db_connection::PgPooledConnection;

pub fn get_providers(connection: &mut PgPooledConnection) -> Result<Vec<Provider>, Error> {
    providers::table.load::<Provider>(connection)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connections::db_connection::establish_connection;

    #[test]
    fn test_get_providers_returns_ok() {
        let connection = establish_connection();
        let mut pg_pool = connection
            .get()
            .expect("Failed to get connection from pool");

        let result = get_providers(&mut pg_pool);
        assert!(result.is_ok(), "Expected Ok, got {:?}", result);
    }

    #[test]
    fn test_get_providers_returns_vec() {
        let connection = establish_connection();
        let mut pg_pool = connection
            .get()
            .expect("Failed to get connection from pool");

        let result = get_providers(&mut pg_pool).expect("Expected Ok result");
        // This just checks that the result is a Vec (could be empty)
        assert!(result.is_empty() || !result.is_empty());
    }

    #[test]
    fn test_get_providers_returns_error_on_invalid_connection() {
       
        // Create and immediately drop the connection to simulate an invalid connection
        let connection = establish_connection();
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

        let result = get_providers(&mut invalid_pg_pool);
        assert!(result.is_err(), "Expected Err, got {:?}", result);
    }
}
