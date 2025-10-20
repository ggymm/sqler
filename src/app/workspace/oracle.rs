use gpui::prelude::*;
use gpui::*;

use gpui_component::{h_flex, v_flex, ActiveTheme as _, StyledExt};

use crate::app::{DataSourceTabState, SqlerApp, TabId};
use crate::option::ConnectionOptions;
use crate::option::DataSourceKind;
use crate::option::{DataSourceOptions, OracleAddress};

pub struct OracleWorkspace<'a> {
    state: &'a DataSourceTabState,
}

impl<'a> OracleWorkspace<'a> {
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
        debug_assert!(matches!(meta.kind, DataSourceKind::Oracle));

        let options = match &meta.options {
            DataSourceOptions::Oracle(opts) => opts,
            other => panic!("OracleWorkspace expects Oracle options, got {}", other.kind().label()),
        };

        let theme = cx.theme();
        let address = match &options.address {
            OracleAddress::ServiceName(name) => format!("ServiceName: {}", name),
            OracleAddress::Sid(sid) => format!("SID: {}", sid),
        };

        let body = v_flex()
            .gap(px(12.))
            .child(div().text_lg().font_semibold().child(meta.name.clone()))
            .child(div().text_sm().text_color(theme.muted_foreground).child(format!(
                "{}@{}:{} [{}]",
                options.username, options.host, options.port, address
            )))
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
                    .child("Oracle 工作区后续将补齐会话与资源管理功能。"),
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
                            .child("Oracle 工作区（精简布局）"),
                    ),
            )
            .child(v_flex().px(px(24.)).py(px(18.)).gap(px(16.)).child(body))
    }
}
