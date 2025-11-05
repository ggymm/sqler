use std::collections::HashMap;

use mysql::{prelude::Queryable, Conn, Opts, OptsBuilder, SslOpts, Value};
use serde::{Deserialize, Serialize};

use super::{
    validate_sql, DatabaseDriver, DatabaseSession, Datatype, DeleteReq, DriverError, InsertReq, Operator, QueryReq,
    QueryResp, UpdateReq, UpdateResp, ValueCond,
};

#[derive(Debug, Clone, Copy)]
pub struct MySQLDriver;

impl DatabaseDriver for MySQLDriver {
    type Config = MySQLOptions;

    fn data_types(&self) -> Vec<Datatype> {
        vec![
            Datatype::TinyInt,
            Datatype::SmallInt,
            Datatype::Int,
            Datatype::BigInt,
            Datatype::Float,
            Datatype::Double,
            Datatype::Decimal,
            Datatype::Char,
            Datatype::VarChar,
            Datatype::Text,
            Datatype::Binary,
            Datatype::VarBinary,
            Datatype::Blob,
            Datatype::Date,
            Datatype::Time,
            Datatype::DateTime,
            Datatype::Timestamp,
            Datatype::Json,
            Datatype::Enum,
            Datatype::Set,
        ]
    }

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
        Ok(Box::new(MySQLSession::new(conn)))
    }
}

struct MySQLSession {
    conn: Conn,
}

impl MySQLSession {
    fn new(conn: Conn) -> Self {
        Self { conn }
    }
}

