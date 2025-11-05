use std::collections::HashMap;

use postgres::{types::Type, Client, Config, Error as PostgresError, NoTls};
use serde::{Deserialize, Serialize};

use super::{
    validate_sql, DatabaseDriver, DatabaseSession, Datatype, DeleteReq, DriverError, InsertReq, Operator, QueryReq,
    QueryResp, UpdateReq, UpdateResp, ValueCond,
};

#[derive(Debug, Clone, Copy)]
pub struct PostgresDriver;

impl DatabaseDriver for PostgresDriver {
    type Config = PostgresOptions;

    fn data_types(&self) -> Vec<Datatype> {
        vec![
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
            Datatype::Date,
            Datatype::Time,
            Datatype::Timestamp,
            Datatype::Boolean,
            Datatype::Json,
            Datatype::Uuid,
            Datatype::Array,
        ]
    }

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
        Ok(Box::new(PostgresSession::new(client)))
    }
}

struct PostgresSession {
    client: Client,
}

impl PostgresSession {
    fn new(client: Client) -> Self {
        Self { client }
    }
}

impl DatabaseSession for PostgresSession {
    fn query(
        &mut self,
        request: QueryReq,
    ) -> Result<QueryResp, DriverError> {
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
                let cols = if columns.is_empty() {
                    "*".to_string()
                } else {
                    columns
                        .iter()
                        .map(|c| format!("\"{}\"", c.replace('"', "\"\"")))
                        .collect::<Vec<_>>()
                        .join(", ")
                };
                let mut sql = format!("SELECT {} FROM \"{}\"", cols, table.replace('"', "\"\""));
                let mut params = Vec::new();
                let mut param_index = 1;

                if !filters.is_empty() {
                    let mut clauses = Vec::new();
                    for filter in &filters {
                        let field = format!("\"{}\"", filter.field.replace('"', "\"\""));
                        match filter.operator {
                            Operator::IsNull => clauses.push(format!("{} IS NULL", field)),
                            Operator::IsNotNull => clauses.push(format!("{} IS NOT NULL", field)),
                            Operator::In => {
                                if let ValueCond::List(ref list) = filter.value {
                                    if !list.is_empty() {
                                        let placeholders: Vec<_> = (0..list.len())
                                            .map(|_| {
                                                let ph = format!("${}", param_index);
                                                param_index += 1;
                                                ph
                                            })
                                            .collect();
                                        clauses.push(format!("{} IN ({})", field, placeholders.join(", ")));
                                        params.extend(list.clone());
                                    }
                                }
                            }
                            Operator::NotIn => {
                                if let ValueCond::List(ref list) = filter.value {
                                    if !list.is_empty() {
                                        let placeholders: Vec<_> = (0..list.len())
                                            .map(|_| {
                                                let ph = format!("${}", param_index);
                                                param_index += 1;
                                                ph
                                            })
                                            .collect();
                                        clauses.push(format!("{} NOT IN ({})", field, placeholders.join(", ")));
                                        params.extend(list.clone());
                                    }
                                }
                            }
                            Operator::Between => {
                                if let ValueCond::Range(ref start, ref end) = filter.value {
                                    clauses.push(format!(
                                        "{} BETWEEN ${} AND ${}",
                                        field,
                                        param_index,
                                        param_index + 1
                                    ));
                                    param_index += 2;
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
                                clauses.push(format!("{} {} ${}", field, op_str, param_index));
                                param_index += 1;
                                match &filter.value {
                                    ValueCond::String(s) => params.push(s.clone()),
                                    ValueCond::Number(n) => params.push(n.to_string()),
                                    ValueCond::Bool(b) => {
                                        params.push(if *b { "true".to_string() } else { "false".to_string() })
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

                if !orders.is_empty() {
                    let order_clauses: Vec<_> = orders
                        .iter()
                        .map(|ord| {
                            format!(
                                "\"{}\" {}",
                                ord.field.replace('"', "\"\""),
                                if ord.ascending { "ASC" } else { "DESC" }
                            )
                        })
                        .collect();
                    sql.push_str(&format!(" ORDER BY {}", order_clauses.join(", ")));
                }

                match (limit, offset) {
                    (Some(l), Some(o)) => sql.push_str(&format!(" LIMIT {} OFFSET {}", l, o)),
                    (Some(l), None) => sql.push_str(&format!(" LIMIT {}", l)),
                    (None, Some(o)) => sql.push_str(&format!(" OFFSET {}", o)),
                    (None, None) => {}
                }

                (sql, params)
            }
            other => {
                return Err(DriverError::InvalidField(format!(
                    "PostgreSQL 查询仅支持 SQL 和 Builder，收到: {:?}",
                    other
                )))
            }
        };

        let param_refs: Vec<&(dyn postgres::types::ToSql + Sync)> = params
            .iter()
            .map(|s| s as &(dyn postgres::types::ToSql + Sync))
            .collect();

        let rows = self
            .client
            .query(&sql, &param_refs[..])
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

    fn insert(
        &mut self,
        request: InsertReq,
    ) -> Result<UpdateResp, DriverError> {
        match request {
            InsertReq::Sql { sql } => {
                validate_sql(&sql)?;
                let affected = self
                    .client
                    .execute(&sql, &[])
                    .map_err(|err| DriverError::Other(format!("执行写入失败: {}", err)))?;
                Ok(UpdateResp { affected })
            }
            other => Err(DriverError::InvalidField(format!(
                "PostgreSQL 插入仅支持 SQL，收到: {:?}",
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
                let affected = self
                    .client
                    .execute(&sql, &[])
                    .map_err(|err| DriverError::Other(format!("执行写入失败: {}", err)))?;
                Ok(UpdateResp { affected })
            }
            other => Err(DriverError::InvalidField(format!(
                "PostgreSQL 更新仅支持 SQL，收到: {:?}",
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
                let affected = self
                    .client
                    .execute(&sql, &[])
                    .map_err(|err| DriverError::Other(format!("执行写入失败: {}", err)))?;
                Ok(UpdateResp { affected })
            }
            other => Err(DriverError::InvalidField(format!(
                "PostgreSQL 删除仅支持 SQL，收到: {:?}",
                other
            ))),
        }
    }

    fn tables(&mut self) -> Result<Vec<String>, DriverError> {
        let sql = "SELECT tablename FROM pg_tables WHERE schemaname = 'public'";
        let rows = self
            .client
            .query(sql, &[])
            .map_err(|err| DriverError::Other(format!("查询表列表失败: {}", err)))?;

        let mut tables = Vec::new();
        for row in rows {
            let table_name: String = row.get(0);
            tables.push(table_name);
        }
        Ok(tables)
    }

    fn columns(
        &mut self,
        table: &str,
    ) -> Result<Vec<String>, DriverError> {
        let sql = "SELECT column_name FROM information_schema.columns WHERE table_schema = 'public' AND table_name = $1 ORDER BY ordinal_position";
        let rows = self
            .client
            .query(sql, &[&table])
            .map_err(|err| DriverError::Other(format!("查询列信息失败: {}", err)))?;

        let mut columns = Vec::new();
        for row in rows {
            let column_name: String = row.get(0);
            columns.push(column_name);
        }
        Ok(columns)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PostgresOptions {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: Option<String>,
    pub use_tls: bool,
}

impl Default for PostgresOptions {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".into(),
            port: 5432,
            database: String::new(),
            username: "postgres".into(),
            password: None,
            use_tls: false,
        }
    }
}

impl PostgresOptions {
    pub fn endpoint(&self) -> String {
        let db = self.database.trim();
        let suffix = if db.is_empty() {
            String::new()
        } else {
            format!("/{}", db)
        };
        format!("postgres://{}:{}{}", self.host, self.port, suffix)
    }

    pub fn overview(&self) -> Vec<(&'static str, String)> {
        vec![
            ("连接地址", format!("{}:{}", self.host, self.port)),
            (
                "数据库",
                if self.database.is_empty() {
                    "未配置".into()
                } else {
                    self.database.clone()
                },
            ),
            (
                "安全性",
                if self.use_tls {
                    "TLS 已启用".into()
                } else {
                    "未启用 TLS".into()
                },
            ),
        ]
    }
}

fn connect(config: &PostgresOptions) -> Result<Client, DriverError> {
    if config.host.trim().is_empty() {
        return Err(DriverError::MissingField("host".into()));
    }
    if config.port == 0 {
        return Err(DriverError::InvalidField("port".into()));
    }
    if config.username.trim().is_empty() {
        return Err(DriverError::MissingField("username".into()));
    }
    if config.use_tls {
        return Err(DriverError::Other("PostgreSQL 暂未支持 TLS 连接".into()));
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

    let client = pg_config
        .connect(NoTls)
        .map_err(|err| DriverError::Other(format!("连接失败: {}", err)))?;

    Ok(client)
}

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
