use serde::{Deserialize, Serialize};

use crate::option::{ConnectionOptions, DataSourceKind};

#[derive(Clone, Serialize, Deserialize)]
pub struct SQLiteOptions {
    pub filepath: String,
    pub password: Option<String>,
    pub read_only: bool,
}

impl Default for SQLiteOptions {
    fn default() -> Self {
        Self {
            filepath: String::new(),
            password: None,
            read_only: false,
        }
    }
}

impl ConnectionOptions for SQLiteOptions {
    fn kind(&self) -> DataSourceKind {
        DataSourceKind::SQLite
    }
}

impl SQLiteOptions {
    pub fn display_endpoint(&self) -> String {
        let path = self.filepath.trim();
        if path.is_empty() {
            "sqlite://<未配置文件>".into()
        } else if self.read_only {
            format!("sqlite://{}?mode=ro", path)
        } else {
            format!("sqlite://{}", path)
        }
    }
}
