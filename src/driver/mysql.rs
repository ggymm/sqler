use super::{
    DatabaseDriver, DatabaseSession, DeleteReq, DriverError, InsertReq, QueryReq, QueryResp, UpdateReq, WriteResp,
};
use crate::option::MySQLOptions;

use mysql::prelude::Queryable;
use mysql::{Conn, Opts, OptsBuilder, SslOpts, Value};
use serde_json::{Map, Number};

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
            QueryReq::Sql { statement } => {
                if statement.trim().is_empty() {
                    return Err(DriverError::InvalidField("statement".into()));
                }
                let rows: Vec<mysql::Row> = self
                    .conn
                    .query(statement)
                    .map_err(|err| DriverError::Other(format!("执行查询失败: {}", err)))?;

                let mut records = Vec::new();
                for row in rows {
                    let columns = row
                        .columns_ref()
                        .iter()
                        .map(|col| col.name_str().to_string())
                        .collect::<Vec<_>>();
                    let raw_values = row.unwrap();

                    let mut map = Map::with_capacity(columns.len());
                    for (idx, name) in columns.into_iter().enumerate() {
                        let value = raw_values.get(idx).cloned().unwrap_or(Value::NULL);
                        map.insert(name, mysql_value_to_json(value));
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
            InsertReq::Sql { statement } => self.exec_write(statement),
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
            UpdateReq::Sql { statement } => self.exec_write(statement),
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
            DeleteReq::Sql { statement } => self.exec_write(statement),
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
        statement: String,
    ) -> Result<WriteResp, DriverError> {
        if statement.trim().is_empty() {
            return Err(DriverError::InvalidField("statement".into()));
        }
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
    if !trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-'))
    {
        return Err(DriverError::InvalidField("charset".into()));
    }
    conn.query_drop(format!("SET NAMES {}", trimmed))
        .map_err(|err| DriverError::Other(format!("设置字符集失败: {}", err)))
}

fn mysql_value_to_json(value: Value) -> serde_json::Value {
    match value {
        Value::NULL => serde_json::Value::Null,
        Value::Bytes(bytes) => serde_json::Value::String(String::from_utf8_lossy(&bytes).into_owned()),
        Value::Int(int) => serde_json::Value::Number(Number::from(int)),
        Value::UInt(uint) => serde_json::Value::Number(Number::from(uint)),
        Value::Float(float) => {
            if let Some(num) = Number::from_f64(float as f64) {
                serde_json::Value::Number(num)
            } else {
                serde_json::Value::String(float.to_string())
            }
        }
        Value::Double(double) => {
            if let Some(num) = Number::from_f64(double) {
                serde_json::Value::Number(num)
            } else {
                serde_json::Value::String(double.to_string())
            }
        }
        Value::Date(year, month, day, hour, minute, second, micros) => serde_json::Value::String(format!(
            "{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02}.{:06}",
            micros
        )),
        Value::Time(neg, days, hours, minutes, seconds, micros) => {
            let sign = if neg { "-" } else { "" };
            serde_json::Value::String(format!(
                "{sign}{days} {:02}:{:02}:{:02}.{:06}",
                hours, minutes, seconds, micros
            ))
        }
    }
}
