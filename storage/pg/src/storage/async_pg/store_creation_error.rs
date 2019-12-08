use std::env::VarError;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum StoreCreationError {
    VarError(VarError),
    ConnectionError(tokio_postgres::error::Error),
}

impl From<tokio_postgres::error::Error> for StoreCreationError {
    fn from(e: tokio_postgres::error::Error) -> Self {
        StoreCreationError::ConnectionError(e)
    }
}

impl From<VarError> for StoreCreationError {
    fn from(e: VarError) -> Self {
        StoreCreationError::VarError(e)
    }
}

impl fmt::Display for StoreCreationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StoreCreationError::VarError(e) => e.fmt(f),
            StoreCreationError::ConnectionError(e) => e.fmt(f),
        }
    }
}

impl Error for StoreCreationError {
    fn description(&self) -> &str {
        match self {
            StoreCreationError::VarError(e) => e.description(),
            StoreCreationError::ConnectionError(e) => e.description(),
        }
    }

    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            StoreCreationError::VarError(e) => Some(e),
            StoreCreationError::ConnectionError(e) => Some(e),
        }
    }
}
