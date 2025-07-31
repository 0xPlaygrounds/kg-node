use grc20_core::error::DatabaseError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to execute query: {0}")]
    DatabaseError(#[from] DatabaseError),
    #[error("Failed to parse UUID: {0}")]
    InvalidUuid(String),
}

impl From<Error> for rmcp::Error {
    fn from(err: Error) -> Self {
        match err {
            Error::DatabaseError(_) => rmcp::Error::internal_error(err.to_string(), None),
            Error::InvalidUuid(_) => rmcp::Error::invalid_params(err.to_string(), None),
        }
    }
}
