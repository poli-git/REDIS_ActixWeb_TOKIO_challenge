use actix_web::{HttpResponse, ResponseError};
use serde::Serialize;
use std::fmt;

#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: ErrorBody,
    pub data: Option<serde_json::Value>, // Always null for errors
}

impl ErrorResponse {
    pub fn new(code: &str, message: &str) -> Self {
        ErrorResponse {
            error: ErrorBody {
                code: code.to_string(),
                message: message.to_string(),
            },
            data: None,
        }
    }

    pub fn bad_request(message: &str) -> HttpResponse {
        HttpResponse::BadRequest().json(ErrorResponse::new("bad_request", message))
    }

    pub fn internal_error(message: &str) -> HttpResponse {
        HttpResponse::InternalServerError().json(ErrorResponse::new("internal_error", message))
    }
    pub fn not_found(message: &str) -> HttpResponse {
        HttpResponse::NotFound().json(ErrorResponse::new("not_found", message))
    }
    pub fn forbidden(message: &str) -> HttpResponse {
        HttpResponse::Forbidden().json(ErrorResponse::new("forbidden", message))
    }
    pub fn unauthorized(message: &str) -> HttpResponse {
        HttpResponse::Unauthorized().json(ErrorResponse::new("unauthorized", message))
    }
    pub fn conflict(message: &str) -> HttpResponse {
        HttpResponse::Conflict().json(ErrorResponse::new("conflict", message))
    }
    pub fn method_not_allowed(message: &str) -> HttpResponse {
        HttpResponse::MethodNotAllowed().json(ErrorResponse::new("method_not_allowed", message))
    }
    pub fn not_acceptable(message: &str) -> HttpResponse {
        HttpResponse::NotAcceptable().json(ErrorResponse::new("not_acceptable", message))
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
