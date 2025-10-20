use gpui::Context;

use crate::app::SqlerApp;
use crate::{DataSourceMeta, DataSourceType};

pub fn render(
    kind: DataSourceType,
    meta: &DataSourceMeta,
    cx: &mut Context<SqlerApp>,
) -> gpui::Div {
    debug_assert!(matches!(kind, DataSourceType::SQLite));

    let notes = vec![
        format!("描述：{}", meta.description.to_string()),
        format!("数据库文件：{}", meta.connection.database.to_string()),
        "SQLite 工作区适合快速检查本地数据，后续将支持直接导入导出。".to_string(),
    ];

    super::render_common_workspace(kind, meta, notes, cx)
}