impl DatabaseSession for MySQLSession {
    fn query(
        &mut self,
        request: QueryReq,
    ) -> Result<QueryResp, DriverError> {
        // 根据请求类型构建 SQL 和参数
        let (sql, params) = match request {
            QueryReq::Sql { sql, args } => {
                validate_sql(&sql)?;
                (sql, args)
            }
            QueryReq::Builder {
                table,
                columns,
                limit,
                offset,
                orders,
                filters,
            } => {
                // 构建 SELECT SQL
                let cols = if columns.is_empty() {
                    "*".to_string()
                } else {
                    columns
                        .iter()
                        .map(|c| format!("`{}`", c.replace('`', "``")))
                        .collect::<Vec<_>>()
                        .join(", ")
                };
                let mut sql = format!("SELECT {} FROM `{}`", cols, table.replace('`', "``"));
                let mut params = Vec::new();

                // WHERE 子句
                if !filters.is_empty() {
                    let mut clauses = Vec::new();
                    for filter in &filters {
                        let field = format!("`{}`", filter.field.replace('`', "``"));
                        match filter.operator {
                            Operator::IsNull => clauses.push(format!("{} IS NULL", field)),
                            Operator::IsNotNull => clauses.push(format!("{} IS NOT NULL", field)),
                            Operator::In => {
                                if let ValueCond::List(ref list) = filter.value {
                                    if !list.is_empty() {
                                        let placeholders = vec!["?"; list.len()].join(", ");
                                        clauses.push(format!("{} IN ({})", field, placeholders));
                                        params.extend(list.clone());
                                    }
                                }
                            }
                            Operator::NotIn => {
                                if let ValueCond::List(ref list) = filter.value {
                                    if !list.is_empty() {
                                        let placeholders = vec!["?"; list.len()].join(", ");
                                        clauses.push(format!("{} NOT IN ({})", field, placeholders));
                                        params.extend(list.clone());
                                    }
                                }
                            }
                            Operator::Between => {
                                if let ValueCond::Range(ref start, ref end) = filter.value {
                                    clauses.push(format!("{} BETWEEN ? AND ?", field));
                                    params.push(start.clone());
                                    params.push(end.clone());
                                }
                            }
                            _ => {
                                let op_str = match filter.operator {
                                    Operator::Equal => "=",
                                    Operator::NotEqual => "!=",
                                    Operator::GreaterThan => ">",
                                    Operator::LessThan => "<",
                                    Operator::GreaterOrEqual => ">=",
                                    Operator::LessOrEqual => "<=",
                                    Operator::Like => "LIKE",
                                    Operator::NotLike => "NOT LIKE",
                                    _ => "=",
                                };
                                clauses.push(format!("{} {} ?", field, op_str));
                                match &filter.value {
                                    ValueCond::String(s) => params.push(s.clone()),
                                    ValueCond::Number(n) => params.push(n.to_string()),
                                    ValueCond::Bool(b) => {
                                        params.push(if *b { "1".to_string() } else { "0".to_string() })
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    if !clauses.is_empty() {
                        sql.push_str(&format!(" WHERE {}", clauses.join(" AND ")));
                    }
                }

                // ORDER BY 子句
                if !orders.is_empty() {
                    let order_clauses: Vec<_> = orders
                        .iter()
                        .map(|ord| {
                            format!(
                                "`{}` {}",
                                ord.field.replace('`', "``"),
                                if ord.ascending { "ASC" } else { "DESC" }
                            )
                        })
                        .collect();
                    sql.push_str(&format!(" ORDER BY {}", order_clauses.join(", ")));
                }

                // LIMIT/OFFSET 子句
                match (limit, offset) {
                    (Some(l), Some(o)) => sql.push_str(&format!(" LIMIT {} OFFSET {}", l, o)),
                    (Some(l), None) => sql.push_str(&format!(" LIMIT {}", l)),
                    (None, Some(o)) => sql.push_str(&format!(" LIMIT 18446744073709551615 OFFSET {}", o)),
                    (None, None) => {}
                }

                (sql, params)
            }
            other => {
                return Err(DriverError::InvalidField(format!(
                    "MySQL 查询仅支持 SQL 和 Builder，收到: {:?}",
                    other
                )))
            }
        };

        // 统一执行查询和转换结果
        let mysql_params: Vec<Value> = params.into_iter().map(Value::from).collect();

        let rows: Vec<mysql::Row> = self
            .conn
            .exec(&sql, mysql_params)
            .map_err(|err| DriverError::Other(format!("执行查询失败: {}", err)))?;

        if rows.is_empty() {
            return Ok(QueryResp::Rows(Vec::new()));
        }

        let column_names: Vec<String> = rows[0]
            .columns_ref()
            .iter()
            .map(|col| col.name_str().to_string())
            .collect();

        let mut records = Vec::with_capacity(rows.len());
        for row in rows {
            let raw_values = row.unwrap();
            let mut map = HashMap::with_capacity(column_names.len());
            for (idx, name) in column_names.iter().enumerate() {
                let value = raw_values.get(idx).cloned().unwrap_or(Value::NULL);
                map.insert(name.clone(), mysql_value_to_string(value));
            }
            records.push(map);
        }

        Ok(QueryResp::Rows(records))
    }

    fn insert(
        &mut self,
        request: InsertReq,
    ) -> Result<UpdateResp, DriverError> {
        match request {
            InsertReq::Sql { sql } => {
                validate_sql(&sql)?;
                self.conn
                    .query_drop(&sql)
                    .map_err(|err| DriverError::Other(format!("执行写入失败: {}", err)))?;
                Ok(UpdateResp {
                    affected: self.conn.affected_rows(),
                })
            }
            other => Err(DriverError::InvalidField(format!(
                "MySQL 插入仅支持 SQL，收到: {:?}",
                other
            ))),
        }
    }

    fn update(
        &mut self,
        request: UpdateReq,
    ) -> Result<UpdateResp, DriverError> {
        match request {
            UpdateReq::Sql { sql } => {
                validate_sql(&sql)?;
                self.conn
                    .query_drop(&sql)
                    .map_err(|err| DriverError::Other(format!("执行写入失败: {}", err)))?;
                Ok(UpdateResp {
                    affected: self.conn.affected_rows(),
                })
            }
            other => Err(DriverError::InvalidField(format!(
                "MySQL 更新仅支持 SQL，收到: {:?}",
                other
            ))),
        }
    }

    fn delete(
        &mut self,
        request: DeleteReq,
    ) -> Result<UpdateResp, DriverError> {
        match request {
            DeleteReq::Sql { sql } => {
                validate_sql(&sql)?;
                self.conn
                    .query_drop(&sql)
                    .map_err(|err| DriverError::Other(format!("执行写入失败: {}", err)))?;
                Ok(UpdateResp {
                    affected: self.conn.affected_rows(),
                })
            }
            other => Err(DriverError::InvalidField(format!(
                "MySQL 删除仅支持 SQL，收到: {:?}",
                other
            ))),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MySQLOptions {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: Option<String>,
    pub database: String,
    pub charset: Option<String>,
    pub use_tls: bool,
}

impl Default for MySQLOptions {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".into(),
            port: 3306,
            username: "root".into(),
            password: None,
            database: String::new(),
            charset: Some("utf8mb4".into()),
            use_tls: false,
        }
    }
}

impl MySQLOptions {
    pub fn display_endpoint(&self) -> String {
        let scheme = if self.use_tls { "mysqls" } else { "mysql" };
        let db = self.database.trim();
        if db.is_empty() {
            format!("{}://{}:{}", scheme, self.host, self.port)
        } else {
            format!("{}://{}:{}/{}", scheme, self.host, self.port, db)
        }
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
