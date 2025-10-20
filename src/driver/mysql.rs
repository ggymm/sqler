use super::{DatabaseDriver, DriverError};
use crate::option::MySQLOptions;

use mysql::prelude::Queryable;
use mysql::{Conn, Opts, OptsBuilder, SslOpts};

#[derive(Debug, Clone, Copy)]
pub struct MySQLDriver;

impl DatabaseDriver for MySQLDriver {
    type Config = MySQLOptions;

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
        if config.username.trim().is_empty() {
            return Err(DriverError::MissingField("username".into()));
        }

        let mut builder = OptsBuilder::new();
        builder = builder.ip_or_hostname(Some(config.host.clone()));
        builder = builder.tcp_port(config.port);
        builder = builder.user(Some(config.username.clone()));

        if let Some(password) = &config.password {
            builder = builder.pass(Some(password.clone()));
        }

        if !config.database.is_empty() {
            builder = builder.db_name(Some(config.database.clone()));
        }

        if config.use_tls {
            builder = builder.ssl_opts(Some(SslOpts::default()));
        }

        let opts = Opts::from(builder);

        let mut conn = Conn::new(opts).map_err(|err| DriverError::Other(format!("连接失败: {}", err)))?;

        if let Some(charset) = &config.charset {
            if !charset.trim().is_empty() {
                if !charset
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-'))
                {
                    return Err(DriverError::InvalidField("charset".into()));
                }
                conn.query_drop(format!("SET NAMES {}", charset))
                    .map_err(|err| DriverError::Other(format!("设置字符集失败: {}", err)))?;
            }
        }

        conn.ping()
            .map_err(|err| DriverError::Other(format!("ping 失败: {}", err)))?;

        Ok(())
    }
}
