use super::{DatabaseDriver, DriverError};
use crate::option::RedisOptions;

use redis::Client;

#[derive(Debug, Clone, Copy)]
pub struct RedisDriver;

impl DatabaseDriver for RedisDriver {
    type Config = RedisOptions;

    fn check_connection(
        &self,
        config: &Self::Config,
    ) -> Result<(), DriverError> {
        if config.host.trim().is_empty() {
            return Err(DriverError::MissingField("host".into()));
        }
        if config.port == 0 {
            return Err(DriverError::InvalidField("port".into()));
        }

        let url = build_connection_url(config)?;
        let client = Client::open(url)
            .map_err(|err| DriverError::Other(format!("创建客户端失败: {}", err)))?;

        let mut conn = client
            .get_connection()
            .map_err(|err| DriverError::Other(format!("建立连接失败: {}", err)))?;

        redis::cmd("SELECT")
            .arg(config.db as i64)
            .query::<()>(&mut conn)
            .map_err(|err| DriverError::Other(format!("选择数据库失败: {}", err)))?;

        redis::cmd("PING")
            .query::<String>(&mut conn)
            .map_err(|err| DriverError::Other(format!("PING 失败: {}", err)))?;

        Ok(())
    }
}

fn build_connection_url(config: &RedisOptions) -> Result<String, DriverError> {
    let scheme = if config.use_tls { "rediss://" } else { "redis://" };
    let mut url = String::from(scheme);

    if let Some(username) = &config.username {
        let username = username.trim();
        if username.is_empty() {
            return Err(DriverError::InvalidField("username".into()));
        }
        url.push_str(username);
        if let Some(password) = &config.password {
            url.push(':');
            url.push_str(password);
        }
        url.push('@');
    } else if let Some(password) = &config.password {
        url.push(':');
        url.push_str(password);
        url.push('@');
    }

    url.push_str(config.host.trim());
    url.push(':');
    url.push_str(&config.port.to_string());
    url.push('/');
    url.push_str(&config.db.to_string());

    Ok(url)
}
