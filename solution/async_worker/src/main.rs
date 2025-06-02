use storage::connections::db::establish_connection;
use storage::provider::get_providers;

mod handler;
use handler::process_provider_events;

#[tokio::main]
async fn main() {
    // Initialize dotenv and logging
    dotenv::dotenv().ok();
    env_logger::init();

    log::info!("Starting async_worker...");

    // Diesel is synchronous, so we use spawn_blocking to avoid blocking async runtime
    let providers = tokio::task::spawn_blocking(|| {
        let connection = establish_connection();
        let mut pg_pool = connection.get().map_err(|e| e.to_string())?;
        get_providers(&mut pg_pool).map_err(|e| e.to_string())
    })
    .await
    .expect("Failed to join blocking task");

    match providers {
        Ok(providers) => {
            let mut handles = vec![];
            for provider in providers {
                let id = provider.id;
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
}
