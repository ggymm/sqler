use crate::context::{Connection, Context, DbKind, RUNTIME};
use crate::error::*;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::os::raw::{c_int};
use std::ptr;

#[no_mangle]
pub extern "C" fn sqler_version() -> *mut c_char {
    cstring_out(env!("CARGO_PKG_VERSION"))
}

#[no_mangle]
pub extern "C" fn sqler_new() -> *mut Context {
    Box::into_raw(Box::new(Context::new()))
}

#[no_mangle]
pub extern "C" fn sqler_free(ctx: *mut Context) {
    if ctx.is_null() { return; }
    unsafe { drop(Box::from_raw(ctx)); }
}

#[no_mangle]
pub extern "C" fn sqler_last_error(ctx: *mut Context) -> *mut c_char {
    if ctx.is_null() { return std::ptr::null_mut(); }
    let ctx = unsafe { &*ctx };
    if let Some(e) = ctx.take_error() {
        cstring_out(&e)
    } else {
        std::ptr::null_mut()
    }
}

#[no_mangle]
pub extern "C" fn sqler_string_free(ptr: *mut c_char) {
    if ptr.is_null() { return; }
    unsafe { drop(CString::from_raw(ptr)); }
}

#[no_mangle]
pub extern "C" fn sqler_connect(ctx: *mut Context, kind: u32, uri: *const c_char) -> u64 {
    if ctx.is_null() || uri.is_null() {
        return 0;
    }
    let ctx = unsafe { &*ctx };
    let kind = match DbKind::from_u32(kind) { Some(k) => k, None => return 0 };
    let uri = match cstr_in(uri) { Ok(s) => s, Err(_) => return 0 };

    match kind {
        DbKind::Postgres | DbKind::Mysql | DbKind::Sqlite => {
            #[cfg(feature = "with-sqlx")]
            {
                let res = RUNTIME.block_on(async {
                    crate::sql_any::connect_any(&uri).await
                });
                match res {
                    Ok(pool) => ctx.insert_conn(Connection::Sqlx(pool)),
                    Err(e) => { ctx.set_error(e.to_string()); 0 }
                }
            }
            #[cfg(not(feature = "with-sqlx"))]
            { ctx.set_error("sqlx backend not enabled"); 0 }
        }
        DbKind::MongoDb => {
            #[cfg(feature = "with-mongodb")]
            {
                let res = RUNTIME.block_on(async { crate::mongo::connect(&uri).await });
                match res {
                    Ok((client, db_name)) => ctx.insert_conn(Connection::Mongo { client, db_name }),
                    Err(e) => { ctx.set_error(e.to_string()); 0 }
                }
            }
            #[cfg(not(feature = "with-mongodb"))]
            { ctx.set_error("mongodb backend not enabled"); 0 }
        }
        DbKind::Redis => {
            #[cfg(feature = "with-redis")]
            {
                match redis::Client::open(uri) {
                    Ok(client) => ctx.insert_conn(Connection::Redis(client)),
                    Err(e) => { ctx.set_error(e.to_string()); 0 }
                }
            }
            #[cfg(not(feature = "with-redis"))]
            { ctx.set_error("redis backend not enabled"); 0 }
        }
        DbKind::SqlServer => {
            #[cfg(feature = "with-mssql")]
            { ctx.set_error("mssql backend not fully implemented yet"); 0 }
            #[cfg(not(feature = "with-mssql"))]
            { ctx.set_error("mssql backend not enabled"); 0 }
        }
        DbKind::Oracle => {
            #[cfg(feature = "with-oracle")]
            {
                match oracle::Connection::connect_from_str(&uri) {
                    Ok(conn) => ctx.insert_conn(Connection::Oracle(conn)),
                    Err(e) => { ctx.set_error(e.to_string()); 0 }
                }
            }
            #[cfg(not(feature = "with-oracle"))]
            { ctx.set_error("oracle backend not enabled"); 0 }
        }
    }
}

#[no_mangle]
pub extern "C" fn sqler_disconnect(ctx: *mut Context, id: u64) -> i32 {
    if ctx.is_null() { return -1; }
    let ctx = unsafe { &*ctx };
    ctx.remove_conn(id);
    0
}

