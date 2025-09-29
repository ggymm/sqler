use crate::error::*;
use crate::types::{ExecResult, QueryResult};
use sqlx::AnyPool;
use sqlx::any::{AnyPoolOptions, AnyRow};
use sqlx::{Row, Column};

pub async fn connect_any(uri: &str) -> Result<AnyPool> {
    let pool = AnyPoolOptions::new()
        .max_connections(5)
        .connect(uri)
        .await?;
    Ok(pool)
}

pub async fn execute(pool: &AnyPool, sql: &str) -> Result<ExecResult> {
    let done = sqlx::query(sql).execute(pool).await?;
    Ok(ExecResult { rows_affected: done.rows_affected() })
}

pub async fn query(pool: &AnyPool, sql: &str) -> Result<QueryResult> {
    let rows: Vec<AnyRow> = sqlx::query(sql).fetch_all(pool).await?;
    let (columns, data) = rows_to_strings(rows);
    Ok(QueryResult { columns, rows: data })
}

fn rows_to_strings(rows: Vec<AnyRow>) -> (Vec<String>, Vec<Vec<String>>) {
    if rows.is_empty() {
        return (Vec::new(), Vec::new());
    }
    let first = &rows[0];
    let columns: Vec<String> = first
        .columns()
        .iter()
        .map(|c| c.name().to_string())
        .collect();

    let mut all_rows = Vec::with_capacity(rows.len());
    for row in rows {
        let mut out = Vec::with_capacity(columns.len());
        for i in 0..columns.len() {
            let s = cell_to_string(&row, i);
            out.push(s);
        }
        all_rows.push(out);
    }
    (columns, all_rows)
}

fn cell_to_string(row: &AnyRow, idx: usize) -> String {
    // Attempt common types; fallback to debug
    macro_rules! try_get {
        ($t:ty, $row:expr, $idx:expr) => {
            match $row.try_get::<$t, _>($idx) {
                Ok(v) => return v.to_string(),
                Err(_) => {}
            }
        };
    }
    try_get!(String, row, idx);
    try_get!(i64, row, idx);
    try_get!(i32, row, idx);
    try_get!(i16, row, idx);
    try_get!(f64, row, idx);
    try_get!(f32, row, idx);
    try_get!(bool, row, idx);
    match row.try_get::<Vec<u8>, _>(idx) {
        Ok(bytes) => {
            use base64::Engine as _;
            base64::engine::general_purpose::STANDARD.encode(bytes)
        }
        Err(_) => {
            // Last resort
            format!("<unrepr>")
        }
    }
}
