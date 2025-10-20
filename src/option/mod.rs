pub mod mongodb;
pub mod mysql;
pub mod oracle;
pub mod postgres;
pub mod redis;
pub mod sqlite;
pub mod sqlserver;

use crate::DataSourceType;
pub use mongodb::{MongoDBHost, MongoDBOptions};
pub use mysql::MySQLOptions;
pub use oracle::{OracleAddress, OracleOptions};
pub use postgres::{PostgreSQLOptions, SslMode};
pub use redis::RedisOptions;
pub use sqlite::SQLiteOptions;
pub use sqlserver::{SQLServerAuth, SQLServerOptions};

pub trait ConnectionOptions {
    fn kind(&self) -> DataSourceType;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoredOptions {
    MySQL(MySQLOptions),
    Oracle(OracleOptions),
    SQLite(SQLiteOptions),
    SQLServer(SQLServerOptions),
    PostgreSQL(PostgreSQLOptions),
    Redis(RedisOptions),
    MongoDB(MongoDBOptions),
}

impl ConnectionOptions for StoredOptions {
    fn kind(&self) -> DataSourceType {
        match self {
            StoredOptions::MySQL(_) => DataSourceType::MySQL,
            StoredOptions::Oracle(_) => DataSourceType::Oracle,
            StoredOptions::SQLite(_) => DataSourceType::SQLite,
            StoredOptions::SQLServer(_) => DataSourceType::SQLServer,
            StoredOptions::PostgreSQL(_) => DataSourceType::PostgreSQL,
            StoredOptions::Redis(_) => DataSourceType::Redis,
            StoredOptions::MongoDB(_) => DataSourceType::MongoDB,
        }
    }
}
