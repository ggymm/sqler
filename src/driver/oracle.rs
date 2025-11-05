use serde::{Deserialize, Serialize};

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
