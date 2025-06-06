use tokio::sync::Mutex;

use actix_web::{web, HttpResponse, Responder};
use actix_web::{web::Json, HttpRequest};
use actix_web::{web::Path, Result};
use serde::{Deserialize, Serialize};
use storage::connections::cache::is_healthy;
use storage::connections::cache::Cache;

#[derive(Serialize, Deserialize)]
pub struct HealthResponse {
    status: String,
}

pub struct CommonHealthResponse {
    pub status: bool,
    pub version: String,
    pub environment: String,
    pub db: bool,
    pub cache: bool,
}

/// Basic healthcheck for services
pub async fn get_health(_req: HttpRequest) -> Result<Json<HealthResponse>> {
    Ok(Json(HealthResponse {
        status: "PEPE DE PAPA - ok".to_owned(),
    }))
}

/// Extended healthcheck to get more information about the healthiness of
/// dependent services.
pub async fn get_full_health(state: web::Data<Mutex<Cache>>) -> impl Responder {
    let cache = state.lock().await;
    let cache_healthy = is_healthy(&*cache).await;

    if cache_healthy {
        HttpResponse::Ok().body("Full health: OK")
    } else {
        HttpResponse::ServiceUnavailable().body("Full health: Redis unavailable")
    }
}
