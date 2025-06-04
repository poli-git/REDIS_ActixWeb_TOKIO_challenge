use crate::models::plans::Plan;
use crate::schema::zones;
use chrono::prelude::*;
use serde::{Deserialize, Serialize}; // Import both Serialize and Deserialize
use uuid::Uuid;

#[derive(
    Debug, Serialize, Deserialize, Associations, Identifiable, Queryable, PartialEq, Clone,
)]
#[diesel(belongs_to(Plan, foreign_key = plans_id))]
#[diesel(table_name = zones)] // Updated attribute or Diesel
#[diesel(primary_key(zones_id))]
pub struct Zone {
    pub zones_id: Uuid,
    pub plans_id: Uuid,
    pub event_zone_id: i64,
    pub name: String,
    pub capacity: i64,
    pub price: f64,
    pub numbered: bool,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Insertable)]
#[diesel(table_name = zones)]
pub struct NewZone {
    pub zones_id: Uuid,
    pub plans_id: Uuid,
    pub event_zone_id: i64,
    pub name: String,
    pub capacity: i64,
    pub price: f64,
    pub numbered: bool,
}

impl From<NewZone> for Zone {
    fn from(zone: NewZone) -> Self {
        let now = Utc::now().naive_utc();

        Zone {
            zones_id: zone.zones_id,
            plans_id: zone.plans_id,
            event_zone_id: zone.event_zone_id,
            name: zone.name,
            capacity: zone.capacity,
            price: zone.price,
            numbered: zone.numbered,
            created_at: now,
            updated_at: now,
        }
    }
}
