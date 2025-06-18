use actix_web::{HttpResponse, ResponseError};
use serde::Serialize;
use std::fmt;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorBody {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: ErrorBody,
    pub data: Option<()>, // Always null for errors
}

impl ErrorResponse {
    pub fn new(code: &str, message: &str) -> Self {
        ErrorResponse {
            error: ErrorBody {
                code: code.to_string(),
                message: message.to_string(),
            },
            data: None, // Always null for errors
        }
    }

    pub fn bad_request(message: &str) -> HttpResponse {
        HttpResponse::BadRequest().json(ErrorResponse::new("bad_request", message))
    }

    pub fn internal_error(message: &str) -> HttpResponse {
        HttpResponse::InternalServerError().json(ErrorResponse::new("internal_error", message))
    }
    pub fn service_unavailable(message: &str) -> HttpResponse {
        HttpResponse::ServiceUnavailable().json(ErrorResponse::new("service_unavailable", message))
    }
}

impl fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.error.code, self.error.message)
    }
}

impl ResponseError for ErrorResponse {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::BadRequest().json(self)
    }
}
