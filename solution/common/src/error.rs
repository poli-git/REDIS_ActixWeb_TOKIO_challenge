use thiserror::Error;

#[derive(Debug, Error)]
pub enum PersistPlansError {
    #[error("Database error: {0}")]
    DbError(String),
    #[error("Redis error: {0}")]
    RedisError(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Deserialization error: {0}")]
    DeserializationError(String),
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Timeout error: {0}")]
    TimeoutError(String),
    #[error("Unknown error: {0}")]
    UnknownError(String),
    #[error("Invalid Plans - Not found: {0}")]
    NotFound(String),
}
