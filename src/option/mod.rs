use std::collections::HashMap;

use gpui::SharedString;
use serde_json::Value;

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

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DataSourceKind {
    MySQL,
    Oracle,
    SQLite,
    SQLServer,
    PostgreSQL,
    Redis,
    MongoDB,
}

impl DataSourceKind {
    pub fn all() -> &'static [DataSourceKind] {
        &[
            DataSourceKind::MySQL,
            DataSourceKind::Oracle,
            DataSourceKind::SQLite,
            DataSourceKind::SQLServer,
            DataSourceKind::PostgreSQL,
            DataSourceKind::Redis,
            DataSourceKind::MongoDB,
        ]
    }

    pub fn image(&self) -> &'static str {
        match self {
            DataSourceKind::MySQL => "icons/mysql.svg",
            DataSourceKind::Oracle => "icons/oracle.svg",
            DataSourceKind::SQLite => "icons/sqlite.svg",
            DataSourceKind::SQLServer => "icons/sqlserver.svg",
            DataSourceKind::PostgreSQL => "icons/postgresql.svg",
            DataSourceKind::Redis => "icons/redis.svg",
            DataSourceKind::MongoDB => "icons/mongodb.svg",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            DataSourceKind::MySQL => "MySQL",
            DataSourceKind::Oracle => "Oracle",
            DataSourceKind::SQLite => "SQLite",
            DataSourceKind::SQLServer => "SQLServer",
            DataSourceKind::PostgreSQL => "PostgreSQL",
            DataSourceKind::Redis => "Redis",
            DataSourceKind::MongoDB => "MongoDB",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            DataSourceKind::MySQL => "开源关系型数据库，读写性能稳定、生态成熟",
            DataSourceKind::Oracle => "商业级事务数据库，强调安全性与可扩展性",
            DataSourceKind::SQLite => "嵌入式文件数据库，零配置、单文件存储",
            DataSourceKind::SQLServer => "微软企业数据库，原生集成 Windows 与 AD",
            DataSourceKind::PostgreSQL => "开源对象关系数据库，扩展能力与标准兼容性强",
            DataSourceKind::Redis => "内存键值数据库，适合缓存、队列与实时计数场景",
            DataSourceKind::MongoDB => "文档型数据库，支持灵活的 JSON 模式与水平扩展",
        }
    }
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

#[derive(Clone)]
pub struct DataSourceMeta {
    pub id: String,
    pub name: SharedString,
    pub desc: SharedString,
    pub kind: DataSourceKind,
    pub options: DataSourceOptions,
    pub extras: Option<HashMap<String, Value>>,
}

impl DataSourceMeta {
    pub fn tables(&self) -> Vec<SharedString> {
        if let Some(extras) = &self.extras {
            if let Some(Value::Array(tables)) = extras.get("tables") {
                return tables
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| SharedString::from(s.to_string())))
                    .collect();
            }
        }
        vec![]
    }
}

pub trait ConnectionOptions {
    fn kind(&self) -> DataSourceKind;
}

impl ConnectionOptions for DataSourceOptions {
    fn kind(&self) -> DataSourceKind {
        match self {
            DataSourceOptions::MySQL(_) => DataSourceKind::MySQL,
            DataSourceOptions::Oracle(_) => DataSourceKind::Oracle,
            DataSourceOptions::SQLite(_) => DataSourceKind::SQLite,
            DataSourceOptions::SQLServer(_) => DataSourceKind::SQLServer,
            DataSourceOptions::PostgreSQL(_) => DataSourceKind::PostgreSQL,
            DataSourceOptions::Redis(_) => DataSourceKind::Redis,
            DataSourceOptions::MongoDB(_) => DataSourceKind::MongoDB,
        }
    }
}
