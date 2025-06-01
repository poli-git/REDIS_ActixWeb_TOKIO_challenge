use crate::models::event::*;
use crate::schema::events;
use diesel::RunQueryDsl;
use crate::error::StorageError;
use crate::connections::db_connection::PgPooledConnection;

pub fn get_events(connection: &mut PgPooledConnection) -> Result<Vec<Event>, StorageError> {
    events::table.load::<Event>(connection).map_err(StorageError::from)
}