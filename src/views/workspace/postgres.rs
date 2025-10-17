use gpui::Context;

use crate::views::{DataSourceMeta, DatabaseKind, SqlerApp};

pub fn render(
    kind: DatabaseKind,
    meta: &DataSourceMeta,
    cx: &mut Context<SqlerApp>,
) -> gpui::Div {
    debug_assert!(matches!(kind, DatabaseKind::Postgres));

    let samples = if meta.tables.is_empty() {
        "暂无表信息".to_string()
    } else {
        let head = meta
            .tables
            .iter()
            .take(3)
            .map(|table| table.to_string())
            .collect::<Vec<_>>();
        format!("示例表：{}", head.join(", "))
    };

    let notes = vec![
        format!("描述：{}", meta.description.to_string()),
        samples,
        "Postgres 工作区将提供 Schema、扩展与权限等管理能力。".to_string(),
    ];

    super::render_common_workspace(kind, meta, notes, cx)
}
