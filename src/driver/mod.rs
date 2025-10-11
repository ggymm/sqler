mod mongodb;
mod mysql;
mod redis;
mod sqlite;

use iced::Task;
use serde_json::Value as JsonValue;

use crate::app::DatabaseKind;
use base64::Engine;

#[derive(Debug, Clone)]
pub struct ConnectionParams {
    pub kind: DatabaseKind,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub database: Option<String>,
    pub file_path: Option<String>,
    pub connection_string: Option<String>,
}

impl ConnectionParams {
    pub fn require_host(&self) -> Result<&str, DriverError> {
        self.host.as_deref().ok_or_else(|| DriverError::MissingField("host"))
    }

    pub fn require_port(&self) -> Result<u16, DriverError> {
        self.port.ok_or_else(|| DriverError::MissingField("port"))
    }

    pub fn require_username(&self) -> Result<&str, DriverError> {
        self.username
            .as_deref()
            .ok_or_else(|| DriverError::MissingField("username"))
    }

    pub fn require_file_path(&self) -> Result<&str, DriverError> {
        self.file_path
            .as_deref()
            .ok_or_else(|| DriverError::MissingField("file_path"))
    }

    pub fn require_connection_string(&self) -> Result<&str, DriverError> {
        self.connection_string
            .as_deref()
            .ok_or_else(|| DriverError::MissingField("connection_string"))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DriverError {
    #[error("缺少字段: {0}")]
    MissingField(&'static str),
    #[error("暂不支持: {0}")]
    Unsupported(String),
    #[error("连接失败: {0}")]
    Connection(String),
    #[error("查询失败: {0}")]
    Query(String),
}

#[derive(Debug, Clone)]
pub enum QueryRequest {
    Sql { statement: String },
    RedisDatabases,
}

#[derive(Debug, Clone)]
pub struct QueryResponse {
    pub payload: QueryPayload,
}

#[derive(Debug, Clone)]
pub enum QueryPayload {
    Tabular {
        columns: Vec<String>,
        rows: Vec<Vec<JsonValue>>,
    },
}

#[derive(Clone, Debug)]
pub struct DriverRegistry {
    mysql: mysql::MysqlDriver,
    sqlite: sqlite::SqliteDriver,
    mongodb: mongodb::MongoDriver,
    redis: redis::RedisDriver,
}

impl Default for DriverRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl DriverRegistry {
    pub fn new() -> Self {
        Self {
            mysql: mysql::MysqlDriver::new(),
            sqlite: sqlite::SqliteDriver::new(),
            mongodb: mongodb::MongoDriver::new(),
            redis: redis::RedisDriver::new(),
        }
    }

    pub fn test_connection(
        &self,
        params: ConnectionParams,
    ) -> Task<Result<(), DriverError>> {
        match params.kind {
            DatabaseKind::Mysql => self.mysql.test_connection(params),
            DatabaseKind::Sqlite => self.sqlite.test_connection(params),
            DatabaseKind::Mongodb => self.mongodb.test_connection(params),
            DatabaseKind::Redis => self.redis.test_connection(params),
            other => Task::done(Err(DriverError::Unsupported(format!("{other:?} 暂未实现")))),
        }
    }

    pub fn query(
        &self,
        params: ConnectionParams,
        request: QueryRequest,
    ) -> Task<Result<QueryResponse, DriverError>> {
        match params.kind {
            DatabaseKind::Mysql => self.mysql.query(params, request),
            DatabaseKind::Sqlite => self.sqlite.query(params, request),
            DatabaseKind::Mongodb => self.mongodb.query(params, request),
            DatabaseKind::Redis => self.redis.query(params, request),
            other => Task::done(Err(DriverError::Unsupported(format!("{other:?} 暂未实现查询功能")))),
        }
    }
}

pub(crate) fn encode_binary(bytes: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

pub(crate) fn value_to_json_f64<T>(value: T) -> JsonValue
where
    T: Into<f64>,
{
    JsonValue::from(value.into())
}

pub(crate) fn map_binary_or_text(bytes: Vec<u8>) -> JsonValue {
    match String::from_utf8(bytes.clone()) {
        Ok(text) => JsonValue::String(text),
        Err(_) => JsonValue::String(format!("base64:{}", encode_binary(&bytes))),
    }
}

pub(crate) fn make_tabular_response(
    columns: Vec<String>,
    rows: Vec<Vec<JsonValue>>,
) -> QueryResponse {
    QueryResponse {
        payload: QueryPayload::Tabular { columns, rows },
    }
}

pub(crate) fn unsupported(message: impl Into<String>) -> DriverError {
    DriverError::Unsupported(message.into())
}
