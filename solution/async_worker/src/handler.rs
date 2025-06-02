use reqwest::Client;
use storage::connections::db::establish_connection;
use storage::event::add_event;
use storage::models::event::NewEvent;

use log::{error, info};
use uuid::Uuid;

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

    // Deserialize the response body as a list of NewEvent
    let events: Vec<NewEvent> = match response.json().await {
        Ok(evts) => evts,
        Err(e) => {
            error!("Failed to deserialize events from {}: {}", url, e);
            return;
        }
    };

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