#[no_mangle]
pub extern "C" fn sqler_run_sql(ctx: *mut Context, id: u64, sql: *const c_char) -> *mut c_char {
    if ctx.is_null() || sql.is_null() { return std::ptr::null_mut(); }
    let ctx = unsafe { &*ctx };
    let sql = match cstr_in(sql) { Ok(s) => s, Err(_) => return std::ptr::null_mut() };

    let Some(cref) = ctx.get_conn(id) else { ctx.set_error("connection not found"); return std::ptr::null_mut(); };

    // SQLx backends (Postgres/MySQL)
    #[cfg(feature = "with-sqlx")]
    if let Some(json) = cref.with(|c| match c {
        Connection::Sqlx(pool) => {
            let is_query = looks_like_select(&sql);
            let res = RUNTIME.block_on(async {
                if is_query {
                    match crate::sql_any::query(pool, &sql).await {
                        Ok(q) => serde_json::to_string(&q).unwrap_or("{}".into()),
                        Err(e) => { ctx.set_error(e.to_string()); return String::new(); }
                    }
                } else {
                    match crate::sql_any::execute(pool, &sql).await {
                        Ok(q) => serde_json::to_string(&q).unwrap_or("{}".into()),
                        Err(e) => { ctx.set_error(e.to_string()); return String::new(); }
                    }
                }
            });
            if res.is_empty() { None } else { Some(res) }
        }
        _ => None,
    }).flatten() {
        return cstring_out(&json);
    }

    // No other backends matched

    // Oracle minimal support (execute only)
    #[cfg(feature = "with-oracle")]
    if let Some(json) = cref.with(|c| match c {
        Connection::Oracle(conn) => {
            if looks_like_select(&sql) {
                ctx.set_error("Oracle SELECT via FFI not implemented");
                return None;
            }
            match conn.execute(&sql, &[]) {
                Ok(r) => {
                    let rows = r.rows_affected().unwrap_or(0) as u64;
                    Some(serde_json::to_string(&serde_json::json!({"rows_affected": rows})).unwrap())
                }
                Err(e) => { ctx.set_error(e.to_string()); None }
            }
        }
        _ => None,
    }) { return cstring_out(&json); }

    ctx.set_error("Connection type does not support sqler_run_sql");
    std::ptr::null_mut()
}

// Redis
#[no_mangle]
pub extern "C" fn sqler_redis_set(ctx: *mut Context, id: u64, key: *const c_char, data: *const u8, len: usize) -> i32 {
    if ctx.is_null() || key.is_null() || data.is_null() { return -1; }
    let ctx = unsafe { &*ctx };
    let key = match cstr_in(key) { Ok(s) => s, Err(_) => return -1 };
    let Some(cref) = ctx.get_conn(id) else { ctx.set_error("connection not found"); return -1; };

    #[cfg(feature = "with-redis")]
    if let Some(code) = cref.with(|c| match c {
        Connection::Redis(client) => {
            let slice = unsafe { std::slice::from_raw_parts(data, len) }.to_vec();
            match RUNTIME.block_on(crate::redis_kv::set(client, &key, &slice)) { Ok(_) => 0, Err(e) => { ctx.set_error(e.to_string()); -1 } }
        }
        _ => -1,
    }) { return code; }

    ctx.set_error("Redis backend not available for this connection");
    -1
}

#[no_mangle]
pub extern "C" fn sqler_redis_get(ctx: *mut Context, id: u64, key: *const c_char) -> *mut c_char {
    if ctx.is_null() || key.is_null() { return std::ptr::null_mut(); }
    let ctx = unsafe { &*ctx };
    let key = match cstr_in(key) { Ok(s) => s, Err(_) => return std::ptr::null_mut() };
    let Some(cref) = ctx.get_conn(id) else { ctx.set_error("connection not found"); return std::ptr::null_mut(); };

    #[cfg(feature = "with-redis")]
    if let Some(json) = cref.with(|c| match c {
        Connection::Redis(client) => {
            match RUNTIME.block_on(crate::redis_kv::get(client, &key)) {
                Ok(Some(bytes)) => {
                    use base64::Engine as _;
                    let b64 = base64::engine::general_purpose::STANDARD.encode(bytes);
                    Some(serde_json::to_string(&serde_json::json!({"data_base64": b64})).unwrap())
                }
                Ok(None) => Some("null".into()),
                Err(e) => { ctx.set_error(e.to_string()); None }
            }
        }
        _ => None,
    }).flatten() { return cstring_out(&json); }

    ctx.set_error("Redis backend not available for this connection");
    std::ptr::null_mut()
}

