use crate::option::ConnectionOptions;
use crate::DataSourceType;

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
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

impl ConnectionOptions for OracleOptions {
    fn kind(&self) -> DataSourceType {
        DataSourceType::Oracle
    }
}
