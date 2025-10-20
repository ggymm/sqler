use gpui::prelude::*;
use gpui::*;

use gpui_component::{h_flex, v_flex, ActiveTheme as _, StyledExt};

use crate::app::{DataSourceTabState, SqlerApp, TabId};
use crate::option::{SQLServerAuth, DataSourceOptions};
use crate::DataSourceType;

pub struct SqlServerWorkspace<'a> {
    state: &'a DataSourceTabState,
}

impl<'a> SqlServerWorkspace<'a> {
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
        debug_assert!(matches!(meta.kind, DataSourceType::SQLServer));

        let options = match &meta.options {
            DataSourceOptions::SQLServer(opts) => opts,
            other => panic!("SqlServerWorkspace expects SQL Server options, got {:?}", other),
        };

        let theme = cx.theme();
        let mut summary = match options.username.clone() {
            Some(user) => format!("{}@", user),
            None => "IntegratedAuth@".to_string(),
        };
        summary.push_str(&format!("{}:{}", options.host, options.port));
        if let Some(instance) = &options.instance {
            summary.push_str(&format!(" ({})", instance));
        }
        summary.push_str(&format!(" / {}", options.database));

        let auth = match options.auth {
            SQLServerAuth::SqlPassword => "认证模式：SQL 密码",
            SQLServerAuth::Integrated => "认证模式：集成认证",
        };

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
                    .child(auth),
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
                    .child("SQL Server 工作区将扩展作业、代理与备份策略面板。"),
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
                            .child("SQL Server 工作区（精简布局）"),
                    ),
            )
            .child(v_flex().px(px(24.)).py(px(18.)).gap(px(16.)).child(body))
    }
}
