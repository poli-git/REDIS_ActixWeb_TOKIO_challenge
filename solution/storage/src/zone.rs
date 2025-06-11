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
        .on_conflict((zones::plans_id, zones::event_zone_id, zones::numbered))
        .do_update()
        .set((
            zones::name.eq(&new_zone.name),
            zones::capacity.eq(&new_zone.capacity),
            zones::price.eq(&new_zone.price),
            zones::updated_at.eq(diesel::dsl::now),
        ))
        .returning(Zone::as_returning())
        .get_result::<Zone>(connection)
        .map_err(StorageError::from)
}
