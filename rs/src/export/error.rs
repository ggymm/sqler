use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExportError {
    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("unsupported database type for this build")]
    UnsupportedDatabase,

    #[error(transparent)]
    Csv(#[from] csv::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, ExportError>;
