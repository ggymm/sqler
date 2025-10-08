mod mysql;

use iced::Task;

use crate::app::DatabaseKind;

#[derive(Debug, Clone)]
#[allow(dead_code)]
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

    pub fn password(&self) -> Option<&str> {
        self.password.as_deref()
    }

    pub fn database(&self) -> Option<&str> {
        self.database.as_deref()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DriverError {
    #[error("缺少字段: {0}")]
    MissingField(&'static str),
    #[error("暂不支持该数据库类型")]
    Unsupported,
    #[error("连接失败: {0}")]
    Connection(String),
}

#[derive(Clone, Debug)]
pub struct DriverRegistry {
    mysql: mysql::MysqlDriver,
}

impl DriverRegistry {
    pub fn new() -> Self {
        Self {
            mysql: mysql::MysqlDriver::new(),
        }
    }

    pub fn test_connection(
        &self,
        params: ConnectionParams,
    ) -> Task<Result<(), DriverError>> {
        match params.kind {
            DatabaseKind::Mysql => self.mysql.run_test_connection(params),
            _ => Task::done(Err(DriverError::Unsupported)),
        }
    }
}
