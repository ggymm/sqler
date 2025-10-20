use super::{DatabaseDriver, DriverError};

#[derive(Debug, Clone)]
pub struct MySQLConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
    pub charset: Option<String>,
    pub use_tls: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct MySQLDriver;

impl DatabaseDriver for MySQLDriver {
    type Config = MySQLConfig;

    fn test_connection(
        &self,
        config: &Self::Config,
    ) -> Result<(), DriverError> {
        Ok(())
    }
}
