use super::{
    DatabaseDriver, DatabaseSession, DeleteReq, DriverError, InsertReq, QueryReq, QueryResp, UpdateReq, WriteResp,
};
use crate::option::SQLServerOptions;

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
    ) -> Result<WriteResp, DriverError> {
        Err(DriverError::Other("SQL Server 插入暂未实现".into()))
    }

    fn update(
        &mut self,
        _request: UpdateReq,
    ) -> Result<WriteResp, DriverError> {
        Err(DriverError::Other("SQL Server 更新暂未实现".into()))
    }

    fn delete(
        &mut self,
        _request: DeleteReq,
    ) -> Result<WriteResp, DriverError> {
        Err(DriverError::Other("SQL Server 删除暂未实现".into()))
    }
}

impl DatabaseDriver for SQLServerDriver {
    type Config = SQLServerOptions;

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
