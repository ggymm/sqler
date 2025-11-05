use std::collections::HashMap;

use gpui::SharedString;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub use mongodb::{MongoDBDriver, MongoDBHost, MongoDBOptions};
pub use mysql::{MySQLDriver, MySQLOptions};
pub use oracle::OracleOptions;
pub use postgres::{PostgreSQLDriver, PostgreSQLOptions};
pub use redis::{RedisDriver, RedisOptions};
pub use sqlite::{SQLiteDriver, SQLiteOptions};
pub use sqlserver::{SQLServerDriver, SQLServerOptions};

pub mod mongodb;
pub mod mysql;
pub mod oracle;
pub mod postgres;
pub mod redis;
pub mod sqlite;
pub mod sqlserver;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Datatype {
    // 整数类型
    TinyInt,
    SmallInt,
    Int,
    BigInt,
    // 浮点类型
    Float,
    Double,
    Decimal,
    // 字符串类型
    Char,
    VarChar,
    Text,
    // 二进制类型
    Binary,
    VarBinary,
    Blob,
    // 日期时间类型
    Date,
    Time,
    DateTime,
    Timestamp,
    // 布尔类型
    Boolean,
    // JSON 类型
    Json,
    // 特殊类型
    Uuid,
    Enum,
    Set,
    // NoSQL 特殊类型
    Document,
    Array,
    // Redis 特殊类型
    String,
    List,
    Hash,
    ZSet,
    // 其他
    Unknown,
}

