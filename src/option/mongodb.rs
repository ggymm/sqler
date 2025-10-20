use crate::option::ConnectionOptions;
use crate::option::DataSourceKind;

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MongoDBOptions {
    pub connection_string: Option<String>,
    pub hosts: Vec<MongoDBHost>,
    pub replica_set: Option<String>,
    pub auth_source: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub tls: bool,
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
            tls: false,
        }
    }
}

impl ConnectionOptions for MongoDBOptions {
    fn kind(&self) -> DataSourceKind {
        DataSourceKind::MongoDB
    }
}
