use serde::{Deserialize, Serialize};

use crate::option::{ConnectionOptions, DataSourceKind};

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

impl ConnectionOptions for MongoDBOptions {
    fn kind(&self) -> DataSourceKind {
        DataSourceKind::MongoDB
    }
}

impl MongoDBOptions {
    pub fn display_endpoint(&self) -> String {
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
