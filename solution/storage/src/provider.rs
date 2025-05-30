use crate::models::provider::*;
use crate::schema::providers;
use diesel::{RunQueryDsl}; // Import the schema

use crate::connections::db_connection::PgPooledConnection;

pub fn get_providers(connection: &PgPooledConnection) -> Vec<Provider> {
    let result = providers::table.load::<Provider>(connection).unwrap();

    result
}
