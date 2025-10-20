use crate::option::ConnectionOptions;
use crate::option::DataSourceKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SslMode {
    Disable,
    Prefer,
    Require,
}

#[derive(Clone)]
pub struct PostgreSQLOptions {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: Option<String>,
    pub schema: Option<String>,
    pub ssl_mode: Option<SslMode>,
}

impl Default for PostgreSQLOptions {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".into(),
            port: 5432,
            database: String::new(),
            username: "postgres".into(),
            password: None,
            schema: None,
            ssl_mode: None,
        }
    }
}

impl ConnectionOptions for PostgreSQLOptions {
    fn kind(&self) -> DataSourceKind {
        DataSourceKind::PostgreSQL
    }
}
