use iced::Task;
use redis::{Client, ConnectionAddr, ConnectionInfo, RedisConnectionInfo};
use serde_json::Value as JsonValue;

use super::{ConnectionParams, DriverError, QueryRequest, QueryResponse, make_tabular_response, unsupported};

#[derive(Clone, Debug)]
pub struct RedisDriver;

impl RedisDriver {
    pub fn new() -> Self {
        Self
    }

    pub fn test_connection(
        &self,
        params: ConnectionParams,
    ) -> Task<Result<(), DriverError>> {
        Task::future(async move {
            let mut connection = Self::open_connection(&params)?;
            redis::cmd("PING")
                .query::<String>(&mut connection)
                .map_err(|err| DriverError::Connection(err.to_string()))?;
            Ok(())
        })
    }

    pub fn query(
        &self,
        params: ConnectionParams,
        request: QueryRequest,
    ) -> Task<Result<QueryResponse, DriverError>> {
        Task::future(async move {
            match request {
                QueryRequest::Sql { .. } => Err(unsupported("Redis 不支持 SQL 查询")),
                QueryRequest::RedisDatabases => Self::load_databases(&params),
            }
        })
    }

    fn load_databases(params: &ConnectionParams) -> Result<QueryResponse, DriverError> {
        let mut connection = Self::open_connection(params)?;

        let info = redis::cmd("INFO")
            .arg("KEYSPACE")
            .query::<String>(&mut connection)
            .map_err(|err| DriverError::Query(err.to_string()))?;

        let mut rows = Vec::new();
        for line in info.lines() {
            let trimmed = line.trim();
            if !trimmed.starts_with("db") {
                continue;
            }

            let mut parts = trimmed.splitn(2, ':');
            let name = parts.next().unwrap_or_default().to_string();
            let stats = parts.next().unwrap_or_default();

            let mut keys = JsonValue::from(0);
            let mut expires = JsonValue::from(0);
            let mut avg_ttl = JsonValue::String(String::new());

            for kv in stats.split(',') {
                let mut pair = kv.splitn(2, '=');
                let key = pair.next().unwrap_or_default();
                let value = pair.next().unwrap_or_default();

                match key {
                    "keys" => {
                        if let Ok(num) = value.parse::<u64>() {
                            keys = JsonValue::from(num);
                        }
                    }
                    "expires" => {
                        if let Ok(num) = value.parse::<u64>() {
                            expires = JsonValue::from(num);
                        }
                    }
                    "avg_ttl" => {
                        if value == "0" {
                            avg_ttl = JsonValue::String(String::new());
                        } else if let Ok(num) = value.parse::<u64>() {
                            avg_ttl = JsonValue::from(num);
                        }
                    }
                    _ => {}
                }
            }

            rows.push(vec![
                JsonValue::String(name),
                keys,
                expires,
                avg_ttl.clone(),
            ]);
        }

        let columns = vec![
            "database".to_string(),
            "keys".to_string(),
            "expires".to_string(),
            "avg_ttl".to_string(),
        ];

        Ok(make_tabular_response(columns, rows))
    }

    fn open_connection(params: &ConnectionParams) -> Result<redis::Connection, DriverError> {
        let host = params.require_host()?.to_string();
        let port = params.require_port()?;

        let addr = ConnectionAddr::Tcp(host, port);
        let password = params.password.clone().and_then(|p| if p.is_empty() { None } else { Some(p) });

        let info = ConnectionInfo {
            addr,
            redis: RedisConnectionInfo {
                db: 0,
                username: None,
                password,
                protocol: Default::default(),
            },
        };

        Client::open(info)
            .and_then(|client| client.get_connection())
            .map_err(|err| DriverError::Connection(err.to_string()))
    }
}
