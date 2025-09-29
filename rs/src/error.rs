use thiserror::Error as ThisError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Unsupported: {0}")]
    Unsupported(String),
    #[error("DB error: {0}")]
    Db(String),
    #[error("Internal error: {0}")]
    Internal(String),
}

#[cfg(feature = "with-sqlx")]
impl From<sqlx::Error> for Error {
    fn from(e: sqlx::Error) -> Self { Self::Db(e.to_string()) }
}

#[cfg(feature = "with-mongodb")]
impl From<mongodb::error::Error> for Error {
    fn from(e: mongodb::error::Error) -> Self { Self::Db(e.to_string()) }
}

#[cfg(feature = "with-redis")]
impl From<redis::RedisError> for Error {
    fn from(e: redis::RedisError) -> Self { Self::Db(e.to_string()) }
}

#[cfg(feature = "with-oracle")]
impl From<oracle::Error> for Error {
    fn from(e: oracle::Error) -> Self { Self::Db(e.to_string()) }
}

