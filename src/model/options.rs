use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct MySQLOptions {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
    pub use_tls: bool,
}

impl Default for MySQLOptions {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".into(),
            port: 3306,
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

impl SQLiteOptions {
    pub fn endpoint(&self) -> String {
        let path = self.filepath.trim();
        if path.is_empty() {
            "sqlite://<未配置文件>".into()
        } else if self.read_only {
            format!("sqlite://{}?mode=ro", path)
        } else {
            format!("sqlite://{}", path)
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
                if self.read_only {
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
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
    pub use_tls: bool,
}

impl Default for PostgresOptions {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".into(),
            port: 5432,
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

#[derive(Clone, Serialize, Deserialize)]
pub struct RedisOptions {
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
    pub use_tls: bool,
}

impl Default for RedisOptions {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".into(),
            port: 6379,
            username: None,
            password: None,
            use_tls: false,
        }
    }
}

impl RedisOptions {
    pub fn endpoint(&self) -> String {
        let scheme = if self.use_tls { "rediss" } else { "redis" };
        format!("{}://{}:{}", scheme, self.host, self.port)
    }

    pub fn overview(&self) -> Vec<(&'static str, String)> {
        vec![
            ("连接地址", format!("{}:{}", self.host, self.port)),
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
            return Self::sanitize_uri(uri);
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
            fields.push(("连接字符串", Self::sanitize_uri(uri)));
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

    fn sanitize_uri(raw: &str) -> String {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return "mongodb://<未配置主机>".into();
        }

        if let Some(scheme_end) = trimmed.find("://") {
            let scheme = &trimmed[..scheme_end];
            let rest = &trimmed[scheme_end + 3..];
            if let Some(at) = rest.find('@') {
                let after = &rest[at + 1..];
                return format!("{}://{}", scheme, after);
            }
        }

        trimmed.to_string()
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
