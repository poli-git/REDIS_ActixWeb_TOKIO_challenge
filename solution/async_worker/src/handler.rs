use reqwest::Client;
use storage::{connections::db::establish_connection, schema::events::is_active};
use storage::event::add_event;
use storage::models::event::NewEvent;
use log::{error, info};
use uuid::Uuid;
use quick_xml::de::from_str;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename = "planList")]
pub struct PlanList {
    pub output: Output,
}

#[derive(Debug, Deserialize)]
pub struct Output {
    #[serde(rename = "base_plan")]
    pub base_plan: Vec<BasePlan>,
}

#[derive(Debug, Deserialize)]
pub struct BasePlan {
    #[serde(rename = "@base_plan_id")]
    pub base_plan_id: Option<u32>,
    #[serde(rename = "@sell_mode")]
    pub sell_mode: Option<String>,
    #[serde(rename = "@organizer_company_id")]
    pub organizer_company_id: Option<u32>,
    #[serde(rename = "@title")]
    pub title: String,
    #[serde(rename = "plan")]
     pub plans: Vec<Plan>,
}

#[derive(Debug, Deserialize)]
pub struct Plan {
    #[serde(rename = "@plan_start_date")]
    pub plan_start_date: String,
    #[serde(rename = "@plan_end_date")]
    pub plan_end_date: String,
    #[serde(rename = "@plan_id")]
    pub plan_id: Option<u32>,
    #[serde(rename = "@sell_from")]
    pub sell_from: Option<String>,
    #[serde(rename = "@sell_to")]
    pub sell_to: Option<String>,
    #[serde(rename = "@sold_out")]
    pub sold_out: Option<bool>,
    #[serde(rename = "zone")]
    pub zones: Vec<Zone>,
}

#[derive(Debug, Deserialize)]
pub struct Zone {
    #[serde(rename = "@zone_id")]
    pub zone_id: Option<u32>,
    #[serde(rename = "@capacity")]
    pub capacity: Option<u32>,
    #[serde(rename = "@price")]
    pub price: Option<f32>,
    #[serde(rename = "@name")]
    pub name: Option<String>,
    #[serde(rename = "@numbered")]
    pub numbered: Option<bool>,
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

    // Status
    let status = response.status();
    if !status.is_success() {
        error!("HTTP error {} from {}", status, url);
        return;
    }

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
    .base_plan
    .into_iter()
    .flat_map(|bp| {
        bp.plans.into_iter().map(move |plan| NewEvent {
            id: uuid::Uuid::new_v4(),
            plan_id: plan.plan_id.,
            providers_id: provider_id,
            name: bp.title.clone(),
            description: format!(
                "Event from {} to {} with {} zones",
                plan.plan_start_date,
                plan.plan_end_date,
                plan.zones.len()
            ),
            is_active: true,
        })
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
