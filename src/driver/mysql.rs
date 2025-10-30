use super::{
    validate_statement, DatabaseDriver, DatabaseSession, DeleteReq, DriverError, InsertReq, QueryReq, QueryResp,
    UpdateReq, WriteResp,
};
use crate::option::MySQLOptions;

use mysql::prelude::Queryable;
use mysql::{Conn, Opts, OptsBuilder, SslOpts, Value};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub struct MySQLDriver;

struct MySQLConnection {
    conn: Conn,
}

impl MySQLConnection {
    fn new(conn: Conn) -> Self {
        Self { conn }
    }
}

impl DatabaseSession for MySQLConnection {
    fn query(
        &mut self,
        request: QueryReq,
    ) -> Result<QueryResp, DriverError> {
        match request {
            QueryReq::Sql { statement, params } => {
                validate_statement(&statement)?;

                // 字符串参数直接转换为 MySQL 值
                let mysql_params: Vec<Value> = params.into_iter().map(Value::from).collect();

                let rows: Vec<mysql::Row> = self
                    .conn
                    .exec(&statement, mysql_params)
                    .map_err(|err| DriverError::Other(format!("执行查询失败: {}", err)))?;

                if rows.is_empty() {
                    return Ok(QueryResp::Rows(Vec::new()));
                }

                // 提取列名（只需一次）
                let columns: Vec<String> = rows[0]
                    .columns_ref()
                    .iter()
                    .map(|col| col.name_str().to_string())
                    .collect();

                // 转换行数据为字符串 Map
                let mut records = Vec::with_capacity(rows.len());
                for row in rows {
                    let raw_values = row.unwrap();
                    let mut map = HashMap::with_capacity(columns.len());
                    for (idx, name) in columns.iter().enumerate() {
                        let value = raw_values.get(idx).cloned().unwrap_or(Value::NULL);
                        map.insert(name.clone(), mysql_value_to_string(value));
                    }
                    records.push(map);
                }

                Ok(QueryResp::Rows(records))
            }
            other => Err(DriverError::InvalidField(format!(
                "MySQL 查询仅支持 SQL，收到: {:?}",
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
                "MySQL 插入仅支持 SQL，收到: {:?}",
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
                "MySQL 更新仅支持 SQL，收到: {:?}",
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
                "MySQL 删除仅支持 SQL，收到: {:?}",
                other
            ))),
        }
    }
}

impl MySQLConnection {
    fn exec_write(
        &mut self,
        statement: &str,
    ) -> Result<WriteResp, DriverError> {
        validate_statement(statement)?;
        self.conn
            .query_drop(statement)
            .map_err(|err| DriverError::Other(format!("执行写入失败: {}", err)))?;
        Ok(WriteResp {
            affected: self.conn.affected_rows() as u64,
        })
    }
}

impl DatabaseDriver for MySQLDriver {
    type Config = MySQLOptions;

    fn check_connection(
        &self,
        config: &Self::Config,
    ) -> Result<(), DriverError> {
        let mut conn = connect(config)?;
        if let Some(charset) = &config.charset {
            apply_charset(charset, &mut conn)?;
        }
        conn.ping()
            .map_err(|err| DriverError::Other(format!("ping 失败: {}", err)))?;
        Ok(())
    }

    fn create_connection(
        &self,
        config: &Self::Config,
    ) -> Result<Box<dyn DatabaseSession>, DriverError> {
        let mut conn = connect(config)?;
        if let Some(charset) = &config.charset {
            apply_charset(charset, &mut conn)?;
        }
        Ok(Box::new(MySQLConnection::new(conn)))
    }
}

fn connect(config: &MySQLOptions) -> Result<Conn, DriverError> {
    if config.host.trim().is_empty() {
        return Err(DriverError::MissingField("host".into()));
    }
    if config.port == 0 {
        return Err(DriverError::InvalidField("port".into()));
    }
    if config.username.trim().is_empty() {
        return Err(DriverError::MissingField("username".into()));
    }

    let mut builder = OptsBuilder::new();
    builder = builder.ip_or_hostname(Some(config.host.clone()));
    builder = builder.tcp_port(config.port);
    builder = builder.user(Some(config.username.clone()));

    if let Some(password) = &config.password {
        builder = builder.pass(Some(password.clone()));
    }

    if !config.database.is_empty() {
        builder = builder.db_name(Some(config.database.clone()));
    }

    if config.use_tls {
        builder = builder.ssl_opts(Some(SslOpts::default()));
    }

    let opts = Opts::from(builder);

    Conn::new(opts).map_err(|err| DriverError::Other(format!("连接失败: {}", err)))
}

fn apply_charset(
    charset: &str,
    conn: &mut Conn,
) -> Result<(), DriverError> {
    let trimmed = charset.trim();
    if trimmed.is_empty() {
        return Ok(());
    }

    let is_valid = trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
    if !is_valid {
        return Err(DriverError::InvalidField("charset".into()));
    }

    conn.query_drop(format!("SET NAMES {}", trimmed))
        .map_err(|err| DriverError::Other(format!("设置字符集失败: {}", err)))
}

/// 将 MySQL 值转换为字符串（用于 UI 显示）
fn mysql_value_to_string(value: Value) -> String {
    match value {
        Value::NULL => String::new(),
        Value::Bytes(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
        Value::Int(int) => int.to_string(),
        Value::UInt(uint) => uint.to_string(),
        Value::Float(float) => float.to_string(),
        Value::Double(double) => double.to_string(),
        Value::Date(year, month, day, hour, minute, second, micros) => {
            format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02}.{micros:06}")
        }
        Value::Time(neg, days, hours, minutes, seconds, micros) => {
            let sign = if neg { "-" } else { "" };
            format!("{sign}{days} {hours:02}:{minutes:02}:{seconds:02}.{micros:06}")
        }
    }
}
