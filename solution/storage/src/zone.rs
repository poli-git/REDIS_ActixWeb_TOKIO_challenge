use crate::connections::db::PgPooledConnection;
use crate::error::StorageError;
use crate::models::zones::{NewZone, Zone};
use crate::schema::zones;
use diesel::insert_into;
use diesel::prelude::*;
use diesel::RunQueryDsl;

pub fn get_zones(connection: &mut PgPooledConnection) -> Result<Vec<Zone>, StorageError> {
    zones::table
        .select(Zone::as_select())
        .load::<Zone>(connection)
        .map_err(StorageError::from)
}
pub fn add_or_update_zone(
    connection: &mut PgPooledConnection,
    new_zone: NewZone,
) -> Result<Zone, StorageError> {
    insert_into(zones::table)
        .values(&new_zone)
        .on_conflict(zones::zones_id)
        .do_update()
        .set((
            zones::name.eq(&new_zone.name),
            zones::updated_at.eq(diesel::dsl::now), // Use current time for updated_at
        ))
        .returning(Zone::as_returning())
        .get_result::<Zone>(connection)
        .map_err(StorageError::from)
}
