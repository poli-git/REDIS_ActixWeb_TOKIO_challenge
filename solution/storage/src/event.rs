use crate::connections::db::PgPooledConnection;
use crate::error::StorageError;
use crate::models::event::*;
use crate::schema::events;
use crate::schema::events::id;
use crate::schema::events::name;
use crate::schema::events::providers_id;
use diesel::pg::upsert::excluded;
use diesel::prelude::*;
use diesel::RunQueryDsl; // This brings in QueryDsl and ExpressionMethods

pub fn get_events(connection: &mut PgPooledConnection) -> Result<Vec<Event>, StorageError> {
    events::table
        .load::<Event>(connection)
        .map_err(StorageError::from)
}

pub fn add_event(
    connection: &mut PgPooledConnection,
    new_event: NewEvent,
) -> Result<Event, StorageError> {
    use diesel::insert_into;

    insert_into(events::table)
        .values(&new_event)
        .on_conflict((id, providers_id))
        .do_update()
        .set(name.eq(excluded(name))) // Handle conflict if the event already exists
        .get_result(connection)
        .map_err(StorageError::from)
}

pub fn update_event_is_active(
    connection: &mut PgPooledConnection,
    event_id: uuid::Uuid,
    is_active: bool,
) -> Result<Event, StorageError> {
    use crate::schema::events::dsl::{events, id, is_active as is_active_col};

    diesel::update(events.filter(id.eq(event_id)))
        .set(is_active_col.eq(is_active))
        .get_result(connection)
        .map_err(StorageError::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connections::db::establish_connection;
    use uuid::Uuid;

    fn create_test_event() -> NewEvent {
        NewEvent {
            id: Uuid::new_v4(),
            providers_id: Uuid::new_v4(),
            name: "Test Event".to_string(),
            description: "A test event".to_string(),
        }
    }

    #[test]
    fn test_db_connection() {
        let connection = establish_connection();
        let _pg_pool = connection
            .get()
            .expect("Failed to get connection from pool");
    }

    #[test]
    fn test_get_events_returns_ok() {
        let connection = establish_connection();
        let mut pg_pool = connection
            .get()
            .expect("Failed to get connection from pool");

        let result = get_events(&mut pg_pool);
        assert!(result.is_ok(), "Expected Ok, got {:?}", result);
    }

    #[test]
    fn test_add_event_and_get_event() {
        let connection = establish_connection();
        let mut pg_pool = connection
            .get()
            .expect("Failed to get connection from pool");

        let new_event = create_test_event();
        let inserted = add_event(&mut pg_pool, new_event.clone()).expect("Failed to add event");

        assert_eq!(inserted.name, new_event.name);
        assert_eq!(inserted.description, new_event.description);

        // Now try to get all events and check if the inserted one is present
        let events = get_events(&mut pg_pool).expect("Failed to get events");
        assert!(events.iter().any(|e| e.id == inserted.id));
    }

    #[test]
    fn test_update_event_is_active() {
        let connection = establish_connection();
        let mut pg_pool = connection
            .get()
            .expect("Failed to get connection from pool");

        let new_event = create_test_event();
        let inserted = add_event(&mut pg_pool, new_event).expect("Failed to add event");

        // Deactivate the event
        let updated = update_event_is_active(&mut pg_pool, inserted.id, false)
            .expect("Failed to update is_active");
        assert!(!updated.is_active);

        // Reactivate the event
        let updated = update_event_is_active(&mut pg_pool, inserted.id, true)
            .expect("Failed to update is_active");
        assert!(updated.is_active);
    }

    #[test]
    fn test_update_event_is_active_invalid_id() {
        let connection = establish_connection();
        let mut pg_pool = connection
            .get()
            .expect("Failed to get connection from pool");

        let random_id = Uuid::new_v4();
        let result = update_event_is_active(&mut pg_pool, random_id, false);
        assert!(result.is_err(), "Expected error for invalid event id");
    }
}
