use gpui::prelude::*;
use gpui::*;

use gpui_component::{v_flex, ActiveTheme as _, StyledExt};

use crate::app::SqlerApp;
use crate::option::{DataSource, DataSourceKind, DataSourceOptions, SQLServerAuth};

pub fn render(
    meta: &DataSource,
    cx: &mut Context<SqlerApp>,
) -> gpui::Div {
    debug_assert!(matches!(meta.kind, DataSourceKind::SQLServer));

    let options = match &meta.options {
        DataSourceOptions::SQLServer(opts) => opts,
        _ => panic!("SQL Server workspace expects SQL Server options"),
    };

    let theme = cx.theme();
    let auth = match options.auth {
        SQLServerAuth::SqlPassword => "认证模式：SQL 密码",
        SQLServerAuth::Integrated => "认证模式：集成认证",
    };

    let user = options.username.as_deref().unwrap_or("IntegratedAuth");

    let mut summary = format!("连接：{}@{}:{}", user, options.host, options.port);
    if let Some(instance) = &options.instance {
        summary.push_str(&format!(" ({})", instance));
    }
    summary.push_str(&format!(" / {}", options.database));

    let mut body = v_flex()
        .gap(px(12.))
        .child(div().text_lg().font_semibold().child(meta.name.clone()))
        .child(
            div()
                .text_sm()
                .text_color(theme.muted_foreground)
                .child(format!("描述：{}", meta.desc)),
        )
        .child(div().text_sm().text_color(theme.muted_foreground).child(summary))
        .child(div().text_sm().text_color(theme.muted_foreground).child(auth))
        .child(
            div()
                .text_sm()
                .text_color(theme.muted_foreground)
                .child("SQL Server 工作区将扩展作业、代理与备份策略面板。"),
        );

    let tables = meta.tables();
    if !tables.is_empty() {
        body = body.child(div().text_sm().text_color(theme.muted_foreground).child(format!(
                    "示例表：{}",
                    tables.iter().take(3).map(|t| t.to_string()).collect::<Vec<_>>().join(", ")
                )));
    }

    v_flex()
        .flex_1()
        .size_full()
        .gap(px(16.))
        .px(px(24.))
        .py(px(20.))
        .child(body)
}