// MongoDB
#[no_mangle]
pub extern "C" fn sqler_mongo_insert_one(ctx: *mut Context, id: u64, coll: *const c_char, json_doc: *const c_char) -> *mut c_char {
    if ctx.is_null() || coll.is_null() || json_doc.is_null() { return std::ptr::null_mut(); }
    let ctx = unsafe { &*ctx };
    let coll = match cstr_in(coll) { Ok(s) => s, Err(_) => return std::ptr::null_mut() };
    let json = match cstr_in(json_doc) { Ok(s) => s, Err(_) => return std::ptr::null_mut() };

    let Some(cref) = ctx.get_conn(id) else { ctx.set_error("connection not found"); return std::ptr::null_mut(); };

    #[cfg(feature = "with-mongodb")]
    if let Some(json_out) = cref.with(|c| match c {
        Connection::Mongo { client, db_name } => {
            let db = client.database(db_name);
            match RUNTIME.block_on(crate::mongo::insert_one(&db, &coll, &json)) {
                Ok(id_json) => Some(id_json),
                Err(e) => { ctx.set_error(e.to_string()); None }
            }
        }
        _ => None,
    }).flatten() { return cstring_out(&json_out); }

    ctx.set_error("Mongo backend not available for this connection");
    std::ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn sqler_mongo_find_one(ctx: *mut Context, id: u64, coll: *const c_char, json_filter: *const c_char) -> *mut c_char {
    if ctx.is_null() || coll.is_null() || json_filter.is_null() { return std::ptr::null_mut(); }
    let ctx = unsafe { &*ctx };
    let coll = match cstr_in(coll) { Ok(s) => s, Err(_) => return std::ptr::null_mut() };
    let json = match cstr_in(json_filter) { Ok(s) => s, Err(_) => return std::ptr::null_mut() };

    let Some(cref) = ctx.get_conn(id) else { ctx.set_error("connection not found"); return std::ptr::null_mut(); };

    #[cfg(feature = "with-mongodb")]
    if let Some(json_out) = cref.with(|c| match c {
        Connection::Mongo { client, db_name } => {
            let db = client.database(db_name);
            match RUNTIME.block_on(crate::mongo::find_one(&db, &coll, &json)) {
                Ok(doc_json) => Some(doc_json),
                Err(e) => { ctx.set_error(e.to_string()); None }
            }
        }
        _ => None,
    }).flatten() { return cstring_out(&json_out); }

    ctx.set_error("Mongo backend not available for this connection");
    std::ptr::null_mut()
}

fn cstring_out<S: AsRef<str>>(s: S) -> *mut c_char {
    CString::new(s.as_ref()).unwrap_or_else(|_| CString::new("").unwrap()).into_raw()
}

fn cstr_in<'a>(ptr: *const c_char) -> Result<String> {
    let c = unsafe { CStr::from_ptr(ptr) };
    Ok(c.to_str().map_err(|_| Error::InvalidInput("invalid UTF-8".into()))?.to_string())
}

fn looks_like_select(sql: &str) -> bool {
    sql.trim_start().to_ascii_lowercase().starts_with("select ")
}

// ========================= Export (merged from sqler-export) =========================

#[no_mangle]
pub extern "C" fn sqler_export_version() -> *const c_char {
    static mut VERSION_CSTR: *const c_char = ptr::null();
    unsafe {
        if VERSION_CSTR.is_null() {
            VERSION_CSTR = CString::new(env!("CARGO_PKG_VERSION")).unwrap().into_raw();
        }
        VERSION_CSTR
    }
}

