use std::time::Duration;

use sqlx::{postgres::{PgConnectOptions, PgConnection, PgSslMode}, Connection};
use tokio::runtime::Builder;

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

        let mut options = PgConnectOptions::new()
            .host(&config.host)
            .port(config.port)
            .database(&config.database)
            .username(&config.username)
            .connect_timeout(Duration::from_secs(5));

        if let Some(password) = &config.password {
            options = options.password(password);
        }

        if let Some(mode) = config.ssl_mode {
            let ssl_mode = match mode {
                SslMode::Disable => PgSslMode::Disable,
                SslMode::Prefer => PgSslMode::Prefer,
                SslMode::Require => PgSslMode::Require,
            };
            options = options.ssl_mode(ssl_mode);
        }

        let runtime = Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|err| DriverError::Other(err.to_string()))?;

        runtime
            .block_on(async {
                let mut conn = PgConnection::connect_with(&options).await?;
                conn.close().await?;
                Ok::<(), sqlx::Error>(())
            })
            .map_err(|err| DriverError::Other(err.to_string()))
    }
}
