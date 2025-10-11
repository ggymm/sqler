use iced::Task;
use mongodb::sync::Client;
use mongodb::bson;

use super::{ConnectionParams, DriverError, QueryRequest, QueryResponse, unsupported};

#[derive(Clone, Debug)]
pub struct MongoDriver;

impl MongoDriver {
    pub fn new() -> Self {
        Self
    }

    pub fn test_connection(
        &self,
        params: ConnectionParams,
    ) -> Task<Result<(), DriverError>> {
        Task::future(async move {
            let client = Self::client(&params)?;
            client
                .database("admin")
                .run_command(bson::doc! { "ping": 1 })
                .run()
                .map_err(|err| DriverError::Connection(err.to_string()))?;
            Ok(())
        })
    }

    pub fn query(
        &self,
        params: ConnectionParams,
        request: QueryRequest,
    ) -> Task<Result<QueryResponse, DriverError>> {
        let _ = params;
        let _ = request;
        Task::done(Err(unsupported("MongoDB 查询功能暂未实现")))
    }

    fn client(params: &ConnectionParams) -> Result<Client, DriverError> {
        let uri = params.require_connection_string()?.to_string();
        Client::with_uri_str(&uri).map_err(|err| DriverError::Connection(err.to_string()))
    }
}
