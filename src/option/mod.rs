use crate::DataSourceType;
pub use mysql::MySQLOptions;
pub use postgres::PostgreSQLOptions;
pub use sqlite::SQLiteOptions;
pub use sqlserver::SQLServerOptions;

pub mod mongodb;
pub mod mysql;
pub mod oracle;
pub mod postgres;
pub mod redis;
pub mod sqlite;
pub mod sqlserver;

pub trait ConnectionOptions {
    fn kind(&self) -> DataSourceType;
}
