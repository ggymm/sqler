use std::collections::HashMap;

use serde_json::{Number, Value};

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

// ==================== 通用工具函数 ====================

/// 验证 SQL 语句是否为空
pub(crate) fn validate_stmt(stmt: &str) -> Result<(), DriverError> {
    if stmt.trim().is_empty() {
        return Err(DriverError::InvalidField("statement".into()));
    }
    Ok(())
}

/// 将 f64 转换为 JSON 数字，无效时转为字符串
pub(crate) fn number_from_f64(value: f64) -> Value {
    Number::from_f64(value)
        .map(Value::Number)
        .unwrap_or_else(|| Value::String(value.to_string()))
}

// ==================== 错误定义 ====================

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
    // 查询
    fn query(
        &mut self,
        request: QueryReq,
    ) -> Result<QueryResp, DriverError>;

    // 插入
    fn insert(
        &mut self,
        request: InsertReq,
    ) -> Result<WriteResp, DriverError>;

    // 更新
    fn update(
        &mut self,
        request: UpdateReq,
    ) -> Result<WriteResp, DriverError>;

    // 删除
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

#[cfg(test)]
mod tests {}
