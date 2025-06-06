use actix_web::{web::Json, HttpRequest};
use actix_web::{web::Path, Result};
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
pub struct HealthResponse {
    status: String,
}

/// Basic healthcheck for services
pub async fn get_health(_req: HttpRequest) -> Result<Json<HealthResponse>> {
    Ok(Json(HealthResponse {
        status: "PEPE DE PAPA - ok".to_owned(),
    }))
}
