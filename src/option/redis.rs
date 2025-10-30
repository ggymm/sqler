use serde::{Deserialize, Serialize};

use crate::option::{ConnectionOptions, DataSourceKind};

#[derive(Clone, Serialize, Deserialize)]
pub struct RedisOptions {
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
    pub db: u8,
    pub use_tls: bool,
}

impl Default for RedisOptions {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".into(),
            port: 6379,
            username: None,
            password: None,
            db: 0,
            use_tls: false,
        }
    }
}

impl ConnectionOptions for RedisOptions {
    fn kind(&self) -> DataSourceKind {
        DataSourceKind::Redis
    }
}

impl RedisOptions {
    pub fn display_endpoint(&self) -> String {
        let scheme = if self.use_tls { "rediss" } else { "redis" };
        format!("{}://{}:{}/{}", scheme, self.host, self.port, self.db)
    }
}
