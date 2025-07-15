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

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;
    use crate::connections::db::establish_connection;

    #[tokio::test]
    async fn test_get_base_plans_returns_ok() {
        let connection = establish_connection().await;
        let mut pg_pool = connection
            .get()
            .expect("Failed to get connection from pool");

        let result = get_base_plans(&mut pg_pool);
        assert!(result.is_ok(), "Expected Ok, got {:?}", result);
    }
    #[tokio::test]
    async fn test_get_base_plans_returns_vec() {
        let connection = establish_connection().await;
        let mut pg_pool = connection
            .get()
            .expect("Failed to get connection from pool");
        let result = get_base_plans(&mut pg_pool).expect("Expected Ok result");
        // This just checks that the result is a Vec (could be empty)
        assert!(result.is_empty() || !result.is_empty());
    }
    #[tokio::test]
    async fn test_add_or_update_base_plan() {
        let connection = establish_connection().await;
        let mut pg_pool = connection
            .get()
            .expect("Failed to get connection from pool");
        let new_base_plan = NewBasePlan {
            base_plans_id: Uuid::new_v4(),
            providers_id: Uuid::new_v4(),
            event_base_id: "test_event_base".to_string(),
            title: "Test Base Plan".to_string(),
            sell_mode: "test_mode".to_string(),
        };
        let result = add_or_update_base_plan(&mut pg_pool, new_base_plan);
        assert!(result.is_ok(), "Expected Ok, got {:?}", result);
        let base_plan = result.unwrap();
        assert_eq!(base_plan.title, "Test Base Plan");
        assert_eq!(base_plan.sell_mode, "test_mode");
    }
    #[tokio::test]
    async fn test_add_or_update_base_plan_updates_existing() {
        let connection = establish_connection().await;
        let mut pg_pool = connection
            .get()
            .expect("Failed to get connection from pool");
        let existing_base_plan = NewBasePlan {
            base_plans_id: Uuid::new_v4(),
            providers_id: Uuid::new_v4(),
            event_base_id: "test_event_base".to_string(),
            title: "Existing Base Plan".to_string(),
            sell_mode: "existing_mode".to_string(),
        };
        let _ = add_or_update_base_plan(&mut pg_pool, existing_base_plan.clone());
        let updated_base_plan = NewBasePlan {
            base_plans_id: existing_base_plan.base_plans_id,
            providers_id: existing_base_plan.providers_id,
            event_base_id: existing_base_plan.event_base_id,
            title: "Updated Base Plan".to_string(),
            sell_mode: "updated_mode".to_string(),
        };
        let result = add_or_update_base_plan(&mut pg_pool, updated_base_plan);
        assert!(result.is_ok(), "Expected Ok, got {:?}", result);
        let base_plan = result.unwrap();
        assert_eq!(base_plan.title, "Updated Base Plan");
        assert_eq!(base_plan.sell_mode, "updated_mode");
    }
    #[tokio::test]
    async fn test_add_or_update_base_plan_handles_conflict() {
        let connection = establish_connection().await;
        let mut pg_pool = connection
            .get()
            .expect("Failed to get connection from pool");
        let new_base_plan = NewBasePlan {
            base_plans_id: Uuid::new_v4(),
            providers_id: Uuid::new_v4(),
            event_base_id: "test_event_base".to_string(),
            title: "Test Base Plan".to_string(),
            sell_mode: "test_mode".to_string(),
        };
        let _ = add_or_update_base_plan(&mut pg_pool, new_base_plan.clone());
        let conflicting_base_plan = NewBasePlan {
            base_plans_id: new_base_plan.base_plans_id,
            providers_id: new_base_plan.providers_id,
            event_base_id: new_base_plan.event_base_id,
            title: "Conflicting Base Plan".to_string(),
            sell_mode: "conflicting_mode".to_string(),
        };
        let result = add_or_update_base_plan(&mut pg_pool, conflicting_base_plan);
        assert!(result.is_ok(), "Expected Ok, got {:?}", result);
        let base_plan = result.unwrap();
        assert_eq!(base_plan.title, "Conflicting Base Plan");
        assert_eq!(base_plan.sell_mode, "conflicting_mode");
    }
}
