use std::collections::HashMap;

use serde_json::Value;

pub use mongodb::MongoDBDriver;
pub use mysql::MySQLDriver;
pub use postgres::PostgresDriver;
pub use redis::RedisDriver;
pub use sqlite::SQLiteDriver;
pub use sqlserver::SQLServerDriver;

use crate::model::{DataSourceKind, DataSourceOptions};

pub use crate::model::{MongoDBHost, MongoDBOptions, MySQLOptions, PostgresOptions, RedisOptions, SQLiteOptions};

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

    fn columns(
        &mut self,
        table: &str,
    ) -> Result<Vec<String>, DriverError>;
}

pub fn get_datatypes(kind: DataSourceKind) -> Vec<Datatype> {
    match kind {
        DataSourceKind::MySQL => MySQLDriver.data_types(),
        DataSourceKind::SQLite => SQLiteDriver.data_types(),
        DataSourceKind::Postgres => PostgresDriver.data_types(),
        DataSourceKind::Oracle => vec![],
        DataSourceKind::SQLServer => SQLServerDriver.data_types(),
        DataSourceKind::MongoDB => MongoDBDriver.data_types(),
        DataSourceKind::Redis => RedisDriver.data_types(),
    }
}

pub fn check_connection(opts: &DataSourceOptions) -> Result<(), DriverError> {
    match opts {
        DataSourceOptions::MySQL(config) => MySQLDriver.check_connection(config),
        DataSourceOptions::SQLite(config) => SQLiteDriver.check_connection(config),
        DataSourceOptions::Postgres(config) => PostgresDriver.check_connection(config),
        DataSourceOptions::Oracle(_) => Err(DriverError::Other("Oracle 驱动暂未实现".into())),
        DataSourceOptions::SQLServer(config) => SQLServerDriver.check_connection(config),
        DataSourceOptions::MongoDB(config) => MongoDBDriver.check_connection(config),
        DataSourceOptions::Redis(config) => RedisDriver.check_connection(config),
    }
}

pub fn create_connection(opts: &DataSourceOptions) -> Result<Box<dyn DatabaseSession>, DriverError> {
    match opts {
        DataSourceOptions::MySQL(config) => MySQLDriver.create_connection(config),
        DataSourceOptions::SQLite(config) => SQLiteDriver.create_connection(config),
        DataSourceOptions::Postgres(config) => PostgresDriver.create_connection(config),
        DataSourceOptions::Oracle(_) => Err(DriverError::Other("Oracle 驱动暂未实现".into())),
        DataSourceOptions::SQLServer(config) => SQLServerDriver.create_connection(config),
        DataSourceOptions::MongoDB(config) => MongoDBDriver.create_connection(config),
        DataSourceOptions::Redis(config) => RedisDriver.create_connection(config),
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
