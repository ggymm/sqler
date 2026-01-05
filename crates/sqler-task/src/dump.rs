use std::{
    collections::HashMap,
    fs::{self, File},
    io::{BufWriter, Result, Write},
    path::Path,
    process,
    time::Instant,
};

use chrono::Utc;

use sqler_core::{ColumnInfo, ColumnKind, DatabaseSession, Paging, QueryReq, QueryResp};

use crate::DumpConfig;

/// 导出表为 SQL 文件任务
pub fn run(
    session: &mut Box<dyn DatabaseSession>,
    config: &DumpConfig,
) {
    tracing::info!("开始导出任务");
    tracing::debug!(
        "导出配置: table={}, file={}, batch={}, insert_batch={}, only_schema={}",
        config.table,
        config.file,
        config.batch,
        config.insert_batch,
        config.only_schema
    );

    // 1. 查询表的列信息
    tracing::info!("分析表结构: {}", config.table);
    let cols = match session.columns(&config.table) {
        Ok(cols) => {
            if cols.is_empty() {
                tracing::error!("表 {} 不存在或没有列", config.table);
                process::exit(1);
            } else {
                cols
            }
        }
        Err(e) => {
            tracing::error!("获取表结构失败: {}", e);
            process::exit(1);
        }
    };

    tracing::info!("表结构分析完成，共 {} 列", cols.len());

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

    // 3. 写入表结构
    tracing::info!("写入表结构");
    if let Err(e) = write_schema(&mut writer, config, &cols) {
        tracing::error!("写入表结构失败: {}", e);
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
    let count = {
        match session.query(QueryReq::Builder {
            table: config.table.clone(),
            columns: vec!["COUNT(*) as total".to_string()],
            paging: None,
            orders: vec![],
            filters: vec![],
        }) {
            Ok(QueryResp::Rows { rows, .. }) => rows
                .first()
                .and_then(|row| row.get("total"))
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            _ => 0,
        }
    };
    tracing::info!("总行数: {}", count);

    if count == 0 {
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
    let mut page = 0;
    let mut completed = 0u64;

    loop {
        tracing::debug!("查询第 {} 页数据", page);

        let resp = match session.query(QueryReq::Builder {
            table: config.table.clone(),
            columns: vec![], // 查询所有列
            paging: Some(Paging::new(page, batch_size)),
            orders: vec![],
            filters: vec![],
        }) {
            Ok(r) => r,
            Err(e) => {
                tracing::error!("查询数据失败: {}", e);
                process::exit(1);
            }
        };

        let (_, rows) = match resp {
            QueryResp::Rows { cols, rows } => (cols, rows),
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
        if let Err(e) = write_insert(&mut writer, &config.table, &cols, &rows, config.insert_batch) {
            tracing::error!("写入 INSERT 语句失败: {}", e);
            process::exit(1);
        }

        page = page.saturating_add(1);
        completed = completed.saturating_add(rows.len() as u64);

        // 报告进度
        let elapsed = start_time.elapsed().as_secs_f64();
        let speed = completed as f64 / elapsed;

        tracing::info!(
            "导出进度: {}/{} ({:.1}%), 速度: {:.0} 行/秒, 已用时: {:.0}s, 预计剩余: {:.0}s",
            completed,
            count,
            (completed as f64 / count as f64) * 100.0,
            speed,
            elapsed,
            if speed > 0.0 {
                (count as f64 - completed as f64) / speed
            } else {
                0.0
            }
        );
        if completed >= count {
            tracing::info!("所有数据已导出，共 {} 行", completed);
            break;
        }

        // 检查缓冲区大小，超过 10MB 就刷新
        let buffer = writer.buffer().len();
        if buffer >= 10 * 1024 * 1024 {
            tracing::debug!("刷新缓冲区（当前 {:.2} MB）", buffer as f64 / 1024.0 / 1024.0);
            if let Err(e) = writer.flush() {
                tracing::error!("刷新文件缓冲失败: {}", e);
                process::exit(1);
            }
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
        "导出完成，共 {} 行，耗时 {:.0} 秒，速度 {:.0} 行/秒",
        completed,
        elapsed,
        completed as f64 / elapsed
    );
}

/// 写入表结构
fn write_schema(
    writer: &mut BufWriter<File>,
    config: &DumpConfig,
    columns: &[ColumnInfo],
) -> Result<()> {
    writeln!(writer, "-- Dump of table: {}", config.table)?;
    writeln!(writer, "-- Generated at: {}", Utc::now().to_rfc3339())?;
    writeln!(writer)?;

    // 生成 CREATE TABLE 语句
    writeln!(writer, "-- Table structure for {}", config.table)?;
    writeln!(writer, "CREATE TABLE IF NOT EXISTS `{}` (", config.table)?;

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

/// 写入插入语句
fn write_insert(
    writer: &mut BufWriter<File>,
    table: &str,
    cols: &[ColumnInfo],
    rows: &[HashMap<String, String>],
    batch_size: usize,
) -> Result<()> {
    if rows.is_empty() {
        return Ok(());
    }

    let kind_map: HashMap<&str, &str> = cols
        .iter()
        .map(|c| (c.name.as_str(), c.kind.as_str()))
        .collect();
    let columns: Vec<&str> = cols.iter().map(|c| c.name.as_str()).collect();

    // 按 insert_batch_size 分批写入
    for chunk in rows.chunks(batch_size) {
        writeln!(writer, "INSERT INTO {} ({}) VALUES", table, columns.join(", "))?;

        for (i, row) in chunk.iter().enumerate() {
            let values: Vec<String> = columns
                .iter()
                .map(|col| {
                    let kind = kind_map.get(col).copied().unwrap_or("");

                    match row.get(*col).map(|s| s.as_str()) {
                        None => "NULL".to_string(),
                        Some(s) if s.is_empty() => "''".to_string(),
                        Some(s) if s.eq_ignore_ascii_case("null") => "NULL".to_string(),
                        Some(s) => {
                            // 根据类型判断是否需要引号
                            if ColumnKind::from_str(kind).needs_quotes() {
                                format!("'{}'", s.replace('\'', "''"))
                            } else {
                                s.to_string()
                            }
                        }
                    }
                })
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
