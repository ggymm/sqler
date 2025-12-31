use std::{
    collections::HashMap,
    fs::{self, File},
    io::{BufWriter, Write},
    path::Path,
    process,
    time::Instant,
};

use chrono::Utc;

use sqler_core::{ColumnInfo, DatabaseSession, QueryReq, QueryResp};

use crate::DumpConfig;

/// 导出表为 SQL 文件任务
pub fn run(
    session: &mut Box<dyn DatabaseSession>,
    config: &DumpConfig,
) {
    tracing::info!("开始导出任务");
    tracing::debug!(
        "导出配置: table={}, file={}, filter={:?}, batch={}, insert_batch={}, only_schema={}",
        config.table,
        config.file,
        config.filter,
        config.batch,
        config.insert_batch,
        config.only_schema
    );

    // 1. 查询表的列信息
    tracing::info!("分析表结构: {}", config.table);
    let columns = match session.columns(&config.table) {
        Ok(cols) => cols,
        Err(e) => {
            tracing::error!("获取表结构失败: {}", e);
            process::exit(1);
        }
    };

    if columns.is_empty() {
        tracing::error!("表 {} 不存在或没有列", config.table);
        process::exit(1);
    }

    let column_names: Vec<String> = columns.iter().map(|c| c.name.clone()).collect();
    tracing::info!("表结构分析完成，共 {} 列", column_names.len());

    // 2. 创建输出文件（确保父目录存在）
    tracing::info!("准备输出文件: {}", config.file);
    if let Some(parent) = Path::new(&config.file).parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            tracing::error!("创建目录失败: {}", e);
            process::exit(1);
        }
    }

    let file = match File::create(&config.file) {
        Ok(f) => {
            tracing::info!("输出文件创建成功");
            f
        }
        Err(e) => {
            tracing::error!("创建输出文件失败: {}", e);
            process::exit(1);
        }
    };
    let mut writer = BufWriter::new(file);

    // 3. 写入文件头和表结构
    tracing::info!("写入文件头和表结构");
    if let Err(e) = write_file_header(&mut writer, config, &columns) {
        tracing::error!("写入文件头失败: {}", e);
        process::exit(1);
    }

    // 4. 如果仅导出结构，完成并返回
    if config.only_schema {
        tracing::info!("仅导出结构模式，跳过数据导出");
        if let Err(e) = writer.flush() {
            tracing::error!("刷新文件缓冲失败: {}", e);
            process::exit(1);
        }
        tracing::info!("导出完成（仅结构）");
        return;
    }

    // 5. 查询总行数
    tracing::info!("统计总行数");
    let total_rows = query_total_rows(session, config);
    tracing::info!("总行数: {}", total_rows);

    if total_rows == 0 {
        tracing::warn!("表为空，跳过数据导出");
        if let Err(e) = writer.flush() {
            tracing::error!("刷新文件缓冲失败: {}", e);
            process::exit(1);
        }
        tracing::info!("导出完成（仅结构）");
        return;
    }

    // 6. 分页导出数据
    tracing::info!("开始分页导出数据，batch={}", config.batch);
    let batch_size = config.batch;
    let start_time = Instant::now();
    let mut exported_rows = 0u64;
    let mut page = 0;

    loop {
        // 查询一页数据
        let query_sql = build_query_sql(config, &column_names, page, batch_size);
        tracing::debug!("查询第 {} 页数据，offset={}", page, page * batch_size);
        let resp = match session.query(QueryReq::Sql {
            sql: query_sql,
            args: vec![],
        }) {
            Ok(r) => r,
            Err(e) => {
                tracing::error!("查询数据失败: {}", e);
                process::exit(1);
            }
        };

        let rows = match resp {
            QueryResp::Rows { rows, .. } => rows,
            _ => {
                tracing::error!("查询响应格式错误");
                process::exit(1);
            }
        };

        if rows.is_empty() {
            tracing::debug!("第 {} 页无数据，导出结束", page);
            break;
        }

        tracing::debug!("第 {} 页返回 {} 行", page, rows.len());

        // 生成 INSERT 语句
        if let Err(e) = write_insert_statements(&mut writer, &config.table, &column_names, &rows, config.insert_batch) {
            tracing::error!("写入 INSERT 语句失败: {}", e);
            process::exit(1);
        }

        exported_rows += rows.len() as u64;

        // 报告进度
        let percentage = (exported_rows as f64 / total_rows as f64) * 100.0;
        let elapsed = start_time.elapsed().as_secs_f64();
        let speed = exported_rows as f64 / elapsed;
        let estimated_seconds = if speed > 0.0 {
            (total_rows as f64 - exported_rows as f64) / speed
        } else {
            0.0
        };

        tracing::info!(
            "导出进度: {}/{} ({:.1}%), 速度: {:.0} 行/秒, 已用时: {:.1}s, 预计剩余: {:.0}s",
            exported_rows,
            total_rows,
            percentage,
            speed,
            elapsed,
            estimated_seconds
        );

        page += 1;

        // 如果已经导出完所有数据
        if exported_rows >= total_rows {
            tracing::info!("所有数据已导出，共 {} 行", exported_rows);
            break;
        }
    }

    // 7. 刷新缓冲区
    tracing::debug!("刷新文件缓冲区");
    if let Err(e) = writer.flush() {
        tracing::error!("刷新文件缓冲失败: {}", e);
        process::exit(1);
    }

    // 8. 输出完成消息
    let elapsed = start_time.elapsed().as_secs_f64();
    tracing::info!(
        "导出完成，共 {} 行，耗时 {:.1} 秒，速度 {:.0} 行/秒",
        exported_rows,
        elapsed,
        exported_rows as f64 / elapsed
    );
}

