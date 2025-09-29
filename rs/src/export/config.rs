use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DbType {
    Sqlite,
    Postgres,
    Mysql,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbConfig {
    pub db_type: DbType,

    // SQLite
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sqlite_path: Option<PathBuf>,

    // Networked DBs
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub database: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<Vec<(String, String)>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    Csv,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportOptions {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub table: Option<String>,

    pub output_path: PathBuf,
    #[serde(default = "default_format")] 
    pub format: ExportFormat,

    // CSV options
    #[serde(default = "default_include_headers")] 
    pub include_headers: bool,
    #[serde(default = "default_delimiter")] 
    pub delimiter: char,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub null_text: Option<String>,
}

fn default_format() -> ExportFormat { ExportFormat::Csv }
fn default_include_headers() -> bool { true }
fn default_delimiter() -> char { ',' }

