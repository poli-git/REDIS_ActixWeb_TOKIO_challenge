use crate::connections::db::PgPooledConnection;
use crate::error::StorageError;
use crate::models::zones::{Zone, NewZone};
use crate::schema::zones;
use diesel::insert_into;
use diesel::prelude::*;

pub fn get_zones(connection: &mut PgPooledConnection) -> Result<Vec<Zone>, StorageError> {
    zones::table
        .load::<Zone>(connection)
        .map_err(StorageError::from)
}

pub fn add_zone(
    connection: &mut PgPooledConnection,
    new_zone: NewZone,
) -> Result<Zone, StorageError> {
    insert_into(zones::table)
        .values(&new_zone)
        .get_result(connection)
        .map_err(StorageError::from)
}