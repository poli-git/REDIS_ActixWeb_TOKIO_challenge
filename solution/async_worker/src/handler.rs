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
                // Cache ONLY plan dates that are associated to a base_plan with sell mode = 'online'
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
