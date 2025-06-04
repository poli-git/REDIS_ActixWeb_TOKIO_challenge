use crate::connections::db::PgPooledConnection;
use crate::error::StorageError;
use crate::models::plans::*;
use crate::schema::plans;
use diesel::insert_into;
use diesel::ExpressionMethods;
use diesel::RunQueryDsl;


pub fn get_plans(connection: &mut PgPooledConnection) -> Result<Vec<Plan>, StorageError> {
    plans::table
        .load::<Plan>(connection)
        .map_err(StorageError::from)
}   
pub fn add_plan(
    connection: &mut PgPooledConnection,
    new_plan: NewPlan,
) -> Result<Plan, StorageError> {
    insert_into(plans::table)
        .values(&new_plan)
        .on_conflict((plans::base_plans_id, plans::event_plan_id))
        .do_update()
        .set((
            plans::plan_start_date.eq(&new_plan.plan_start_date),
            plans::plan_end_date.eq(&new_plan.plan_end_date),
            plans::sell_from.eq(&new_plan.sell_from),
            plans::sell_to.eq(&new_plan.sell_to),
            plans::sold_out.eq(&new_plan.sold_out),
            plans::updated_at.eq(diesel::dsl::now), // Use current time for updated_at
        )) // Handle conflict if the plan already exists
        .get_result(connection)
        .map_err(StorageError::from)
}   