/// Initialize a simple env logger. Safe to call multiple times.
#[no_mangle]
pub extern "C" fn sqler_export_init_logger() {
    let _ = env_logger::builder().is_test(false).try_init();
}

#[no_mangle]
pub unsafe extern "C" fn sqler_export_run_json(
    db_config_json: *const c_char,
    export_options_json: *const c_char,
    out_err: *mut *mut c_char,
) -> c_int {
    use crate::export::{export_database, DbConfig, ExportOptions, ExportError};
    let result = (|| -> std::result::Result<(), ExportError> {
        if db_config_json.is_null() || export_options_json.is_null() {
            return Err(ExportError::InvalidInput("null pointer passed for JSON".into()));
        }
        let cfg_str = CStr::from_ptr(db_config_json)
            .to_str()
            .map_err(|_| ExportError::InvalidInput("invalid UTF-8 in db_config_json".into()))?;
        let opts_str = CStr::from_ptr(export_options_json)
            .to_str()
            .map_err(|_| ExportError::InvalidInput("invalid UTF-8 in export_options_json".into()))?;

        let cfg: DbConfig = serde_json::from_str(cfg_str)
            .map_err(|e| ExportError::InvalidInput(format!("parse db_config_json failed: {}", e)))?;
        let opts: ExportOptions = serde_json::from_str(opts_str)
            .map_err(|e| ExportError::InvalidInput(format!("parse export_options_json failed: {}", e)))?;

        export_database(&cfg, &opts)
    })();

    match result {
        Ok(()) => 0,
        Err(e) => {
            if !out_err.is_null() {
                let msg = format!("{}", e);
                let cmsg = CString::new(msg).unwrap_or_else(|_| CString::new("error").unwrap());
                *out_err = cmsg.into_raw();
            }
            1
        }
    }
}

/// Free a C string allocated by this library (alias)
#[no_mangle]
pub extern "C" fn sqler_export_free_string(s: *mut c_char) {
    sqler_string_free(s)
}

// ----- C-struct based export API -----

