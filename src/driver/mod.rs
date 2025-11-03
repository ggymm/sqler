use std::collections::HashMap;

use serde_json::Value;

use crate::option::{ConnectionOptions, DataSourceKind, DataSourceOptions};

pub mod mongodb;
pub mod mysql;
pub mod oracle;
pub mod postgres;
pub mod redis;
pub mod sqlite;
pub mod sqlserver;

pub use mongodb::MongoDBDriver;
pub use mysql::MySQLDriver;
pub use postgres::PostgreSQLDriver;
pub use redis::RedisDriver;
pub use sqlite::SQLiteDriver;
pub use sqlserver::SQLServerDriver;

#[derive(Debug, thiserror::Error)]
pub enum DriverError {
    #[error("配置字段缺失: {0}")]
    MissingField(String),
    #[error("配置字段非法: {0}")]
    InvalidField(String),
    #[error("{0}")]
    Other(String),
}

#[derive(Clone, Debug)]
pub enum QueryReq {
    Sql { stmt: String, params: Vec<String> },
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
pub enum InsertReq {
    Sql { stmt: String },
    Command { name: String, args: Vec<Value> },
    Document { collection: String, document: Value },
}

#[derive(Clone, Debug)]
pub enum UpdateReq {
    Sql {
        stmt: String,
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
    Sql { stmt: String },
    Command { name: String, args: Vec<Value> },
    Document { collection: String, filter: Value },
}

#[derive(Clone, Copy, Debug, Default)]
pub struct WriteResp {
    pub affected: u64,
}

pub trait DatabaseSession: Send {
    fn query(
        &mut self,
        request: QueryReq,
    ) -> Result<QueryResp, DriverError>;

    fn insert(
        &mut self,
        request: InsertReq,
    ) -> Result<WriteResp, DriverError>;

    fn update(
        &mut self,
        request: UpdateReq,
    ) -> Result<WriteResp, DriverError>;

    fn delete(
        &mut self,
        request: DeleteReq,
    ) -> Result<WriteResp, DriverError>;
}

pub trait DatabaseDriver {
    type Config;

    fn check_connection(
        &self,
        config: &Self::Config,
    ) -> Result<(), DriverError>;

    fn create_connection(
        &self,
        config: &Self::Config,
    ) -> Result<Box<dyn DatabaseSession>, DriverError>;
}

pub fn check_connection(
    kind: DataSourceKind,
    opts: &DataSourceOptions,
) -> Result<(), DriverError> {
    if opts.kind() != kind {
        return Err(DriverError::InvalidField(format!("数据源类型不匹配: {}", kind.label())));
    }

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

pub fn create_connection(
    kind: DataSourceKind,
    opts: &DataSourceOptions,
) -> Result<Box<dyn DatabaseSession>, DriverError> {
    if opts.kind() != kind {
        return Err(DriverError::InvalidField(format!("数据源类型不匹配: {}", kind.label())));
    }

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
pub enum ConditionValue {
    String(String),
    Number(f64),
    Bool(bool),
    Null,
    List(Vec<String>),
    Range(String, String),
}

#[derive(Clone, Debug)]
pub struct OrderCond {
    pub field: String,
    pub ascending: bool,
}

#[derive(Clone, Debug)]
pub struct FilterCond {
    pub field: String,
    pub operator: Operator,
    pub value: ConditionValue,
}

#[derive(Clone, Debug, Default)]
pub struct QueryConditions {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub orders: Vec<OrderCond>,
    pub filters: Vec<FilterCond>,
}

pub trait QueryBuilder {
    fn build_order_clause(
        &self,
        orders: &[OrderCond],
    ) -> String;

    fn build_where_clause(
        &self,
        filters: &[FilterCond],
    ) -> (String, Vec<String>);

    fn build_limit_clause(
        &self,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> String;

    fn escape_identifier(
        &self,
        identifier: &str,
    ) -> String;

    fn placeholder(
        &self,
        index: usize,
    ) -> String;

    fn build_select_query(
        &self,
        table: &str,
        columns: &[&str],
        conditions: &QueryConditions,
    ) -> (String, Vec<String>) {
        let cols = if columns.is_empty() {
            "*".to_string()
        } else {
            columns
                .iter()
                .map(|c| self.escape_identifier(c))
                .collect::<Vec<_>>()
                .join(", ")
        };

        let mut sql = format!("SELECT {} FROM {}", cols, self.escape_identifier(table));
        let mut params = Vec::new();

        // WHERE 子句
        if !conditions.filters.is_empty() {
            let (where_clause, where_params) = self.build_where_clause(&conditions.filters);
            sql.push_str(&format!(" WHERE {}", where_clause));
            params.extend(where_params);
        }

        // ORDER BY 子句
        if !conditions.orders.is_empty() {
            let order_clause = self.build_order_clause(&conditions.orders);
            if !order_clause.is_empty() {
                sql.push_str(&format!(" {}", order_clause));
            }
        }

        // LIMIT/OFFSET 子句
        let limit_clause = self.build_limit_clause(conditions.limit, conditions.offset);
        if !limit_clause.is_empty() {
            sql.push_str(&format!(" {}", limit_clause));
        }

        (sql, params)
    }

    fn build_count_query(
        &self,
        table: &str,
        conditions: &QueryConditions,
    ) -> (String, Vec<String>) {
        let mut sql = format!("SELECT COUNT(*) FROM {}", self.escape_identifier(table));
        let mut params = Vec::new();

        // WHERE 子句
        if !conditions.filters.is_empty() {
            let (where_clause, where_params) = self.build_where_clause(&conditions.filters);
            sql.push_str(&format!(" WHERE {}", where_clause));
            params.extend(where_params);
        }

        (sql, params)
    }
}

pub fn create_builder(kind: DataSourceKind) -> Box<dyn QueryBuilder> {
    match kind {
        DataSourceKind::MySQL => Box::new(mysql::MySQLBuilder),
        DataSourceKind::PostgreSQL => Box::new(postgres::PostgreSQLBuilder),
        DataSourceKind::SQLite => Box::new(sqlite::SQLiteBuilder),
        _ => Box::new(mysql::MySQLBuilder), // 默认使用 MySQL
    }
}

pub fn validate_stmt(stmt: &str) -> Result<(), DriverError> {
    if stmt.trim().is_empty() {
        return Err(DriverError::InvalidField("statement".into()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {}
