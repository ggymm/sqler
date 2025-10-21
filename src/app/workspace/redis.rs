use gpui::prelude::*;
use gpui::*;

use gpui_component::{v_flex, ActiveTheme as _, StyledExt};

use crate::app::SqlerApp;
use crate::option::{DataSource, DataSourceKind, DataSourceOptions};

pub fn render(
    meta: &DataSource,
    cx: &mut Context<SqlerApp>,
) -> gpui::Div {
    debug_assert!(matches!(meta.kind, DataSourceKind::Redis));

    let options = match &meta.options {
        DataSourceOptions::Redis(opts) => opts,
        _ => panic!("Redis workspace expects Redis options"),
    };

    let theme = cx.theme();
    let tls = if options.use_tls {
        "TLS：开启"
    } else {
        "TLS：关闭"
    };
    let user = options.username.as_deref().unwrap_or("default");

    let body = v_flex()
        .gap(px(12.))
        .child(div().text_lg().font_semibold().child(meta.name.clone()))
        .child(
            div()
                .text_sm()
                .text_color(theme.muted_foreground)
                .child(format!("描述：{}", meta.desc)),
        )
        .child(div().text_sm().text_color(theme.muted_foreground).child(format!(
            "连接：{}@{}:{} db={}",
            user, options.host, options.port, options.db
        )))
        .child(
            div()
                .text_sm()
                .text_color(theme.muted_foreground)
                .child(format!("账号：{}", user)),
        )
        .child(div().text_sm().text_color(theme.muted_foreground).child(tls))
        .child(
            div()
                .text_sm()
                .text_color(theme.muted_foreground)
                .child("Redis 工作区计划提供键空间浏览与监控。"),
        );

    v_flex()
        .flex_1()
        .size_full()
        .gap(px(16.))
        .px(px(24.))
        .py(px(20.))
        .child(body)
}
