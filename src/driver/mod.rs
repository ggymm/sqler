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

use crate::option::{MySQLOptions, PostgreSQLOptions, SQLServerOptions, SQLiteOptions};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriverKind {
    Postgres,
    MySql,
    Sqlite,
    SqlServer,
}

/// 测试连接请求参数。
#[derive(Debug, Clone)]
pub enum TestConnectionRequest {
    Postgres(PostgreSQLOptions),
    MySql(MySQLOptions),
    Sqlite(SQLiteOptions),
    SqlServer(SQLServerOptions),
}

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
    fn test_connection(
        &self,
        config: &Self::Config,
    ) -> Result<(), DriverError>;
}

/// 按数据源类型测试连接。
pub fn test_connection(request: TestConnectionRequest) -> Result<(), DriverError> {
    match request {
        TestConnectionRequest::Postgres(config) => PostgreSQLDriver.test_connection(&config),
        TestConnectionRequest::MySql(config) => MySQLDriver.test_connection(&config),
        TestConnectionRequest::Sqlite(config) => SQLiteDriver.test_connection(&config),
        TestConnectionRequest::SqlServer(config) => SQLServerDriver.test_connection(&config),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mysql_driver() {
    }
}
