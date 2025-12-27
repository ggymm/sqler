use std::env;
use std::error::Error;
use std::fs;
use std::io::stdout;
use std::path::{Path, PathBuf};
use std::process::exit;

use serde::{Deserialize, Serialize};
use tracing_appender::{non_blocking, rolling::never};
use tracing_subscriber::{EnvFilter, fmt::layer, layer::SubscriberExt, util::SubscriberInitExt};

use sqler_core::{AppCache, DataSource, create_connection};

mod dump;
mod exec;
mod export;
mod import;

/// 操作类型
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Operation {
    Exec,   // 执行 SQL 文件
    Dump,   // 导出表为 SQL
    Import, // 复杂导入（CSV/JSON -> DB）
    Export, // 复杂导出（DB -> CSV/JSON）
}

/// 执行 SQL 配置
#[derive(Debug, Deserialize)]
pub struct ExecConfig {
    pub file: String,
    #[serde(default = "default_batch_size")]
    pub batch: usize,
    #[serde(default = "default_timeout_seconds")]
    pub timeout_seconds: u64,
}

/// 表导出配置
#[derive(Debug, Deserialize)]
pub struct DumpConfig {
    pub file: String,
    pub table: String,
    pub filter: Option<String>,
    #[serde(default = "default_batch_size")]
    pub batch: usize,
    #[serde(default = "default_insert_batch_size")]
    pub insert_batch: usize,
    #[serde(default = "default_timeout_seconds")]
    pub timeout_seconds: u64,
    #[serde(default = "default_include_schema")]
    pub include_schema: bool,
}

fn default_batch_size() -> usize {
    1000
}

fn default_insert_batch_size() -> usize {
    1000
}

fn default_timeout_seconds() -> u64 {
    3600
}

fn default_include_schema() -> bool {
    true
}

/// 导入配置（预留）
#[derive(Debug, Deserialize)]
pub struct ImportConfig {
    // TODO: Phase 2
}

/// 导出配置（预留）
#[derive(Debug, Deserialize)]
pub struct ExportConfig {
    // TODO: Phase 2
}

/// 统一的任务配置
#[derive(Debug, Deserialize)]
pub struct TaskConfig {
    pub task_id: String,
    pub source_id: String,
    pub operation: Operation,
    pub created_at: String,

    // 各操作的配置（可选，根据 operation 字段确定使用哪一个）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dump: Option<DumpConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exec: Option<ExecConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub import: Option<ImportConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub export: Option<ExportConfig>,
}

/// 进度输出消息（写入 stdout 的 JSON Lines）
#[derive(Debug, Serialize)]
pub struct ProgressMessage {
    kind: MessageKind,
    data: serde_json::Value,
}

/// 消息类型
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageKind {
    Progress,
    Status,
    Error,
    Completed,
}

/// 初始化任务日志系统
fn init_task_logging(task_dir: &Path) -> non_blocking::WorkerGuard {
    let log_file = never(task_dir, "task.log");
    let (non_blocking, guard) = non_blocking(log_file);

    tracing_subscriber::registry()
        .with(EnvFilter::new("info"))
        .with(layer().with_writer(stdout))
        .with(layer().with_writer(non_blocking).with_ansi(false))
        .init();

    guard
}

fn main() {
    // 1. 解析命令行参数
    let mut task_dir: Option<PathBuf> = None;
    let args: Vec<String> = env::args().collect();
    for i in 0..args.len() {
        if args[i] == "--task-dir" && i + 1 < args.len() {
            task_dir = Some(PathBuf::from(&args[i + 1]));
            break;
        }
    }
    let task_dir = match task_dir {
        Some(dir) => dir,
        None => {
            print_error("fatal", "缺少 --task-dir 参数");
            eprintln!("用法: sqler-task --task-dir <DIR>");
            exit(1);
        }
    };

    // 2. 初始化日志系统
    let _log_guard = init_task_logging(&task_dir);
    tracing::info!("任务进程启动，task_dir: {:?}", task_dir);

    // 3. 读取任务配置
    let config_path = task_dir.join("config.json");
    let config_content = match fs::read_to_string(&config_path) {
        Ok(content) => content,
        Err(e) => {
            print_error("fatal", &format!("无法读取配置文件: {}", e));
            exit(1);
        }
    };

    // 4. 解析统一的任务配置
    let config: TaskConfig = match serde_json::from_str(&config_content) {
        Ok(cfg) => cfg,
        Err(e) => {
            print_error("fatal", &format!("配置文件格式错误: {}", e));
            exit(1);
        }
    };
    tracing::info!("任务配置解析成功: task_id={}", config.task_id);

    // 5. 从加密缓存读取数据源
    tracing::info!("加载数据源: {}", config.source_id);
    let datasource = match load_data_source(&config.source_id) {
        Ok(ds) => ds,
        Err(e) => {
            print_error("fatal", &format!("无法加载数据源: {}", e));
            exit(1);
        }
    };

    // 6. 建立数据库连接
    tracing::info!("正在连接数据库...");
    let mut session = match create_connection(&datasource.options) {
        Ok(s) => s,
        Err(e) => {
            print_error("fatal", &format!("数据库连接失败: {}", e));
            exit(1);
        }
    };
    tracing::info!("数据库连接成功");

    // 7. 根据 operation 分发处理
    match config.operation {
        Operation::Dump => {
            let dump_config = config.dump.as_ref().expect("Dump 配置缺失");
            dump::run(&mut session, dump_config);
        }
        Operation::Exec => {
            let exec_config = config.exec.as_ref().expect("Exec 配置缺失");
            exec::run(&mut session, exec_config, &task_dir);
        }
        Operation::Import => {
            print_error("fatal", "Import 功能尚未实现");
            exit(1);
        }
        Operation::Export => {
            print_error("fatal", "Export 功能尚未实现");
            exit(1);
        }
    }
}

/// 从加密缓存读取数据源配置
fn load_data_source(id: &str) -> Result<DataSource, Box<dyn Error>> {
    let cache = AppCache::init()?;
    let cache_guard = cache.read().unwrap();
    let sources = cache_guard.sources();

    sources
        .iter()
        .find(|s| s.id == id)
        .cloned()
        .ok_or_else(|| format!("数据源不存在: {}", id).into())
}

pub fn print_error(
    severity: &str,
    message: &str,
) {
    print_progress(ProgressMessage {
        kind: MessageKind::Error,
        data: serde_json::json!({
            "severity": severity,
            "message": message,
        }),
    });
}

pub fn print_completed(data: serde_json::Value) {
    print_progress(ProgressMessage {
        kind: MessageKind::Completed,
        data,
    });
}

pub fn print_progress(msg: ProgressMessage) {
    if let Ok(json) = serde_json::to_string(&msg) {
        println!("{}", json);
    }
}
