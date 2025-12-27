use serde_json::json;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

const SOURCE_ID: &str = "8dd456c4-bf37-46b1-bdf2-38f544a74463";

/// 示例 1：导出单个表的数据（category 表）
fn dump_single_table() {
    let task_dir = PathBuf::from("/tmp/sqler-tasks/dump-category");
    let output_file = PathBuf::from("/tmp/category.sql");

    // 清除上次任务的信息
    if task_dir.exists() {
        fs::remove_dir_all(&task_dir).unwrap();
    }
    if output_file.exists() {
        fs::remove_file(&output_file).unwrap();
    }
    fs::create_dir_all(&task_dir).unwrap();

    let config = json!({
        "task_id": "dump-category-001",
        "source_id": SOURCE_ID,
        "operation": "dump",
        "created_at": "2025-12-27T10:00:00Z",
        "dump": {
            "file": output_file,
            "table": "category",
            "batch": 1000,
            "insert_batch": 1000,
            "timeout_seconds": 3600,
            "include_schema": true
        }
    });

    fs::write(
        task_dir.join("config.json"),
        serde_json::to_string_pretty(&config).unwrap(),
    )
    .unwrap();

    println!("配置文件已创建: {:?}/config.json", task_dir);
    println!("正在执行任务...\n");

    // 直接调用二进制执行任务
    let status = Command::new("cargo")
        .args(&["run", "-p", "sqler-task", "--", "--task-dir"])
        .arg(&task_dir)
        .status()
        .expect("创建任务失败");

    if status.success() {
        println!("\n✓ 任务执行成功");
    } else {
        println!("\n✗ 任务执行失败");
    }
}

fn main() {
    println!("=== Sqler Task Dump 示例 ===\n");

    println!("示例 1: 导出单个表的全部数据");
    println!("----------------------------------------");
    dump_single_table();
    println!();

    println!("注意:");
    println!("  1. 确保数据源 ID '{}' 存在于 ~/.sqler/sources.db", SOURCE_ID);
    println!("  2. 确保目标数据库中存在对应的表");
    println!("  3. 输出文件将保存到 /tmp/ 目录");
    println!("  4. 导出包含表结构（CREATE TABLE）和数据（INSERT 语句）");
}
