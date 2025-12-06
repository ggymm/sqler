use std::collections::HashMap;

use postgres::{Client, Config, Error as PostgresError, NoTls, fallible_iterator::FallibleIterator, types::Type};

use crate::model::{ColumnInfo, ColumnKind, PostgresOptions, TableInfo};

use super::{
    DatabaseDriver, DatabaseSession, DriverError, ExecReq, ExecResp, Operator, QueryReq, QueryResp, ValueCond,
    escape_quote, validate_sql,
};

#[derive(Debug, Clone, Copy)]
pub struct PostgresDriver;

impl DatabaseDriver for PostgresDriver {
    type Config = PostgresOptions;

    fn supp_kinds(&self) -> Vec<ColumnKind> {
        vec![
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
            ColumnKind::Date,
            ColumnKind::Time,
            ColumnKind::Timestamp,
            ColumnKind::Boolean,
            ColumnKind::Json,
            ColumnKind::Uuid,
            ColumnKind::Array,
        ]
    }

    fn check_connection(
        &self,
        config: &Self::Config,
    ) -> Result<(), DriverError> {
        let mut client = open_conn(config)?;
        client
            .simple_query("SELECT 1")
            .map_err(|err| DriverError::Other(format!("校验查询失败: {}", err)))?;
        Ok(())
    }

    fn create_connection(
        &self,
        config: &Self::Config,
    ) -> Result<Box<dyn DatabaseSession>, DriverError> {
        let client = open_conn(config)?;
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
    fn exec(
        &mut self,
        req: ExecReq,
    ) -> Result<ExecResp, DriverError> {
        match req {
            ExecReq::Sql { sql } => {
                validate_sql(&sql)?;
                let affected = self
                    .client
                    .execute(&sql, &[])
                    .map_err(|err| DriverError::Other(format!("执行失败: {}", err)))?;
                Ok(ExecResp { affected })
            }
            other => Err(DriverError::InvalidField(format!(
                "PostgreSQL 仅支持 SQL，收到: {:?}",
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
                let mut param_index = 1;

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
                                escape_quote(&ord.field),
                                if ord.ascending { "ASC" } else { "DESC" }
                            )
                        })
                        .collect();
                    sql.push_str(&format!(" ORDER BY {}", order_clauses.join(", ")));
                }

                // 分页子句 - PostgreSQL 建议使用 OFFSET 时必须有 ORDER BY
                if let Some(page) = paging {
                    if orders.is_empty() {
                        tracing::warn!("PostgreSQL: 使用 OFFSET 但没有 ORDER BY，结果顺序可能不确定");
                    }
                    sql.push_str(&format!(" LIMIT {} OFFSET {}", page.limit(), page.offset()));
                }

                (sql, params)
            }
            other => {
                return Err(DriverError::InvalidField(format!(
                    "PostgreSQL 查询仅支持 SQL 和 Builder，收到: {:?}",
                    other
                )));
            }
        };

        let param_refs: Vec<&(dyn postgres::types::ToSql + Sync)> = params
            .iter()
            .map(|s| s as &(dyn postgres::types::ToSql + Sync))
            .collect();

        let mut iter = self
            .client
            .query_raw(&sql, param_refs)
            .map_err(|err| DriverError::Other(format!("执行查询失败: {}", err)))?;

