use std::path::Path;

use sqler_core::DatabaseSession;

use crate::{ExecConfig, print_completed};

/// 执行 SQL 文件任务
pub fn run(
    _session: &mut Box<dyn DatabaseSession>,
    config: &ExecConfig,
    _task_dir: &Path,
) {
    tracing::info!("准备执行 SQL 文件");
    tracing::debug!("配置: file={}, batch={}", config.file, config.batch);

    // TODO: 实现 SQL 文件读取和执行逻辑
    // 1. 读取 SQL 文件 (config.file)
    // 2. 按分号分割语句
    // 3. 逐条执行并记录进度
    // 4. 保存检查点
    // 5. 输出完成消息

    print_completed(serde_json::json!({
        "status": "not_implemented",
        "message": "exec 功能尚未实现"
    }));
}
