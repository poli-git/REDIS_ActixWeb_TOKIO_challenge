use tokio::sync::Mutex;

use actix_web::{web, HttpRequest, HttpResponse, Responder};
use actix_web::{web::Json, web::Query, Result};
use serde::{Deserialize, Serialize};
use storage::connections::cache::is_healthy;
use storage::connections::cache::Cache;

#[derive(Serialize, Deserialize)]
pub struct HealthResponse {
    status: String,
}

#[derive(Deserialize)]
pub struct GetSearchRequest {
    starts_at: String,
    ends_at: String,
}

/// Basic healthcheck for services
pub async fn get_health(_req: HttpRequest) -> Result<Json<HealthResponse>> {
    Ok(Json(HealthResponse {
        status: "Basic healthcheck for Web-service: - OK".to_owned(),
    }))
}

/// Extended healthcheck to get more information about the healthiness of
/// dependent services.
pub async fn get_full_health(state: web::Data<Mutex<Cache>>) -> impl Responder {
    let cache = state.lock().await;
    let cache_healthy = is_healthy(&cache).await;

    if cache_healthy {
        HttpResponse::Ok().body("Full health: OK")
    } else {
        HttpResponse::ServiceUnavailable().body("Full health: Redis unavailable")
    }
}

/// Search for available events based on the provided time range.
pub async fn search_available_events(
    state: web::Data<Mutex<Cache>>,
    req: Query<GetSearchRequest>,
) -> impl Responder {
    // Example response using the fields from GetRequest
    HttpResponse::Ok().body(format!(
        "Start - End - {}   :   {}",
        req.starts_at, req.ends_at
    ))
}
