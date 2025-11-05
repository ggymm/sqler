use serde::{Deserialize, Serialize};

use super::{
    DatabaseDriver, DatabaseSession, Datatype, DeleteReq, DriverError, InsertReq, QueryReq, QueryResp, UpdateReq,
    UpdateResp,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SQLServerAuth {
    SqlPassword,
    Integrated,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SQLServerOptions {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub auth: SQLServerAuth,
    pub instance: Option<String>,
}

impl Default for SQLServerOptions {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".into(),
            port: 1433,
            database: String::new(),
            username: None,
            password: None,
            auth: SQLServerAuth::SqlPassword,
            instance: None,
        }
    }
}

impl SQLServerOptions {
    pub fn endpoint(&self) -> String {
        let mut authority = format!("{}:{}", self.host, self.port);
        if let Some(instance) = &self.instance {
            let trimmed = instance.trim();
            if !trimmed.is_empty() {
                authority = format!("{}\\{}", authority, trimmed);
            }
        }

        let db = self.database.trim();
        if db.is_empty() {
            format!("sqlserver://{}", authority)
        } else {
            format!("sqlserver://{}/{}", authority, db)
        }
    }

    pub fn overview(&self) -> Vec<(&'static str, String)> {
        let mut fields = vec![("连接地址", format!("{}:{}", self.host, self.port))];

        if let Some(instance) = &self.instance {
            if !instance.is_empty() {
                fields.push(("实例名", instance.clone()));
            }
        }

        fields.push((
            "数据库",
            if self.database.is_empty() {
                "未配置".into()
            } else {
                self.database.clone()
            },
        ));
        fields
    }
}

/// SQL Server 驱动占位实现。
#[derive(Debug, Clone, Copy)]
pub struct SQLServerDriver;

struct SqlServerConnection;

impl DatabaseSession for SqlServerConnection {
    fn query(
        &mut self,
        _request: QueryReq,
    ) -> Result<QueryResp, DriverError> {
        Err(DriverError::Other("SQL Server 查询暂未实现".into()))
    }

    fn insert(
        &mut self,
        _request: InsertReq,
    ) -> Result<UpdateResp, DriverError> {
        Err(DriverError::Other("SQL Server 插入暂未实现".into()))
    }

    fn update(
        &mut self,
        _request: UpdateReq,
    ) -> Result<UpdateResp, DriverError> {
        Err(DriverError::Other("SQL Server 更新暂未实现".into()))
    }

    fn delete(
        &mut self,
        _request: DeleteReq,
    ) -> Result<UpdateResp, DriverError> {
        Err(DriverError::Other("SQL Server 删除暂未实现".into()))
    }

    fn tables(&mut self) -> Result<Vec<String>, DriverError> {
        Err(DriverError::Other("SQL Server 查询表列表暂未实现".into()))
    }

    fn columns(
        &mut self,
        _table: &str,
    ) -> Result<Vec<String>, DriverError> {
        Err(DriverError::Other("SQL Server 查询列信息暂未实现".into()))
    }
}

impl DatabaseDriver for SQLServerDriver {
    type Config = SQLServerOptions;

    fn data_types(&self) -> Vec<Datatype> {
        vec![
            Datatype::TinyInt,
            Datatype::SmallInt,
            Datatype::Int,
            Datatype::BigInt,
            Datatype::Float,
            Datatype::Double,
            Datatype::Decimal,
            Datatype::Char,
            Datatype::VarChar,
            Datatype::Text,
            Datatype::Binary,
            Datatype::VarBinary,
            Datatype::Date,
            Datatype::Time,
            Datatype::DateTime,
            Datatype::Timestamp,
            Datatype::Boolean,
            Datatype::Uuid,
        ]
    }

    fn check_connection(
        &self,
        config: &Self::Config,
    ) -> Result<(), DriverError> {
        Ok(())
    }

    fn create_connection(
        &self,
        _config: &Self::Config,
    ) -> Result<Box<dyn DatabaseSession>, DriverError> {
        Err(DriverError::Other("SQL Server 驱动暂未实现连接能力".into()))
    }
}
