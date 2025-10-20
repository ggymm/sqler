pub use mysql::MySQLDriver;
pub use postgres::PostgreSQLDriver;
pub use sqlite::SQLiteDriver;
pub use sqlserver::SQLServerDriver;

pub mod mongodb;
pub mod mysql;
pub mod oracle;
pub mod postgres;
pub mod redis;
pub mod sqlite;
pub mod sqlserver;

use crate::option::{ConnectionOptions, StoredOptions};
use crate::DataSourceType;

/// 统一的驱动错误。
#[derive(Debug, thiserror::Error)]
pub enum DriverError {
    #[error("配置字段缺失: {0}")]
    MissingField(String),
    #[error("配置字段非法: {0}")]
    InvalidField(String),
    #[error("{0}")]
    Other(String),
}

/// 数据源驱动统一接口。
pub trait DatabaseDriver {
    type Config;

    /// 测试连接；成功返回 `Ok(())`，失败返回 [`DriverError`].
    fn check_connection(
        &self,
        config: &Self::Config,
    ) -> Result<(), DriverError>;
}

/// 按数据源类型测试连接。
pub fn check_connection(
    kind: DataSourceType,
    options: &StoredOptions,
) -> Result<(), DriverError> {
    if options.kind() != kind {
        return Err(DriverError::InvalidField(format!(
            "数据源类型不匹配: {}", kind.label()
        )));
    }

    match options {
        StoredOptions::MySQL(config) => MySQLDriver.check_connection(config),
        StoredOptions::PostgreSQL(config) => PostgreSQLDriver.check_connection(config),
        StoredOptions::SQLite(config) => SQLiteDriver.check_connection(config),
        StoredOptions::SQLServer(config) => SQLServerDriver.check_connection(config),
        StoredOptions::Oracle(_) => Err(DriverError::Other("Oracle 驱动暂未实现".into())),
        StoredOptions::Redis(_) => Err(DriverError::Other("Redis 驱动暂未实现".into())),
        StoredOptions::MongoDB(_) => Err(DriverError::Other("MongoDB 驱动暂未实现".into())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::option::StoredOptions;
    use crate::DataSourceType;

    #[test]
    fn mysql_missing_host_is_error() {
        let mut config = crate::option::MySQLOptions::default();
        config.host.clear();
        let options = StoredOptions::MySQL(config);

        let result = check_connection(DataSourceType::MySQL, &options);
        assert!(matches!(
            result,
            Err(DriverError::MissingField(field)) if field == "host"
        ));
    }
}
