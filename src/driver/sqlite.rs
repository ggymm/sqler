use std::{collections::HashMap, fs, path::Path};

use rusqlite::{types::ValueRef, Connection, OpenFlags};

use super::{
    validate_stmt, ConditionValue, DatabaseDriver, DatabaseSession, DeleteReq, DriverError, FilterCond, InsertReq,
    Operator, OrderCond, QueryBuilder, QueryReq, QueryResp, UpdateReq, WriteResp,
};
use crate::option::SQLiteOptions;

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
        match request {
            QueryReq::Sql {
                stmt: statement,
                params,
            } => {
                validate_stmt(&statement)?;

                let mut stmt = self
                    .conn
                    .prepare(&statement)
                    .map_err(|err| DriverError::Other(format!("准备查询失败: {}", err)))?;
                let names = stmt.column_names().iter().map(|s| s.to_string()).collect::<Vec<_>>();

                // 字符串参数直接传递（SQLite 自动处理类型）
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
            other => Err(DriverError::InvalidField(format!(
                "SQLite 查询仅支持 SQL，收到: {:?}",
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
                let affected = self
                    .conn
                    .execute(&statement, [])
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
            UpdateReq::Sql { stmt: statement } => {
                validate_stmt(&statement)?;
                let affected = self
                    .conn
                    .execute(&statement, [])
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
            DeleteReq::Sql { stmt: statement } => {
                validate_stmt(&statement)?;
                let affected = self
                    .conn
                    .execute(&statement, [])
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

// ==================== SQL 构建器实现 ====================

/// SQLite 查询构建器
pub struct SQLiteBuilder;

impl QueryBuilder for SQLiteBuilder {
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
            (None, Some(o)) => format!("LIMIT -1 OFFSET {}", o), // SQLite uses -1 for unlimited
            (None, None) => String::new(),
        }
    }

    fn escape_identifier(
        &self,
        identifier: &str,
    ) -> String {
        // SQLite supports both double quotes and square brackets
        format!("\"{}\"", identifier.replace('"', "\"\""))
    }

    fn placeholder(
        &self,
        _index: usize,
    ) -> String {
        "?".to_string()
    }
}
