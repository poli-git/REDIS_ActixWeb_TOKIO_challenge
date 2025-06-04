use diesel::Identifiable;
use diesel::Insertable;
use diesel::Queryable;

use crate::models::base_plans::BasePlan;
use crate::schema::plans;
use chrono::prelude::*;
// Import both Serialize and Deserialize
use uuid::Uuid;

#[derive(
    Debug, Serialize, Deserialize, Associations, Identifiable, Queryable, PartialEq, Clone,
)]
#[diesel(belongs_to(BasePlan, foreign_key = id))]
#[diesel(table_name = plans)]
pub struct Plan {
    pub id: Uuid,
    pub base_plan_id: Uuid,
    pub plan_id: i64,
    pub plan_start_date: chrono::NaiveDateTime,
    pub plan_end_date: chrono::NaiveDateTime,
    pub sell_from: Option<chrono::NaiveDateTime>,
    pub sell_to: Option<chrono::NaiveDateTime>,
    pub sold_out: bool,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = plans)]
pub struct NewPlan {
    pub id: uuid::Uuid,
    pub base_plan_id: uuid::Uuid,
    pub plan_id: i64,
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
            plan_id: plan.plan_id,
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
