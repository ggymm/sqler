use serde::Deserialize;
use serde::Serialize;

use crate::option::ConnectionOptions;
use crate::option::DataSourceKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SQLServerAuth {
    SqlPassword,
    Integrated,
}

#[derive(Clone, Serialize, Deserialize)]
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
    fn kind(&self) -> DataSourceKind {
        DataSourceKind::SQLServer
    }
}

impl SQLServerOptions {
    pub fn display_endpoint(&self) -> String {
        let mut authority = format!("{}:{}", self.host, self.port);
        if let Some(instance) = &self.instance {
            let trimmed = instance.trim();
            if !trimmed.is_empty() {
                authority = format!("{}\\{}", authority, trimmed);
            }
        }

        let db = self.database.trim();
        if db.is_empty() {
            format!("sqlserver://{}", authority)
        } else {
            format!("sqlserver://{}/{}", authority, db)
        }
    }
}
