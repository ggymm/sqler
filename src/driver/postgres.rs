use std::collections::HashMap;

use postgres::{types::Type, Client, Config, Error as PostgresError, NoTls};

use super::{
    validate_statement, DatabaseDriver, DatabaseSession, DeleteReq, DriverError, InsertReq, QueryReq, QueryResp,
    UpdateReq, WriteResp,
};
use crate::option::{PostgreSQLOptions, SslMode};

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
            QueryReq::Sql { statement, params } => {
                validate_statement(&statement)?;

                // 字符串参数转换为 PostgreSQL 参数引用
                let param_refs: Vec<&(dyn postgres::types::ToSql + Sync)> = params
                    .iter()
                    .map(|s| s as &(dyn postgres::types::ToSql + Sync))
                    .collect();

                let rows = self
                    .client
                    .query(&statement, &param_refs[..])
                    .map_err(|err| DriverError::Other(format!("执行查询失败: {}", err)))?;

                let mut records = Vec::with_capacity(rows.len());
                for row in rows {
                    let mut record = HashMap::with_capacity(row.len());
                    for (idx, column) in row.columns().iter().enumerate() {
                        let value = postgres_value_to_string(&row, idx)?;
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
            InsertReq::Sql { statement } => self.exec_write(&statement),
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
            UpdateReq::Sql { statement } => self.exec_write(&statement),
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
            DeleteReq::Sql { statement } => self.exec_write(&statement),
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
        statement: &str,
    ) -> Result<WriteResp, DriverError> {
        validate_statement(statement)?;
        let affected = self
            .client
            .execute(statement, &[])
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

/// 将 PostgreSQL 值转换为字符串（用于 UI 显示）
fn postgres_value_to_string(
    row: &postgres::Row,
    idx: usize,
) -> Result<String, DriverError> {
    let column = row
        .columns()
        .get(idx)
        .ok_or_else(|| DriverError::Other(format!("列索引越界: {}", idx)))?;
    let ty = column.type_();

    // 所有类型统一转换为字符串
    let value = match *ty {
        Type::BOOL => {
            let val: Option<bool> = row.try_get(idx).map_err(map_pg_err)?;
            val.map(|b| b.to_string()).unwrap_or_default()
        }
        Type::INT2 => {
            let val: Option<i16> = row.try_get(idx).map_err(map_pg_err)?;
            val.map(|v| v.to_string()).unwrap_or_default()
        }
        Type::INT4 => {
            let val: Option<i32> = row.try_get(idx).map_err(map_pg_err)?;
            val.map(|v| v.to_string()).unwrap_or_default()
        }
        Type::INT8 => {
            let val: Option<i64> = row.try_get(idx).map_err(map_pg_err)?;
            val.map(|v| v.to_string()).unwrap_or_default()
        }
        Type::FLOAT4 => {
            let val: Option<f32> = row.try_get(idx).map_err(map_pg_err)?;
            val.map(|v| v.to_string()).unwrap_or_default()
        }
        Type::FLOAT8 => {
            let val: Option<f64> = row.try_get(idx).map_err(map_pg_err)?;
            val.map(|v| v.to_string()).unwrap_or_default()
        }
        _ => {
            // 其他所有类型都尝试转为字符串
            let text: Option<String> = row.try_get(idx).map_err(map_pg_err)?;
            text.unwrap_or_default()
        }
    };
    Ok(value)
}

fn map_pg_err(err: PostgresError) -> DriverError {
    DriverError::Other(format!("PostgreSQL 解析字段失败: {}", err))
}
