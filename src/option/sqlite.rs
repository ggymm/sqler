use serde::Deserialize;
use serde::Serialize;

use crate::option::ConnectionOptions;
use crate::option::DataSourceKind;

#[derive(Clone, Serialize, Deserialize)]
pub struct SQLiteOptions {
    pub file_path: String,
    pub password: Option<String>,
    pub read_only: bool,
}

impl Default for SQLiteOptions {
    fn default() -> Self {
        Self {
            file_path: String::new(),
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
        let path = self.file_path.trim();
        if path.is_empty() {
            "sqlite://<未配置文件>".into()
        } else if self.read_only {
            format!("sqlite://{}?mode=ro", path)
        } else {
            format!("sqlite://{}", path)
        }
    }
}
