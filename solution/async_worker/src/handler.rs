use reqwest::Client;
use storage::connections::db::establish_connection;
use storage::event::add_event;
use storage::models::event::NewEvent;

use log::{error, info};
use uuid::Uuid;

use quick_xml::de::from_str;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct PlanList {
    output: Output,
}

#[derive(Debug, Deserialize)]
struct Output {
    #[serde(rename = "base_plan")]
    base_plans: Vec<BasePlan>,
}

#[derive(Debug, Deserialize)]
struct BasePlan {
    #[serde(rename = "base_plan_id", default)]
    base_plan_id: Option<u32>,
    title: String,
    plan: Plan,
}

#[derive(Debug, Deserialize)]
struct Plan {
    #[serde(rename = "plan_id", default)]
    plan_id: Option<u32>,
    #[serde(rename = "plan_start_date")]
    plan_start_date: String,
    #[serde(rename = "plan_end_date")]
    plan_end_date: String,
    #[serde(rename = "zone")]
    zones: Vec<Zone>,
}

#[derive(Debug, Deserialize)]
struct Zone {
    #[serde(rename = "zone_id", default)]
    zone_id: Option<u32>,
    capacity: u32,
    price: f32,
    name: String,
    numbered: bool,
}

pub async fn process_provider_events(provider_id: Uuid, provider_name: String, url: String) {
    info!(
        "Fetching events for provider: {} - {}",
        provider_id, provider_name
    );

    // Send GET request to the provider's URL asynchronously
    let client = Client::new();
    let response = match client.get(&url).send().await {
        Ok(resp) => resp,
        Err(e) => {
            error!("Failed to fetch events from {}: {}", url, e);
            return;
        }
    };

    // Fetch the XML body as text
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

    // Map PlanList into Vec<NewEvent>
    let events: Vec<NewEvent> = plan_list
        .output
        .base_plans
        .into_iter()
        .map(|bp| NewEvent {
            id: uuid::Uuid::new_v4(),
            providers_id: provider_id,
            name: bp.title,
            description: format!(
                "Plan from {} to {} with {} zones",
                bp.plan.plan_start_date,
                bp.plan.plan_end_date,
                bp.plan.zones.len()
            ),
        })
        .collect();

    info!("Fetched {} events from {}", events.len(), url);

    // Get DB connection
    let connection = establish_connection();
    let mut pg_pool = match connection.get() {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to get DB connection: {}", e);
            return;
        }
    };

    // Add each event to the database
    for event in events {
        match add_event(&mut pg_pool, event) {
            Ok(inserted) => info!("Added event: {}", inserted.name),
            Err(e) => error!("Failed to add event: {}", e),
        }
    }
}
