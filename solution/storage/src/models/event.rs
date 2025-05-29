use diesel::prelude::*;
use diesel::Selectable;
use serde::{Deserialize, Serialize};

use chrono::prelude::*;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name=crate::schema::events)]
pub struct Event {
    pub id: Uuid,
    pub providers_id: Uuid,
    pub name: String,
    pub description: String,
    pub is_active: bool,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct NewEvent {
    pub id: Uuid,
    pub providers_id: Uuid,
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
