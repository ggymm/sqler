use crate::option::ConnectionOptions;
use crate::option::DataSourceKind;

#[derive(Clone)]
pub struct MySQLOptions {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: Option<String>,
    pub database: String,
    pub charset: Option<String>,
    pub use_tls: bool,
}

impl Default for MySQLOptions {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".into(),
            port: 3306,
            username: "root".into(),
            password: None,
            database: String::new(),
            charset: Some("utf8mb4".into()),
            use_tls: false,
        }
    }
}

impl ConnectionOptions for MySQLOptions {
    fn kind(&self) -> DataSourceKind {
        DataSourceKind::MySQL
    }
}
