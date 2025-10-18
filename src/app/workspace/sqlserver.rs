use gpui::Context;

use crate::app::{DataSourceMeta, DatabaseKind, SqlerApp};

pub fn render(kind: DatabaseKind, meta: &DataSourceMeta, cx: &mut Context<SqlerApp>) -> gpui::Div {
    debug_assert!(matches!(kind, DatabaseKind::SqlServer));

    let notes = vec![
        format!("描述：{}", meta.description.to_string()),
        format!("表数量：{}", meta.tables.len()),
        "SQL Server 工作区将扩展作业、代理与备份策略面板。".to_string(),
    ];

    super::render_common_workspace(kind, meta, notes, cx)
}
