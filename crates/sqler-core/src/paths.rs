use std::{fs::create_dir_all, path::PathBuf};

/// 获取 sqler 根目录（~/.sqler）
///
/// Fallback: 如果 home_dir 失败，使用 ./.sqler
pub fn root_dir() -> PathBuf {
    dirs::home_dir()
        .map(|home| home.join(".sqler"))
        .unwrap_or_else(|| PathBuf::from(".sqler"))
}

/// 获取数据源配置文件路径（~/.sqler/sources.db）
pub fn sources_db() -> PathBuf {
    root_dir().join("sources.db")
}

/// 获取日志目录（~/.sqler/logs）
pub fn logs_dir() -> PathBuf {
    let dir = root_dir().join("logs");
    let _ = create_dir_all(&dir);
    dir
}

/// 获取缓存根目录（~/.sqler/cache）
pub fn caches_dir() -> PathBuf {
    let dir = root_dir().join("cache");
    let _ = create_dir_all(&dir);
    dir
}

/// 获取指定数据源的缓存目录（~/.sqler/cache/{uuid}）
pub fn cache_dir(id: &str) -> PathBuf {
    let dir = caches_dir().join(id);
    let _ = create_dir_all(&dir);
    dir
}

/// 获取指定数据源的表信息缓存文件（~/.sqler/cache/{uuid}/tables.json）
pub fn cache_tables(uuid: &str) -> PathBuf {
    cache_dir(uuid).join("tables.json")
}

/// 获取指定数据源的查询缓存文件（~/.sqler/cache/{uuid}/queries.json）
pub fn cache_queries(uuid: &str) -> PathBuf {
    cache_dir(uuid).join("queries.json")
}

/// 获取任务根目录（~/.sqler/tasks）
pub fn tasks_dir() -> PathBuf {
    let dir = root_dir().join("tasks");
    let _ = create_dir_all(&dir);
    dir
}

/// 获取指定任务的目录（~/.sqler/tasks/{task_id}）
pub fn task_dir(id: &str) -> PathBuf {
    let dir = tasks_dir().join(id);
    let _ = create_dir_all(&dir);
    dir
}
