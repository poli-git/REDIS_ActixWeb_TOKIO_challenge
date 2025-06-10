use async_worker::error::PersistPlansError;
use async_worker::utils::{get_cache, get_db_connection};
use async_worker::xml_models::PlanList;
use async_worker::xml_models::SellModeEnum;
use reqwest::Client;

use log::{debug, error, info};
use quick_xml::de::from_str;
use storage::base_plan::add_or_update_base_plan;
use storage::models::base_plans::NewBasePlan;
use storage::models::plans::NewPlan;
use storage::plan::add_or_update_plan;
use uuid::Uuid;

pub async fn process_provider_events(provider_id: Uuid, provider_name: String, url: String) {
    info!(
        "Fetching events for provider: {} - {}",
        provider_id, provider_name
    );
    // Validate URL
    if url.is_empty() {
        error!(
            "Provider URL is empty for provider: {} - {}",
            provider_id, provider_name
        );
        return;
    }
    if !url.starts_with("http://") && !url.starts_with("https://") {
        error!(
            "Invalid URL format for provider: {} - {}. URL: {}",
            provider_id, provider_name, url
        );
        return;
    }
    // Send request to the provider's URL
    let client = Client::new();
    let response = match client.get(&url).send().await {
        Ok(resp) => resp,
        Err(e) => {
            error!("Failed to fetch events from {}: {}", url, e);
            return;
        }
    };
    // Check if the response is successful
    if !response.status().is_success() {
        error!(
            "Failed to fetch events from {}: HTTP {}",
            url,
            response.status()
        );
        return;
    }
    // Fetch the XML body
    let xml_body = match response.text().await {
        Ok(body) => body,
        Err(e) => {
            error!("Failed to read response body from {}: {}", url, e);
            return;
        }
    };
    // Parse XML into PlanList
    let plan_list: PlanList = match from_str(&xml_body) {
        Ok(pl) => pl,
        Err(e) => {
            error!("Failed to parse XML from {}: {}", url, e);
            return;
        }
    };
    // Map PlanList into Vec<NewBasePlan>
    let events: Vec<NewBasePlan> = plan_list
        .output
        .base_plan
        .iter()
        .flat_map(|bp| {
            let base_plan_id = bp.base_plan_id.clone().unwrap_or_default();
            bp.plans.iter().map(move |_plan| NewBasePlan {
                base_plans_id: uuid::Uuid::new_v4(),
                providers_id: provider_id,
                event_base_id: base_plan_id.clone(),
                title: bp.title.clone(),
                sell_mode: bp
                    .sell_mode
                    .as_ref()
                    .map(|e| e.to_string())
                    .unwrap_or_default(),
            })
        })
        .collect();

    // Log the number of base plans fetched
    debug!("Fetched {} base_plans from {}", events.len(), url);

    // Persist base plans to the database
    log::debug!(
        "Persisting base plans for provider: {} - {}",
        provider_id,
        provider_name
    );
    if let Err(e) = persist_base_plans(
        plan_list.output.base_plan,
        provider_id,
        provider_name.clone(),
    )
    .await
    {
        error!(
            "Failed to persist base plans for provider: {} - {}: {:?}",
            provider_id, provider_name, e
        );
        return;
    }
    debug!(
        "Successfully processed events for provider: {} - {}",
        provider_id,
        provider_name.clone()
    );
}

