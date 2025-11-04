use serde::{Deserialize, Serialize};

use crate::option::{ConnectionOptions, DataSourceKind};

#[derive(Clone, Serialize, Deserialize)]
pub struct PostgreSQLOptions {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: Option<String>,
    pub use_tls: bool,
}

impl Default for PostgreSQLOptions {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".into(),
            port: 5432,
            database: String::new(),
            username: "postgres".into(),
            password: None,
            use_tls: false,
        }
    }
}

impl ConnectionOptions for PostgreSQLOptions {
    fn kind(&self) -> DataSourceKind {
        DataSourceKind::PostgreSQL
    }
}

impl PostgreSQLOptions {
    pub fn display_endpoint(&self) -> String {
        let db = self.database.trim();
        let suffix = if db.is_empty() {
            String::new()
        } else {
            format!("/{}", db)
        };
        format!("postgres://{}:{}{}", self.host, self.port, suffix)
    }
}
