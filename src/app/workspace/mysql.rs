use gpui::Context;

use crate::app::{DataSourceMeta, DatabaseKind, SqlerApp};

pub fn render(
    kind: DatabaseKind,
    meta: &DataSourceMeta,
    cx: &mut Context<SqlerApp>,
) -> gpui::Div {
    debug_assert!(matches!(kind, DatabaseKind::MySql));

    let notes = vec![
        format!("描述：{}", meta.description.to_string()),
        format!("表数量：{}", meta.tables.len()),
        "MySQL 工作区规划包含连接池与慢查询分析面板。".to_string(),
    ];

    super::render_common_workspace(kind, meta, notes, cx)
}
