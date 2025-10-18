use std::path::Path;

use sqlx::{sqlite::{SqliteConnectOptions, SqliteConnection}, Connection};
use tokio::runtime::Builder;

use super::{DatabaseDriver, DriverError};

/// SQLite 连接配置。
#[derive(Debug, Clone)]
pub struct SqliteConfig {
    pub file_path: String,
    pub read_only: bool,
}

/// SQLite 驱动占位实现。
#[derive(Debug, Clone, Copy)]
pub struct SqliteDriver;

impl DatabaseDriver for SqliteDriver {
    type Config = SqliteConfig;

    fn test_connection(&self, config: &Self::Config) -> Result<(), DriverError> {
        if config.file_path.trim().is_empty() {
            return Err(DriverError::MissingField("file_path".into()));
        }

        let path = Path::new(&config.file_path);
        if config.read_only && !path.exists() {
            return Err(DriverError::InvalidField(
                "read_only 模式下要求文件必须存在".into(),
            ));
        }

        let mut options = SqliteConnectOptions::new().filename(&config.file_path);
        if config.read_only {
            options = options.read_only(true);
        } else {
            options = options.create_if_missing(true);
        }

        let runtime = Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|err| DriverError::Other(err.to_string()))?;

        runtime
            .block_on(async {
                let mut conn = SqliteConnection::connect_with(&options).await?;
                conn.ping().await?;
                conn.close().await?;
                Ok::<(), sqlx::Error>(())
            })
            .map_err(|err| DriverError::Other(err.to_string()))
    }
}
