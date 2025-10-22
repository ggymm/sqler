use serde_json::Map;
use serde_json::Value;

use crate::option::ConnectionOptions;
use crate::option::DataSourceKind;
use crate::option::DataSourceOptions;

pub use mongodb::MongoDBDriver;
pub use mysql::MySQLDriver;
pub use postgres::PostgreSQLDriver;
pub use redis::RedisDriver;
pub use sqlite::SQLiteDriver;
pub use sqlserver::SQLServerDriver;

pub mod mongodb;
pub mod mysql;
pub mod oracle;
pub mod postgres;
pub mod redis;
pub mod sqlite;
pub mod sqlserver;

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
    Sql { statement: String },
    Command { name: String, args: Vec<Value> },
    Document { collection: String, filter: Value },
}

#[derive(Clone, Debug)]
pub enum QueryResp {
    Rows(Vec<Map<String, Value>>),
    Value(Value),
    Documents(Vec<Value>),
}

#[derive(Clone, Debug)]
pub enum InsertReq {
    Sql { statement: String },
    Command { name: String, args: Vec<Value> },
    Document { collection: String, document: Value },
}

#[derive(Clone, Debug)]
pub enum UpdateReq {
    Sql {
        statement: String,
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
    Sql { statement: String },
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
