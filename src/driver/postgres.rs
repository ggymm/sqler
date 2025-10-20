use super::{DatabaseDriver, DriverError};
use crate::option::PostgreSQLOptions;

/// Postgres 驱动占位实现。
#[derive(Debug, Clone, Copy)]
pub struct PostgreSQLDriver;

impl DatabaseDriver for PostgreSQLDriver {
    type Config = PostgreSQLOptions;

    fn check_connection(
        &self,
        config: &Self::Config,
    ) -> Result<(), DriverError> {
        Ok(())
    }
}
