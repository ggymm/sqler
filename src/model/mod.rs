use serde::{Deserialize, Serialize};

pub use kind::{ColumnKind, DataSourceKind};
pub use options::{
    DataSourceOptions, MongoDBHost, MongoDBOptions, MySQLOptions, OracleOptions, PostgresOptions, RedisOptions,
    SQLServerOptions, SQLiteOptions,
};

mod kind;
mod options;

#[derive(Clone, Serialize, Deserialize)]
pub struct DataSource {
    pub id: String,
    pub name: String,
    pub kind: DataSourceKind,
    pub options: DataSourceOptions,
}

impl DataSource {
    pub fn new(
        id: String,
        name: String,
        kind: DataSourceKind,
        options: DataSourceOptions,
    ) -> Self {
        Self {
            id,
            name,
            kind,
            options,
        }
    }

    pub fn display_endpoint(&self) -> String {
        match &self.options {
            DataSourceOptions::MySQL(opts) => opts.endpoint(),
            DataSourceOptions::SQLite(opts) => opts.endpoint(),
            DataSourceOptions::Postgres(opts) => opts.endpoint(),
            DataSourceOptions::Oracle(opts) => opts.endpoint(),
            DataSourceOptions::SQLServer(opts) => opts.endpoint(),
            DataSourceOptions::Redis(opts) => opts.endpoint(),
            DataSourceOptions::MongoDB(opts) => opts.endpoint(),
        }
    }

    pub fn display_overview(&self) -> Vec<(&'static str, String)> {
        match &self.options {
            DataSourceOptions::MySQL(opts) => opts.overview(),
            DataSourceOptions::SQLite(opts) => opts.overview(),
            DataSourceOptions::Postgres(opts) => opts.overview(),
            DataSourceOptions::Oracle(opts) => opts.overview(),
            DataSourceOptions::SQLServer(opts) => opts.overview(),
            DataSourceOptions::Redis(opts) => opts.overview(),
            DataSourceOptions::MongoDB(opts) => opts.overview(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueryRecord {
    pub id: String,
    pub datasource_id: String,
    pub sql: String,
    pub executed_at: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommandRecord {
    pub id: String,
    pub datasource_id: String,
    pub command: String,
    pub executed_at: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TableInfo {
    pub name: String,
    pub row_count: Option<u64>,
    pub size_bytes: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: ColumnKind,
    pub nullable: bool,
    pub default_value: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TableSchema {
    pub table_name: String,
    pub columns: Vec<ColumnInfo>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecentItems {
    pub datasource_id: String,
    pub tables: Vec<String>,
    pub updated_at: i64,
}
