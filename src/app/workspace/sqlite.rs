use gpui::prelude::*;
use gpui::*;

use gpui_component::{v_flex, ActiveTheme as _, StyledExt};

use crate::app::SqlerApp;
use crate::option::{DataSource, DataSourceKind, DataSourceOptions};

pub fn render(
    meta: &DataSource,
    cx: &mut Context<SqlerApp>,
) -> gpui::Div {
    debug_assert!(matches!(meta.kind, DataSourceKind::SQLite));

    let options = match &meta.options {
        DataSourceOptions::SQLite(opts) => opts,
        _ => panic!("SQLite workspace expects SQLite options"),
    };

    let theme = cx.theme();

    let mut body = v_flex()
        .gap(px(12.))
        .child(div().text_lg().font_semibold().child(meta.name.clone()))
        .child(
            div()
                .text_sm()
                .text_color(theme.muted_foreground)
                .child(format!("描述：{}", meta.desc)),
        )
        .child(
            div()
                .text_sm()
                .text_color(theme.muted_foreground)
                .child(format!("数据库文件：{}", options.file_path)),
        )
        .child(
            div()
                .text_sm()
                .text_color(theme.muted_foreground)
                .child(if options.read_only {
                    "访问模式：只读"
                } else {
                    "访问模式：读写"
                }),
        )
        .child(
            div()
                .text_sm()
                .text_color(theme.muted_foreground)
                .child("SQLite 工作区适合快速检查本地数据，后续将支持直接导入导出。"),
        );

    let tables = meta.tables();
    if !tables.is_empty() {
        body =
            body.child(div().text_sm().text_color(theme.muted_foreground).child(
                format!("示例表：{}", tables.iter().take(3).map(|t| t.to_string()).collect::<Vec<_>>().join(", ")),
            ));
    }

    v_flex()
        .flex_1()
        .size_full()
        .gap(px(16.))
        .px(px(24.))
        .py(px(20.))
        .child(body)
}
