use super::{DatabaseDriver, DriverError};

#[derive(Debug, Clone)]
pub struct SQLiteConfig {
    pub file_path: String,
    pub read_only: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct SQLiteDriver;

impl DatabaseDriver for SQLiteDriver {
    type Config = SQLiteConfig;

    fn test_connection(&self, config: &Self::Config) -> Result<(), DriverError> {
        Ok(())
    }
}
