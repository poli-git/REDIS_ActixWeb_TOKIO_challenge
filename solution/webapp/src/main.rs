use actix_web::{web, App, HttpServer};
use anyhow::Result;
use dotenv::dotenv;
use storage::connections::cache::Cache;
use storage::error::StorageError;
use tokio::sync::Mutex;

use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
//
mod config;
mod errors;
mod handler;
mod service;
use service::ApiDoc;

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
            .service(SwaggerUi::new("/swagger-ui/{_:.*}").url("/openapi.json", ApiDoc::openapi()))
            .app_data(app_data.clone())
            .configure(service::configure)
            .app_data(web::Data::new(ApiDoc::openapi()))
            .app_data(web::Data::new(ApiDoc::openapi().info.clone()))
            .app_data(web::Data::new(ApiDoc::openapi().paths.clone()))
    })
    .disable_signals()
    .bind(config.web_app_server)?
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
    .run()
    .await?;

    Ok(())
}
