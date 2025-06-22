use log::{debug, error, info};
use quick_xml::de::from_str;
use reqwest::Client;
use storage::models::base_plans::NewBasePlan;
use uuid::Uuid;

use common::persist::persist_base_plans;
use common::xml_models::PlanList;

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
