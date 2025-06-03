use crate::models::base_plans::BasePlan;
use crate::schema::plans;
use chrono::prelude::*;
use serde::{Deserialize, Serialize}; // Import both Serialize and Deserialize
use uuid::Uuid; 

#[derive(
    Debug, Serialize, Deserialize, Associations, Identifiable, Queryable, PartialEq, Clone,
)]
#[diesel(belongs_to(BasePlan, foreign_key = base_plan_id))]
#[diesel(table_name = plans)] // Updated attribute for Diesel
pub struct Plan {
    pub id: Uuid,
    #[serde(rename = "base_plan_id")]
    #[serde(skip_serializing_if = "Uuid::is_nil")]
    pub base_plan_id: Uuid,
    pub plan_id: i64,
    #[serde(rename = "plan_start_date")]        
    pub plan_start_date: chrono::NaiveDateTime,
    #[serde(rename = "plan_end_date")]
    pub plan_end_date: chrono::NaiveDateTime,
    #[serde(rename = "sell_from")]
    pub sell_from: Option<chrono::NaiveDateTime>,
    #[serde(rename = "sell_to")]
    pub sell_to: Option<chrono::NaiveDateTime>,
    #[serde(rename = "sold_out")]
    pub sold_out: bool,
    #[serde(rename = "created_at")]
    pub created_at: chrono::NaiveDateTime,
    #[serde(rename = "updated_at")]
    pub updated_at: chrono::NaiveDateTime,
}
 
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Insertable)]
#[diesel(table_name = events)]
pub struct NewPlan {
    pub id: Uuid,
    pub base_plan_id: Uuid,
    pub plan_id: i64,
    pub plan_start_date: chrono::NaiveDateTime,
    pub plan_end_date: chrono::NaiveDateTime,
    pub sell_from: Option<chrono::NaiveDateTime>,
    pub sell_to: Option<chrono::NaiveDateTime>,
    pub sold_out: bool,
}

impl From<NewPlan> for Plan {
    fn from(plan: NewBasePlan) -> Self {
        let now = Utc::now().naive_utc();

        Plan {
            id: plan.id,
            plan_id: plan.plan_id,
            plan_start_date: plan.plan_start_date,
            plan_end_date: plan.plan_end_date,
            sell_from: plan.sell_from,
            sell_to: plan.sell_to,
            sold_out: plan.sold_out,
            base_plan_id: plan.base_plan_id,
            created_at: now,
            updated_at: now,
        }
         
    }
}
