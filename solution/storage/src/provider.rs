use crate::models::provider::*;
use crate::schema::providers;
use diesel::{RunQueryDsl};
use diesel::result::Error; // Import Diesel's error type

use crate::connections::db_connection::PgPooledConnection;

pub fn get_providers(connection: &PgPooledConnection) -> Result<Vec<Provider>, Error> {
    providers::table.load::<Provider>(connection)
}