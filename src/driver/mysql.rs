use super::{DatabaseDriver, DriverError};

/// MySQL 连接配置。
#[derive(Debug, Clone)]
pub struct MySqlConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: Option<String>,
    pub charset: Option<String>,
    pub use_tls: bool,
}

/// MySQL 驱动占位实现。
#[derive(Debug, Clone, Copy)]
pub struct MySqlDriver;

impl DatabaseDriver for MySqlDriver {
    type Config = MySqlConfig;

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

        if let Some(charset) = &config.charset {
            if charset.trim().is_empty() {
                return Err(DriverError::InvalidField("charset 不能为空字符串".into()));
            }
        }

        // TODO: 在此处接入实际 MySQL 连接逻辑。
        Ok(())
    }
}
