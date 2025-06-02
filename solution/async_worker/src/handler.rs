use reqwest::Client;
use storage::connections::db::establish_connection;
use storage::event::add_event;
use storage::models::event::NewEvent;

use crate::event_xml::{EventList, BaseEvent};

use log::{error, info};
use uuid::Uuid;

use quick_xml::de::from_str;

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
    let plan_list: EventList = match from_str(&xml_body) {
        Ok(pl) => pl,
        Err(e) => {
            error!("Failed to parse XML from {}: {}", url, e);
            return;
        }
    };
    // Map EventList into Vec<NewEvent>
    let events: Vec<NewEvent> = event_list
        .output
        .base_events
        .into_iter()
        .map(|bp| NewEvent {
            id: uuid::Uuid::new_v4(),
            providers_id: provider_id,
            name: bp.title,
            description: format!(
                "Event from {} to {} with {} zones",
                bp.event.event_start_date,
                bp.event.event_end_date,
                bp.event.zones.len()
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
