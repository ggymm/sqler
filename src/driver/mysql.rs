use iced::Task;
use mysql::prelude::Queryable;
use mysql::{Conn, OptsBuilder, Value as MysqlValue};
use serde_json::Value as JsonValue;

use super::{
    execution_response, make_tabular_response, map_binary_or_text, unsupported, ConnectionParams,
    DriverError, ExecuteRequest, ExecuteResponse, QueryRequest, QueryResponse,
};

#[derive(Clone, Copy, Debug)]
pub struct MysqlDriver;

impl MysqlDriver {
    pub fn new() -> Self {
        Self
    }

    pub fn test_connection(
        &self,
        params: ConnectionParams,
    ) -> Task<Result<(), DriverError>> {
        Task::future(async move {
            let mut conn = Self::connect(&params)?;
            conn.ping().map_err(|err| DriverError::Connection(err.to_string()))
        })
    }

    pub fn query(
        &self,
        params: ConnectionParams,
        request: QueryRequest,
    ) -> Task<Result<QueryResponse, DriverError>> {
        Task::future(async move {
            match request {
                QueryRequest::Sql { statement } => {
                    let mut conn = Self::connect(&params)?;
                    let result = conn
                        .query_iter(statement.clone())
                        .map_err(|err| DriverError::Query(err.to_string()))?;

                    let columns = result
                        .columns()
                        .as_ref()
                        .iter()
                        .map(|c| c.name_str().to_string())
                        .collect::<Vec<_>>();

                    let mut rows = Vec::new();
                    for row in result {
                        let row = row.map_err(|err| DriverError::Query(err.to_string()))?;
                        let values = row
                            .unwrap()
                            .into_iter()
                            .map(mysql_value_to_json)
                            .collect();
                        rows.push(values);
                    }

                    Ok(make_tabular_response(columns, rows))
                }
                QueryRequest::KeyValue(_) => Err(unsupported("MySQL 仅支持 SQL 查询")),
            }
        })
    }

    pub fn execute(
        &self,
        params: ConnectionParams,
        request: ExecuteRequest,
    ) -> Task<Result<ExecuteResponse, DriverError>> {
        Task::future(async move {
            match request {
                ExecuteRequest::Sql { statement } => {
                    let mut conn = Self::connect(&params)?;
                    conn.query_drop(statement.clone())
                        .map_err(|err| DriverError::Execution(err.to_string()))?;
                    let affected = conn.affected_rows();
                    let last_id = conn.last_insert_id();
                    let generated = if last_id == 0 {
                        None
                    } else {
                        Some(JsonValue::from(last_id as i64))
                    };

                    Ok(execution_response(affected, generated))
                }
                ExecuteRequest::KeyValue(_) => Err(unsupported("MySQL 仅支持 SQL 执行")),
            }
        })
    }

    fn connect(params: &ConnectionParams) -> Result<Conn, DriverError> {
        let host = params.require_host()?.to_string();
        let port = params.require_port()?;
        let user = params.require_username()?.to_string();
        let password = params.password.clone().unwrap_or_default();
        let database = params.database.clone();

        let mut builder = OptsBuilder::new()
            .ip_or_hostname(Some(host))
            .tcp_port(port)
            .user(Some(user))
            .pass(if password.is_empty() { None } else { Some(password) });

        if let Some(db) = database {
            if !db.is_empty() {
                builder = builder.db_name(Some(db));
            }
        }

        let opts = mysql::Opts::from(builder);
        Conn::new(opts).map_err(|err| DriverError::Connection(err.to_string()))
    }
}

fn mysql_value_to_json(value: MysqlValue) -> JsonValue {
    match value {
        MysqlValue::NULL => JsonValue::Null,
        MysqlValue::Bytes(bytes) => map_binary_or_text(bytes),
        MysqlValue::Int(v) => JsonValue::from(v),
        MysqlValue::UInt(v) => JsonValue::from(v),
        MysqlValue::Float(v) => JsonValue::from(v),
        MysqlValue::Double(v) => JsonValue::from(v),
        MysqlValue::Date(year, month, day, hour, minute, second, micros) => JsonValue::String(
            format!(
                "{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02}.{:06}",
                micros
            )
            .trim_end_matches(".000000")
            .to_string(),
        ),
        MysqlValue::Time(is_neg, days, hours, minutes, seconds, micros) => {
            let sign = if is_neg { "-" } else { "" };
            JsonValue::String(
                format!(
                    "{sign}{total_hours:02}:{minutes:02}:{seconds:02}.{:06}",
                    micros,
                    total_hours = days * 24 + u32::from(hours)
                )
                .trim_end_matches(".000000")
                .to_string(),
            )
        }
    }
}
