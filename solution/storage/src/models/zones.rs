use crate::models::plans::Plan;
use crate::schema::zones;
use chrono::prelude::*;
use serde::{Deserialize, Serialize}; // Import both Serialize and Deserialize
use uuid::Uuid;

#[derive(
    Debug, Serialize, Deserialize, Associations, Identifiable, Queryable, PartialEq, Clone,
)]
#[diesel(belongs_to(Plan, foreign_key = id))]
#[diesel(table_name = zones)] // Updated attribute for Diesel
pub struct Zone {
    pub id: Uuid,
    #[serde(rename = "plan_id")]
    #[serde(skip_serializing_if = "Uuid::is_nil")]
    pub plan_id: Uuid,
    #[serde(rename = "zone_id")]
    pub zone_id: i64,
    #[serde(rename = "name")]
    #[serde(skip_serializing_if = "String::is_empty")]
    pub name: String,
    #[serde(rename = "capacity")]
    pub capacity: i64,
    #[serde(rename = "price")]
    pub price: f64,
    #[serde(rename = "numbered")]
    pub numbered: bool,
    #[serde(rename = "created_at")]
    pub created_at: chrono::NaiveDateTime,
    #[serde(rename = "updated_at")]
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Insertable)]
#[diesel(table_name = zones)]
pub struct NewZone {
    pub id: Uuid,
    pub plan_id: Uuid,
    pub zone_id: i64,
    pub name: String,
    pub capacity: i64,
    pub price: f64,
    pub numbered: bool,
}

impl From<NewZone> for Zone {
    fn from(zone: NewZone) -> Self {
        let now = Utc::now().naive_utc();

        Zone {
            id: zone.id,
            plan_id: zone.plan_id,
            zone_id: zone.zone_id,
            name: zone.name,
            capacity: zone.capacity,
            price: zone.price,
            numbered: zone.numbered,
            created_at: now,
            updated_at: now,
        }
    }
}
