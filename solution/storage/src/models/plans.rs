use crate::models::base_plans::BasePlan;
use crate::schema::plans;
use chrono::prelude::*;
use serde::{Deserialize, Serialize}; 
use uuid::Uuid;

#[derive(
    Debug, Serialize, Deserialize, Associations, Identifiable, Queryable, PartialEq, Clone,
)]
#[diesel(belongs_to(BasePlan, foreign_key = id))]
#[diesel(table_name = plans)]
pub struct Plan {
    pub id: Uuid,
    pub base_plan_id: Uuid,
    pub event_plan_id: i64,
    pub plan_start_date: chrono::NaiveDateTime,
    pub plan_end_date: chrono::NaiveDateTime,
    pub sell_from: Option<chrono::NaiveDateTime>,
    pub sell_to: Option<chrono::NaiveDateTime>,
    pub sold_out: bool,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Insertable)]
#[diesel(table_name = plans)]
pub struct NewPlan {
    pub id: uuid::Uuid,
    pub base_plan_id: uuid::Uuid,
    pub event_plan_id: i64,
    pub plan_start_date: chrono::NaiveDateTime,
    pub plan_end_date: chrono::NaiveDateTime,
    pub sell_from: Option<chrono::NaiveDateTime>,
    pub sell_to: Option<chrono::NaiveDateTime>,
    pub sold_out: bool,
}

impl From<NewPlan> for Plan {
    fn from(plan: NewPlan) -> Self {
        let now = Utc::now().naive_utc();

        Plan {
            id: plan.id,
            base_plan_id: plan.base_plan_id,
            event_plan_id: plan.event_plan_id,
            plan_start_date: plan.plan_start_date,
            plan_end_date: plan.plan_end_date,
            sell_from: plan.sell_from,
            sell_to: plan.sell_to,
            sold_out: plan.sold_out,
            created_at: now,
            updated_at: now,
        }
    }
}
