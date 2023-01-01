use thiserror::Error;

/// Result
pub type DalResult<T> = Result<T, Error>;

/// Enumeration of all possible errors
/// which can be returned by a `dal` function
#[derive(Debug, Error)]
pub enum Error {
    /// A mysql error
    #[error("{0}")]
    Mysql(#[from] mysql::Error),
    /// A migration error
    #[error("{0}")]
    Refinery(#[from] refinery::Error),
    /// The database is in an invalid state
    #[error("Invalid state: {0}")]
    InvalidState(String),
}
