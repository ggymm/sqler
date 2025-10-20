use super::{DatabaseDriver, DriverError};
use crate::option::SQLServerOptions;

/// SQL Server 驱动占位实现。
#[derive(Debug, Clone, Copy)]
pub struct SQLServerDriver;

impl DatabaseDriver for SQLServerDriver {
    type Config = SQLServerOptions;

    fn check_connection(
        &self,
        config: &Self::Config,
    ) -> Result<(), DriverError> {
        Ok(())
    }
}
