use tokio::sync::Mutex;

use actix_web::{web, HttpRequest, HttpResponse, Responder};
use actix_web::{web::Json, web::Query, Result};
use serde::{Deserialize, Serialize};
use storage::connections::cache::is_healthy;
use storage::connections::cache::Cache;

#[derive(Deserialize)]
pub struct SetRequest {
    pub key: String,
    pub value: String,
    pub seconds: Option<usize>, // Optional expiration
}

#[derive(Serialize, Deserialize)]
pub struct HealthResponse {
    status: String,
}

#[derive(Deserialize)]
pub struct GetRequest {
    pub key: String,
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
    let cache_healthy = is_healthy(&cache).await;

    if cache_healthy {
        HttpResponse::Ok().body("Full health: OK")
    } else {
        HttpResponse::ServiceUnavailable().body("Full health: Redis unavailable")
    }
}

// Updated /set endpoint to use GET and query parameters
pub async fn set_key_value(
    state: web::Data<Mutex<Cache>>,
    req: Query<SetRequest>,
) -> impl Responder {
    let cache = state.lock().await;
    let seconds = req.seconds.unwrap_or(3600); // default to 1 hour if not provided

    let _res = cache
        .zrange_by_score("12".to_string(), "-inf", "+inf")
        .await;
    // Example usage of zrange_by_score, can be removed if not needed

    match cache.set_ex(&req.key, &req.value, seconds).await {
        Ok(_) => HttpResponse::Ok().body("Value set"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Redis error: {}", e)),
    }
}

pub async fn get_key_value(
    state: web::Data<Mutex<Cache>>,
    req: Query<GetRequest>,
) -> impl Responder {
    let cache = state.lock().await;
    match cache.get(req.key.clone()).await {
        Ok(val) => HttpResponse::Ok().body(val),
        Err(e) => HttpResponse::InternalServerError().body(format!("Redis error: {}", e)),
    }
}
