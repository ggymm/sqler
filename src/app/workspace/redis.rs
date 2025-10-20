use gpui::prelude::*;
use gpui::*;

use gpui_component::{h_flex, v_flex, ActiveTheme as _, StyledExt};

use crate::app::{DataSourceTabState, SqlerApp, TabId};
use crate::option::StoredOptions;
use crate::DataSourceType;

pub struct RedisWorkspace<'a> {
    state: &'a DataSourceTabState,
}

impl<'a> RedisWorkspace<'a> {
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
        debug_assert!(matches!(meta.kind, DataSourceType::Redis));

        let options = match &meta.options {
            StoredOptions::Redis(opts) => opts,
            other => panic!("RedisWorkspace expects Redis options, got {:?}", other),
        };

        let theme = cx.theme();
        let tls = if options.use_tls { " (TLS)" } else { "" };
        let user = options.username.clone().unwrap_or_else(|| "default".into());
        let summary = format!(
            "{}@{}:{} db={}{}",
            user, options.host, options.port, options.db, tls
        );

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
                    .child("Redis 工作区计划提供键空间浏览与监控。"),
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
                            .child("Redis 工作区（精简布局）"),
                    ),
            )
            .child(v_flex().px(px(24.)).py(px(18.)).gap(px(16.)).child(body))
    }
}
