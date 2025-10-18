use super::{DatabaseDriver, DriverError};

/// Postgres 连接配置。
#[derive(Debug, Clone)]
pub struct PostgresConfig {
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
pub struct PostgresDriver;

impl DatabaseDriver for PostgresDriver {
    type Config = PostgresConfig;

    fn test_connection(&self, config: &Self::Config) -> Result<(), DriverError> {
        if config.host.trim().is_empty() {
            return Err(DriverError::MissingField("host".into()));
        }
        if config.port == 0 {
            return Err(DriverError::InvalidField("port 必须大于 0".into()));
        }
        if config.database.trim().is_empty() {
            return Err(DriverError::MissingField("database".into()));
        }
        if config.username.trim().is_empty() {
            return Err(DriverError::MissingField("username".into()));
        }

        // TODO: 在此处接入实际连接逻辑。
        Ok(())
    }
}
