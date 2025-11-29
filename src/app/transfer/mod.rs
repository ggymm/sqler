mod export;
mod import;

pub use export::ExportWindowBuilder;
pub use import::ImportWindowBuilder;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransferKind {
    Csv,
    Json,
    Sql,
}

impl TransferKind {
    pub fn all() -> &'static [TransferKind] {
        &[TransferKind::Csv, TransferKind::Json, TransferKind::Sql]
    }

    pub fn label(&self) -> &'static str {
        match self {
            TransferKind::Csv => "CSV",
            TransferKind::Json => "JSON",
            TransferKind::Sql => "SQL",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            TransferKind::Csv => "逗号分隔值文件，适用于表格数据",
            TransferKind::Json => "JSON 格式文件，适用于结构化数据",
            TransferKind::Sql => "SQL 脚本文件，包含完整的建表和插入语句",
        }
    }

    pub fn from_label(label: &str) -> Option<Self> {
        match label {
            "CSV" => Some(TransferKind::Csv),
            "JSON" => Some(TransferKind::Json),
            "SQL" => Some(TransferKind::Sql),
            _ => None,
        }
    }
}