        let mut columns = vec![];
        let mut records = vec![];
        let mut count = 0;
        while let Some(row) = iter
            .next()
            .map_err(|err| DriverError::Other(format!("读取结果失败: {}", err)))?
        {
            if count >= 1000 {
                break;
            }

            // 从第一行提取列名
            if count == 0 {
                columns = row.columns().iter().map(|col| col.name().to_string()).collect();
            }

            let mut record = HashMap::with_capacity(row.len());
            for (idx, column) in row.columns().iter().enumerate() {
                let value = parse_value(&row, idx)?;
                record.insert(column.name().to_string(), value);
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
        let sql = "SELECT
            t.tablename,
            s.n_live_tup,
            pg_total_relation_size(quote_ident(t.schemaname)||'.'||quote_ident(t.tablename))
        FROM pg_tables t
        LEFT JOIN pg_stat_user_tables s ON t.tablename = s.relname
        WHERE t.schemaname = 'public'";
        let rows = self
            .client
            .query(sql, &[])
            .map_err(|err| DriverError::Other(format!("查询表列表失败: {}", err)))?;

        let mut tables = vec![];
        for row in rows {
            let name: String = row.get(0);
            let row_count: Option<i64> = row.get(1);
            let size_bytes: Option<i64> = row.get(2);

            tables.push(TableInfo {
                name,
                row_count: row_count.map(|n| n as u64),
                size_bytes: size_bytes.map(|n| n as u64),
                last_accessed: None,
            });
        }
        Ok(tables)
    }

    fn columns(
        &mut self,
        table: &str,
    ) -> Result<Vec<ColumnInfo>, DriverError> {
        let sql = "SELECT
            c.column_name,
            c.data_type,
            c.is_nullable,
            c.column_default,
            c.character_maximum_length,
            CASE WHEN pk.column_name IS NOT NULL THEN true ELSE false END as is_primary_key,
            COALESCE(pgd.description, '') as comment
        FROM information_schema.columns c
        LEFT JOIN (
            SELECT ku.column_name
            FROM information_schema.table_constraints tc
            JOIN information_schema.key_column_usage ku
                ON tc.constraint_name = ku.constraint_name
                AND tc.table_schema = ku.table_schema
            WHERE tc.constraint_type = 'PRIMARY KEY'
                AND tc.table_schema = 'public'
                AND tc.table_name = $1
        ) pk ON c.column_name = pk.column_name
        LEFT JOIN pg_catalog.pg_statio_all_tables st
            ON st.schemaname = c.table_schema AND st.relname = c.table_name
        LEFT JOIN pg_catalog.pg_description pgd
            ON pgd.objoid = st.relid
            AND pgd.objsubid = c.ordinal_position
        WHERE c.table_schema = 'public' AND c.table_name = $1
        ORDER BY c.ordinal_position";

        let rows = self
            .client
            .query(sql, &[&table])
            .map_err(|err| DriverError::Other(format!("查询列信息失败: {}", err)))?;

        let mut columns = vec![];
        for row in rows {
            let name: String = row.get(0);
            let data_type: String = row.get(1);
            let is_nullable: String = row.get(2);
            let nullable = is_nullable.to_uppercase() == "YES";
            let default_value: Option<String> = row.get(3);
            let max_length: Option<i32> = row.get(4);
            let is_primary_key: bool = row.get(5);
            let comment: String = row.get(6);

            let auto_increment = default_value
                .as_ref()
                .map(|d| d.starts_with("nextval("))
                .unwrap_or(false);

            columns.push(ColumnInfo {
                name,
                kind: data_type,
                comment,
                nullable,
                primary_key: is_primary_key,
                default_value: default_value.unwrap_or_default(),
                max_length: max_length.map(|n| n as u64).unwrap_or(0),
                auto_increment,
            });
        }
        Ok(columns)
    }
}

fn open_conn(config: &PostgresOptions) -> Result<Client, DriverError> {
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
    if config.use_tls {
        return Err(DriverError::Other("PostgreSQL 暂未支持 TLS 连接".into()));
    }

    let mut pg_config = Config::new();
    pg_config.host(config.host.trim());
    pg_config.port(config.port.parse().unwrap_or(5432));
    pg_config.user(config.username.trim());
    pg_config.password(config.password.as_str());
    pg_config.dbname(config.database.trim());

    let client = pg_config
        .connect(NoTls)
        .map_err(|err| DriverError::Other(format!("连接失败: {}", err)))?;
    Ok(client)
}

fn parse_value(
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

fn format_columns(columns: &[String]) -> String {
    if columns.is_empty() {
        return "*".to_string();
    }

    // PostgreSQL 保留关键字列表
    #[rustfmt::skip]
    let keywords = [
        "SELECT", "FROM", "WHERE", "INSERT", "UPDATE", "DELETE",
        "CREATE", "DROP", "ALTER", "TABLE", "INDEX", "VIEW",
        "JOIN", "LEFT", "RIGHT", "INNER", "OUTER", "ON",
        "GROUP", "ORDER", "BY", "HAVING", "LIMIT", "OFFSET",
        "AS", "AND", "OR", "NOT", "IN", "IS", "NULL",
        "PRIMARY", "KEY", "FOREIGN", "REFERENCES", "CONSTRAINT",
        "DEFAULT", "UNIQUE", "CHECK",
        "COUNT", "SUM", "AVG", "MAX", "MIN",
        "DISTINCT", "ALL", "BETWEEN", "LIKE", "EXISTS",
        "CASE", "WHEN", "THEN", "ELSE", "END",
        "UNION", "INTERSECT", "EXCEPT",
        "INT", "VARCHAR", "TEXT", "DATE", "TIMESTAMP",
        "CHAR", "DECIMAL", "FLOAT", "DOUBLE", "BOOLEAN",
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

fn map_pg_err(err: PostgresError) -> DriverError {
    DriverError::Other(format!("PostgreSQL 解析字段失败: {}", err))
}
