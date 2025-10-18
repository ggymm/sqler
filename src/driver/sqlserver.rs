use super::{DatabaseDriver, DriverError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SQLServerAuth {
    SqlPassword,
    Integrated,
}

#[derive(Debug, Clone)]
pub struct SQLServerConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub auth: SQLServerAuth,
    pub instance: Option<String>,
}

/// SQL Server 驱动占位实现。
#[derive(Debug, Clone, Copy)]
pub struct SQLServerDriver;

impl DatabaseDriver for SQLServerDriver {
    type Config = SQLServerConfig;

    fn test_connection(&self, config: &Self::Config) -> Result<(), DriverError> {
        Ok(())
    }
}
