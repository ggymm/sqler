use std::{collections::HashMap, fs, path::Path};

use rusqlite::{Connection, OpenFlags, types::ValueRef};

use crate::{ColumnInfo, ColumnKind, SQLiteOptions, TableInfo};

use super::{
    DatabaseDriver, DatabaseSession, DriverError, ExecReq, ExecResp, Operator, QueryReq, QueryResp, ValueCond,
    escape_quote, validate_sql,
};

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
    fn exec(
        &mut self,
        req: ExecReq,
    ) -> Result<ExecResp, DriverError> {
        match req {
            ExecReq::Sql { sql } => {
                validate_sql(&sql)?;
                let affected = self
                    .conn
                    .execute(&sql, [])
                    .map_err(|err| DriverError::Other(format!("执行失败: {}", err)))?;
                Ok(ExecResp {
                    affected: affected as u64,
                })
            }
            other => Err(DriverError::InvalidField(format!(
                "SQLite 仅支持 SQL，收到: {:?}",
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
                (sql, args)
            }
            QueryReq::Builder {
                table,
                columns,
                paging,
                orders,
                filters,
            } => {
                let mut sql = format!("SELECT {} FROM \"{}\"", format_columns(&columns), escape_quote(&table));
                let mut params = vec![];

                if !filters.is_empty() {
                    let mut clauses = vec![];
                    for filter in &filters {
                        let field = format!("\"{}\"", escape_quote(&filter.field));
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
                                escape_quote(&ord.field),
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
                    "SQLite 查询仅支持 SQL 和 Builder，收到: {:?}",
                    other
                )));
            }
        };

        let mut stmt = self
            .conn
            .prepare(&sql)
            .map_err(|err| DriverError::Other(format!("准备查询失败: {}", err)))?;
        let columns = stmt.column_names().iter().map(|s| s.to_string()).collect::<Vec<_>>();

        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|s| s as &dyn rusqlite::ToSql).collect();

        let mut rows = stmt
            .query(&param_refs[..])
            .map_err(|err| DriverError::Other(format!("执行查询失败: {}", err)))?;

        let mut records = vec![];
        let mut count = 0;
        while let Some(row) = rows
            .next()
            .map_err(|err| DriverError::Other(format!("读取结果失败: {}", err)))?
        {
            if count >= 1000 {
                break;
            }

            let mut record = HashMap::with_capacity(columns.len());
            for (idx, name) in columns.iter().enumerate() {
                let value = row
                    .get_ref(idx)
                    .map_err(|err| DriverError::Other(format!("读取列 {name} 失败: {}", err)))?;
                record.insert(name.clone(), parse_value(value));
            }
            records.push(record);
            count += 1;
        }

        Ok(QueryResp::Rows {
            cols: columns,
            rows: records,
        })
    }

    fn tables(&mut self) -> Result<Vec<TableInfo>, DriverError> {
        let sql = "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name";
        let mut stmt = self
            .conn
            .prepare(sql)
            .map_err(|err| DriverError::Other(format!("查询表列表失败: {}", err)))?;

        let mut tables = vec![];
        let rows = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|err| DriverError::Other(format!("查询表列表失败: {}", err)))?;

        for row in rows {
            let name = row.map_err(|err| DriverError::Other(format!("读取表名失败: {}", err)))?;

            let row_count = self
                .conn
                .query_row(
                    &format!("SELECT COUNT(*) FROM \"{}\"", escape_quote(&name)),
                    [],
                    |row| row.get::<_, i64>(0),
                )
                .ok()
                .map(|n| n as u64);

            tables.push(TableInfo {
                name,
                row_count,
                size_bytes: None,
                last_accessed: None,
            });
        }
        Ok(tables)
    }

    fn columns(
        &mut self,
        table: &str,
    ) -> Result<Vec<ColumnInfo>, DriverError> {
        let sql = format!("PRAGMA table_info(\"{}\")", escape_quote(table));
        let mut stmt = self
            .conn
            .prepare(&sql)
            .map_err(|err| DriverError::Other(format!("查询列信息失败: {}", err)))?;

        let mut columns = vec![];
        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i32>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, i32>(5)?,
                ))
            })
            .map_err(|err| DriverError::Other(format!("查询列信息失败: {}", err)))?;

        for row in rows {
            let (name, data_type, notnull, default_value, pk) =
                row.map_err(|err| DriverError::Other(format!("读取列信息失败: {}", err)))?;

            columns.push(ColumnInfo {
                name,
                kind: data_type,
                comment: String::new(),
                nullable: notnull == 0,
                primary_key: pk > 0,
                default_value: default_value.unwrap_or_default(),
                max_length: 0,
                auto_increment: false,
            });
        }
        Ok(columns)
    }
}

