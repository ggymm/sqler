use std::time::Duration;

use native_tls::TlsConnector;
use tokio::{net::TcpStream, runtime::Builder, time::timeout};
use tokio_native_tls::TlsConnector as TokioTlsConnector;
use tokio_util::compat::TokioAsyncWriteCompatExt;

use tiberius::{AuthMethod, Client, Config as TdsConfig, EncryptionLevel};

use super::{DatabaseDriver, DriverError};

/// SQL Server 连接认证方式。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SqlServerAuth {
    SqlPassword,
    Integrated,
}

/// SQL Server 连接配置。
#[derive(Debug, Clone)]
pub struct SqlServerConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub auth: SqlServerAuth,
    pub instance: Option<String>,
}

/// SQL Server 驱动占位实现。
#[derive(Debug, Clone, Copy)]
pub struct SqlServerDriver;

impl DatabaseDriver for SqlServerDriver {
    type Config = SqlServerConfig;

    fn test_connection(&self, config: &Self::Config) -> Result<(), DriverError> {
        if config.host.trim().is_empty() {
            return Err(DriverError::MissingField("host".into()));
        }
        if config.port == 0 {
            return Err(DriverError::InvalidField("port 必须大于 0".into()));
        }
        if config.database.trim().is_empty() {
            return Err(DriverError::MissingField("database".into()));
        }

        match config.auth {
            SqlServerAuth::SqlPassword => {
                if config.username.as_deref().unwrap_or("").trim().is_empty() {
                    return Err(DriverError::MissingField("username".into()));
                }
            }
            SqlServerAuth::Integrated => {
                // Windows 集成认证无需额外判断。
            }
        }

        let timeout_duration = Duration::from_secs(5);

        let runtime = Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|err| DriverError::Other(err.to_string()))?;

        runtime
            .block_on(async {
                let mut config_tds = TdsConfig::new();
                config_tds.host(&config.host);
                config_tds.port(config.port);
                config_tds.database(&config.database);
                config_tds.encryption(EncryptionLevel::Required);
                config_tds.trust_cert();

                if let Some(instance) = &config.instance {
                    config_tds.instance_name(instance);
                }

                match config.auth {
                    SqlServerAuth::SqlPassword => {
                        let username = config
                            .username
                            .clone()
                            .ok_or_else(|| DriverError::MissingField("username".into()))?;
                        let password = config.password.clone().unwrap_or_default();
                        config_tds.authentication(AuthMethod::sql_server(username, password));
                    }
                    SqlServerAuth::Integrated => {
                        #[cfg(windows)]
                        {
                            config_tds.authentication(AuthMethod::Integrated);
                        }
                        #[cfg(not(windows))]
                        {
                            return Err(DriverError::Other(
                                "集成认证仅在 Windows 平台上受支持".into(),
                            ));
                        }
                    }
                }

                let addr = (config.host.as_str(), config.port);
                let stream = timeout(timeout_duration, TcpStream::connect(addr))
                    .await
                    .map_err(|_| DriverError::Other("连接 SQL Server 超时".into()))??;
                stream.set_nodelay(true).map_err(|err| DriverError::Other(err.to_string()))?;

                let tls = TlsConnector::builder()
                    .danger_accept_invalid_certs(true)
                    .build()
                    .map_err(|err| DriverError::Other(err.to_string()))?;
                let tls = TokioTlsConnector::from(tls);

                let mut client = Client::connect(config_tds, stream.compat_write(), tls)
                    .await
                    .map_err(|err| DriverError::Other(err.to_string()))?;
                client
                    .close()
                    .await
                    .map_err(|err| DriverError::Other(err.to_string()))?;

                Ok::<(), DriverError>(())
            })
    }
}
