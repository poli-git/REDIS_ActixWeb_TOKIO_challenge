use diesel::result::Error as DieselError;
use std::error::Error;
use std::fmt;

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
