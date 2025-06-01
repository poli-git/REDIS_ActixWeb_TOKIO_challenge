use std::thread;
use storage::connections::db::establish_connection;
use storage::provider::get_providers;
use uuid::Uuid;

fn dummy_function(provider_id: Uuid, provider_name: String) {
    println!(
        "Dummy function called for provider: {} - {}",
        provider_id, provider_name
    );
}

#[tokio::main]
async fn main() {
    // Initialize dotenv and logging
    dotenv::dotenv().ok();
    env_logger::init();

    log::info!("Starting async_worker...");

    // Diesel is synchronous, so we use spawn_blocking to avoid blocking async runtime
    let providers = tokio::task::spawn_blocking(|| {
        let connection = establish_connection();
        let mut pg_pool = connection.get().unwrap();
        get_providers(&mut pg_pool)
    })
    .await
    .expect("Failed to join blocking task");

    match providers {
        Ok(providers) => {
            let mut handles = vec![];
            for provider in providers {
                let id = provider.id;
                let name = provider.name.clone();
                // Spawn a real OS thread for each provider
                let handle = thread::spawn(move || {
                    dummy_function(id, name);
                });
                handles.push(handle);
            }
            for handle in handles {
                let _ = handle.join();
            }
        }
        Err(e) => {
            log::error!("Error fetching providers: {}", e);
        }
    }
}