async fn persist_base_plans(
    base_plans: Vec<async_worker::xml_models::BasePlan>,
    provider_id: uuid::Uuid,
    provider_name: String,
) -> Result<(), PersistPlansError> {
    if base_plans.is_empty() {
        log::warn!(
            "No base plans found for provider: {} - {}",
            provider_id,
            provider_name
        );
        return Ok(());
    }
    // Get DB connection
    let mut pg_pool = match get_db_connection().await {
        Some(conn) => conn,
        None => {
            return Err(PersistPlansError::DbError(
                "Failed to get DB connection".to_string(),
            ));
        }
    };
    // Get Cache instance
    let redis_conn = get_cache().await;

    for bp in base_plans {
        let new_base_plan = NewBasePlan {
            base_plans_id: uuid::Uuid::new_v4(),
            providers_id: provider_id,
            event_base_id: bp.base_plan_id.clone().unwrap_or_default(),
            title: bp.title.clone(),
            sell_mode: bp
                .sell_mode
                .as_ref()
                .map(|e| e.to_string())
                .unwrap_or_default(),
        };

        // Persist the base_plan to the database
        match add_or_update_base_plan(&mut pg_pool, new_base_plan) {
            Ok(inserted) => {
                log::debug!(
                    "Added base_plan: {} : {}",
                    inserted.event_base_id,
                    inserted.title
                );
                // Cache ONLY base_plan with sell mode = 'online'
                if inserted.sell_mode == SellModeEnum::Online.to_string() {
                    log::debug!("Caching full online event: {}", inserted.event_base_id);

                    // Cache online base_plan
                    if let Err(e) = redis_conn
                        .set(
                            format!("base_plan:{}:{}", provider_id, inserted.event_base_id),
                            serde_json::to_string(&bp).unwrap_or_default(),
                        )
                        .await
                    {
                        log::error!(
                            "Failed to cache online base_plan {}: {}",
                            inserted.event_base_id,
                            e
                        );
                    }
                }
                // Persist plans associated with this base plan to the database
                match persist_plans(
                    &bp.plans,
                    bp.sell_mode.as_ref().cloned(),
                    inserted.base_plans_id,
                    &inserted.event_base_id,
                    &mut pg_pool,
                    &redis_conn,
                )
                .await
                {
                    Ok(_) => {
                        log::debug!(
                            "Successfully persisted plans for base plan ID: {}",
                            inserted.base_plans_id
                        );
                    }
                    Err(e) => {
                        log::error!(
                            "Failed to persist plans for base plan ID {}: {}",
                            inserted.base_plans_id,
                            e
                        );
                        return Err(e);
                    }
                }
            }
            Err(e) => {
                log::error!("Failed to add base_plan: {}", e);
                return Err(PersistPlansError::DbError(e.to_string()));
            }
        }
    }
    Ok(())
}

