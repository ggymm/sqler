use super::{DatabaseDriver, DriverError};
use crate::option::{MongoDBHost, MongoDBOptions};

use mongodb::bson::doc;
use mongodb::sync::Client;

#[derive(Debug, Clone, Copy)]
pub struct MongoDBDriver;

impl DatabaseDriver for MongoDBDriver {
    type Config = MongoDBOptions;

    fn check_connection(
        &self,
        config: &Self::Config,
    ) -> Result<(), DriverError> {
        let uri = build_uri(config)?;

        let client = Client::with_uri_str(&uri)
            .map_err(|err| DriverError::Other(format!("连接失败: {}", err)))?;

        let database_name = config
            .auth_source
            .as_deref()
            .filter(|name| !name.is_empty())
            .unwrap_or("admin");

        client
            .database(database_name)
            .run_command(doc! { "ping": 1 })
            .run()
            .map_err(|err| DriverError::Other(format!("ping 失败: {}", err)))?;

        Ok(())
    }
}

fn build_uri(config: &MongoDBOptions) -> Result<String, DriverError> {
    if let Some(uri) = &config.connection_string {
        let trimmed = uri.trim();
        if trimmed.is_empty() {
            return Err(DriverError::InvalidField("connection_string".into()));
        }
        return Ok(trimmed.to_string());
    }

    if config.hosts.is_empty() {
        return Err(DriverError::MissingField("hosts".into()));
    }

    let mut uri = String::from("mongodb://");

    if let Some(username) = &config.username {
        let username = username.trim();
        if username.is_empty() {
            return Err(DriverError::InvalidField("username".into()));
        }
        uri.push_str(username);
        if let Some(password) = &config.password {
            uri.push(':');
            uri.push_str(password);
        }
        uri.push('@');
    } else if config.password.is_some() {
        return Err(DriverError::InvalidField("username".into()));
    }

    let hosts = config
        .hosts
        .iter()
        .map(|MongoDBHost { host, port }| {
            let host = host.trim();
            if host.is_empty() {
                Err(DriverError::InvalidField("host".into()))
            } else {
                Ok(format!("{}:{}", host, port))
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    uri.push_str(&hosts.join(","));

    let mut params = Vec::new();
    if let Some(auth) = &config.auth_source {
        if !auth.is_empty() {
            params.push(format!("authSource={}", auth));
        }
    }
    if let Some(rs) = &config.replica_set {
        if !rs.is_empty() {
            params.push(format!("replicaSet={}", rs));
        }
    }
    if config.tls {
        params.push("tls=true".to_string());
    }

    if !params.is_empty() {
        uri.push('/');
        uri.push('?');
        uri.push_str(&params.join("&"));
    }

    Ok(uri)
}
