use std::time::Duration;
use storage::connections::db::establish_connection;
use storage::provider::get_active_providers;

mod config;
mod handler;
use handler::process_provider_events;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    env_logger::init();
    let config = config::build();
    let interval_secs = config.async_worker_interval_sec;

    log::info!("Starting async_worker...");

    // Main loop to fetch providers and process events
    loop {
        log::info!("Fetching active providers...");
        // Establish the connection asynchronously before entering the blocking task
        let connection = establish_connection().await;
        let providers = tokio::task::spawn_blocking(move || {
            let mut pg_pool = connection.get().map_err(|e| e.to_string())?;
            get_active_providers(&mut pg_pool).map_err(|e| e.to_string())
        })
        .await
        .expect("Failed to join blocking task");

        match providers {
            Ok(providers) => {
                let mut handles = vec![];
                for provider in providers {
                    let id = provider.providers_id;
                    let name = provider.name.clone();
                    let url = provider.url.clone();

                    log::info!("Processing provider: {} - {}", id, name);

                    // Spawn an async task for each provider
                    let handle = tokio::spawn(async move {
                        process_provider_events(id, name, url).await;
                    });
                    handles.push(handle);
                }
                for handle in handles {
                    let _ = handle.await;
                }
            }
            Err(e) => {
                log::error!("Error fetching providers: {}", e);
            }
        }

        log::info!("Sleeping for {} seconds before next run...", interval_secs);
        tokio::time::sleep(Duration::from_secs(interval_secs.into())).await;
    }
}
