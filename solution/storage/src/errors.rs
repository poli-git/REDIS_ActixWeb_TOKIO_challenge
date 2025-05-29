use diesel::r2d2::{Error as R2d2Error, PoolError};
use uuid::Uuid;

pub use diesel::result::{DatabaseErrorKind, Error as DieselError};
use std::convert::From;
use tokio::task::JoinError;

pub type StorageResult<T> = Result<T, StorageError>;

pub static INTERNAL_ERROR_CODE: &str = "InternalServerError";
pub static VALIDATION_ERROR_CODE: &str = "ValidationError";

#[derive(Serialize, Deserialize, Debug)]
pub struct ErrorResponse {
    pub errors: Vec<CodedError>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CodedError {
    pub code: String,
    pub message: String,
}

#[non_exhaustive]
#[derive(strum_macros::EnumMessage, Fail, Debug)]
pub enum StorageError {
    #[fail(
        display = "Delete developer wallet request failed: It is not possible to delete a Developer Wallet that contains funds"
    )]
    CannotDeleteDevWallet(),

    #[fail(display = "unable to retrieve balance: {}", _0)]
    CannotGetBalance(String),

    #[fail(display = "unable to retrieve block info: {}", _0)]
    CannotGetBlockError(String),

    #[fail(display = "Fail on get developer wallet key pair: {}", _0)]
    CannotGetKeyPair(String),

    #[fail(display = "Fail on sign transaction: {}", _0)]
    CannotSignTransaction(String),

    #[fail(display = "Celery Error: {}", _0)]
    CeleryError(String),

    #[fail(display = "Connection to the Wallet DB failed: {}", _0)]
    DatabaseConnectionError(String),

    #[fail(
        display = "Wallet already exist: user_uid={}, app_uid={}",
        user_uid, app_uid
    )]
    DuplicateUser { user_uid: Uuid, app_uid: Uuid },

    #[fail(display = "IO Error: {}", _0)]
    IoError(String),

    #[fail(display = "Insufficient funds: {}", _0)]
    InsufficientFundsError(String),

    #[fail(display = "Invalid contract type found: {}", _0)]
    InvalidContractTypeError(String),

    #[fail(display = "Deploy contract failed no address found")]
    InvalidDeployNoAddress(),

    #[fail(display = "KMS config Error: {}", _0)]
    KmsConfigError(String),

    #[fail(display = "KMS Error: {}", _0)]
    KmsError(String),

    #[fail(display = "KMS Region Error: {}", _0)]
    KmsRegionError(String),

    #[fail(display = "Not found: {}", _0)]
    NotFound(String),

    #[fail(display = "Error parsing private key: {}", _0)]
    ParsingPrivateKeyError(String),

    #[fail(display = "Lock is poisoned: {}", _0)]
    PoisonedLock(String),

    #[fail(display = "Secure Request Error: {}", _0)]
    SecureRequestError(String),

    #[fail(display = "Unknown Error: {}", _0)]
    UnknownError(String),

    #[fail(display = "Duplicate entry: {}", _0)]
    #[strum(message = "StorageDuplicateEntryError")]
    DuplicateEntryError(String),

    #[fail(display = "Unknown Database error: {}", _0)]
    UnknownDatabaseError(String),

    #[fail(display = "User does not exist: {}", _0)]
    UserDoesNotExist(String),

    #[fail(display = "Error creating a connection pool: {}", _0)]
    #[strum(message = "StorageConnectionPoolError")]
    ConnectionPoolError(String),

    #[fail(display = "Entry not found: {}", _0)]
    #[strum(message = "StorageNotFoundError")]
    NotFoundError(String),

    #[fail(display = "Failed to parse entry: {}", _0)]
    #[strum(message = "StorageParseError")]
    ParseError(String),
}

impl From<R2d2Error> for StorageError {
    fn from(error: R2d2Error) -> Self {
        StorageError::ConnectionPoolError(error.to_string())
    }
}

impl From<DieselError> for StorageError {
    fn from(error: DieselError) -> Self {
        match error {
            DieselError::DatabaseError(kind, info) => {
                if let DatabaseErrorKind::UniqueViolation = kind {
                    let message = info.details().unwrap_or_else(|| info.message()).to_string();
                    return StorageError::DuplicateEntryError(message);
                }
                let description = format!("{:?}", info);
                StorageError::UnknownError(description)
            }
            DieselError::NotFound => {
                StorageError::NotFoundError("Requested record was not found".into())
            }
            _ => StorageError::UnknownError("Unknown error".into()),
        }
    }
}

impl From<PoolError> for StorageError {
    fn from(error: PoolError) -> Self {
        StorageError::ConnectionPoolError(error.to_string())
    }
}

impl From<JoinError> for StorageError {
    fn from(error: JoinError) -> Self {
        // TODO: log instead of return?
        StorageError::UnknownError(error.to_string())
    }
}
