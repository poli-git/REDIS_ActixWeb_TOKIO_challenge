use crate::connections::db::PgPooledConnection;
use crate::error::StorageError;
use crate::models::base_plans::*;
use crate::schema::base_plans;
use crate::schema::base_plans::sell_mode;
use crate::schema::base_plans::title;
use crate::schema::base_plans::updated_at;
use diesel::insert_into;
use diesel::prelude::*;
use diesel::RunQueryDsl;
use uuid::Uuid; // This brings in QueryDsl and ExpressionMethods

pub fn get_base_plans(connection: &mut PgPooledConnection) -> Result<Vec<BasePlan>, StorageError> {
    base_plans::table
        .load::<BasePlan>(connection)
        .map_err(StorageError::from)
}

pub fn add_or_update_base_plan(
    connection: &mut PgPooledConnection,
    new_base_plan: NewBasePlan,
) -> Result<BasePlan, StorageError> {
    insert_into(base_plans::table)
        .values(&new_base_plan)
        .on_conflict((base_plans::providers_id, base_plans::event_base_id))
        .do_update()
        .set((
            title.eq(&new_base_plan.title),
            sell_mode.eq(&new_base_plan.sell_mode),
            updated_at.eq(diesel::dsl::now), // Use current time for updated_at
        )) // Handle conflict if the event already exists
        .get_result(connection)
        .map_err(StorageError::from)
}

pub fn get_base_plan_by_id(
    connection: &mut PgPooledConnection,
    base_plans_id: Uuid,
) -> Result<Option<BasePlan>, StorageError> {
    base_plans::table
        .filter(base_plans::base_plans_id.eq(base_plans_id))
        .first::<BasePlan>(connection)
        .optional()
        .map_err(StorageError::from)
}
