use std::{collections::HashMap, fs, path::Path};

use rusqlite::{types::ValueRef, Connection, OpenFlags};
use serde::{Deserialize, Serialize};

use super::{
    validate_sql, DatabaseDriver, DatabaseSession, Datatype, DeleteReq, DriverError, InsertReq, Operator, QueryReq,
    QueryResp, UpdateReq, ValueCond, WriteResp,
};

#[derive(Clone, Serialize, Deserialize)]
pub struct SQLiteOptions {
    pub filepath: String,
    pub password: Option<String>,
    pub read_only: bool,
}

impl Default for SQLiteOptions {
    fn default() -> Self {
        Self {
            filepath: String::new(),
            password: None,
            read_only: false,
        }
    }
}
impl SQLiteOptions {
    pub fn display_endpoint(&self) -> String {
        let path = self.filepath.trim();
        if path.is_empty() {
            "sqlite://<未配置文件>".into()
        } else if self.read_only {
            format!("sqlite://{}?mode=ro", path)
        } else {
            format!("sqlite://{}", path)
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SQLiteDriver;

struct SQLiteConnection {
    conn: Connection,
}

impl SQLiteConnection {
    fn new(conn: Connection) -> Self {
        Self { conn }
    }
}

impl DatabaseSession for SQLiteConnection {
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
                filters,
                orders,
                limit,
                offset,
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
                    (None, Some(o)) => sql.push_str(&format!(" LIMIT -1 OFFSET {}", o)),
                    (None, None) => {}
                }

                (sql, params)
            }
            other => {
                return Err(DriverError::InvalidField(format!(
                    "SQLite 查询仅支持 SQL 和 Builder，收到: {:?}",
                    other
                )))
            }
        };

        let mut stmt = self
            .conn
            .prepare(&sql)
            .map_err(|err| DriverError::Other(format!("准备查询失败: {}", err)))?;
        let names = stmt.column_names().iter().map(|s| s.to_string()).collect::<Vec<_>>();

        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|s| s as &dyn rusqlite::ToSql).collect();

        let mut rows = stmt
            .query(&param_refs[..])
            .map_err(|err| DriverError::Other(format!("执行查询失败: {}", err)))?;

        let mut records = Vec::new();
        while let Some(row) = rows
            .next()
            .map_err(|err| DriverError::Other(format!("读取结果失败: {}", err)))?
        {
            let mut record = HashMap::with_capacity(names.len());
            for (idx, name) in names.iter().enumerate() {
                let value = row
                    .get_ref(idx)
                    .map_err(|err| DriverError::Other(format!("读取列 {name} 失败: {}", err)))?;
                record.insert(name.clone(), sqlite_value_to_string(value));
            }
            records.push(record);
        }

        Ok(QueryResp::Rows(records))
    }

    fn insert(
        &mut self,
        request: InsertReq,
    ) -> Result<WriteResp, DriverError> {
        match request {
            InsertReq::Sql { sql } => {
                validate_sql(&sql)?;
                let affected = self
                    .conn
                    .execute(&sql, [])
                    .map_err(|err| DriverError::Other(format!("执行写入失败: {}", err)))?;
                Ok(WriteResp {
                    affected: affected as u64,
                })
            }
            other => Err(DriverError::InvalidField(format!(
                "SQLite 插入仅支持 SQL，收到: {:?}",
                other
            ))),
        }
    }

    fn update(
        &mut self,
        request: UpdateReq,
    ) -> Result<WriteResp, DriverError> {
        match request {
            UpdateReq::Sql { sql } => {
                validate_sql(&sql)?;
                let affected = self
                    .conn
                    .execute(&sql, [])
                    .map_err(|err| DriverError::Other(format!("执行写入失败: {}", err)))?;
                Ok(WriteResp {
                    affected: affected as u64,
                })
            }
            other => Err(DriverError::InvalidField(format!(
                "SQLite 更新仅支持 SQL，收到: {:?}",
                other
            ))),
        }
    }

    fn delete(
        &mut self,
        request: DeleteReq,
    ) -> Result<WriteResp, DriverError> {
        match request {
            DeleteReq::Sql { sql } => {
                validate_sql(&sql)?;
                let affected = self
                    .conn
                    .execute(&sql, [])
                    .map_err(|err| DriverError::Other(format!("执行写入失败: {}", err)))?;
                Ok(WriteResp {
                    affected: affected as u64,
                })
            }
            other => Err(DriverError::InvalidField(format!(
                "SQLite 删除仅支持 SQL，收到: {:?}",
                other
            ))),
        }
    }
}

impl DatabaseDriver for SQLiteDriver {
    type Config = SQLiteOptions;

    fn data_types(&self) -> Vec<Datatype> {
        vec![
            Datatype::Int,
            Datatype::BigInt,
            Datatype::Float,
            Datatype::Double,
            Datatype::Char,
            Datatype::VarChar,
            Datatype::Text,
            Datatype::Blob,
            Datatype::Date,
            Datatype::DateTime,
            Datatype::Boolean,
        ]
    }

    fn check_connection(
        &self,
        config: &Self::Config,
    ) -> Result<(), DriverError> {
        let conn = open_connection(config)?;
        conn.query_row("SELECT 1", [], |_| Ok::<_, rusqlite::Error>(()))
            .map_err(|err| DriverError::Other(format!("校验查询失败: {}", err)))?;
        Ok(())
    }

    fn create_connection(
        &self,
        config: &Self::Config,
    ) -> Result<Box<dyn DatabaseSession>, DriverError> {
        let conn = open_connection(config)?;
        Ok(Box::new(SQLiteConnection::new(conn)))
    }
}

fn open_connection(config: &SQLiteOptions) -> Result<Connection, DriverError> {
    let path_str = config.filepath.trim();
    if path_str.is_empty() {
        return Err(DriverError::MissingField("file_path".into()));
    }

    let path = Path::new(path_str);

    if config.read_only {
        if !path.exists() {
            return Err(DriverError::InvalidField("file_path 不存在".into()));
        }
    } else if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|err| DriverError::Other(format!("创建目录失败: {}", err)))?;
        }
    }

    let flags = if config.read_only {
        OpenFlags::SQLITE_OPEN_READ_ONLY
    } else {
        OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE
    };

    Connection::open_with_flags(path, flags).map_err(|err| DriverError::Other(format!("打开 SQLite 失败: {}", err)))
}

/// 将 SQLite 值转换为字符串（用于 UI 显示）
fn sqlite_value_to_string(value: ValueRef<'_>) -> String {
    match value {
        ValueRef::Null => String::new(),
        ValueRef::Integer(int) => int.to_string(),
        ValueRef::Real(real) => real.to_string(),
        ValueRef::Text(text) => String::from_utf8_lossy(text).into_owned(),
        ValueRef::Blob(blob) => {
            // Blob 显示为十六进制字符串
            blob.iter().map(|b| format!("{:02x}", b)).collect::<String>()
        }
    }
}
