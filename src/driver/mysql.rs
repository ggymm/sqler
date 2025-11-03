use std::collections::HashMap;

use mysql::{prelude::Queryable, Conn, Opts, OptsBuilder, SslOpts, Value};

use super::{
    validate_stmt, ConditionValue, DatabaseDriver, DatabaseSession, DeleteReq, DriverError, FilterCond, InsertReq,
    Operator, OrderCond, QueryBuilder, QueryReq, QueryResp, UpdateReq, WriteResp,
};
use crate::option::MySQLOptions;

pub struct MySQLBuilder;

impl QueryBuilder for MySQLBuilder {
    fn build_order_clause(
        &self,
        sorts: &[OrderCond],
    ) -> String {
        let orders: Vec<_> = sorts
            .iter()
            .map(|sort| {
                format!(
                    "{} {}",
                    self.escape_identifier(&sort.field),
                    if sort.ascending { "ASC" } else { "DESC" }
                )
            })
            .collect();

        if orders.is_empty() {
            String::new()
        } else {
            format!("ORDER BY {}", orders.join(", "))
        }
    }

    fn build_where_clause(
        &self,
        conditions: &[FilterCond],
    ) -> (String, Vec<String>) {
        let mut clauses = Vec::new();
        let mut params = Vec::new();
        let mut param_index = 0;

        for condition in conditions.iter() {
            let field = self.escape_identifier(&condition.field);

            match condition.operator {
                Operator::IsNull => {
                    clauses.push(format!("{} IS NULL", field));
                }
                Operator::IsNotNull => {
                    clauses.push(format!("{} IS NOT NULL", field));
                }
                Operator::In => {
                    if let ConditionValue::List(ref list) = condition.value {
                        if list.is_empty() {
                            continue;
                        }
                        let placeholders: Vec<_> = (0..list.len())
                            .map(|_| {
                                let ph = self.placeholder(param_index);
                                param_index += 1;
                                ph
                            })
                            .collect();
                        clauses.push(format!("{} IN ({})", field, placeholders.join(", ")));
                        params.extend(list.clone());
                    }
                }
                Operator::NotIn => {
                    if let ConditionValue::List(ref list) = condition.value {
                        if list.is_empty() {
                            continue;
                        }
                        let placeholders: Vec<_> = (0..list.len())
                            .map(|_| {
                                let ph = self.placeholder(param_index);
                                param_index += 1;
                                ph
                            })
                            .collect();
                        clauses.push(format!("{} NOT IN ({})", field, placeholders.join(", ")));
                        params.extend(list.clone());
                    }
                }
                Operator::Between => {
                    if let ConditionValue::Range(ref start, ref end) = condition.value {
                        let ph1 = self.placeholder(param_index);
                        param_index += 1;
                        let ph2 = self.placeholder(param_index);
                        param_index += 1;
                        clauses.push(format!("{} BETWEEN {} AND {}", field, ph1, ph2));
                        params.push(start.clone());
                        params.push(end.clone());
                    }
                }
                _ => {
                    let op_str = match condition.operator {
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

                    let ph = self.placeholder(param_index);
                    param_index += 1;
                    clauses.push(format!("{} {} {}", field, op_str, ph));

                    match &condition.value {
                        ConditionValue::String(s) => params.push(s.clone()),
                        ConditionValue::Number(n) => params.push(n.to_string()),
                        ConditionValue::Bool(b) => params.push(if *b { "1".to_string() } else { "0".to_string() }),
                        _ => {}
                    }
                }
            }
        }

        if clauses.is_empty() {
            ("1=1".to_string(), params)
        } else {
            (clauses.join(" AND "), params)
        }
    }

    fn build_limit_clause(
        &self,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> String {
        match (limit, offset) {
            (Some(l), Some(o)) => format!("LIMIT {} OFFSET {}", l, o),
            (Some(l), None) => format!("LIMIT {}", l),
            (None, Some(o)) => format!("LIMIT 18446744073709551615 OFFSET {}", o), // MySQL requires LIMIT with OFFSET
            (None, None) => String::new(),
        }
    }

    fn escape_identifier(
        &self,
        identifier: &str,
    ) -> String {
        format!("`{}`", identifier.replace('`', "``"))
    }

    fn placeholder(
        &self,
        _index: usize,
    ) -> String {
        "?".to_string()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MySQLDriver;

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
        match request {
            QueryReq::Sql { stmt, args } => {
                validate_stmt(&stmt)?;

                // 字符串参数直接转换为 MySQL 值
                let mysql_params: Vec<Value> = args.into_iter().map(Value::from).collect();

                let rows: Vec<mysql::Row> = self
                    .conn
                    .exec(&stmt, mysql_params)
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
            InsertReq::Sql { stmt: statement } => {
                validate_stmt(&statement)?;
                self.conn
                    .query_drop(&statement)
                    .map_err(|err| DriverError::Other(format!("执行写入失败: {}", err)))?;
                Ok(WriteResp {
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
    ) -> Result<WriteResp, DriverError> {
        match request {
            UpdateReq::Sql { stmt: statement } => {
                validate_stmt(&statement)?;
                self.conn
                    .query_drop(&statement)
                    .map_err(|err| DriverError::Other(format!("执行写入失败: {}", err)))?;
                Ok(WriteResp {
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
    ) -> Result<WriteResp, DriverError> {
        match request {
            DeleteReq::Sql { stmt: statement } => {
                validate_stmt(&statement)?;
                self.conn
                    .query_drop(&statement)
                    .map_err(|err| DriverError::Other(format!("执行写入失败: {}", err)))?;
                Ok(WriteResp {
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
