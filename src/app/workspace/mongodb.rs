use gpui::prelude::*;
use gpui::*;

use gpui_component::{v_flex, ActiveTheme as _, StyledExt};

use crate::app::SqlerApp;
use crate::option::{DataSource, DataSourceKind, DataSourceOptions};

pub fn render(
    meta: &DataSource,
    cx: &mut Context<SqlerApp>,
) -> gpui::Div {
    debug_assert!(matches!(meta.kind, DataSourceKind::MongoDB));

    let options = match &meta.options {
        DataSourceOptions::MongoDB(opts) => opts,
        _ => panic!("MongoDB workspace expects MongoDB options"),
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
                .child(format!("描述：{}", meta.desc)),
        )
        .child(div().text_sm().text_color(theme.muted_foreground).child(summary))
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
        .gap(px(16.))
        .px(px(24.))
        .py(px(20.))
        .child(body)
}
