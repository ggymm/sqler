use iced::Task;
use rusqlite::{Connection, types::ValueRef};
use serde_json::Value as JsonValue;

use super::{
    ConnectionParams, DriverError, QueryRequest, QueryResponse, make_tabular_response, map_binary_or_text,
    unsupported, value_to_json_f64,
};

#[derive(Clone, Copy, Debug)]
pub struct SqliteDriver;

impl SqliteDriver {
    pub fn new() -> Self {
        Self
    }

    pub fn test_connection(
        &self,
        params: ConnectionParams,
    ) -> Task<Result<(), DriverError>> {
        Task::future(async move {
            let path = params.require_file_path()?.to_string();
            let conn = Connection::open(&path).map_err(|err| DriverError::Connection(err.to_string()))?;
            conn.execute_batch("SELECT 1;")
                .map_err(|err| DriverError::Connection(err.to_string()))
        })
    }

    pub fn query(
        &self,
        params: ConnectionParams,
        request: QueryRequest,
    ) -> Task<Result<QueryResponse, DriverError>> {
        Task::future(async move {
            let statement = match request {
                QueryRequest::Sql { statement } => statement,
                QueryRequest::RedisDatabases => return Err(unsupported("SQLite 不支持该查询类型")),
            };
            let path = params.require_file_path()?.to_string();
            let conn = Connection::open(&path).map_err(|err| DriverError::Connection(err.to_string()))?;
            let mut stmt = conn
                .prepare(&statement)
                .map_err(|err| DriverError::Query(err.to_string()))?;

            let column_names = stmt
                .column_names()
                .iter()
                .map(|name| name.to_string())
                .collect::<Vec<_>>();

            let mut rows = Vec::new();
            let mut query = stmt.query([]).map_err(|err| DriverError::Query(err.to_string()))?;

            while let Some(row) = query.next().map_err(|err| DriverError::Query(err.to_string()))? {
                let mut values = Vec::with_capacity(column_names.len());
                for idx in 0..column_names.len() {
                    let value = row.get_ref(idx).map_err(|err| DriverError::Query(err.to_string()))?;
                    values.push(sqlite_value_to_json(value));
                }
                rows.push(values);
            }

            Ok(make_tabular_response(column_names, rows))
        })
    }

}

fn sqlite_value_to_json(value: ValueRef<'_>) -> JsonValue {
    match value {
        ValueRef::Null => JsonValue::Null,
        ValueRef::Integer(v) => JsonValue::from(v),
        ValueRef::Real(v) => value_to_json_f64(v),
        ValueRef::Text(bytes) => JsonValue::String(std::str::from_utf8(bytes).unwrap_or_default().to_string()),
        ValueRef::Blob(bytes) => map_binary_or_text(bytes.to_vec()),
    }
}
