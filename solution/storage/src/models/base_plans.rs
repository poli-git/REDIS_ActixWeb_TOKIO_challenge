use crate::models::providers::Provider;
use crate::schema::base_plans;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(
    Debug,
    Serialize,
    Deserialize,
    Associations,
    Identifiable,
    Queryable,
    PartialEq,
    Clone,
    Selectable,
)]
#[diesel(belongs_to(Provider, foreign_key = providers_id))]
#[diesel(table_name = base_plans)]
#[diesel(primary_key(base_plans_id))]
pub struct BasePlan {
    pub base_plans_id: Uuid,
    pub providers_id: Uuid,
    pub event_base_id: String,
    pub title: String,
    pub sell_mode: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Insertable)]
#[diesel(table_name = base_plans)]
pub struct NewBasePlan {
    pub base_plans_id: Uuid,
    pub providers_id: Uuid,
    pub event_base_id: String,
    pub title: String,
    pub sell_mode: String,
}

impl From<NewBasePlan> for BasePlan {
    fn from(base_plan: NewBasePlan) -> Self {
        let now = Utc::now().naive_utc();

        BasePlan {
            base_plans_id: base_plan.base_plans_id,
            event_base_id: base_plan.event_base_id,
            title: base_plan.title,
            sell_mode: base_plan.sell_mode,
            providers_id: base_plan.providers_id,
            created_at: now,
            updated_at: now,
        }
    }
}
