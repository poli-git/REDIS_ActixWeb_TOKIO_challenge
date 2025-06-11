use actix_web::{web, App, HttpServer};
use anyhow::Result;
use dotenv::dotenv;
use storage::connections::cache::Cache;
use storage::error::StorageError;
use tokio::sync::Mutex;
use webapp::service::get_full_health;
use webapp::service::get_health;
use webapp::service::search_available_events;

mod config;
mod errors;
mod helpers;
mod service;

async fn get_cache() -> Cache {
    Cache::new()
        .await
        .map_err(|e| {
            log::error!("Failed to connect to Redis: {}", e);
            StorageError::from(e)
        })
        .unwrap()
}

#[actix_web::main]
async fn main() -> Result<()> {
    dotenv().ok();
    env_logger::init();
    let config = config::build();

    let redis_conn = get_cache().await;
    let app_data = web::Data::new(Mutex::new(redis_conn));

    log::info!("Starting webapp on {}", config.web_app_server);

    HttpServer::new(move || {
        App::new()
            .app_data(app_data.clone())
            .service(web::resource("/health").route(web::get().to(get_health)))
            .service(web::resource("/health/full").route(web::get().to(get_full_health)))
            .service(web::resource("/search").route(web::get().to(search_available_events)))
    })
    .client_disconnect_timeout(std::time::Duration::from_millis(
        config.actix_client_shutdown_ms as u64,
    ))
    .client_request_timeout(std::time::Duration::from_millis(
        config.actix_client_timeout_ms as u64,
    ))
    .shutdown_timeout(config.actix_shutdown_timeout_s)
    .keep_alive(std::time::Duration::from_secs(
        config.actix_keepalive_seconds as u64,
    ))
    .workers(config.actix_num_workers)
    .bind(config.web_app_server)?
    .run()
    .await?;

    Ok(())
}