impl Datatype {
    pub fn label(&self) -> &'static str {
        match self {
            Datatype::TinyInt => "TINYINT",
            Datatype::SmallInt => "SMALLINT",
            Datatype::Int => "INT",
            Datatype::BigInt => "BIGINT",
            Datatype::Float => "FLOAT",
            Datatype::Double => "DOUBLE",
            Datatype::Decimal => "DECIMAL",
            Datatype::Char => "CHAR",
            Datatype::VarChar => "VARCHAR",
            Datatype::Text => "TEXT",
            Datatype::Binary => "BINARY",
            Datatype::VarBinary => "VARBINARY",
            Datatype::Blob => "BLOB",
            Datatype::Date => "DATE",
            Datatype::Time => "TIME",
            Datatype::DateTime => "DATETIME",
            Datatype::Timestamp => "TIMESTAMP",
            Datatype::Boolean => "BOOLEAN",
            Datatype::Json => "JSON",
            Datatype::Uuid => "UUID",
            Datatype::Enum => "ENUM",
            Datatype::Set => "SET",
            Datatype::Document => "DOCUMENT",
            Datatype::Array => "ARRAY",
            Datatype::String => "STRING",
            Datatype::List => "LIST",
            Datatype::Hash => "HASH",
            Datatype::ZSet => "ZSET",
            Datatype::Unknown => "UNKNOWN",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Operator {
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
    GreaterOrEqual,
    LessOrEqual,
    Like,
    NotLike,
    In,
    NotIn,
    Between,
    IsNull,
    IsNotNull,
}

impl Operator {
    pub fn all() -> Vec<Self> {
        vec![
            Self::Equal,
            Self::NotEqual,
            Self::GreaterThan,
            Self::LessThan,
            Self::GreaterOrEqual,
            Self::LessOrEqual,
            Self::Like,
            Self::NotLike,
            Self::IsNull,
            Self::IsNotNull,
        ]
    }

    pub fn label(&self) -> &str {
        match self {
            Self::Equal => "=",
            Self::NotEqual => "!=",
            Self::GreaterThan => ">",
            Self::LessThan => "<",
            Self::GreaterOrEqual => ">=",
            Self::LessOrEqual => "<=",
            Self::Like => "LIKE",
            Self::NotLike => "NOT LIKE",
            Self::IsNull => "IS NULL",
            Self::IsNotNull => "IS NOT NULL",
            Self::In => "IN",
            Self::NotIn => "NOT IN",
            Self::Between => "BETWEEN",
        }
    }

    pub fn from_label(label: &str) -> Self {
        match label {
            "=" => Self::Equal,
            "!=" => Self::NotEqual,
            ">" => Self::GreaterThan,
            "<" => Self::LessThan,
            ">=" => Self::GreaterOrEqual,
            "<=" => Self::LessOrEqual,
            "LIKE" => Self::Like,
            "NOT LIKE" => Self::NotLike,
            "IS NULL" => Self::IsNull,
            "IS NOT NULL" => Self::IsNotNull,
            "IN" => Self::In,
            "NOT IN" => Self::NotIn,
            "BETWEEN" => Self::Between,
            _ => Self::Equal, // 默认
        }
    }
}

#[derive(Clone, Debug)]
pub enum QueryReq {
    Sql {
        sql: String,
        args: Vec<String>,
    },
    Builder {
        table: String,
        columns: Vec<String>,
        limit: Option<usize>,
        offset: Option<usize>,
        orders: Vec<OrderCond>,
        filters: Vec<FilterCond>,
    },
    Command {
        name: String,
        args: Vec<Value>,
    },
    Document {
        collection: String,
        filter: Value,
    },
}

#[derive(Clone, Debug)]
pub enum InsertReq {
    Sql { sql: String },
    Command { name: String, args: Vec<Value> },
    Document { collection: String, document: Value },
}

#[derive(Clone, Debug)]
pub enum UpdateReq {
    Sql {
        sql: String,
    },
    Command {
        name: String,
        args: Vec<Value>,
    },
    Document {
        collection: String,
        filter: Value,
        update: Value,
    },
}

#[derive(Clone, Debug)]
pub enum DeleteReq {
    Sql { sql: String },
    Command { name: String, args: Vec<Value> },
    Document { collection: String, filter: Value },
}

#[derive(Clone, Debug)]
pub enum QueryResp {
    Rows(Vec<HashMap<String, String>>),
    Value(Value),
    Documents(Vec<Value>),
}

#[derive(Clone, Debug)]
pub struct UpdateResp {
    pub affected: u64,
}

#[derive(Clone, Debug)]
pub struct OrderCond {
    pub field: String,
    pub ascending: bool,
}

#[derive(Clone, Debug)]
pub enum ValueCond {
    Null,
    Bool(bool),
    String(String),
    Number(f64),
    List(Vec<String>),
    Range(String, String),
}

#[derive(Clone, Debug)]
pub struct FilterCond {
    pub field: String,
    pub operator: Operator,
    pub value: ValueCond,
}

#[derive(Debug, thiserror::Error)]
pub enum DriverError {
    #[error("{0}")]
    Other(String),
    #[error("配置字段缺失: {0}")]
    MissingField(String),
    #[error("配置字段非法: {0}")]
    InvalidField(String),
}

pub trait DatabaseDriver {
    type Config;

    fn data_types(&self) -> Vec<Datatype>;

    fn check_connection(
        &self,
        config: &Self::Config,
    ) -> Result<(), DriverError>;

    fn create_connection(
        &self,
        config: &Self::Config,
    ) -> Result<Box<dyn DatabaseSession>, DriverError>;
}

pub trait DatabaseSession: Send {
    fn query(
        &mut self,
        request: QueryReq,
    ) -> Result<QueryResp, DriverError>;

    fn insert(
        &mut self,
        request: InsertReq,
    ) -> Result<UpdateResp, DriverError>;

    fn update(
        &mut self,
        request: UpdateReq,
    ) -> Result<UpdateResp, DriverError>;

    fn delete(
        &mut self,
        request: DeleteReq,
    ) -> Result<UpdateResp, DriverError>;

    fn tables(&mut self) -> Result<Vec<String>, DriverError>;
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DataSource {
    pub id: String,
    pub name: String,
    pub desc: String,
    pub kind: DataSourceKind,
    pub options: DataSourceOptions,
    pub extras: Option<HashMap<String, Value>>,
}

impl DataSource {
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

    pub fn display_endpoint(&self) -> String {
        match &self.options {
            DataSourceOptions::MySQL(opts) => opts.endpoint(),
            DataSourceOptions::Oracle(opts) => opts.endpoint(),
            DataSourceOptions::SQLite(opts) => opts.endpoint(),
            DataSourceOptions::SQLServer(opts) => opts.endpoint(),
            DataSourceOptions::PostgreSQL(opts) => opts.endpoint(),
            DataSourceOptions::Redis(opts) => opts.endpoint(),
            DataSourceOptions::MongoDB(opts) => opts.endpoint(),
        }
    }

    pub fn display_overview(&self) -> Vec<(&'static str, String)> {
        match &self.options {
            DataSourceOptions::MySQL(opts) => opts.overview(),
            DataSourceOptions::Oracle(opts) => opts.overview(),
            DataSourceOptions::SQLite(opts) => opts.overview(),
            DataSourceOptions::SQLServer(opts) => opts.overview(),
            DataSourceOptions::PostgreSQL(opts) => opts.overview(),
            DataSourceOptions::Redis(opts) => opts.overview(),
            DataSourceOptions::MongoDB(opts) => opts.overview(),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
            DataSourceKind::MySQL => "开源关系型数据库,读写性能稳定、生态成熟",
            DataSourceKind::Oracle => "商业级事务数据库,强调安全性与可扩展性",
            DataSourceKind::SQLite => "嵌入式文件数据库,零配置、单文件存储",
            DataSourceKind::SQLServer => "微软企业数据库,原生集成 Windows 与 AD",
            DataSourceKind::PostgreSQL => "开源对象关系数据库,扩展能力与标准兼容性强",
            DataSourceKind::Redis => "内存键值数据库,适合缓存、队列与实时计数场景",
            DataSourceKind::MongoDB => "文档型数据库,支持灵活的 JSON 模式与水平扩展",
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum DataSourceOptions {
    MySQL(MySQLOptions),
    Oracle(OracleOptions),
    SQLite(SQLiteOptions),
    SQLServer(SQLServerOptions),
    PostgreSQL(PostgreSQLOptions),
    Redis(RedisOptions),
    MongoDB(MongoDBOptions),
}

pub fn get_datatypes(kind: DataSourceKind) -> Vec<Datatype> {
    match kind {
        DataSourceKind::MySQL => MySQLDriver.data_types(),
        DataSourceKind::PostgreSQL => PostgreSQLDriver.data_types(),
        DataSourceKind::SQLite => SQLiteDriver.data_types(),
        DataSourceKind::SQLServer => SQLServerDriver.data_types(),
        DataSourceKind::MongoDB => MongoDBDriver.data_types(),
        DataSourceKind::Redis => RedisDriver.data_types(),
        DataSourceKind::Oracle => vec![],
    }
}

pub fn check_connection(opts: &DataSourceOptions) -> Result<(), DriverError> {
    match opts {
        DataSourceOptions::MySQL(config) => MySQLDriver.check_connection(config),
        DataSourceOptions::PostgreSQL(config) => PostgreSQLDriver.check_connection(config),
        DataSourceOptions::SQLite(config) => SQLiteDriver.check_connection(config),
        DataSourceOptions::SQLServer(config) => SQLServerDriver.check_connection(config),
        DataSourceOptions::MongoDB(config) => MongoDBDriver.check_connection(config),
        DataSourceOptions::Redis(config) => RedisDriver.check_connection(config),
        DataSourceOptions::Oracle(_) => Err(DriverError::Other("Oracle 驱动暂未实现".into())),
    }
}

pub fn create_connection(opts: &DataSourceOptions) -> Result<Box<dyn DatabaseSession>, DriverError> {
    match opts {
        DataSourceOptions::MySQL(config) => MySQLDriver.create_connection(config),
        DataSourceOptions::PostgreSQL(config) => PostgreSQLDriver.create_connection(config),
        DataSourceOptions::SQLite(config) => SQLiteDriver.create_connection(config),
        DataSourceOptions::SQLServer(config) => SQLServerDriver.create_connection(config),
        DataSourceOptions::MongoDB(config) => MongoDBDriver.create_connection(config),
        DataSourceOptions::Redis(config) => RedisDriver.create_connection(config),
        DataSourceOptions::Oracle(_) => Err(DriverError::Other("Oracle 驱动暂未实现".into())),
    }
}

pub fn validate_sql(sql: &str) -> Result<(), DriverError> {
    if sql.trim().is_empty() {
        return Err(DriverError::InvalidField("sql".into()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {}
