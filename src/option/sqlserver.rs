use crate::option::ConnectionOptions;
use crate::DataSourceType;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SQLServerAuth {
    SqlPassword,
    Integrated,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SQLServerOptions {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub auth: SQLServerAuth,
    pub instance: Option<String>,
}

impl Default for SQLServerOptions {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".into(),
            port: 1433,
            database: String::new(),
            username: None,
            password: None,
            auth: SQLServerAuth::SqlPassword,
            instance: None,
        }
    }
}

impl ConnectionOptions for SQLServerOptions {
    fn kind(&self) -> DataSourceType {
        DataSourceType::SQLServer
    }
}
