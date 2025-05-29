use crate::schema::events;
use chrono::prelude::Utc;
use std::convert::From;

#[derive(
    Debug,
    Serialize,
    Deserialize,
    Associations,
    Identifiable,
    Insertable,
    Queryable,
    PartialEq,
    Clone,
)]

pub struct Event {
    pub id: i32,
    pub providers_id: i32,
    pub name: String,
    pub description: String,
    pub is_active: bool,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct NewEvent {
    pub id: i32,
    pub providers_id: i32,
    pub name: String,
    pub description: String,
}

impl From<NewEvent> for Event {
    fn from(event: NewEvent) -> Self {
        let now = Utc::now().naive_utc();

        Event {
            id: event.id,
            providers_id: event.providers_id,
            name: event.name,
            description: event.description,
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }
}
