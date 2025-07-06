use crate::connections::db::PgPooledConnection;
use crate::error::StorageError;
use crate::models::base_plans::BasePlan;
use crate::models::plans::{NewPlan, Plan};
use crate::schema::base_plans::dsl as base_plans_dsl;
use crate::schema::plans::{self, plan_end_date, plan_start_date};
use diesel::insert_into;
use diesel::prelude::*;

pub fn get_plans(connection: &mut PgPooledConnection) -> Result<Vec<Plan>, StorageError> {
    plans::table
        .load::<Plan>(connection)
        .map_err(StorageError::from)
}

pub fn add_or_update_plan(
    connection: &mut PgPooledConnection,
    new_plan: NewPlan,
) -> Result<Plan, StorageError> {
    insert_into(plans::table)
        .values(&new_plan)
        .on_conflict((plans::base_plans_id, plans::event_plan_id))
        .do_update()
        .set((
            plan_start_date.eq(&new_plan.plan_start_date),
            plan_end_date.eq(&new_plan.plan_end_date),
            plans::sell_from.eq(&new_plan.sell_from),
            plans::sell_to.eq(&new_plan.sell_to),
            plans::sold_out.eq(&new_plan.sold_out),
            plans::updated_at.eq(diesel::dsl::now), // Use current time for updated_at
        )) // Handle conflict if the event already exists
        .get_result(connection)
        .map_err(StorageError::from)
}

pub fn get_all_base_plans_with_plans_by_ids(
    connection: &mut PgPooledConnection,
    event_base_id: &str,
    event_plan_id: &str,
) -> Result<Vec<(BasePlan, Plan)>, StorageError> {
    base_plans_dsl::base_plans
        .filter(base_plans_dsl::event_base_id.eq(event_base_id))
        .inner_join(plans::table.on(base_plans_dsl::base_plans_id.eq(plans::base_plans_id)))
        .filter(plans::event_plan_id.eq(event_plan_id))
        .select((BasePlan::as_select(), Plan::as_select()))
        .load::<(BasePlan, Plan)>(connection)
        .map_err(StorageError::from)
}
