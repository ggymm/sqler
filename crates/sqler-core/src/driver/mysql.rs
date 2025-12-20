use std::collections::HashMap;

use mysql::{Conn, Opts, OptsBuilder, SslOpts, Value, prelude::Queryable};

use crate::{ColumnInfo, ColumnKind, MySQLOptions, TableInfo};

use super::{
    DatabaseDriver, DatabaseSession, DriverError, ExecReq, ExecResp, Operator, QueryReq, QueryResp, ValueCond,
    escape_backtick, validate_sql,
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
    fn exec(
        &mut self,
        req: ExecReq,
    ) -> Result<ExecResp, DriverError> {
        match req {
            ExecReq::Sql { sql } => {
                validate_sql(&sql)?;
                self.conn
                    .query_drop(&sql)
                    .map_err(|err| DriverError::Other(format!("执行失败: {}", err)))?;
                Ok(ExecResp {
                    affected: self.conn.affected_rows(),
                })
            }
            other => Err(DriverError::InvalidField(format!(
                "MySQL 仅支持 SQL，收到: {:?}",
                other
            ))),
        }
    }

    fn query(
        &mut self,
        req: QueryReq,
    ) -> Result<QueryResp, DriverError> {
        let (sql, params) = match req {
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
                let mut sql = format!("SELECT {} FROM `{}`", format_columns(&columns), escape_backtick(&table));
                let mut params: Vec<Value> = vec![];

                // WHERE 子句
                if !filters.is_empty() {
                    let mut clauses = vec![];
                    for filter in &filters {
                        let field = format!("`{}`", escape_backtick(&filter.field));
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
                                escape_backtick(&ord.field),
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
                )));
            }
        };
        tracing::debug!(sql = %sql);

        let iter = self
            .conn
            .exec_iter(&sql, params)
            .map_err(|err| DriverError::Other(format!("执行查询失败: {}", err)))?;

        let rows: Vec<mysql::Row> = iter
            .take(1000)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|err| DriverError::Other(format!("读取结果失败: {}", err)))?;

        if rows.is_empty() {
            return Ok(QueryResp::Rows {
                cols: vec![],
                rows: vec![],
            });
        }

        let columns: Vec<String> = rows[0]
            .columns_ref()
            .iter()
            .map(|col| col.name_str().to_string())
            .collect();
        let mut records = Vec::with_capacity(rows.len());

        for row in rows {
            let raw = row.unwrap();
            let mut map = HashMap::with_capacity(columns.len());
            for (idx, name) in columns.iter().enumerate() {
                let value = raw.get(idx).cloned().unwrap_or(Value::NULL);
                map.insert(name.clone(), parse_value(value));
            }
            records.push(map);
        }

        Ok(QueryResp::Rows {
            cols: columns,
            rows: records,
        })
    }

    fn tables(&mut self) -> Result<Vec<TableInfo>, DriverError> {
        let sql = "SHOW TABLE STATUS";
        let rows: Vec<mysql::Row> = self
            .conn
            .query(sql)
            .map_err(|err| DriverError::Other(format!("查询表列表失败: {}", err)))?;

        let mut tables = vec![];
        for row in rows {
            let name: String = row
                .get("Name")
                .ok_or_else(|| DriverError::Other("缺少 Name 字段".into()))?;

            let row_count: Option<u64> = row.get("Rows");
            let size_bytes: Option<u64> = row.get("Data_length");

            tables.push(TableInfo {
                name,
                row_count,
                size_bytes,
                last_accessed: None,
            });
        }
        Ok(tables)
    }

    fn columns(
        &mut self,
        table: &str,
    ) -> Result<Vec<ColumnInfo>, DriverError> {
        let sql = format!("SHOW FULL COLUMNS FROM `{}`", escape_backtick(table));
        let rows: Vec<mysql::Row> = self
            .conn
            .query(&sql)
            .map_err(|err| DriverError::Other(format!("查询列信息失败: {}", err)))?;

        let mut columns = vec![];
        for row in rows {
            let name: String = row
                .get("Field")
                .ok_or_else(|| DriverError::Other("缺少 Field 字段".into()))?;

            let mut col = ColumnInfo {
                name,
                kind: String::new(),
                comment: String::new(),
                nullable: false,
                primary_key: false,
                default_value: String::new(),
                max_length: 0,
                auto_increment: false,
            };

            // 批量获取并设置字段值
            for field in ["Type", "Null", "Key", "Extra", "Default", "Comment"] {
                let value = row.get::<Value, _>(field).unwrap_or(Value::NULL);
                let parsed = parse_value(value);
                match field {
                    "Key" => col.primary_key = parsed == "PRI",
                    "Null" => col.nullable = parsed.to_uppercase() == "YES",
                    "Extra" => col.auto_increment = parsed.contains("auto_increment"),
                    "Type" => col.kind = parsed,
                    "Comment" => col.comment = parsed,
                    "Default" => col.default_value = parsed,
                    _ => {}
                }
            }

            columns.push(col);
        }
        Ok(columns)
    }
}

fn open_conn(config: &MySQLOptions) -> Result<Conn, DriverError> {
    if config.host.trim().is_empty() {
        return Err(DriverError::MissingField("host".into()));
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
    builder = builder.tcp_port(config.port.parse().unwrap_or(3306));
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

fn format_columns(columns: &[String]) -> String {
    if columns.is_empty() {
        return "*".to_string();
    }

    // MySQL 保留关键字列表
    // 参考: https://dev.mysql.com/doc/refman/8.0/en/keywords.html
    #[rustfmt::skip]
    let keywords = [
        "SELECT", "FROM", "WHERE", "INSERT", "UPDATE", "DELETE",
        "CREATE", "DROP", "ALTER", "TABLE", "INDEX", "VIEW",
        "JOIN", "LEFT", "RIGHT", "INNER", "OUTER", "ON",
        "GROUP", "ORDER", "BY", "HAVING", "LIMIT", "OFFSET",
        "AS", "AND", "OR", "NOT", "IN", "IS", "NULL",
        "PRIMARY", "KEY", "FOREIGN", "REFERENCES", "CONSTRAINT",
        "DEFAULT", "AUTO_INCREMENT", "UNIQUE", "CHECK",
        "COUNT", "SUM", "AVG", "MAX", "MIN",
        "DISTINCT", "ALL", "BETWEEN", "LIKE", "EXISTS",
        "CASE", "WHEN", "THEN", "ELSE", "END",
        "UNION", "INTERSECT", "EXCEPT",
        "DATABASE", "SCHEMA", "TRIGGER", "PROCEDURE", "FUNCTION",
        "INT", "VARCHAR", "TEXT", "DATE", "DATETIME", "TIMESTAMP",
        "CHAR", "DECIMAL", "FLOAT", "DOUBLE", "BOOLEAN",
    ];

    columns
        .iter()
        .map(|c| {
            if keywords.contains(&c.trim().to_uppercase().as_str()) {
                format!("`{}`", escape_backtick(c))
            } else {
                c.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}
