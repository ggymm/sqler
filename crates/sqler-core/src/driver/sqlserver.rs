use crate::{ColumnInfo, ColumnKind, SQLServerOptions, TableInfo};

use super::{DatabaseDriver, DatabaseSession, DriverError, ExecReq, ExecResp, QueryReq, QueryResp};

/// SQL Server 驱动占位实现。
#[derive(Debug, Clone, Copy)]
pub struct SQLServerDriver;

struct SqlServerConnection;

impl DatabaseSession for SqlServerConnection {
    fn exec(
        &mut self,
        _req: ExecReq,
    ) -> Result<ExecResp, DriverError> {
        Err(DriverError::Other("SQL Server 执行暂未实现".into()))
    }

    fn query(
        &mut self,
        req: QueryReq,
    ) -> Result<QueryResp, DriverError> {
        Err(DriverError::Other("SQL Server 查询暂未实现".into()))
    }

    fn tables(&mut self) -> Result<Vec<TableInfo>, DriverError> {
        Err(DriverError::Other("SQL Server 查询表列表暂未实现".into()))
    }

    fn columns(
        &mut self,
        _table: &str,
    ) -> Result<Vec<ColumnInfo>, DriverError> {
        Err(DriverError::Other("SQL Server 查询列信息暂未实现".into()))
    }
}

impl DatabaseDriver for SQLServerDriver {
    type Config = SQLServerOptions;

    fn supp_kinds(&self) -> Vec<ColumnKind> {
        vec![
            ColumnKind::TinyInt,
            ColumnKind::SmallInt,
            ColumnKind::Int,
            ColumnKind::BigInt,
            ColumnKind::Float,
            ColumnKind::Double,
            ColumnKind::Decimal,
            ColumnKind::Char,
            ColumnKind::VarChar,
            ColumnKind::Text,
            ColumnKind::Binary,
            ColumnKind::VarBinary,
            ColumnKind::Date,
            ColumnKind::Time,
            ColumnKind::DateTime,
            ColumnKind::Timestamp,
            ColumnKind::Boolean,
            ColumnKind::Uuid,
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
