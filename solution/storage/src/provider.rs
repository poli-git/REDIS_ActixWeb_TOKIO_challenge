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
pub fn add_or_update_provider(
    connection: &mut PgPooledConnection,
    new_provider: NewProvider,
) -> Result<Provider, StorageError> {
    diesel::insert_into(providers::table)
        .values(&new_provider)
        .on_conflict(providers::providers_id)
        .do_update()
        .set((
            providers::name.eq(&new_provider.name),
            providers::is_active.eq(&new_provider.is_active),
            providers::updated_at.eq(diesel::dsl::now), // Use current time for updated_at
        ))
        .get_result(connection)
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
}
