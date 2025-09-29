use crate::export::config::{DbConfig, DbType, ExportFormat, ExportOptions};
use crate::export::error::{ExportError, Result};

pub fn export_database(cfg: &DbConfig, opts: &ExportOptions) -> Result<()> {
    match (&cfg.db_type, &opts.format) {
        (DbType::Sqlite, ExportFormat::Csv) => export_via_sqlx(build_sqlite_uri(cfg)?, opts),
        (DbType::Postgres, ExportFormat::Csv) => {
            let uri = build_pg_uri(cfg)?; export_via_sqlx(uri, opts)
        }
        (DbType::Mysql, ExportFormat::Csv) => {
            let uri = build_mysql_uri(cfg)?; export_via_sqlx(uri, opts)
        }
    }
}

fn export_via_sqlx(uri: String, opts: &ExportOptions) -> Result<()> {
    // Build query
    let query = if let Some(q) = &opts.query {
        q.clone()
    } else if let Some(table) = &opts.table {
        format!("SELECT * FROM {}", table)
    } else {
        return Err(ExportError::InvalidInput("either 'query' or 'table' must be provided".into()));
    };

    // Fetch rows synchronously via global runtime
    let qres = crate::context::RUNTIME.block_on(async move {
        let pool = crate::sql_any::connect_any(&uri).await?;
        let q = crate::sql_any::query(&pool, &query).await?;
        Ok::<_, crate::error::Error>(q)
    }).map_err(|e| ExportError::InvalidInput(e.to_string()))?;

    // Write CSV
    let file = std::fs::File::create(&opts.output_path)?;
    let mut wtr = csv::WriterBuilder::new()
        .has_headers(false)
        .delimiter(opts.delimiter as u8)
        .from_writer(file);
    if opts.include_headers {
        wtr.write_record(&qres.columns)?;
    }
    for row in qres.rows {
        wtr.write_record(row)?;
    }
    wtr.flush()?;
    Ok(())
}

fn build_sqlite_uri(cfg: &DbConfig) -> Result<String> {
    let path = cfg.sqlite_path.as_ref().ok_or_else(|| ExportError::InvalidInput("sqlite_path is required".into()))?;
    Ok(format!("sqlite://{}", path.display()))
}

fn build_pg_uri(cfg: &DbConfig) -> Result<String> {
    let host = cfg.host.as_deref().unwrap_or("localhost");
    let port = cfg.port.unwrap_or(5432);
    let db = cfg.database.as_deref().ok_or_else(|| ExportError::InvalidInput("database is required".into()))?;
    let user = cfg.username.as_deref().unwrap_or("");
    let pass = cfg.password.as_deref().unwrap_or("");
    let auth = if !user.is_empty() { format!("{}:{}@", user, pass) } else { String::new() };
    Ok(format!("postgres://{}{}:{}/{}", auth, host, port, db))
}

fn build_mysql_uri(cfg: &DbConfig) -> Result<String> {
    let host = cfg.host.as_deref().unwrap_or("localhost");
    let port = cfg.port.unwrap_or(3306);
    let db = cfg.database.as_deref().ok_or_else(|| ExportError::InvalidInput("database is required".into()))?;
    let user = cfg.username.as_deref().unwrap_or("");
    let pass = cfg.password.as_deref().unwrap_or("");
    let auth = if !user.is_empty() { format!("{}:{}@", user, pass) } else { String::new() };
    Ok(format!("mysql://{}{}:{}/{}", auth, host, port, db))
}
