use diesel::result::Error as DieselError;
use log::{debug, error};
use redis::{ErrorKind, RedisError};
use std::error::Error;
use std::fmt;
use thiserror::Error;

#[derive(Debug)]
pub enum StorageError {
    Diesel(DieselError),
    NotFound(String),
    InvalidInput(String),
    PoolError(String),
    Other(String),
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageError::Diesel(e) => write!(f, "Database error: {}", e),
            StorageError::NotFound(msg) => write!(f, "Not found: {}", msg),
            StorageError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            StorageError::PoolError(msg) => write!(f, "Pool error: {}", msg),
            StorageError::Other(msg) => write!(f, "Other error: {}", msg),
        }
    }
}

impl Error for StorageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            StorageError::Diesel(e) => Some(e),
            _ => None,
        }
    }
}

impl From<DieselError> for StorageError {
    fn from(err: DieselError) -> Self {
        StorageError::Diesel(err)
    }
}

#[derive(Clone, strum_macros::EnumMessage, Debug, Error, PartialEq)]
pub enum CacheError {
    #[error("Cannot delete key {}", _0)]
    CannotDelete(String),

    #[error("Cannot check for exists on key {}", _0)]
    #[strum(message = "CannotCheckKeyError")]
    CannotExists(String),

    #[error("Cannot set expiry for key {}", _0)]
    CannotExpire(String),

    #[error("Cannot get and delete for key {}", _0)]
    CannotGetDelete(String),

    #[error("Cannot getset key {}", _0)]
    CannotGetSet(String),

    #[error("Cannot get transaction: {}", _0)]
    CannotGetTransaction(String),

    #[error("Cannot increment value at key {} {}", _0, _1)]
    CannotIncrement(String, String),

    #[error("Cannot rename key {} with new key {}", _0, _1)]
    CannotRename(String, String),

    #[error("Cannot get mget: {}", _0)]
    CannotMget(String),

    #[error("Cannot parse order {} {}", _0, _1)]
    CannotParse(String, String),

    #[error("Cannot remove transaction: {}", _0)]
    CannotRemoveTransaction(String),

    #[error("Cannot save transaction: {}", _0)]
    CannotSaveTransaction(String),

    #[error("Cannot scan with pattern {}", _0)]
    CannotScan(String),

    #[error("Cannot set key {}", _0)]
    CannotSet(String),

    #[error("Cannot set_nx key {}", _0)]
    CannotSetNx(String),

    #[error("Cannot set_ex key {}", _0)]
    CannotSetEx(String),

    #[error("Cannot unwatch key {}", _0)]
    CannotUnwatch(String),

    #[error("Cannot watch key {}", _0)]
    CannotWatch(String),

    #[error("Cannot zadd key {}", _0)]
    CannotZadd(String),

    #[error("Cannot zcount with key {}", _0)]
    CannotZcount(String),

    #[error("Cannot zcard with key {}", _0)]
    CannotZcard(String),

    #[error("Cannot zscan with key {}", _0)]
    CannotZscan(String),

    #[error("Cannot zrangebyscore with key {}", _0)]
    CannotZrangeByScore(String),

    #[error("Cannot zrange with key {}", _0)]
    CannotZrange(String),

    #[error("Cannot zrem key {}", _0)]
    CannotZrem(String),

    #[error("Cannot zrem with key {} and value {}", _0, _1)]
    CannotRemoveZelement(String, String),

    #[error("Error {}", _0)]
    Error(String),

    #[error("Cannot parse URL")]
    CannotParseUrl,

    #[error("Not connected")]
    NotConnected,

    #[error("Cannot locate key {}", _0)]
    #[strum(message = "NotFoundError")]
    NotFound(String),

    #[error("Cannot find element {} in order with key: {}", _0, _1)]
    #[strum(message = "NotFoundOrderElement")]
    NotFoundOrderElement(String, String),

    #[error("Unknown error: {}", _0)]
    Unknown(String),
}

/// Utility to make transforming a LibError into an ErrorResponse
// use crate::error_response::ErrorResponse;
// use crate::error_response::CodedError;
impl From<RedisError> for CacheError {
    fn from(err: RedisError) -> CacheError {
        let message = format!("{}", err);
        debug!("redis Error {:?} ({:?})", err.kind(), err);
        match err.kind() {
            ErrorKind::ResponseError => CacheError::Error(message),
            ErrorKind::IoError => CacheError::NotConnected,
            ErrorKind::InvalidClientConfig => CacheError::CannotParseUrl,
            _ => CacheError::Unknown(message),
        }
    }
}

impl From<serde_json::Error> for CacheError {
    fn from(error: serde_json::Error) -> Self {
        error!("[SerdeJsonError] {}", error);
        CacheError::Unknown(error.to_string())
    }
}

impl From<CacheError> for StorageError {
    fn from(err: CacheError) -> Self {
        StorageError::Other(err.to_string())
    }
}

pub type CacheResult<T> = Result<T, CacheError>;
