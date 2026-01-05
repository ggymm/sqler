use std::{fs::File, io::Read, process, time::Instant};

use sqler_core::{DatabaseSession, ExecReq};

use crate::ExecConfig;

const CHUNK_SIZE: usize = 100 * 1024 * 1024; // 100MB per chunk

/// 执行 SQL 文件任务
pub fn run(
    session: &mut Box<dyn DatabaseSession>,
    config: &ExecConfig,
) {
    tracing::info!("开始执行任务");
    tracing::debug!("执行配置: file={}, batch={}", config.file, config.batch);

    // 1. 打开文件
    tracing::info!("打开 SQL 文件: {}", config.file);
    let mut file = match File::open(&config.file) {
        Ok(f) => {
            let size = f.metadata().map(|m| m.len()).unwrap_or(0);
            tracing::info!(
                "文件打开成功，大小: {} 字节 ({:.2} MB)",
                size,
                size as f64 / 1024.0 / 1024.0
            );
            f
        }
        Err(e) => {
            tracing::error!("文件打开失败: {}", e);
            process::exit(1);
        }
    };

    // 2. 流式读取并执行
    tracing::info!("开始流式执行 SQL 语句");
    let start_time = Instant::now();
    let mut affected = 0u64;
    let mut completed = 0i64;

    let mut buffer = vec![0u8; CHUNK_SIZE];
    let mut remainder = String::new();
    loop {
        // 读取一个块
        let bytes = match file.read(&mut buffer) {
            Ok(n) => n,
            Err(e) => {
                tracing::error!("文件读取失败: {}", e);
                process::exit(1);
            }
        };

        if bytes == 0 {
            break; // EOF
        }

        // 将块转换为字符串（处理UTF-8）
        let chunk = match str::from_utf8(&buffer[..bytes]) {
            Ok(s) => s,
            Err(e) => {
                tracing::error!("文件编码错误（需要UTF-8）: {}", e);
                process::exit(1);
            }
        };

        // 拼接上次的剩余部分
        let content = format!("{}{}", remainder, chunk);

        // 按分号分割
        let parts: Vec<&str> = content.split(';').collect();

        // 执行除最后一个外的所有语句（最后一个可能不完整）
        for sql in &parts[..parts.len() - 1] {
            let sql = sql.trim();
            if sql.is_empty() {
                continue; // 跳过空语句
            }

            completed = completed.saturating_add(1);

            // 执行SQL
            match session.exec(ExecReq::Sql { sql: sql.to_string() }) {
                Ok(resp) => {
                    affected = affected.saturating_add(resp.affected);
                    tracing::debug!("第 {} 条执行成功，影响 {} 行", completed, resp.affected);

                    // 每100条报告一次进度
                    if completed % 100 == 0 {
                        let elapsed = start_time.elapsed().as_secs_f64();
                        let speed = completed as f64 / elapsed;
                        tracing::info!(
                            "执行进度: {} 条完成, 累计影响 {} 行, 速度: {:.0} 条/秒, 已用时: {:.0}s",
                            completed,
                            affected,
                            speed,
                            elapsed
                        );
                    }
                }
                Err(e) => {
                    tracing::error!("第 {} 条执行失败: {}\nSQL: {}", completed, e, sql);
                    process::exit(1);
                }
            }
        }

        // 保留最后一个不完整的部分
        remainder = parts.last().unwrap().to_string();
    }

    // 3. 处理文件末尾的剩余语句
    let remainder = remainder.trim();
    if !remainder.is_empty() {
        completed = completed.saturating_add(1);

        match session.exec(ExecReq::Sql {
            sql: remainder.to_string(),
        }) {
            Ok(resp) => {
                affected += resp.affected;
                tracing::debug!("第 {} 条执行成功，影响 {} 行", completed, resp.affected);
            }
            Err(e) => {
                tracing::error!("第 {} 条执行失败: {}\nSQL: {}", completed, e, remainder);
                process::exit(1);
            }
        }
    }

    // 4. 输出完成消息
    let elapsed = start_time.elapsed().as_secs_f64();
    tracing::info!(
        "执行完成，共 {} 条SQL语句，累计影响 {} 行，耗时 {:.0} 秒，速度 {:.0} 条/秒",
        completed,
        affected,
        elapsed,
        completed as f64 / elapsed
    );
}
