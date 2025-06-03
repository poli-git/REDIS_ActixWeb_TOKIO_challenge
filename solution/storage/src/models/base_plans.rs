use crate::models::providers::Provider;
use crate::schema::base_plans;
use chrono::prelude::*;
use serde::{Deserialize, Serialize}; // Import both Serialize and Deserialize
use uuid::Uuid;

#[derive(
    Debug, Serialize, Deserialize, Associations, Identifiable, Queryable, PartialEq, Clone,
)]
#[diesel(belongs_to(Provider, foreign_key = providers_id))]
#[diesel(table_name = base_plans)] // Updated attribute for Diesel
pub struct BasePlan {
    pub id: Uuid,
    #[serde(rename = "providers_id")]
    #[serde(skip_serializing_if = "Uuid::is_nil")]
    pub providers_id: Uuid,
    pub base_plan_id: i64,
    #[serde(rename = "title")]
    #[serde(skip_serializing_if = "String::is_empty")]
    pub title: String,
    #[serde(rename = "sell_mode")]
    #[serde(skip_serializing_if = "String::is_empty")]
    pub sell_mode: String,
    #[serde(rename = "created_at")]
    pub created_at: chrono::NaiveDateTime,
    #[serde(rename = "updated_at")]
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Insertable)]
#[diesel(table_name = events)]
pub struct NewBasePlan {
    pub id: Uuid,
    pub providers_id: Uuid,
    pub base_plan_id: i64,
    pub title: String,
    pub sell_mode: String,
}

impl From<NewBasePlan> for BasePlan {
    fn from(base_plan: NewBasePlan) -> Self {
        let now = Utc::now().naive_utc();

        BasePlan {
            id: base_plan.id,
            base_plan_id: base_plan.base_plan_id,
            title: base_plan.title,
            sell_mode: base_plan.sell_mode,
            providers_id: base_plan.providers_id,
            created_at: now,
            updated_at: now,
        }
    }
}
