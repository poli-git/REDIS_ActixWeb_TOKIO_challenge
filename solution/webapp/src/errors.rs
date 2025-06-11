use actix_web::{HttpResponse, ResponseError};
use serde::Serialize;
use std::fmt;

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

impl ErrorResponse {
    pub fn bad_request(msg: &str) -> HttpResponse {
        HttpResponse::BadRequest().json(ErrorResponse {
            error: msg.to_string(),
        })
    }

    pub fn internal_error(msg: &str) -> HttpResponse {
        HttpResponse::InternalServerError().json(ErrorResponse {
            error: msg.to_string(),
        })
    }
}

impl fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.error)
    }
}

impl ResponseError for ErrorResponse {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::BadRequest().json(self)
    }
}
