use std::collections::HashMap;

use serde_json::Value;

use crate::model::{ColumnInfo, ColumnKind, DataSourceKind, DataSourceOptions, TableInfo};

pub use mongodb::MongoDBDriver;
pub use mysql::MySQLDriver;
pub use postgres::PostgresDriver;
pub use redis::RedisDriver;
pub use sqlite::SQLiteDriver;
pub use sqlserver::SQLServerDriver;

mod mongodb;
mod mysql;
mod oracle;
mod postgres;
mod redis;
mod sqlite;
mod sqlserver;

#[derive(Clone, Debug)]
pub struct Paging {
    page: usize,
    size: usize,
}

impl Paging {
    pub fn new(
        page: usize,
        size: usize,
    ) -> Self {
        Self { page, size }
    }

    pub fn limit(&self) -> usize {
        self.size
    }

    pub fn offset(&self) -> usize {
        self.page * self.size
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
        paging: Option<Paging>,
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
pub enum ExecReq {
    Sql { sql: String },
    Command { name: String, args: Vec<Value> },
    Document { collection: String, operation: DocumentOp },
}

#[derive(Clone, Debug)]
pub struct ExecResp {
    pub affected: u64,
}

#[derive(Clone, Debug)]
pub enum DocumentOp {
    Insert { document: Value },
    Update { filter: Value, update: Value },
    Delete { filter: Value },
}

#[derive(Clone, Debug)]
pub enum QueryResp {
    Rows {
        cols: Vec<String>,
        rows: Vec<HashMap<String, String>>,
    },
    Value(Value),
    Documents(Vec<Value>),
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

    fn supp_kinds(&self) -> Vec<ColumnKind>;

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
    fn exec(
        &mut self,
        req: ExecReq,
    ) -> Result<ExecResp, DriverError>;

    fn query(
        &mut self,
        req: QueryReq,
    ) -> Result<QueryResp, DriverError>;

    fn tables(&mut self) -> Result<Vec<TableInfo>, DriverError>;

    fn columns(
        &mut self,
        table: &str,
    ) -> Result<Vec<ColumnInfo>, DriverError>;
}

pub fn supp_kinds(kind: DataSourceKind) -> Vec<ColumnKind> {
    match kind {
        DataSourceKind::MySQL => MySQLDriver.supp_kinds(),
        DataSourceKind::SQLite => SQLiteDriver.supp_kinds(),
        DataSourceKind::Postgres => PostgresDriver.supp_kinds(),
        DataSourceKind::Oracle => vec![],
        DataSourceKind::SQLServer => SQLServerDriver.supp_kinds(),
        DataSourceKind::MongoDB => MongoDBDriver.supp_kinds(),
        DataSourceKind::Redis => RedisDriver.supp_kinds(),
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

pub fn escape_quote(s: &str) -> String {
    s.replace('"', "\"\"")
}

pub fn escape_backtick(s: &str) -> String {
    s.replace('`', "``")
}

#[cfg(test)]
mod tests {}
