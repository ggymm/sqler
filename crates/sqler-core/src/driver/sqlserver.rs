use crate::{ColumnInfo, ColumnKind, SQLServerOptions, TableInfo};

use super::{DatabaseDriver, DatabaseSession, DriverError, ExecReq, ExecResp, QueryReq, QueryResp};

/// SQL Server 驱动 - 暂未实现
#[derive(Debug, Clone, Copy)]
pub struct SQLServerDriver;

struct SQLServerConnection;

impl DatabaseSession for SQLServerConnection {
    fn exec(
        &mut self,
        _req: ExecReq,
    ) -> Result<ExecResp, DriverError> {
        Err(DriverError::Other("SQL Server 驱动暂未实现".into()))
    }

    fn query(
        &mut self,
        _req: QueryReq,
    ) -> Result<QueryResp, DriverError> {
        Err(DriverError::Other("SQL Server 驱动暂未实现".into()))
    }

    fn tables(&mut self) -> Result<Vec<TableInfo>, DriverError> {
        Err(DriverError::Other("SQL Server 驱动暂未实现".into()))
    }

    fn columns(
        &mut self,
        _table: &str,
    ) -> Result<Vec<ColumnInfo>, DriverError> {
        Err(DriverError::Other("SQL Server 驱动暂未实现".into()))
    }
}

impl DatabaseDriver for SQLServerDriver {
    type Config = SQLServerOptions;

    fn supp_kinds(&self) -> Vec<ColumnKind> {
        vec![]
    }

    fn check_connection(
        &self,
        _config: &Self::Config,
    ) -> Result<(), DriverError> {
        Err(DriverError::Other("SQL Server 驱动暂未实现".into()))
    }

    fn create_connection(
        &self,
        _config: &Self::Config,
    ) -> Result<Box<dyn DatabaseSession>, DriverError> {
        Err(DriverError::Other("SQL Server 驱动暂未实现".into()))
    }
}
