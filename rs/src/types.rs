use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ExecResult {
    pub rows_affected: u64,
}

#[derive(Debug, Serialize)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>, // stringified cells for portability
}

