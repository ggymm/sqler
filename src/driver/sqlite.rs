use super::{DatabaseDriver, DriverError};
use crate::option::SQLiteOptions;

#[derive(Debug, Clone, Copy)]
pub struct SQLiteDriver;

impl DatabaseDriver for SQLiteDriver {
    type Config = SQLiteOptions;

    fn test_connection(
        &self,
        config: &Self::Config,
    ) -> Result<(), DriverError> {
        Ok(())
    }
}
