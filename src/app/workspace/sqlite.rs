use gpui::prelude::*;
use gpui::*;

use gpui_component::{h_flex, v_flex, ActiveTheme as _, StyledExt};

use crate::app::{DataSourceTabState, SqlerApp, TabId};
use crate::option::DataSourceOptions;
use crate::DataSourceType;

pub struct SqliteWorkspace<'a> {
    state: &'a DataSourceTabState,
}

impl<'a> SqliteWorkspace<'a> {
    pub fn new(state: &'a DataSourceTabState) -> Self {
        Self { state }
    }

    pub fn render(
        &self,
        _tab_id: TabId,
        _window: &mut Window,
        cx: &mut Context<SqlerApp>,
    ) -> gpui::Div {
        let meta = &self.state.meta;
        debug_assert!(matches!(meta.kind, DataSourceType::SQLite));

        let options = match &meta.options {
            DataSourceOptions::SQLite(opts) => opts,
            other => panic!("SqliteWorkspace expects SQLite options, got {:?}", other),
        };

        let theme = cx.theme();
        let summary = format!("数据库文件：{}", options.file_path);

        let body = v_flex()
            .gap(px(12.))
            .child(div().text_lg().font_semibold().child(meta.name.clone()))
            .child(
                div()
                    .text_sm()
                    .text_color(theme.muted_foreground)
                    .child(summary),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(theme.muted_foreground)
                    .child(format!("描述：{}", meta.desc.to_string())),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(theme.muted_foreground)
                    .child(if options.read_only { "访问模式：只读" } else { "访问模式：读写" }),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(theme.muted_foreground)
                    .child("SQLite 工作区将聚焦本地调试与快速数据浏览。"),
            );

        v_flex()
            .flex_1()
            .size_full()
            .child(
                h_flex()
                    .px(px(18.))
                    .py(px(12.))
                    .bg(theme.tab_bar)
                    .border_b_1()
                    .border_color(theme.border)
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme.muted_foreground)
                            .child("SQLite 工作区（精简布局）"),
                    ),
            )
            .child(v_flex().px(px(24.)).py(px(18.)).gap(px(16.)).child(body))
    }
}
