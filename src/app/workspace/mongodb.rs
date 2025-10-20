use gpui::prelude::*;
use gpui::*;

use gpui_component::{h_flex, v_flex, ActiveTheme as _, StyledExt};

use crate::app::{DataSourceTabState, SqlerApp, TabId};
use crate::option::StoredOptions;
use crate::DataSourceType;

pub struct MongoWorkspace<'a> {
    state: &'a DataSourceTabState,
}

impl<'a> MongoWorkspace<'a> {
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
        debug_assert!(matches!(meta.kind, DataSourceType::MongoDB));

        let options = match &meta.options {
            StoredOptions::MongoDB(opts) => opts,
            other => panic!("MongoWorkspace expects MongoDB options, got {:?}", other),
        };

        let theme = cx.theme();
        let summary = if let Some(uri) = &options.connection_string {
            uri.clone()
        } else if !options.hosts.is_empty() {
            let hosts = options
                .hosts
                .iter()
                .map(|host| format!("{}:{}", host.host, host.port))
                .collect::<Vec<_>>()
                .join(",");
            let mut text = format!("mongodb://{}", hosts);
            if let Some(rs) = &options.replica_set {
                text.push_str(&format!("?replicaSet={}", rs));
            }
            text
        } else {
            "mongodb://<empty>".to_string()
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
                    .child(format!("描述：{}", meta.desc.to_string())),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(theme.muted_foreground)
                    .child(if options.tls { "TLS：开启" } else { "TLS：关闭" }),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(theme.muted_foreground)
                    .child("MongoDB 工作区将提供集合浏览与聚合调试功能。"),
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
                            .child("MongoDB 工作区（精简布局）"),
                    ),
            )
            .child(v_flex().px(px(24.)).py(px(18.)).gap(px(16.)).child(body))
    }
}