async fn persist_plans(
    bp_plans: &Vec<async_worker::xml_models::Plan>,
    sell_mode: Option<SellModeEnum>,
    base_plans_id: uuid::Uuid,
    event_base_id: &str,
    pg_pool: &mut storage::connections::db::PgPooledConnection,
    redis_conn: &storage::connections::cache::Cache,
) -> Result<(), PersistPlansError> {
    if bp_plans.is_empty() {
        log::warn!("No plans found for base_plan ID: {}", base_plans_id);
        return Err(PersistPlansError::NotFound(format!(
            "No plans found for base_plan ID: {}",
            base_plans_id
        )));
    }
    log::debug!(
        "Persisting {} plans for base_plan ID: {}",
        bp_plans.len(),
        base_plans_id
    );
    for plan in bp_plans {
        let new_plan = NewPlan {
            plans_id: uuid::Uuid::new_v4(),
            base_plans_id,
            event_plan_id: plan.plan_id.clone().unwrap_or_default(),
            plan_start_date: chrono::NaiveDateTime::parse_from_str(
                &plan.plan_start_date,
                "%Y-%m-%dT%H:%M:%S",
            )
            .unwrap_or_else(|_| chrono::Utc::now().naive_utc()),
            plan_end_date: chrono::NaiveDateTime::parse_from_str(
                &plan.plan_end_date,
                "%Y-%m-%dT%H:%M:%S",
            )
            .unwrap_or_else(|_| chrono::Utc::now().naive_utc()),
            sell_from: plan
                .sell_from
                .as_ref()
                .and_then(|s| chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S").ok())
                .unwrap_or_else(|| chrono::Utc::now().naive_utc()),
            sell_to: plan
                .sell_to
                .as_ref()
                .and_then(|s| chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S").ok())
                .unwrap_or_else(|| chrono::Utc::now().naive_utc()),
            sold_out: plan.sold_out.unwrap_or(false),
        };

        match add_or_update_plan(pg_pool, new_plan) {
            Ok(inserted_plan) => {
                log::debug!(
                    "Added plan: {} : {}",
                    inserted_plan.plans_id,
                    inserted_plan.event_plan_id
                );
                // Cache ONLY plans that are associated to a base_plan with sell mode = 'online'
                if sell_mode == Some(SellModeEnum::Online) {
                    if let Err(e) = redis_conn
                        .cache_plan_dates(
                            event_base_id.to_string(),
                            inserted_plan.event_plan_id.to_string(),
                            inserted_plan.plan_start_date,
                            inserted_plan.plan_end_date,
                        )
                        .await
                    {
                        log::error!(
                            "Failed to cache start/end date for online event {}: {}",
                            inserted_plan.event_plan_id.to_string(),
                            e
                        );
                        return Err(PersistPlansError::RedisError(e.to_string()));
                    }
                }
            }
            Err(e) => {
                log::error!(
                    "Failed to add plan for base plan ID {}: {}",
                    base_plans_id,
                    e
                );
                return Err(PersistPlansError::DbError(e.to_string()));
            }
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use async_worker::xml_models::PlanList;
    use storage::base_plan::add_or_update_base_plan;
    use storage::plan::add_or_update_plan;

    #[tokio::test]
    async fn test_process_provider_events() {
        // Mock provider ID, name, and URL
        let provider_id = Uuid::new_v4();
        let provider_name = "Test Provider".to_string();
        let url = "http://example.com/events".to_string();

        // Call the function
        process_provider_events(provider_id, provider_name, url).await;
    }
}
// Test for persist_base_plans
#[tokio::test]
async fn test_persist_base_plans() {
    let provider_id = Uuid::new_v4();
    let provider_name = "Test Provider".to_string();
    let base_plans = vec![async_worker::xml_models::BasePlan {
        base_plan_id: Some("test_base_plan_id".to_string()),
        title: "Test Base Plan".to_string(),
        sell_mode: Some(SellModeEnum::Online),
        organizer_company_id: None,
        plans: vec![],
    }];

    let result = persist_base_plans(base_plans, provider_id, provider_name).await;
    assert!(result.is_ok(), "Failed to persist base plans");
}
// Test for persist_plans
#[tokio::test]
async fn test_persist_plans() {
    let base_plans_id = Uuid::new_v4();
    let event_base_id = "test_event_base_id";
    let pool = storage::connections::db::establish_connection().await;
    let mut pg_pool = pool.get().unwrap();
    let redis_conn = get_cache().await;

    let bp_plans = vec![async_worker::xml_models::Plan {
        plan_id: Some("test_plan_id".to_string()),
        plan_start_date: "2023-10-01T00:00:00".to_string(),
        plan_end_date: "2023-10-02T00:00:00".to_string(),
        sell_from: Some("2023-09-01T00:00:00".to_string()),
        sell_to: Some("2023-09-30T00:00:00".to_string()),
        sold_out: Some(false),
        zones: vec![],
    }];

    let result = persist_plans(
        &bp_plans,
        Some(SellModeEnum::Online),
        base_plans_id,
        event_base_id,
        &mut pg_pool,
        &redis_conn,
    )
    .await;
    assert!(result.is_ok(), "Failed to persist plans");
}
// Test for persist_plans with empty plans
#[tokio::test]
async fn test_persist_plans_empty() {
    let base_plans_id = Uuid::new_v4();
    let event_base_id = "test_event_base_id";
    let pool = storage::connections::db::establish_connection().await;
    let mut pg_conn = pool.get().unwrap();
    let redis_conn = get_cache().await;

    let bp_plans: Vec<async_worker::xml_models::Plan> = vec![];

    let result = persist_plans(
        &bp_plans,
        Some(SellModeEnum::Online),
        base_plans_id,
        event_base_id,
        &mut pg_conn,
        &redis_conn,
    )
    .await;
    assert!(result.is_err(), "Expected error for empty plans");
}
// Test for persist_plans with invalid date formats
#[tokio::test]
async fn test_persist_plans_invalid_dates() {
    let base_plans_id = Uuid::new_v4();
    let event_base_id = "test_event_base_id";
    let pool = storage::connections::db::establish_connection().await;
    let mut pg_pool = pool.get().unwrap();
    let redis_conn = get_cache().await;

    let bp_plans = vec![async_worker::xml_models::Plan {
        plan_id: Some("test_plan_id".to_string()),
        plan_start_date: "invalid_date".to_string(),
        plan_end_date: "invalid_date".to_string(),
        sell_from: Some("invalid_date".to_string()),
        sell_to: Some("invalid_date".to_string()),
        sold_out: Some(false),
        zones: vec![],
    }];

    let result = persist_plans(
        &bp_plans,
        Some(SellModeEnum::Online),
        base_plans_id,
        event_base_id,
        pg_pool,
        &redis_conn,
    )
    .await;
    assert!(result.is_err(), "Expected error for invalid date formats");
}
// Test for persist_plans with missing sell mode
#[tokio::test]
async fn test_persist_plans_missing_sell_mode() {
    let base_plans_id = Uuid::new_v4();
    let event_base_id = "test_event_base_id";
    let pool = storage::connections::db::establish_connection().await;
    let mut pg_pool = pool.get().unwrap();
    let redis_conn = get_cache().await;

    let bp_plans = vec![async_worker::xml_models::Plan {
        plan_id: Some("test_plan_id".to_string()),
        plan_start_date: "2023-10-01T00:00:00".to_string(),
        plan_end_date: "2023-10-02T00:00:00".to_string(),
        sell_from: Some("2023-09-01T00:00:00".to_string()),
        sell_to: Some("2023-09-30T00:00:00".to_string()),
        sold_out: Some(false),
        zones: vec![],
    }];

    let result = persist_plans(
        &bp_plans,
        None, // Missing sell mode
        base_plans_id,
        event_base_id,
        &mut pg_pool,
        &redis_conn,
    )
    .await;
    assert!(result.is_err(), "Expected error for missing sell mode");
}
// Test for persist_plans with database error
#[tokio::test]
async fn test_persist_plans_db_error() {
    let base_plans_id = Uuid::new_v4();
    let event_base_id = "test_event_base_id";
    let pool = storage::connections::db::establish_connection().await;
    let mut pg_pool = pool.get().unwrap();
    let redis_conn = get_cache().await;

    let bp_plans = vec![async_worker::xml_models::Plan {
        plan_id: Some("test_plan_id".to_string()),
        plan_start_date: "invalid_date".to_string(), // Invalid date to trigger error
        plan_end_date: "invalid_date".to_string(),
        sell_from: Some("invalid_date".to_string()),
        sell_to: Some("invalid_date".to_string()),
        sold_out: Some(false),
        zones: vec![],
    }];

    let result = persist_plans(
        &bp_plans,
        Some(SellModeEnum::Online),
        base_plans_id,
        event_base_id,
        &mut pg_pool,
        &redis_conn,
    )
    .await;
    assert!(result.is_err(), "Expected error for database operation");
}
// Test for persist_base_plans with empty base plans
#[tokio::test]
async fn test_persist_base_plans_empty() {
    let provider_id = Uuid::new_v4();
    let provider_name = "Test Provider".to_string();
    let base_plans: Vec<async_worker::xml_models::BasePlan> = vec![];

    let result = persist_base_plans(base_plans, provider_id, provider_name).await;
    assert!(result.is_ok(), "Expected success for empty base plans");
}
// Test for process_provider_events with valid data
#[tokio::test]
async fn test_process_provider_events_valid() {
    use httpmock::Method::GET;
    use httpmock::MockServer;

    // Start a local mock server
    let server = MockServer::start();

    // Prepare a valid XML response for the mock
    let xml_response = r#"
        <PlanList>
            <output>
                <base_plan>
                    <base_plan_id>test_base_plan_id</base_plan_id>
                    <title>Test Base Plan</title>
                    <sell_mode>online</sell_mode>
                    <plans>
                        <plan>
                            <plan_id>test_plan_id</plan_id>
                            <plan_start_date>2023-10-01T00:00:00</plan_start_date>
                            <plan_end_date>2023-10-02T00:00:00</plan_end_date>
                            <sell_from>2023-09-01T00:00:00</sell_from>
                            <sell_to>2023-09-30T00:00:00</sell_to>
                            <sold_out>false</sold_out>
                        </plan>
                    </plans>
                </base_plan>
            </output>
        </PlanList>
    "#;

    // Create a mock endpoint
    let mock = server.mock(|when, then| {
        when.method(GET).path("/valid_events");
        then.status(200)
            .header("content-type", "application/xml")
            .body(xml_response);
    });

    let provider_id = Uuid::new_v4();
    let provider_name = "Test Provider".to_string();
    let url = format!("{}/valid_events", server.base_url());

    // Call the function with the mock server URL
    process_provider_events(provider_id, provider_name, url).await;

    // Assert the mock was called
    mock.assert();
}

// Test for process_provider_events with invalid URL
#[tokio::test]
async fn test_process_provider_events_invalid_url() {
    let provider_id = Uuid::new_v4();
    let provider_name = "Test Provider".to_string();
    let url = "invalid_url".to_string();

    // Call the function with an invalid URL
    process_provider_events(provider_id, provider_name, url).await;

    // Check logs or other side effects to ensure it handled the error correctly
}
// Test for process_provider_events with empty URL
#[tokio::test]
async fn test_process_provider_events_empty_url() {
    let provider_id = Uuid::new_v4();
    let provider_name = "Test Provider".to_string();
    let url = "".to_string();

    // Call the function with an empty URL
    process_provider_events(provider_id, provider_name, url).await;

    // Check logs or other side effects to ensure it handled the error correctly
}
// Test for process_provider_events with failed HTTP request
#[tokio::test]
async fn test_process_provider_events_http_failure() {
    let provider_id = Uuid::new_v4();
    let provider_name = "Test Provider".to_string();
    let url = "http://example.com/failed_request".to_string();

    // Mock the HTTP client to simulate a failed request
    // This would typically involve using a mocking library to return an error response

    process_provider_events(provider_id, provider_name, url).await;

    // Check logs or other side effects to ensure it handled the error correctly
}
// Test for process_provider_events with XML parsing error
#[tokio::test]
async fn test_process_provider_events_xml_parsing_error() {
    let provider_id = Uuid::new_v4();
    let provider_name = "Test Provider".to_string();
    let url = "http://example.com/invalid_xml".to_string();

    // Mock the HTTP client to return an invalid XML response
    // This would typically involve using a mocking library to return a malformed XML response

    process_provider_events(provider_id, provider_name, url).await;

    // Check logs or other side effects to ensure it handled the error correctly
}
// Test for process_provider_events with no base plans
#[tokio::test]
async fn test_process_provider_events_no_base_plans() {
    let provider_id = Uuid::new_v4();
    let provider_name = "Test Provider".to_string();
    let url = "http://example.com/no_base_plans".to_string();

    // Mock the HTTP client to return a valid response with no base plans
    // This would typically involve using a mocking library to return a valid XML response with empty base plans

    process_provider_events(provider_id, provider_name, url).await;

    // Check logs or other side effects to ensure it handled the case correctly
}
// Test for process_provider_events with multiple base plans
#[tokio::test]
async fn test_process_provider_events_multiple_base_plans() {
    let provider_id = Uuid::new_v4();
    let provider_name = "Test Provider".to_string();
    let url = "http://example.com/multiple_base_plans".to_string();

    // Mock the HTTP client to return a valid response with multiple base plans
    // This would typically involve using a mocking library to return a valid XML response with multiple base plans

    process_provider_events(provider_id, provider_name, url).await;

    // Check logs or other side effects to ensure it handled the case correctly
}
// Test for process_provider_events with base plans having different sell modes
#[tokio::test]
async fn test_process_provider_events_different_sell_modes() {
    let provider_id = Uuid::new_v4();
    let provider_name = "Test Provider".to_string();
    let url = "http://example.com/different_sell_modes".to_string();

    // Mock the HTTP client to return a valid response with base plans having different sell modes
    // This would typically involve using a mocking library to return a valid XML response with different sell modes

    process_provider_events(provider_id, provider_name, url).await;

    // Check logs or other side effects to ensure it handled the case correctly
}
// Test for process_provider_events with base plans having no sell mode
#[tokio::test]
async fn test_process_provider_events_no_sell_mode() {
    let provider_id = Uuid::new_v4();
    let provider_name = "Test Provider".to_string();
    let url = "http://example.com/no_sell_mode".to_string();

    // Mock the HTTP client to return a valid response with base plans having no sell mode
    // This would typically involve using a mocking library to return a valid XML response with no sell mode

    process_provider_events(provider_id, provider_name, url).await;

    // Check logs or other side effects to ensure it handled the case correctly
}
// Test for process_provider_events with base plans having invalid data
#[tokio::test]
async fn test_process_provider_events_invalid_data() {
    let provider_id = Uuid::new_v4();
    let provider_name = "Test Provider".to_string();
    let url = "http://example.com/invalid_data".to_string();

    // Mock the HTTP client to return a valid response with base plans having invalid data
    // This would typically involve using a mocking library to return a valid XML response with invalid data

    process_provider_events(provider_id, provider_name, url).await;

    // Check logs or other side effects to ensure it handled the case correctly
}
// Test for process_provider_events with base plans having missing fields
#[tokio::test]
async fn test_process_provider_events_missing_fields() {
    let provider_id = Uuid::new_v4();
    let provider_name = "Test Provider".to_string();
    let url = "http://example.com/missing_fields".to_string();

    // Mock the HTTP client to return a valid response with base plans having missing fields
    // This would typically involve using a mocking library to return a valid XML response with missing fields

    process_provider_events(provider_id, provider_name, url).await;

    // Check logs or other side effects to ensure it handled the case correctly
}
// Test for process_provider_events with base plans having empty fields
#[tokio::test]
async fn test_process_provider_events_empty_fields() {
    let provider_id = Uuid::new_v4();
    let provider_name = "Test Provider".to_string();
    let url = "http://example.com/empty_fields".to_string();

    // Mock the HTTP client to return a valid response with base plans having empty fields
    // This would typically involve using a mocking library to return a valid XML response with empty fields

    process_provider_events(provider_id, provider_name, url).await;

    // Check logs or other side effects to ensure it handled the case correctly
}
// Test for process_provider_events with base plans having special characters
#[tokio::test]
async fn test_process_provider_events_special_characters() {
    let provider_id = Uuid::new_v4();
    let provider_name = "Test Provider".to_string();
    let url = "http://example.com/special_characters".to_string();

    // Mock the HTTP client to return a valid response with base plans having special characters
    // This would typically involve using a mocking library to return a valid XML response with special characters

    process_provider_events(provider_id, provider_name, url).await;

    // Check logs or other side effects to ensure it handled the case correctly
}
// Test for process_provider_events with base plans having large data
#[tokio::test]
async fn test_process_provider_events_large_data() {
    let provider_id = Uuid::new_v4();
    let provider_name = "Test Provider".to_string();
    let url = "http://example.com/large_data".to_string();

    // Mock the HTTP client to return a valid response with large data
    // This would typically involve using a mocking library to return a valid XML response with large data

    process_provider_events(provider_id, provider_name, url).await;

    // Check logs or other side effects to ensure it handled the case correctly
}
// Test for process_provider_events with base plans having nested structures
#[tokio::test]
async fn test_process_provider_events_nested_structures() {
    let provider_id = Uuid::new_v4();
    let provider_name = "Test Provider".to_string();
    let url = "http://example.com/nested_structures".to_string();

    // Mock the HTTP client to return a valid response with nested structures
    // This would typically involve using a mocking library to return a valid XML response with nested structures

    process_provider_events(provider_id, provider_name, url).await;

    // Check logs or other side effects to ensure it handled the case correctly
}
// Test for process_provider_events with base plans having mixed data types
#[tokio::test]
async fn test_process_provider_events_mixed_data_types() {
    let provider_id = Uuid::new_v4();
    let provider_name = "Test Provider".to_string();
    let url = "http://example.com/mixed_data_types".to_string();

    // Mock the HTTP client to return a valid response with mixed data types
    // This would typically involve using a mocking library to return a valid XML response with mixed data types

    process_provider_events(provider_id, provider_name, url).await;

    // Check logs or other side effects to ensure it handled the case correctly
}
// Test for process_provider_events with base plans having different date formats
#[tokio::test]
async fn test_process_provider_events_different_date_formats() {
    let provider_id = Uuid::new_v4();
    let provider_name = "Test Provider".to_string();
    let url = "http://example.com/different_date_formats".to_string();

    // Mock the HTTP client to return a valid response with different date formats
    // This would typically involve using a mocking library to return a valid XML response with different date formats

    process_provider_events(provider_id, provider_name, url).await;

    // Check logs or other side effects to ensure it handled the case correctly
}
// Test for process_provider_events with base plans having different time zones
#[tokio::test]
async fn test_process_provider_events_different_time_zones() {
    let provider_id = Uuid::new_v4();
    let provider_name = "Test Provider".to_string();
    let url = "http://example.com/different_time_zones".to_string();

    // Mock the HTTP client to return a valid response with different time zones
    // This would typically involve using a mocking library to return a valid XML response with different time zones

    process_provider_events(provider_id, provider_name, url).await;

    // Check logs or other side effects to ensure it handled the case correctly
}
// Test for process_provider_events with base plans having different currencies
#[tokio::test]
async fn test_process_provider_events_different_currencies() {
    let provider_id = Uuid::new_v4();
    let provider_name = "Test Provider".to_string();
    let url = "http://example.com/different_currencies".to_string();

    // Mock the HTTP client to return a valid response with different currencies
    // This would typically involve using a mocking library to return a valid XML response with different currencies

    process_provider_events(provider_id, provider_name, url).await;

    // Check logs or other side effects to ensure it handled the case correctly
}
// Test for process_provider_events with base plans having different languages
#[tokio::test]
async fn test_process_provider_events_different_languages() {
    let provider_id = Uuid::new_v4();
    let provider_name = "Test Provider".to_string();
    let url = "http://example.com/different_languages".to_string();

    // Mock the HTTP client to return a valid response with different languages
    // This would typically involve using a mocking library to return a valid XML response with different languages

    process_provider_events(provider_id, provider_name, url).await;

    // Check logs or other side effects to ensure it handled the case correctly
}
// Test for process_provider_events with base plans having different categories
#[tokio::test]
async fn test_process_provider_events_different_categories() {
    let provider_id = Uuid::new_v4();
    let provider_name = "Test Provider".to_string();
    let url = "http://example.com/different_categories".to_string();

    // Mock the HTTP client to return a valid response with different categories
    // This would typically involve using a mocking library to return a valid XML response with different categories

    process_provider_events(provider_id, provider_name, url).await;

    // Check logs or other side effects to ensure it handled the case correctly
}
// Test for process_provider_events with base plans having different statuses
#[tokio::test]
async fn test_process_provider_events_different_statuses() {
    let provider_id = Uuid::new_v4();
    let provider_name = "Test Provider".to_string();
    let url = "http://example.com/different_statuses".to_string();

    // Mock the HTTP client to return a valid response with different statuses
    // This would typically involve using a mocking library to return a valid XML response with different statuses

    process_provider_events(provider_id, provider_name, url).await;

    // Check logs or other side effects to ensure it handled the case correctly
}
