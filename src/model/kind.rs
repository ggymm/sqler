use serde::{Deserialize, Serialize};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataSourceKind {
    MySQL,
    SQLite,
    Postgres,
    Oracle,
    SQLServer,
    Redis,
    MongoDB,
}

impl DataSourceKind {
    pub fn all() -> &'static [DataSourceKind] {
        &[
            DataSourceKind::MySQL,
            DataSourceKind::SQLite,
            DataSourceKind::Postgres,
            DataSourceKind::Oracle,
            DataSourceKind::SQLServer,
            DataSourceKind::Redis,
            DataSourceKind::MongoDB,
        ]
    }

    pub fn image(&self) -> &'static str {
        match self {
            DataSourceKind::MySQL => "icons/mysql.svg",
            DataSourceKind::SQLite => "icons/sqlite.svg",
            DataSourceKind::Postgres => "icons/postgresql.svg",
            DataSourceKind::Oracle => "icons/oracle.svg",
            DataSourceKind::SQLServer => "icons/sqlserver.svg",
            DataSourceKind::Redis => "icons/redis.svg",
            DataSourceKind::MongoDB => "icons/mongodb.svg",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            DataSourceKind::MySQL => "MySQL",
            DataSourceKind::SQLite => "SQLite",
            DataSourceKind::Postgres => "PostgreSQL",
            DataSourceKind::Oracle => "Oracle",
            DataSourceKind::SQLServer => "SQLServer",
            DataSourceKind::Redis => "Redis",
            DataSourceKind::MongoDB => "MongoDB",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            DataSourceKind::MySQL => "开源关系型数据库,读写性能稳定、生态成熟",
            DataSourceKind::SQLite => "嵌入式文件数据库,零配置、单文件存储",
            DataSourceKind::Postgres => "开源对象关系数据库,扩展能力与标准兼容性强",
            DataSourceKind::Oracle => "商业级事务数据库,强调安全性与可扩展性",
            DataSourceKind::SQLServer => "微软企业数据库,原生集成 Windows 与 AD",
            DataSourceKind::Redis => "内存键值数据库,适合缓存、队列与实时计数场景",
            DataSourceKind::MongoDB => "文档型数据库,支持灵活的 JSON 模式与水平扩展",
        }
    }
}
