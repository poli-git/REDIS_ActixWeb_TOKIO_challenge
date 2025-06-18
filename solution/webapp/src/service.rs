use crate::errors::*;
use crate::helpers::*;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use actix_web::{web::Json, web::Query, Result};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use storage::connections::cache::is_healthy;
use storage::connections::cache::Cache;
use tokio::sync::Mutex;
use utoipa::ToSchema;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        get_health,
        search_available_events
    ),
    components(
        schemas(HealthResponse, GetSearchRequest)
    ),
    tags(
        (name = "Webapp", description = "API endpoints")
    )
)]
pub struct ApiDoc;

/// Configures the web service routes.
/// It registers the `/search` route for searching available events and the `/health` route for health checks.
/// The `/search` route accepts GET requests with query parameters for `starts_at` and `ends_at`.
/// The `/health` route provides a basic health check response.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/search").route(web::get().to(search_available_events)));
    cfg.service(web::resource("/health").route(web::get().to(get_health)));
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct HealthResponse {
    status: String,
}

#[derive(Deserialize, ToSchema)]
pub struct GetSearchRequest {
    starts_at: String,
    ends_at: String,
}

#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Healthcheck", body = HealthResponse)
    ),
    tag = "api"
)]
/// Get the health status of the service.
pub async fn get_health(_req: HttpRequest) -> Result<Json<HealthResponse>> {
    Ok(Json(HealthResponse {
        status: "Basic healthcheck for Web-service: - OK".to_owned(),
    }))
}

#[utoipa::path(
    get,
    path = "/search",
    params(
        ("starts_at" = String, Query, description = "Start datetime in %Y-%m-%dT%H:%M:%S format"),
        ("ends_at" = String, Query, description = "End datetime in %Y-%m-%dT%H:%M:%S format")
    ),
    responses(
        (status = 200, description = "List of available events"),
        (status = 400, description = "Bad request"),
        (status = 503, description = "Service unavailable"),
        (status = 500, description = "Internal error")
    ),
    tag = "api"
)]
/// Search for available events based on the provided time range.
/// request with query parameters for `starts_at` and `ends_at`
pub async fn search_available_events(
    state: web::Data<Mutex<Cache>>,
    req: Query<GetSearchRequest>,
) -> impl Responder {
    // Validate the time range
    if req.starts_at.is_empty() || req.ends_at.is_empty() {
        return ErrorResponse::bad_request("Both starts_at and ends_at must be provided.");
    }
    if req.starts_at >= req.ends_at {
        return ErrorResponse::bad_request("starts_at must be before ends_at.");
    }
    // Parse the datetime strings
    let starts_at = match NaiveDateTime::parse_from_str(&req.starts_at, "%Y-%m-%dT%H:%M:%S") {
        Ok(dt) => dt,
        Err(_) => {
            return ErrorResponse::bad_request("Invalid starts_at format. Use %Y-%m-%dT%H:%M:%S");
        }
    };
    let ends_at = match NaiveDateTime::parse_from_str(&req.ends_at, "%Y-%m-%dT%H:%M:%S") {
        Ok(dt) => dt,
        Err(_) => {
            return ErrorResponse::bad_request("Invalid ends_at format. Use %Y-%m-%dT%H:%M:%S");
        }
    };

    if starts_at >= ends_at {
        return ErrorResponse::bad_request("starts_at must be before ends_at.");
    }
    // Lock the cache state to ensure thread safety
    let cache = state.lock().await;
    // Check if the cache is healthy
    if !is_healthy(&cache).await {
        return ErrorResponse::service_unavailable("Cache is not healthy.");
    }
    // Fetch matched plans from the cache
    match cache.get_matched_plans(starts_at, ends_at).await {
        Ok(events) => {
            let response = map_provider_events_to_response_dto(&events);
            HttpResponse::Ok().json(response)
        }
        Err(_) => ErrorResponse::internal_error("Failed to fetch events"),
    }
}
