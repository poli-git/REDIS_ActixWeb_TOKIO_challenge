use crate::connections::db::PgPooledConnection;
use crate::error::StorageError;
use crate::models::plans::{NewPlan, Plan};
use crate::schema::plans;
use diesel::insert_into;
use diesel::prelude::*;

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
        .get_result(connection)
        .map_err(StorageError::from)
}
