use super::{DatabaseDriver, DriverError};
use crate::option::SQLiteOptions;

use rusqlite::{Connection, OpenFlags};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy)]
pub struct SQLiteDriver;

impl DatabaseDriver for SQLiteDriver {
    type Config = SQLiteOptions;

    fn check_connection(
        &self,
        config: &Self::Config,
    ) -> Result<(), DriverError> {
        let path_str = config.file_path.trim();
        if path_str.is_empty() {
            return Err(DriverError::MissingField("file_path".into()));
        }

        let path = Path::new(path_str);

        if config.read_only {
            if !path.exists() {
                return Err(DriverError::InvalidField("file_path 不存在".into()));
            }
        } else if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).map_err(|err| DriverError::Other(format!("创建目录失败: {}", err)))?;
            }
        }

        let flags = if config.read_only {
            OpenFlags::SQLITE_OPEN_READ_ONLY
        } else {
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE
        };

        let conn = Connection::open_with_flags(path, flags)
            .map_err(|err| DriverError::Other(format!("打开 SQLite 失败: {}", err)))?;

        conn.query_row("SELECT 1", [], |_| Ok::<_, rusqlite::Error>(()))
            .map_err(|err| DriverError::Other(format!("校验查询失败: {}", err)))?;

        Ok(())
    }
}
