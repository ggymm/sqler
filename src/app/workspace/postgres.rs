use gpui::prelude::*;
use gpui::*;

use gpui_component::{h_flex, v_flex, ActiveTheme as _, StyledExt};

use crate::app::{DataSourceTabState, SqlerApp, TabId};
use crate::option::{SslMode, StoredOptions};
use crate::DataSourceType;

pub struct PostgresWorkspace<'a> {
    state: &'a DataSourceTabState,
}

impl<'a> PostgresWorkspace<'a> {
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
        debug_assert!(matches!(meta.kind, DataSourceType::PostgreSQL));

        let options = match &meta.options {
            StoredOptions::PostgreSQL(opts) => opts,
            other => panic!("PostgresWorkspace expects Postgres options, got {:?}", other),
        };

        let theme = cx.theme();
        let summary = format!(
            "{}@{}:{} / {}",
            options.username, options.host, options.port, options.database
        );

        let mut info = v_flex()
            .gap(px(8.))
            .child(div().text_sm().text_color(theme.muted_foreground).child(summary));

        if let Some(schema) = &options.schema {
            info = info.child(
                div()
                    .text_sm()
                    .text_color(theme.muted_foreground)
                    .child(format!("默认 Schema：{}", schema)),
            );
        }
        if let Some(mode) = options.ssl_mode {
            let mode_str = match mode {
                SslMode::Disable => "Disable",
                SslMode::Prefer => "Prefer",
                SslMode::Require => "Require",
            };
            info = info.child(
                div()
                    .text_sm()
                    .text_color(theme.muted_foreground)
                    .child(format!("SSL 模式：{}", mode_str)),
            );
        }

        let notes = if self.state.tables.is_empty() {
            vec!["暂无表信息，后续可以通过同步功能加载。".to_string()]
        } else {
            let head = self
                .state
                .tables
                .iter()
                .take(3)
                .map(|table| table.to_string())
                .collect::<Vec<_>>();
            vec![format!("示例表：{}", head.join(", "))]
        };

        let mut body = v_flex()
            .gap(px(12.))
            .child(div().text_lg().font_semibold().child(meta.name.clone()))
            .child(info)
            .child(
                div()
                    .text_sm()
                    .text_color(theme.muted_foreground)
                    .child(format!("描述：{}", meta.desc.to_string())),
            );

        for note in notes {
            body = body.child(
                div()
                    .text_sm()
                    .text_color(theme.muted_foreground)
                    .child(note),
            );
        }

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
                            .child("PostgreSQL 工作区（布局预留，后续扩展）"),
                    ),
            )
            .child(v_flex().px(px(24.)).py(px(18.)).gap(px(16.)).child(body))
    }
}
