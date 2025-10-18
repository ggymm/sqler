use std::path::Path;

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

        // TODO: 在此处接入实际 SQLite 连接逻辑。
        Ok(())
    }
}
