use crate::option::ConnectionOptions;
use crate::option::DataSourceKind;
use crate::option::DataSourceOptions;

pub use mongodb::MongoDBDriver;
pub use mysql::MySQLDriver;
pub use postgres::PostgreSQLDriver;
pub use redis::RedisDriver;
pub use sqlite::SQLiteDriver;
pub use sqlserver::SQLServerDriver;

pub mod mongodb;
pub mod mysql;
pub mod oracle;
pub mod postgres;
pub mod redis;
pub mod sqlite;
pub mod sqlserver;

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
    kind: DataSourceKind,
    options: &DataSourceOptions,
) -> Result<(), DriverError> {
    if options.kind() != kind {
        return Err(DriverError::InvalidField(format!("数据源类型不匹配: {}", kind.label())));
    }

    match options {
        DataSourceOptions::MySQL(config) => MySQLDriver.check_connection(config),
        DataSourceOptions::PostgreSQL(config) => PostgreSQLDriver.check_connection(config),
        DataSourceOptions::SQLite(config) => SQLiteDriver.check_connection(config),
        DataSourceOptions::SQLServer(config) => SQLServerDriver.check_connection(config),
        DataSourceOptions::MongoDB(config) => MongoDBDriver.check_connection(config),
        DataSourceOptions::Redis(config) => RedisDriver.check_connection(config),
        DataSourceOptions::Oracle(_) => Err(DriverError::Other("Oracle 驱动暂未实现".into())),
    }
}

#[cfg(test)]
mod tests {}
