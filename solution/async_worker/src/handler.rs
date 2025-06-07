use async_worker::xml_models::PlanList;
use reqwest::Client;
use storage::connections::db::establish_connection;

use log::{error, info};
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
    // Send GET request to the provider's URL asynchronously
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
                sell_mode: bp.sell_mode.clone().unwrap_or_default(),
            })
        })
        .collect();

    info!("Fetched {} base_plans from {}", events.len(), url);

    // Get DB connection
    let connection = establish_connection();
    let mut pg_pool = match connection.get() {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to get DB connection: {}", e);
            return;
        }
    };
    // Persist base plans to the database
    log::info!(
        "Persisting base plans for provider: {} - {}",
        provider_id,
        provider_name
    );
    persist_base_plans(plan_list.output.base_plan, provider_id, &mut pg_pool);
    info!(
        "Successfully processed events for provider: {} - {}",
        provider_id, provider_name
    );
}

fn persist_base_plans(
    base_plans: Vec<async_worker::xml_models::BasePlan>,
    provider_id: uuid::Uuid,
    pg_pool: &mut storage::connections::db::PgPooledConnection,
) {
    if base_plans.is_empty() {
        log::warn!("No base plans found for provider ID: {}", provider_id);
        return;
    }
    for bp in base_plans {
        let new_base_plan = NewBasePlan {
            base_plans_id: uuid::Uuid::new_v4(),
            providers_id: provider_id,
            event_base_id: bp.base_plan_id.clone().unwrap_or_default(),
            title: bp.title.clone(),
            sell_mode: bp.sell_mode.clone().unwrap_or_default(),
        };

        match add_or_update_base_plan(pg_pool, new_base_plan) {
            Ok(inserted) => {
                log::info!(
                    "Added base_plan: {} : {}",
                    inserted.title,
                    inserted.base_plans_id
                );
                // Persist plans for this base plan
                persist_plans(bp.plans, inserted.base_plans_id, pg_pool);
            }
            Err(e) => {
                log::error!("Failed to add base_plan: {}", e);
                continue;
            }
        }
    }
}

fn persist_plans(
    bp_plans: Vec<async_worker::xml_models::Plan>,
    base_plans_id: uuid::Uuid,
    pg_pool: &mut storage::connections::db::PgPooledConnection,
) {
    if bp_plans.is_empty() {
        log::warn!("No plans found for base plan ID: {}", base_plans_id);
        return;
    }
    log::info!(
        "Persisting {} plans for base plan ID: {}",
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
            Ok(inserted_plan) => log::info!(
                "Added plan: {} : {}",
                inserted_plan.event_plan_id,
                inserted_plan.plans_id
            ),
            Err(e) => log::error!("Failed to add plan: {}", e),
        }
    }
}
