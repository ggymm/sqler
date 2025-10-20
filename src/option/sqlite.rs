use crate::option::ConnectionOptions;
use crate::DataSourceType;

#[derive(Debug, Clone, PartialEq, Eq)]
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
    fn kind(&self) -> DataSourceType {
        DataSourceType::SQLite
    }
}
