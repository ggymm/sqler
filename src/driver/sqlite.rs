use std::{collections::HashMap, fs, path::Path};

use rusqlite::{types::ValueRef, Connection, OpenFlags};

use super::{
    validate_stmt, DatabaseDriver, DatabaseSession, DeleteReq, DriverError, InsertReq, QueryReq, QueryResp, UpdateReq,
    WriteResp,
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
            InsertReq::Sql { stmt: statement } => self.exec_write(&statement),
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
            UpdateReq::Sql { stmt: statement } => self.exec_write(&statement),
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
            DeleteReq::Sql { stmt: statement } => self.exec_write(&statement),
            other => Err(DriverError::InvalidField(format!(
                "SQLite 删除仅支持 SQL，收到: {:?}",
                other
            ))),
        }
    }
}

impl SQLiteConnection {
    fn exec_write(
        &mut self,
        statement: &str,
    ) -> Result<WriteResp, DriverError> {
        validate_stmt(statement)?;
        let affected = self
            .conn
            .execute(statement, [])
            .map_err(|err| DriverError::Other(format!("执行写入失败: {}", err)))?;
        Ok(WriteResp {
            affected: affected as u64,
        })
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
    let path_str = config.file_path.trim();
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
