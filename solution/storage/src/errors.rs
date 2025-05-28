use diesel::r2d2::{Error as R2d2Error, PoolError};
pub use diesel::result::{DatabaseErrorKind, Error as DieselError};
use log::*;
use std::convert::From;
use strum::EnumMessage;
use tokio::task::JoinError;

#[derive(strum_macros::EnumMessage, Fail, Debug)]
pub enum Error {
    #[fail(display = "Error creating a connection pool: {}", _0)]
    #[strum(message = "StorageConnectionPoolError")]
    ConnectionPoolError(String),

    #[fail(display = "Duplicate entry: {}", _0)]
    #[strum(message = "StorageDuplicateEntryError")]
    DuplicateEntryError(String),

    #[fail(display = "Entry not found: {}", _0)]
    #[strum(message = "StorageNotFoundError")]
    NotFoundError(String),

    #[fail(display = "Failed to parse entry: {}", _0)]
    #[strum(message = "StorageParseError")]
    ParseError(String),

    #[fail(display = "Unknown error: {}", _0)]
    #[strum(message = "StorageUnknownError")]
    UnknownError(String),
}

impl From<PoolError> for Error {
    fn from(error: PoolError) -> Self {
        Error::ConnectionPoolError(error.to_string())
    }
}

impl From<R2d2Error> for Error {
    fn from(error: R2d2Error) -> Self {
        Error::ConnectionPoolError(error.to_string())
    }
}

impl From<DieselError> for Error {
    fn from(error: DieselError) -> Self {
        match error {
            DieselError::DatabaseError(kind, info) => {
                if let DatabaseErrorKind::UniqueViolation = kind {
                    let message = info.details().unwrap_or_else(|| info.message()).to_string();
                    return Error::DuplicateEntryError(message);
                }
                let description = format!("{:?}", info);
                Error::UnknownError(description)
            }
            DieselError::NotFound => Error::NotFoundError("Requested record was not found".into()),
            _ => Error::UnknownError("Unknown error".into()),
        }
    }
}

impl From<JoinError> for Error {
    fn from(error: JoinError) -> Self {
        // TODO: log instead of return?
        Error::UnknownError(error.to_string())
    }
}
