use super::{DatabaseDriver, DriverError};
use crate::option::MySQLOptions;

#[derive(Debug, Clone, Copy)]
pub struct MySQLDriver;

impl DatabaseDriver for MySQLDriver {
    type Config = MySQLOptions;

    fn test_connection(
        &self,
        config: &Self::Config,
    ) -> Result<(), DriverError> {
        Ok(())
    }
}
