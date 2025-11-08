use std::collections::HashMap;

use mysql::{prelude::Queryable, Conn, Opts, OptsBuilder, SslOpts, Value};

use crate::model::{ColumnKind, MySQLOptions};

use super::{
    validate_sql, DatabaseDriver, DatabaseSession, DeleteReq, DriverError, InsertReq, Operator, QueryReq, QueryResp,
    UpdateReq, UpdateResp, ValueCond,
};

#[derive(Debug, Clone, Copy)]
pub struct MySQLDriver;

impl DatabaseDriver for MySQLDriver {
    type Config = MySQLOptions;

    fn supp_kinds(&self) -> Vec<ColumnKind> {
        vec![
            ColumnKind::TinyInt,
            ColumnKind::SmallInt,
            ColumnKind::Int,
            ColumnKind::BigInt,
            ColumnKind::Float,
            ColumnKind::Double,
            ColumnKind::Decimal,
            ColumnKind::Char,
            ColumnKind::VarChar,
            ColumnKind::Text,
            ColumnKind::Binary,
            ColumnKind::VarBinary,
            ColumnKind::Blob,
            ColumnKind::Date,
            ColumnKind::Time,
            ColumnKind::DateTime,
            ColumnKind::Timestamp,
            ColumnKind::Json,
            ColumnKind::Enum,
            ColumnKind::Set,
        ]
    }

    fn check_connection(
        &self,
        config: &Self::Config,
    ) -> Result<(), DriverError> {
        let mut conn = open_conn(config)?;
        conn.ping()
            .map_err(|err| DriverError::Other(format!("ping 失败: {}", err)))?;
        Ok(())
    }

    fn create_connection(
        &self,
        config: &Self::Config,
    ) -> Result<Box<dyn DatabaseSession>, DriverError> {
        let conn = open_conn(config)?;
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
        let (sql, params) = match request {
            QueryReq::Sql { sql, args } => {
                validate_sql(&sql)?;
                let params: Vec<Value> = args.into_iter().map(Value::from).collect();
                (sql, params)
            }
            QueryReq::Builder {
                table,
                columns,
                paging,
                orders,
                filters,
            } => {
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
                let mut params: Vec<Value> = Vec::new();

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
                                        for item in list {
                                            params.push(Value::from(item.as_str()));
                                        }
                                    }
                                }
                            }
                            Operator::NotIn => {
                                if let ValueCond::List(ref list) = filter.value {
                                    if !list.is_empty() {
                                        let placeholders = vec!["?"; list.len()].join(", ");
                                        clauses.push(format!("{} NOT IN ({})", field, placeholders));
                                        for item in list {
                                            params.push(Value::from(item.as_str()));
                                        }
                                    }
                                }
                            }
                            Operator::Between => {
                                if let ValueCond::Range(ref start, ref end) = filter.value {
                                    clauses.push(format!("{} BETWEEN ? AND ?", field));
                                    params.push(Value::from(start.as_str()));
                                    params.push(Value::from(end.as_str()));
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
                                    ValueCond::String(s) => params.push(Value::from(s.as_str())),
                                    ValueCond::Number(n) => params.push(Value::from(*n)),
                                    ValueCond::Bool(b) => params.push(Value::from(*b)),
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

                // 分页子句
                if let Some(page) = paging {
                    sql.push_str(&format!(" LIMIT {} OFFSET {}", page.limit(), page.offset()));
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

        tracing::debug!(sql = %sql);

        let rows: Vec<mysql::Row> = self
            .conn
            .exec(&sql, params)
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
                map.insert(name.clone(), parse_value(value));
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

    fn tables(&mut self) -> Result<Vec<String>, DriverError> {
        let sql = "SHOW TABLES";
        let rows: Vec<mysql::Row> = self
            .conn
            .query(sql)
            .map_err(|err| DriverError::Other(format!("查询表列表失败: {}", err)))?;

        let mut tables = Vec::new();
        for row in rows {
            let raw_values = row.unwrap();
            if let Some(value) = raw_values.get(0) {
                tables.push(parse_value(value.clone()));
            }
        }
        Ok(tables)
    }

    fn columns(
        &mut self,
        table: &str,
    ) -> Result<Vec<String>, DriverError> {
        let sql = format!("SHOW COLUMNS FROM `{}`", table.replace('`', "``"));
        let rows: Vec<mysql::Row> = self
            .conn
            .query(&sql)
            .map_err(|err| DriverError::Other(format!("查询列信息失败: {}", err)))?;

        let mut columns = Vec::new();
        for row in rows {
            let raw_values = row.unwrap();
            if let Some(value) = raw_values.get(0) {
                columns.push(parse_value(value.clone()));
            }
        }
        Ok(columns)
    }
}

fn open_conn(config: &MySQLOptions) -> Result<Conn, DriverError> {
    if config.host.trim().is_empty() {
        return Err(DriverError::MissingField("host".into()));
    }
    if config.port == 0 {
        return Err(DriverError::InvalidField("port".into()));
    }
    if config.username.trim().is_empty() {
        return Err(DriverError::MissingField("username".into()));
    }
    if config.password.trim().is_empty() {
        return Err(DriverError::MissingField("password".into()));
    }
    if config.database.trim().is_empty() {
        return Err(DriverError::MissingField("database".into()));
    }

    let mut builder = OptsBuilder::new();
    builder = builder.ip_or_hostname(Some(config.host.clone()));
    builder = builder.tcp_port(config.port);
    builder = builder.user(Some(config.username.clone()));
    builder = builder.pass(Some(config.password.clone()));
    builder = builder.db_name(Some(config.database.clone()));

    if config.use_tls {
        builder = builder.ssl_opts(Some(SslOpts::default()));
    }

    let opts = Opts::from(builder);
    Conn::new(opts).map_err(|err| DriverError::Other(format!("连接失败: {}", err)))
}

fn parse_value(value: Value) -> String {
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
