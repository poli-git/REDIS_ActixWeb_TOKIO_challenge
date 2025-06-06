use actix_web::{web, App, HttpServer};
use storage::connections::cache::Cache;
use storage::error::StorageError;
use anyhow::Result;
use dotenv::dotenv;
use tokio::sync::Mutex;
use webapp::service::get_full_health;
use webapp::service::get_health;

mod config;

pub async fn get_cache() -> Cache {
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

    let redis_conn = get_cache().await;
    let app_data = web::Data::new(Mutex::new(redis_conn));

    log::info!("Starting webapp on 0.0.0.0:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(app_data.clone())
            .service(web::resource("/health").route(web::get().to(get_health)))
            .service(web::resource("/health/full").route(web::get().to(get_full_health)))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await?;

    Ok(())
}
