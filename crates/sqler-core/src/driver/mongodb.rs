use crate::{ColumnInfo, ColumnKind, MongoDBOptions, TableInfo};

use super::{DatabaseDriver, DatabaseSession, DriverError, ExecReq, ExecResp, QueryReq, QueryResp};

/// MongoDB 驱动 - 暂未实现
#[derive(Debug, Clone, Copy)]
pub struct MongoDBDriver;

struct MongoDBConnection;

impl DatabaseSession for MongoDBConnection {
    fn exec(
        &mut self,
        _req: ExecReq,
    ) -> Result<ExecResp, DriverError> {
        Err(DriverError::Other("MongoDB 驱动暂未实现".into()))
    }

    fn query(
        &mut self,
        _req: QueryReq,
    ) -> Result<QueryResp, DriverError> {
        Err(DriverError::Other("MongoDB 驱动暂未实现".into()))
    }

    fn tables(&mut self) -> Result<Vec<TableInfo>, DriverError> {
        Err(DriverError::Other("MongoDB 驱动暂未实现".into()))
    }

    fn columns(
        &mut self,
        _table: &str,
    ) -> Result<Vec<ColumnInfo>, DriverError> {
        Err(DriverError::Other("MongoDB 驱动暂未实现".into()))
    }
}

impl DatabaseDriver for MongoDBDriver {
    type Config = MongoDBOptions;

    fn supp_kinds(&self) -> Vec<ColumnKind> {
        vec![]
    }

    fn check_connection(
        &self,
        _config: &Self::Config,
    ) -> Result<(), DriverError> {
        Err(DriverError::Other("MongoDB 驱动暂未实现".into()))
    }

    fn create_connection(
        &self,
        _config: &Self::Config,
    ) -> Result<Box<dyn DatabaseSession>, DriverError> {
        Err(DriverError::Other("MongoDB 驱动暂未实现".into()))
    }
}
