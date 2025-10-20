use gpui::Context;

use crate::app::SqlerApp;
use crate::{DataSourceMeta, DataSourceType};

pub fn render(
    kind: DataSourceType,
    meta: &DataSourceMeta,
    cx: &mut Context<SqlerApp>,
) -> gpui::Div {
    debug_assert!(matches!(kind, DataSourceType::PostgreSQL));

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
        format!("描述：{}", meta.desc.to_string()),
        samples,
        "Postgres 工作区将提供 Schema、扩展与权限等管理能力。".to_string(),
    ];

    super::render_common_workspace(kind, meta, notes, cx)
}