/// 查询总行数
fn query_total_rows(
    session: &mut Box<dyn DatabaseSession>,
    config: &DumpConfig,
) -> u64 {
    let count_sql = if let Some(filter) = &config.filter {
        format!("SELECT COUNT(*) as total FROM {} WHERE {}", config.table, filter)
    } else {
        format!("SELECT COUNT(*) as total FROM {}", config.table)
    };

    match session.query(QueryReq::Sql {
        sql: count_sql,
        args: vec![],
    }) {
        Ok(QueryResp::Rows { rows, .. }) => {
            if let Some(row) = rows.first() {
                if let Some(total_str) = row.get("total") {
                    return total_str.parse().unwrap_or(0);
                }
            }
            0
        }
        _ => 0,
    }
}

/// 构建查询 SQL
fn build_query_sql(
    config: &DumpConfig,
    columns: &[String],
    page: usize,
    page_size: usize,
) -> String {
    let cols = columns.join(", ");
    let offset = page * page_size;

    if let Some(filter) = &config.filter {
        format!(
            "SELECT {} FROM {} WHERE {} LIMIT {} OFFSET {}",
            cols, config.table, filter, page_size, offset
        )
    } else {
        format!(
            "SELECT {} FROM {} LIMIT {} OFFSET {}",
            cols, config.table, page_size, offset
        )
    }
}

/// 写入文件头和表结构
fn write_file_header(
    writer: &mut BufWriter<File>,
    config: &DumpConfig,
    columns: &[ColumnInfo],
) -> std::io::Result<()> {
    writeln!(writer, "-- Dump of table: {}", config.table)?;
    writeln!(writer, "-- Generated at: {}", Utc::now().to_rfc3339())?;
    if let Some(filter) = &config.filter {
        writeln!(writer, "-- Filter: {}", filter)?;
    }
    writeln!(writer)?;

    // 生成 CREATE TABLE 语句
    writeln!(writer, "-- Table structure for {}", config.table)?;
    writeln!(writer, "DROP TABLE IF EXISTS `{}`;", config.table)?;
    writeln!(writer, "CREATE TABLE `{}` (", config.table)?;

    for (i, col) in columns.iter().enumerate() {
        let mut line = format!("  `{}` {}", col.name, col.kind);

        // 添加 NOT NULL 约束
        if !col.nullable {
            line.push_str(" NOT NULL");
        }

        // 添加 AUTO_INCREMENT
        if col.auto_increment {
            line.push_str(" AUTO_INCREMENT");
        }

        // 添加默认值
        if !col.default_value.is_empty() && col.default_value != "NULL" {
            line.push_str(&format!(" DEFAULT {}", col.default_value));
        }

        // 添加注释
        if !col.comment.is_empty() {
            line.push_str(&format!(" COMMENT '{}'", col.comment.replace('\'', "''")));
        }

        // 检查是否需要主键，如果有主键则所有列都要加逗号
        let has_primary_key = columns.iter().any(|c| c.primary_key);
        if has_primary_key || i < columns.len() - 1 {
            line.push(',');
        }

        writeln!(writer, "{}", line)?;
    }

    // 添加主键定义
    let primary_keys: Vec<&str> = columns
        .iter()
        .filter(|c| c.primary_key)
        .map(|c| c.name.as_str())
        .collect();

    if !primary_keys.is_empty() {
        writeln!(
            writer,
            "  PRIMARY KEY ({})",
            primary_keys
                .iter()
                .map(|k| format!("`{}`", k))
                .collect::<Vec<_>>()
                .join(", ")
        )?;
    }

    writeln!(writer, ");")?;
    writeln!(writer)?;
    writeln!(writer, "-- Data for table {}", config.table)?;
    writeln!(writer)?;

    Ok(())
}

/// 写入 INSERT 语句（按 insert_batch_size 分批）
fn write_insert_statements(
    writer: &mut BufWriter<File>,
    table: &str,
    columns: &[String],
    rows: &[HashMap<String, String>],
    insert_batch_size: usize,
) -> std::io::Result<()> {
    if rows.is_empty() {
        return Ok(());
    }

    let cols = columns.join(", ");

    // 按 insert_batch_size 分批写入
    for chunk in rows.chunks(insert_batch_size) {
        writeln!(writer, "INSERT INTO {} ({}) VALUES", table, cols)?;

        for (i, row) in chunk.iter().enumerate() {
            let values: Vec<String> = columns
                .iter()
                .map(|col| format_sql_value(row.get(col).map(|s| s.as_str())))
                .collect();

            if i == chunk.len() - 1 {
                writeln!(writer, "  ({});", values.join(", "))?;
            } else {
                writeln!(writer, "  ({}),", values.join(", "))?;
            }
        }

        writeln!(writer)?;
    }

    Ok(())
}

/// 格式化 SQL 值
fn format_sql_value(value: Option<&str>) -> String {
    match value {
        None => "NULL".to_string(),
        Some(s) if s.is_empty() => "''".to_string(),
        Some(s) if s.eq_ignore_ascii_case("null") => "NULL".to_string(),
        Some(s) => {
            // 尝试解析为数字
            if s.parse::<f64>().is_ok() {
                s.to_string()
            } else {
                // 字符串需要转义单引号
                format!("'{}'", s.replace('\'', "''"))
            }
        }
    }
}
