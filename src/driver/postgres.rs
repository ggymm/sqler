use super::{DatabaseDriver, DriverError};

#[derive(Debug, Clone)]
pub struct PostgreSQLConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: Option<String>,
    pub ssl_mode: Option<SslMode>,
}

/// Postgres SSL 模式占位。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SslMode {
    Disable,
    Prefer,
    Require,
}

/// Postgres 驱动占位实现。
#[derive(Debug, Clone, Copy)]
pub struct PostgreSQLDriver;

impl DatabaseDriver for PostgreSQLDriver {
    type Config = PostgreSQLConfig;

    fn test_connection(&self, config: &Self::Config) -> Result<(), DriverError> {
        Ok(())
    }
}
