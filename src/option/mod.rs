use crate::DataSourceType;
pub use mongodb::MongoDBHost;
pub use mongodb::MongoDBOptions;
pub use mysql::MySQLOptions;
pub use oracle::OracleAddress;
pub use oracle::OracleOptions;
pub use postgres::PostgreSQLOptions;
pub use postgres::SslMode;
pub use redis::RedisOptions;
pub use sqlite::SQLiteOptions;
pub use sqlserver::SQLServerAuth;
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataSourceOptions {
    MySQL(MySQLOptions),
    Oracle(OracleOptions),
    SQLite(SQLiteOptions),
    SQLServer(SQLServerOptions),
    PostgreSQL(PostgreSQLOptions),
    Redis(RedisOptions),
    MongoDB(MongoDBOptions),
}

impl ConnectionOptions for DataSourceOptions {
    fn kind(&self) -> DataSourceType {
        match self {
            DataSourceOptions::MySQL(_) => DataSourceType::MySQL,
            DataSourceOptions::Oracle(_) => DataSourceType::Oracle,
            DataSourceOptions::SQLite(_) => DataSourceType::SQLite,
            DataSourceOptions::SQLServer(_) => DataSourceType::SQLServer,
            DataSourceOptions::PostgreSQL(_) => DataSourceType::PostgreSQL,
            DataSourceOptions::Redis(_) => DataSourceType::Redis,
            DataSourceOptions::MongoDB(_) => DataSourceType::MongoDB,
        }
    }
}
