use crate::models::provider::Provider;
use crate::schema::events;
use chrono::prelude::*;
use serde::{Deserialize, Serialize}; // Import both Serialize and Deserialize
use uuid::Uuid;

#[derive(
    Debug, Serialize, Deserialize, Associations, Identifiable, Queryable, PartialEq, Clone,
)]
#[diesel(belongs_to(Provider, foreign_key = providers_id))]
#[diesel(table_name = events)] // Updated attribute for Diesel
pub struct Event {
    pub id: Uuid,
    pub providers_id: Uuid,
    pub name: String,
    pub description: String,
    pub is_active: bool,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Insertable)]
#[diesel(table_name = events)]
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
