use std::time::Duration;

use sqlx::{mysql::{MySqlConnectOptions, MySqlConnection, MySqlSslMode}, Connection};
use tokio::runtime::Builder;

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

        let mut options = MySqlConnectOptions::new()
            .host(&config.host)
            .port(config.port)
            .database(&config.database)
            .username(&config.username)
            .ssl_mode(if config.use_tls {
                MySqlSslMode::Preferred
            } else {
                MySqlSslMode::Disabled
            })
            .connect_timeout(Duration::from_secs(5));

        if let Some(password) = &config.password {
            options = options.password(password);
        }

        if let Some(charset) = &config.charset {
            if charset.trim().is_empty() {
                return Err(DriverError::InvalidField("charset 不能为空字符串".into()));
            }
            options = options.charset(charset);
        }

        let runtime = Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|err| DriverError::Other(err.to_string()))?;

        runtime
            .block_on(async {
                let mut conn = MySqlConnection::connect_with(&options).await?;
                conn.ping().await?;
                conn.close().await?;
                Ok::<(), sqlx::Error>(())
            })
            .map_err(|err| DriverError::Other(err.to_string()))
    }
}
