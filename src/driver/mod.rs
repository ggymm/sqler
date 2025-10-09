mod mongodb;
mod mysql;
mod sqlite;

use iced::Task;
use serde::{Deserialize, Serialize};
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
    #[error("请求不合法: {0}")]
    InvalidRequest(String),
    #[error("暂不支持: {0}")]
    Unsupported(String),
    #[error("连接失败: {0}")]
    Connection(String),
    #[error("查询失败: {0}")]
    Query(String),
    #[error("执行失败: {0}")]
    Execution(String),
}

#[derive(Debug, Clone)]
pub enum QueryRequest {
    Sql { statement: String },
    KeyValue(KeyQuery),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KeyQuery {
    pub database: String,
    pub collection: String,
    #[serde(default)]
    pub filter: JsonValue,
    #[serde(default)]
    pub projection: Option<JsonValue>,
    #[serde(default)]
    pub limit: Option<i64>,
    #[serde(default)]
    pub skip: Option<u64>,
    #[serde(default)]
    pub sort: Option<JsonValue>,
}

#[derive(Debug, Clone)]
pub enum ExecuteRequest {
    Sql { statement: String },
    KeyValue(KeyCommand),
}

#[derive(Debug, Clone)]
pub struct KeyCommand {
    pub database: String,
    pub collection: String,
    pub action: KeyAction,
}

#[derive(Debug, Clone)]
pub enum KeyAction {
    InsertOne { document: JsonValue },
    UpdateMany { filter: JsonValue, update: JsonValue },
    DeleteMany { filter: JsonValue },
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
    Documents {
        documents: Vec<JsonValue>,
    },
}

#[derive(Debug, Clone)]
pub struct ExecuteResponse {
    pub affected_count: u64,
    pub generated: Option<JsonValue>,
}

#[derive(Clone, Debug)]
pub struct DriverRegistry {
    mysql: mysql::MysqlDriver,
    sqlite: sqlite::SqliteDriver,
    mongodb: mongodb::MongoDriver,
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
            other => Task::done(Err(DriverError::Unsupported(format!("{other:?} 暂未实现查询功能")))),
        }
    }

    pub fn execute(
        &self,
        params: ConnectionParams,
        request: ExecuteRequest,
    ) -> Task<Result<ExecuteResponse, DriverError>> {
        match params.kind {
            DatabaseKind::Mysql => self.mysql.execute(params, request),
            DatabaseKind::Sqlite => self.sqlite.execute(params, request),
            DatabaseKind::Mongodb => self.mongodb.execute(params, request),
            other => Task::done(Err(DriverError::Unsupported(format!("{other:?} 暂未实现执行功能")))),
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

pub(crate) fn make_document_response(documents: Vec<JsonValue>) -> QueryResponse {
    QueryResponse {
        payload: QueryPayload::Documents { documents },
    }
}

pub(crate) fn execution_response(
    affected_count: u64,
    generated: Option<JsonValue>,
) -> ExecuteResponse {
    ExecuteResponse {
        affected_count,
        generated,
    }
}

pub(crate) fn invalid_request(message: impl Into<String>) -> DriverError {
    DriverError::InvalidRequest(message.into())
}

pub(crate) fn unsupported(message: impl Into<String>) -> DriverError {
    DriverError::Unsupported(message.into())
}
