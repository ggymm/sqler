use super::{
    DatabaseDriver, DatabaseSession, DeleteReq, DriverError, InsertReq, QueryReq, QueryResp, UpdateReq, WriteResp,
};
use crate::option::{PostgreSQLOptions, SslMode};

use postgres::types::Type;
use postgres::{Client, Config, Error as PostgresError, NoTls};
use serde_json::{Map, Number, Value};

/// Postgres 驱动实现。
#[derive(Debug, Clone, Copy)]
pub struct PostgreSQLDriver;

struct PostgresConnection {
    client: Client,
}

impl PostgresConnection {
    fn new(client: Client) -> Self {
        Self { client }
    }
}

impl DatabaseSession for PostgresConnection {
    fn query(
        &mut self,
        request: QueryReq,
    ) -> Result<QueryResp, DriverError> {
        match request {
            QueryReq::Sql { statement } => {
                if statement.trim().is_empty() {
                    return Err(DriverError::InvalidField("statement".into()));
                }
                let rows = self
                    .client
                    .query(statement.as_str(), &[])
                    .map_err(|err| DriverError::Other(format!("执行查询失败: {}", err)))?;

                let mut records = Vec::with_capacity(rows.len());
                for row in rows {
                    let mut record = Map::with_capacity(row.len());
                    for (idx, column) in row.columns().iter().enumerate() {
                        let value = postgres_value_to_json(&row, idx)?;
                        record.insert(column.name().to_string(), value);
                    }
                    records.push(record);
                }

                Ok(QueryResp::Rows(records))
            }
            other => Err(DriverError::InvalidField(format!(
                "PostgreSQL 查询仅支持 SQL，收到: {:?}",
                other
            ))),
        }
    }

    fn insert(
        &mut self,
        request: InsertReq,
    ) -> Result<WriteResp, DriverError> {
        match request {
            InsertReq::Sql { statement } => self.exec_write(statement),
            other => Err(DriverError::InvalidField(format!(
                "PostgreSQL 插入仅支持 SQL，收到: {:?}",
                other
            ))),
        }
    }

    fn update(
        &mut self,
        request: UpdateReq,
    ) -> Result<WriteResp, DriverError> {
        match request {
            UpdateReq::Sql { statement } => self.exec_write(statement),
            other => Err(DriverError::InvalidField(format!(
                "PostgreSQL 更新仅支持 SQL，收到: {:?}",
                other
            ))),
        }
    }

    fn delete(
        &mut self,
        request: DeleteReq,
    ) -> Result<WriteResp, DriverError> {
        match request {
            DeleteReq::Sql { statement } => self.exec_write(statement),
            other => Err(DriverError::InvalidField(format!(
                "PostgreSQL 删除仅支持 SQL，收到: {:?}",
                other
            ))),
        }
    }
}

impl PostgresConnection {
    fn exec_write(
        &mut self,
        statement: String,
    ) -> Result<WriteResp, DriverError> {
        if statement.trim().is_empty() {
            return Err(DriverError::InvalidField("statement".into()));
        }
        let affected = self
            .client
            .execute(statement.as_str(), &[])
            .map_err(|err| DriverError::Other(format!("执行写入失败: {}", err)))?;
        Ok(WriteResp { affected })
    }
}

impl DatabaseDriver for PostgreSQLDriver {
    type Config = PostgreSQLOptions;

    fn check_connection(
        &self,
        config: &Self::Config,
    ) -> Result<(), DriverError> {
        let mut client = connect(config)?;
        client
            .simple_query("SELECT 1")
            .map_err(|err| DriverError::Other(format!("校验查询失败: {}", err)))?;
        Ok(())
    }

    fn create_connection(
        &self,
        config: &Self::Config,
    ) -> Result<Box<dyn DatabaseSession>, DriverError> {
        let client = connect(config)?;
        Ok(Box::new(PostgresConnection::new(client)))
    }
}