#[repr(C)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SqlerDbType {
    Sqlite = 0,
    Postgres = 1,
    Mysql = 2,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SqlerExportFormat {
    Csv = 0,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SqlerDbConfig {
    pub db_type: SqlerDbType,
    // SQLite
    pub sqlite_path: *const c_char,
    // Network DBs
    pub host: *const c_char,
    pub port: u16,
    pub database: *const c_char,
    pub username: *const c_char,
    pub password: *const c_char,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SqlerExportOptions {
    pub query: *const c_char,
    pub table: *const c_char,
    pub output_path: *const c_char,
    pub format: SqlerExportFormat,
    pub include_headers: u8,
    pub delimiter: c_char,
    pub null_text: *const c_char,
}

unsafe fn cstr_opt_utf8(ptr: *const c_char, field: &str) -> std::result::Result<Option<String>, String> {
    if ptr.is_null() {
        return Ok(None);
    }
    let s = CStr::from_ptr(ptr).to_str().map_err(|_| format!("invalid UTF-8 in {}", field))?;
    Ok(Some(s.to_string()))
}

unsafe fn cstr_req_utf8(ptr: *const c_char, field: &str) -> std::result::Result<String, String> {
    cstr_opt_utf8(ptr, field)?.ok_or_else(|| format!("{} is required", field))
}

fn map_db_type(t: SqlerDbType) -> crate::export::DbType {
    match t {
        SqlerDbType::Sqlite => crate::export::DbType::Sqlite,
        SqlerDbType::Postgres => crate::export::DbType::Postgres,
        SqlerDbType::Mysql => crate::export::DbType::Mysql,
    }
}

fn map_format(f: SqlerExportFormat) -> crate::export::ExportFormat {
    match f {
        SqlerExportFormat::Csv => crate::export::ExportFormat::Csv,
    }
}

unsafe fn convert_cfg(c: *const SqlerDbConfig) -> std::result::Result<crate::export::DbConfig, String> {
    if c.is_null() { return Err("cfg is NULL".into()); }
    let c = &*c;
    let db_type = map_db_type(c.db_type);
    let sqlite_path = cstr_opt_utf8(c.sqlite_path, "sqlite_path")?;
    let host = cstr_opt_utf8(c.host, "host")?;
    let database = cstr_opt_utf8(c.database, "database")?;
    let username = cstr_opt_utf8(c.username, "username")?;
    let password = cstr_opt_utf8(c.password, "password")?;

    Ok(crate::export::DbConfig {
        db_type,
        sqlite_path: sqlite_path.map(Into::into),
        host,
        port: if c.port == 0 { None } else { Some(c.port) },
        database,
        username,
        password,
        params: None,
    })
}

unsafe fn convert_opts(c: *const SqlerExportOptions) -> std::result::Result<crate::export::ExportOptions, String> {
    if c.is_null() { return Err("opts is NULL".into()); }
    let c = &*c;
    let query = cstr_opt_utf8(c.query, "query")?;
    let table = cstr_opt_utf8(c.table, "table")?;
    let output_path = cstr_req_utf8(c.output_path, "output_path")?;
    let null_text = cstr_opt_utf8(c.null_text, "null_text")?;
    let delimiter_u8 = c.delimiter as u8;
    let delimiter = delimiter_u8 as char;

    Ok(crate::export::ExportOptions {
        query,
        table,
        output_path: output_path.into(),
        format: map_format(c.format),
        include_headers: c.include_headers != 0,
        delimiter,
        null_text,
    })
}

#[no_mangle]
pub unsafe extern "C" fn sqler_export_run(
    cfg: *const SqlerDbConfig,
    opts: *const SqlerExportOptions,
    out_err: *mut *mut c_char,
) -> c_int {
    use crate::export::export_database;
    let result = (|| -> std::result::Result<(), String> {
        let cfg = convert_cfg(cfg)?;
        let opts = convert_opts(opts)?;
        export_database(&cfg, &opts).map_err(|e| format!("{}", e))
    })();

    match result {
        Ok(()) => 0,
        Err(e) => {
            if !out_err.is_null() {
                let cmsg = CString::new(e).unwrap_or_else(|_| CString::new("error").unwrap());
                *out_err = cmsg.into_raw();
            }
            1
        }
    }
}

// Optional convenience: export a SQL query to CSV using an existing connection (sqlx backends)
#[no_mangle]
pub extern "C" fn sqler_export_query_to_csv(
    ctx: *mut Context,
    id: u64,
    query: *const c_char,
    output_path: *const c_char,
    include_headers: u8,
    delimiter: u8,
) -> c_int {
    if ctx.is_null() || query.is_null() || output_path.is_null() { return -1; }
    let ctx = unsafe { &*ctx };
    let query = match cstr_in(query) { Ok(s) => s, Err(_) => return -1 };
    let path = match cstr_in(output_path) { Ok(s) => s, Err(_) => return -1 };
    let Some(cref) = ctx.get_conn(id) else { ctx.set_error("connection not found"); return -1; };

    #[cfg(feature = "with-sqlx")]
    if let Some(code) = cref.with(|c| match c {
        Connection::Sqlx(pool) => {
            let res = RUNTIME.block_on(async { crate::sql_any::query(pool, &query).await });
            match res {
                Ok(q) => {
                    let file = match std::fs::File::create(&path) { Ok(f) => f, Err(e) => { ctx.set_error(e.to_string()); return -1; } };
                    let mut wtr = csv::WriterBuilder::new()
                        .has_headers(false)
                        .delimiter(delimiter)
                        .from_writer(file);
                    if include_headers != 0 {
                        if let Err(e) = wtr.write_record(&q.columns) { ctx.set_error(e.to_string()); return -1; }
                    }
                    for row in q.rows {
                        if let Err(e) = wtr.write_record(row) { ctx.set_error(e.to_string()); return -1; }
                    }
                    if let Err(e) = wtr.flush() { ctx.set_error(e.to_string()); return -1; }
                    0
                }
                Err(e) => { ctx.set_error(e.to_string()); -1 }
            }
        }
        _ => -1,
    }) { return code; }

    ctx.set_error("Connection type does not support CSV export");
    -1
}