impl DatabaseDriver for SQLiteDriver {
    type Config = SQLiteOptions;

    fn supp_kinds(&self) -> Vec<ColumnKind> {
        vec![
            ColumnKind::Int,
            ColumnKind::BigInt,
            ColumnKind::Float,
            ColumnKind::Double,
            ColumnKind::Char,
            ColumnKind::VarChar,
            ColumnKind::Text,
            ColumnKind::Blob,
            ColumnKind::Date,
            ColumnKind::DateTime,
            ColumnKind::Boolean,
        ]
    }

    fn check_connection(
        &self,
        config: &Self::Config,
    ) -> Result<(), DriverError> {
        let conn = open_conn(config)?;
        conn.query_row("SELECT 1", [], |_| Ok::<_, rusqlite::Error>(()))
            .map_err(|err| DriverError::Other(format!("校验查询失败: {}", err)))?;
        Ok(())
    }

    fn create_connection(
        &self,
        config: &Self::Config,
    ) -> Result<Box<dyn DatabaseSession>, DriverError> {
        let conn = open_conn(config)?;
        Ok(Box::new(SQLiteConnection::new(conn)))
    }
}

fn open_conn(config: &SQLiteOptions) -> Result<Connection, DriverError> {
    let path_str = config.filepath.trim();
    if path_str.is_empty() {
        return Err(DriverError::MissingField("file_path".into()));
    }

    let path = Path::new(path_str);

    if config.readonly {
        if !path.exists() {
            return Err(DriverError::InvalidField("file_path 不存在".into()));
        }
    } else if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|err| DriverError::Other(format!("创建目录失败: {}", err)))?;
        }
    }

    let flags = if config.readonly {
        OpenFlags::SQLITE_OPEN_READ_ONLY
    } else {
        OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE
    };

    Connection::open_with_flags(path, flags).map_err(|err| DriverError::Other(format!("打开 SQLite 失败: {}", err)))
}

fn parse_value(value: ValueRef<'_>) -> String {
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

fn format_columns(columns: &[String]) -> String {
    if columns.is_empty() {
        return "*".to_string();
    }

    // SQLite 保留关键字列表
    #[rustfmt::skip]
    let keywords = [
        "SELECT", "FROM", "WHERE", "INSERT", "UPDATE", "DELETE",
        "CREATE", "DROP", "ALTER", "TABLE", "INDEX", "VIEW",
        "JOIN", "LEFT", "RIGHT", "INNER", "OUTER", "ON",
        "GROUP", "ORDER", "BY", "HAVING", "LIMIT", "OFFSET",
        "AS", "AND", "OR", "NOT", "IN", "IS", "NULL",
        "PRIMARY", "KEY", "FOREIGN", "REFERENCES", "CONSTRAINT",
        "DEFAULT", "UNIQUE", "CHECK", "AUTOINCREMENT",
        "COUNT", "SUM", "AVG", "MAX", "MIN",
        "DISTINCT", "ALL", "BETWEEN", "LIKE", "EXISTS", "GLOB",
        "CASE", "WHEN", "THEN", "ELSE", "END",
        "UNION", "INTERSECT", "EXCEPT",
        "INT", "INTEGER", "TEXT", "REAL", "BLOB",
        "VARCHAR", "DATE", "DATETIME", "TIMESTAMP",
    ];

    columns
        .iter()
        .map(|c| {
            if keywords.contains(&c.trim().to_uppercase().as_str()) {
                format!("\"{}\"", escape_quote(c))
            } else {
                c.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}
