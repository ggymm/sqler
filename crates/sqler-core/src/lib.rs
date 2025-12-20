use std::path::Path;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// 核心模块导出
pub mod cache;
pub mod driver;

// 重新导出 driver 和 cache 类型
pub use driver::{
    check_connection, create_connection, supp_kinds, DatabaseDriver, DatabaseSession, DriverError,
    ExecReq, ExecResp, FilterCond, Operator, OrderCond, Paging, QueryReq, QueryResp, ValueCond,
};

pub use cache::{AppCache, ArcCache, CacheError};

// ============================================================================
// Model Types (原 model.rs 内容)
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TableInfo {
    pub name: String,
    pub row_count: Option<u64>,
    pub size_bytes: Option<u64>,
    pub last_accessed: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ColumnInfo {
    pub name: String,
    pub kind: String,
    pub comment: String,
    pub nullable: bool,
    pub primary_key: bool,
    pub default_value: String,
    pub max_length: u64,
    pub auto_increment: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SavedQuery {
    pub name: String,
    pub content: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DataSource {
    pub id: String,
    pub name: String,
    pub kind: DataSourceKind,
    pub options: DataSourceOptions,
}

impl DataSource {
    pub fn new(
        name: String,
        kind: DataSourceKind,
        options: DataSourceOptions,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColumnKind {
    TinyInt,
    SmallInt,
    Int,
    BigInt,
    Float,
    Double,
    Decimal,
    Char,
    VarChar,
    Text,
    Binary,
    VarBinary,
    Blob,
    Date,
    Time,
    DateTime,
    Timestamp,
    Boolean,
    Json,
    Uuid,
    Enum,
    Set,
    Document,
    Array,
    String,
    List,
    Hash,
    ZSet,
    Unknown,
}

impl ColumnKind {
    pub fn label(&self) -> &'static str {
        match self {
            ColumnKind::TinyInt => "TINYINT",
            ColumnKind::SmallInt => "SMALLINT",
            ColumnKind::Int => "INT",
            ColumnKind::BigInt => "BIGINT",
            ColumnKind::Float => "FLOAT",
            ColumnKind::Double => "DOUBLE",
            ColumnKind::Decimal => "DECIMAL",
            ColumnKind::Char => "CHAR",
            ColumnKind::VarChar => "VARCHAR",
            ColumnKind::Text => "TEXT",
            ColumnKind::Binary => "BINARY",
            ColumnKind::VarBinary => "VARBINARY",
            ColumnKind::Blob => "BLOB",
            ColumnKind::Date => "DATE",
            ColumnKind::Time => "TIME",
            ColumnKind::DateTime => "DATETIME",
            ColumnKind::Timestamp => "TIMESTAMP",
            ColumnKind::Boolean => "BOOLEAN",
            ColumnKind::Json => "JSON",
            ColumnKind::Uuid => "UUID",
            ColumnKind::Enum => "ENUM",
            ColumnKind::Set => "SET",
            ColumnKind::Document => "DOCUMENT",
            ColumnKind::Array => "ARRAY",
            ColumnKind::String => "STRING",
            ColumnKind::List => "LIST",
            ColumnKind::Hash => "HASH",
            ColumnKind::ZSet => "ZSET",
            ColumnKind::Unknown => "UNKNOWN",
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataSourceKind {
    MySQL,
    SQLite,
    Postgres,
    Oracle,
    SQLServer,
    Redis,
    MongoDB,
}

impl DataSourceKind {
    pub fn all() -> &'static [DataSourceKind] {
        &[
            DataSourceKind::MySQL,
            DataSourceKind::SQLite,
            DataSourceKind::Postgres,
            DataSourceKind::Oracle,
            DataSourceKind::SQLServer,
            DataSourceKind::Redis,
            DataSourceKind::MongoDB,
        ]
    }

    pub fn image(&self) -> &'static str {
        match self {
            DataSourceKind::MySQL => "icons/mysql.svg",
            DataSourceKind::SQLite => "icons/sqlite.svg",
            DataSourceKind::Postgres => "icons/postgresql.svg",
            DataSourceKind::Oracle => "icons/oracle.svg",
            DataSourceKind::SQLServer => "icons/sqlserver.svg",
            DataSourceKind::Redis => "icons/redis.svg",
            DataSourceKind::MongoDB => "icons/mongodb.svg",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            DataSourceKind::MySQL => "MySQL",
            DataSourceKind::SQLite => "SQLite",
            DataSourceKind::Postgres => "PostgreSQL",
            DataSourceKind::Oracle => "Oracle",
            DataSourceKind::SQLServer => "SQLServer",
            DataSourceKind::Redis => "Redis",
            DataSourceKind::MongoDB => "MongoDB",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            DataSourceKind::MySQL => "开源关系型数据库,读写性能稳定、生态成熟",
            DataSourceKind::SQLite => "嵌入式文件数据库,零配置、单文件存储",
            DataSourceKind::Postgres => "开源对象关系数据库,扩展能力与标准兼容性强",
            DataSourceKind::Oracle => "商业级事务数据库,强调安全性与可扩展性",
            DataSourceKind::SQLServer => "微软企业数据库,原生集成 Windows 与 AD",
            DataSourceKind::Redis => "内存键值数据库,适合缓存、队列与实时计数场景",
            DataSourceKind::MongoDB => "文档型数据库,支持灵活的 JSON 模式与水平扩展",
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MySQLOptions {
    pub host: String,
    pub port: String,
    pub username: String,
    pub password: String,
    pub database: String,
    pub use_tls: bool,
}

impl Default for MySQLOptions {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".into(),
            port: "3306".into(),
            username: "root".into(),
            password: "".into(),
            database: String::new(),
            use_tls: false,
        }
    }
}

impl MySQLOptions {
    pub fn endpoint(&self) -> String {
        let scheme = if self.use_tls { "mysqls" } else { "mysql" };
        let db = self.database.trim();
        if db.is_empty() {
            format!("{}://{}:{}", scheme, self.host, self.port)
        } else {
            format!("{}://{}:{}/{}", scheme, self.host, self.port, db)
        }
    }

    pub fn overview(&self) -> Vec<(&'static str, String)> {
        vec![
            ("连接地址", format!("{}:{}", self.host, self.port)),
            ("账号", self.username.clone()),
            (
                "数据库",
                if self.database.is_empty() {
                    "未配置".into()
                } else {
                    self.database.clone()
                },
            ),
        ]
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SQLiteOptions {
    pub readonly: bool,
    pub filepath: String,
    pub password: Option<String>,
}

impl Default for SQLiteOptions {
    fn default() -> Self {
        Self {
            readonly: false,
            filepath: String::new(),
            password: None,
        }
    }
}

impl SQLiteOptions {
    pub fn endpoint(&self) -> String {
        let path = self.filepath.trim();
        if path.is_empty() {
            return "sqlite://<未配置文件>".into();
        }

        let name = Path::new(path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(path);

        if self.readonly {
            format!("sqlite://{}?mode=ro", name)
        } else {
            format!("sqlite://{}", name)
        }
    }

    pub fn overview(&self) -> Vec<(&'static str, String)> {
        vec![
            (
                "文件路径",
                if self.filepath.is_empty() {
                    "未配置".into()
                } else {
                    self.filepath.clone()
                },
            ),
            (
                "访问模式",
                if self.readonly {
                    "只读".into()
                } else {
                    "读写".into()
                },
            ),
        ]
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PostgresOptions {
    pub host: String,
    pub port: String,
    pub database: String,
    pub username: String,
    pub password: String,
    pub use_tls: bool,
}

impl Default for PostgresOptions {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".into(),
            port: "5432".into(),
            database: String::new(),
            username: "postgres".into(),
            password: "".into(),
            use_tls: false,
        }
    }
}

impl PostgresOptions {
    pub fn endpoint(&self) -> String {
        let db = self.database.trim();
        let suffix = if db.is_empty() {
            String::new()
        } else {
            format!("/{}", db)
        };
        format!("postgres://{}:{}{}", self.host, self.port, suffix)
    }

    pub fn overview(&self) -> Vec<(&'static str, String)> {
        vec![
            ("连接地址", format!("{}:{}", self.host, self.port)),
            (
                "数据库",
                if self.database.is_empty() {
                    "未配置".into()
                } else {
                    self.database.clone()
                },
            ),
            (
                "安全性",
                if self.use_tls {
                    "TLS 已启用".into()
                } else {
                    "未启用 TLS".into()
                },
            ),
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OracleAddress {
    ServiceName(String),
    Sid(String),
}

impl Default for OracleAddress {
    fn default() -> Self {
        OracleAddress::ServiceName("xe".into())
    }
}

impl OracleAddress {
    pub fn value(&self) -> &str {
        match self {
            OracleAddress::ServiceName(value) => value,
            OracleAddress::Sid(value) => value,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct OracleOptions {
    pub host: String,
    pub port: u16,
    pub address: OracleAddress,
    pub username: String,
    pub password: Option<String>,
    pub wallet_path: Option<String>,
}

impl Default for OracleOptions {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".into(),
            port: 1521,
            address: OracleAddress::default(),
            username: "system".into(),
            password: None,
            wallet_path: None,
        }
    }
}

impl OracleOptions {
    pub fn endpoint(&self) -> String {
        let hint = match &self.address {
            OracleAddress::ServiceName(value) => format!("svc={}", value),
            OracleAddress::Sid(value) => format!("sid={}", value),
        };
        format!("oracle://{}:{}?{}", self.host, self.port, hint)
    }

    pub fn overview(&self) -> Vec<(&'static str, String)> {
        let mut fields = vec![("连接地址", format!("{}:{}", self.host, self.port))];

        match &self.address {
            OracleAddress::ServiceName(name) => fields.push(("服务名", name.clone())),
            OracleAddress::Sid(sid) => fields.push(("SID", sid.clone())),
        }

        if let Some(wallet) = &self.wallet_path {
            if !wallet.is_empty() {
                fields.push(("Wallet 路径", wallet.clone()));
            }
        }

        fields
    }
}

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

impl SQLServerOptions {
    pub fn endpoint(&self) -> String {
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

    pub fn overview(&self) -> Vec<(&'static str, String)> {
        let mut fields = vec![("连接地址", format!("{}:{}", self.host, self.port))];

        if let Some(instance) = &self.instance {
            if !instance.is_empty() {
                fields.push(("实例名", instance.clone()));
            }
        }

        fields.push((
            "数据库",
            if self.database.is_empty() {
                "未配置".into()
            } else {
                self.database.clone()
            },
        ));
        fields
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RedisKind {
    Cluster,
    Standalone,
}

impl Default for RedisKind {
    fn default() -> Self {
        RedisKind::Standalone
    }
}

impl RedisKind {
    pub fn all() -> &'static [RedisKind] {
        &[RedisKind::Cluster, RedisKind::Standalone]
    }

    pub fn label(&self) -> &'static str {
        match self {
            RedisKind::Cluster => "集群",
            RedisKind::Standalone => "单机",
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RedisOptions {
    pub kind: RedisKind,
    pub host: String,
    pub port: String,
    pub nodes: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub use_tls: bool,
}

impl Default for RedisOptions {
    fn default() -> Self {
        Self {
            kind: RedisKind::Standalone,
            host: "127.0.0.1".into(),
            port: "6379".into(),
            nodes: String::new(),
            username: None,
            password: None,
            use_tls: false,
        }
    }
}

impl RedisOptions {
    pub fn endpoint(&self) -> String {
        let scheme = if self.use_tls { "rediss" } else { "redis" };
        match self.kind {
            RedisKind::Standalone => {
                format!("{}://{}:{}", scheme, self.host, self.port)
            }
            RedisKind::Cluster => {
                if self.nodes.is_empty() {
                    format!("{}://<未配置集群节点>", scheme)
                } else {
                    format!("{}://{}", scheme, self.nodes)
                }
            }
        }
    }

    pub fn overview(&self) -> Vec<(&'static str, String)> {
        let mut fields = vec![("连接类型", self.kind.label().into())];

        match self.kind {
            RedisKind::Standalone => {
                fields.push(("连接地址", format!("{}:{}", self.host, self.port)));
            }
            RedisKind::Cluster => {
                if self.nodes.is_empty() {
                    fields.push(("集群节点", "未配置".into()));
                } else {
                    fields.push(("集群节点", self.nodes.replace(',', ", ")));
                }
            }
        }

        fields.push((
            "安全性",
            if self.use_tls {
                "TLS 已启用".into()
            } else {
                "未启用 TLS".into()
            },
        ));

        fields
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MongoDBHost {
    pub host: String,
    pub port: u16,
}

impl Default for MongoDBHost {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".into(),
            port: 27017,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MongoDBOptions {
    pub connection_string: Option<String>,
    pub hosts: Vec<MongoDBHost>,
    pub replica_set: Option<String>,
    pub auth_source: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub use_tls: bool,
}

impl Default for MongoDBOptions {
    fn default() -> Self {
        Self {
            connection_string: None,
            hosts: vec![MongoDBHost::default()],
            replica_set: None,
            auth_source: None,
            username: None,
            password: None,
            use_tls: false,
        }
    }
}

impl MongoDBOptions {
    pub fn endpoint(&self) -> String {
        if let Some(uri) = &self.connection_string {
            return uri.into();
        }

        if self.hosts.is_empty() {
            return "mongodb://<未配置主机>".into();
        }

        let hosts = self
            .hosts
            .iter()
            .map(|MongoDBHost { host, port }| format!("{}:{}", host, port))
            .collect::<Vec<_>>()
            .join(",");

        let mut suffix = String::new();
        if let Some(auth) = &self.auth_source {
            let trimmed = auth.trim();
            if !trimmed.is_empty() {
                suffix = format!("?db={}", trimmed);
            }
        }

        format!("mongodb://{}{}", hosts, suffix)
    }

    pub fn overview(&self) -> Vec<(&'static str, String)> {
        let mut fields = vec![];

        if let Some(uri) = &self.connection_string {
            fields.push(("连接字符串", uri.into()));
        } else if !self.hosts.is_empty() {
            let hosts = self
                .hosts
                .iter()
                .map(|h| format!("{}:{}", h.host, h.port))
                .collect::<Vec<_>>()
                .join(", ");
            fields.push(("主机列表", hosts));
        } else {
            fields.push(("主机列表", "未配置".into()));
        }

        if let Some(db) = &self.auth_source {
            if !db.is_empty() {
                fields.push(("认证数据库", db.clone()));
            }
        }

        fields.push((
            "安全性",
            if self.use_tls {
                "TLS 已启用".into()
            } else {
                "未启用 TLS".into()
            },
        ));
        fields
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum DataSourceOptions {
    MySQL(MySQLOptions),
    SQLite(SQLiteOptions),
    Postgres(PostgresOptions),
    Oracle(OracleOptions),
    SQLServer(SQLServerOptions),
    Redis(RedisOptions),
    MongoDB(MongoDBOptions),
}
