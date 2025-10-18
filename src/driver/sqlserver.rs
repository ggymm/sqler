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

        // TODO: 在此处接入实际 SQL Server 连接逻辑。
        Ok(())
    }
}