fn connect(config: &PostgreSQLOptions) -> Result<Client, DriverError> {
    if config.host.trim().is_empty() {
        return Err(DriverError::MissingField("host".into()));
    }
    if config.port == 0 {
        return Err(DriverError::InvalidField("port".into()));
    }
    if config.username.trim().is_empty() {
        return Err(DriverError::MissingField("username".into()));
    }
    if matches!(config.ssl_mode, Some(SslMode::Require | SslMode::Prefer)) {
        return Err(DriverError::Other("PostgreSQL 暂未支持 SSL 模式连接".into()));
    }

    let mut pg_config = Config::new();
    pg_config.host(config.host.trim());
    pg_config.port(config.port);
    pg_config.user(config.username.trim());
    if let Some(password) = &config.password {
        pg_config.password(password.as_str());
    }
    if !config.database.trim().is_empty() {
        pg_config.dbname(config.database.trim());
    }

    let mut client = pg_config
        .connect(NoTls)
        .map_err(|err| DriverError::Other(format!("连接失败: {}", err)))?;

    if let Some(schema) = &config.schema {
        let trimmed = schema.trim();
        if !trimmed.is_empty() {
            client
                .batch_execute(format!("SET search_path TO {}", trimmed).as_str())
                .map_err(|err| DriverError::Other(format!("设置 search_path 失败: {}", err)))?;
        }
    }

    Ok(client)
}

fn postgres_value_to_json(
    row: &postgres::Row,
    idx: usize,
) -> Result<Value, DriverError> {
    let column = row
        .columns()
        .get(idx)
        .ok_or_else(|| DriverError::Other(format!("列索引越界: {}", idx)))?;
    let ty = column.type_();
    let value = match *ty {
        Type::BOOL => Value::Bool(row.try_get::<usize, bool>(idx).map_err(map_pg_err)?),
        Type::INT2 => {
            let value: i16 = row.try_get(idx).map_err(map_pg_err)?;
            Value::Number(Number::from(value as i64))
        }
        Type::INT4 => {
            let value: i32 = row.try_get(idx).map_err(map_pg_err)?;
            Value::Number(Number::from(value as i64))
        }
        Type::INT8 => {
            let value: i64 = row.try_get(idx).map_err(map_pg_err)?;
            Value::Number(Number::from(value))
        }
        Type::FLOAT4 => number_from_f64(row.try_get::<usize, f32>(idx).map_err(map_pg_err)? as f64),
        Type::FLOAT8 => number_from_f64(row.try_get::<usize, f64>(idx).map_err(map_pg_err)?),
        Type::NUMERIC => {
            let text: Option<String> = row.try_get(idx).map_err(map_pg_err)?;
            text.map_or(Value::Null, |s| Value::String(s))
        }
        Type::TEXT | Type::VARCHAR | Type::BPCHAR | Type::NAME => {
            let text: Option<String> = row.try_get(idx).map_err(map_pg_err)?;
            text.map_or(Value::Null, Value::String)
        }
        Type::JSON | Type::JSONB => {
            let text: Option<String> = row.try_get(idx).map_err(map_pg_err)?;
            match text {
                Some(raw) => serde_json::from_str(&raw).unwrap_or(Value::String(raw)),
                None => Value::Null,
            }
        }
        Type::UUID => {
            let text: Option<String> = row.try_get(idx).map_err(map_pg_err)?;
            text.map_or(Value::Null, Value::String)
        }
        Type::TIMESTAMP | Type::TIMESTAMPTZ | Type::DATE | Type::TIME => {
            let text: Option<String> = row.try_get(idx).map_err(map_pg_err)?;
            text.map_or(Value::Null, Value::String)
        }
        _ => {
            let text: Option<String> = row.try_get(idx).map_err(map_pg_err)?;
            text.map_or(Value::Null, Value::String)
        }
    };
    Ok(value)
}

fn number_from_f64(value: f64) -> Value {
    if let Some(num) = Number::from_f64(value) {
        Value::Number(num)
    } else {
        Value::String(value.to_string())
    }
}

fn map_pg_err(err: PostgresError) -> DriverError {
    DriverError::Other(format!("PostgreSQL 解析字段失败: {}", err))
}
