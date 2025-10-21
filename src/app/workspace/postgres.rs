use gpui::prelude::*;
use gpui::*;

use gpui_component::{v_flex, ActiveTheme as _, StyledExt};

use crate::app::SqlerApp;
use crate::option::{DataSource, DataSourceKind, DataSourceOptions, SslMode};

pub fn render(
    meta: &DataSource,
    cx: &mut Context<SqlerApp>,
) -> gpui::Div {
    debug_assert!(matches!(meta.kind, DataSourceKind::PostgreSQL));

    let options = match &meta.options {
        DataSourceOptions::PostgreSQL(opts) => opts,
        _ => panic!("Postgres workspace expects PostgreSQL options"),
    };

    let theme = cx.theme();

    let mut summary = vec![format!(
        "连接：{}@{}:{} / {}",
        options.username, options.host, options.port, options.database
    )];

    if let Some(schema) = &options.schema {
        if !schema.is_empty() {
            summary.push(format!("默认 Schema：{}", schema));
        }
    }
    if let Some(mode) = options.ssl_mode {
        let mode_str = match mode {
            SslMode::Disable => "Disable",
            SslMode::Prefer => "Prefer",
            SslMode::Require => "Require",
        };
        summary.push(format!("SSL 模式：{}", mode_str));
    }

    let notes = meta
        .tables()
        .into_iter()
        .take(3)
        .map(|table| table.to_string())
        .collect::<Vec<_>>();

    let mut body = v_flex()
        .gap(px(12.))
        .child(div().text_lg().font_semibold().child(meta.name.clone()))
        .child(
            div()
                .text_sm()
                .text_color(theme.muted_foreground)
                .child(format!("描述：{}", meta.desc)),
        );

    for line in summary {
        body = body.child(div().text_sm().text_color(theme.muted_foreground).child(line));
    }

    if notes.is_empty() {
        body = body.child(
            div()
                .text_sm()
                .text_color(theme.muted_foreground)
                .child("暂无表信息，后续可通过同步功能加载。"),
        );
    } else {
        body = body.child(
            div()
                .text_sm()
                .text_color(theme.muted_foreground)
                .child(format!("示例表：{}", notes.join(", "))),
        );
    }

    v_flex()
        .flex_1()
        .size_full()
        .gap(px(16.))
        .px(px(24.))
        .py(px(20.))
        .child(body)
}
