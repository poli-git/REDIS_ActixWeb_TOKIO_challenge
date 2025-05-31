use crate::models::provider::*;
use crate::schema::providers;
use diesel::result::Error;
use diesel::RunQueryDsl; // Import Diesel's error type

use crate::connections::db_connection::PgPooledConnection;

pub fn get_providers(connection: &mut PgPooledConnection) -> Result<Vec<Provider>, Error> {
    providers::table.load::<Provider>(connection)
}
