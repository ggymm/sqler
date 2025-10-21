use gpui::prelude::*;
use gpui::*;

use gpui_component::{v_flex, ActiveTheme as _, StyledExt};

use crate::app::SqlerApp;
use crate::option::{DataSource, DataSourceKind, DataSourceOptions, OracleAddress};

pub fn render(
    meta: &DataSource,
    cx: &mut Context<SqlerApp>,
) -> gpui::Div {
    debug_assert!(matches!(meta.kind, DataSourceKind::Oracle));

    let options = match &meta.options {
        DataSourceOptions::Oracle(opts) => opts,
        _ => panic!("Oracle workspace expects Oracle options"),
    };

    let theme = cx.theme();
    let address = match &options.address {
        OracleAddress::ServiceName(name) => format!("ServiceName: {}", name),
        OracleAddress::Sid(sid) => format!("SID: {}", sid),
    };

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
                .child(format!("{}@{}:{}", options.username, options.host, options.port)),
        )
        .child(div().text_sm().text_color(theme.muted_foreground).child(address))
        .child(
            div()
                .text_sm()
                .text_color(theme.muted_foreground)
                .child("Oracle 工作区后续将补齐会话与资源管理功能。"),
        );

    if options.wallet_path.is_some() {
        body = body.child(div().text_sm().text_color(theme.muted_foreground).child("钱包：已配置"));
    }

    v_flex()
        .flex_1()
        .size_full()
        .gap(px(16.))
        .px(px(24.))
        .py(px(20.))
        .child(body)
}
