mod export;
mod import;

pub use export::ExportWindow;
pub use import::ImportWindow;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransferFormat {
    Csv,
    Json,
    Sql,
}

impl TransferFormat {
    pub fn all() -> &'static [TransferFormat] {
        &[TransferFormat::Csv, TransferFormat::Json, TransferFormat::Sql]
    }

    pub fn label(&self) -> &'static str {
        match self {
            TransferFormat::Csv => "CSV",
            TransferFormat::Json => "JSON",
            TransferFormat::Sql => "SQL",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            TransferFormat::Csv => "逗号分隔值文件，适用于表格数据",
            TransferFormat::Json => "JSON 格式文件，适用于结构化数据",
            TransferFormat::Sql => "SQL 脚本文件，包含完整的建表和插入语句",
        }
    }

    pub fn from_label(label: &str) -> Option<Self> {
        match label {
            "CSV" => Some(TransferFormat::Csv),
            "JSON" => Some(TransferFormat::Json),
            "SQL" => Some(TransferFormat::Sql),
            _ => None,
        }
    }
}
