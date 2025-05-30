use crate::models::provider::*;
use crate::schema::providers;
use diesel::prelude::*; // Import the schema

pub fn get_providers(connection: &mut PgConnection) -> Vec<Provider> {
    providers::table.load::<Provider>(connection).unwrap()
}
