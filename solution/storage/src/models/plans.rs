use crate::models::base_plans::BasePlan;
use crate::schema::plans;
use chrono::prelude::*;
use diesel::sql_types::Integer;
use serde::{Deserialize, Serialize}; 
use uuid::Uuid;

#[derive(
    Debug, Serialize, Deserialize, Associations, Identifiable, Queryable, PartialEq, Clone,
)]
#[diesel(belongs_to(BasePlan, foreign_key = base_plans_id))]
#[diesel(table_name = plans)]
#[diesel(primary_key(plans_id))]
pub struct Plan {
    pub plans_id: Uuid,
    pub base_plans_id: Uuid,
    pub event_plan_id: i32,
    pub plan_start_date: chrono::NaiveDateTime,
    pub plan_end_date: chrono::NaiveDateTime,
    pub sell_from: chrono::NaiveDateTime,
    pub sell_to: chrono::NaiveDateTime,
    pub sold_out: bool,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Insertable)]
#[diesel(table_name = plans)]
pub struct NewPlan {
    pub plans_id: uuid::Uuid,
    pub base_plans_id: uuid::Uuid,
    pub event_plan_id: i32,
    pub plan_start_date: chrono::NaiveDateTime,
    pub plan_end_date: chrono::NaiveDateTime,
    pub sell_from: chrono::NaiveDateTime,
    pub sell_to: chrono::NaiveDateTime,
    pub sold_out: bool,
}

impl From<NewPlan> for Plan {
    fn from(plan: NewPlan) -> Self {
        let now = Utc::now().naive_utc();

        Plan {
            plans_id: plan.plans_id,
            base_plans_id: plan.base_plans_id,
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
