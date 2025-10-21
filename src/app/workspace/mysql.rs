use gpui::prelude::*;
use gpui::*;

use gpui_component::{
    form::{form_field, v_form},
    h_flex, v_flex, ActiveTheme as _, StyledExt,
};

use crate::app::SqlerApp;
use crate::option::{DataSource, DataSourceKind, DataSourceOptions, MySQLOptions};

pub fn render(
    meta: &DataSource,
    cx: &mut Context<SqlerApp>,
) -> gpui::Div {
    debug_assert!(matches!(meta.kind, DataSourceKind::MySQL));

    let options = match &meta.options {
        DataSourceOptions::MySQL(opts) => opts,
        _ => panic!("MySQL workspace expects MySQL options"),
    };

    let tables = meta.tables();
    let tables_len = tables.len();

    let mut table_list = v_flex()
        .px(px(12.))
        .py(px(8.))
        .gap(px(6.))
        .flex_1()
        .id("mysql-table-list")
        .overflow_scroll();
    {
        let style = table_list.style();
        style.min_size.height = Some(Length::Definite(px(0.).into()));
        style.min_size.width = Some(Length::Definite(px(0.).into()));
    }
    table_list = table_list.children(tables.iter().map(|table| {
        div()
            .px(px(10.))
            .py(px(6.))
            .rounded(px(4.))
            .hover(|this| this.bg(cx.theme().sidebar_accent))
            .child(table.clone())
    }));

    let mut left_panel = v_flex()
        .w(px(240.))
        .flex_shrink_0()
        .bg(cx.theme().sidebar)
        .border_r_1()
        .border_color(cx.theme().border)
        .child(
            div()
                .px(px(16.))
                .py(px(18.))
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("表列表"),
        )
        .child(table_list);
    {
        let style = left_panel.style();
        style.min_size.height = Some(Length::Definite(px(0.).into()));
        style.min_size.width = Some(Length::Definite(px(0.).into()));
    }

    let detail_panel = render_detail(meta, options, tables_len, cx);

    h_flex().flex_1().size_full().child(left_panel).child(
        v_flex()
            .flex_1()
            .gap(px(16.))
            .px(px(24.))
            .py(px(20.))
            .child(detail_panel),
    )
}

fn render_detail(
    meta: &DataSource,
    options: &MySQLOptions,
    tables_len: usize,
    cx: &mut Context<SqlerApp>,
) -> gpui::Div {
    let mut config = v_form()
        .gap(px(12.))
        .child(form_field().label("名称").child(div().child(meta.name.clone())))
        .child(form_field().label("类型").child(div().child(meta.kind.label())))
        .child(form_field().label("主机").child(div().child(options.host.clone())))
        .child(form_field().label("端口").child(div().child(options.port.to_string())))
        .child(
            form_field()
                .label("数据库")
                .child(div().child(options.database.clone())),
        )
        .child(form_field().label("账号").child(div().child(options.username.clone())));

    if let Some(charset) = &options.charset {
        config = config.child(form_field().label("字符集").child(div().child(charset.clone())));
    }
    if options.use_tls {
        config = config.child(form_field().label("TLS").child(div().child("开启")));
    }
    if options.password.is_some() {
        config = config.child(form_field().label("密码").child(div().child("已设置")));
    }

    let notes = vec![
        format!("描述：{}", meta.desc),
        format!("表数量：{}", tables_len),
        "MySQL 工作区规划包含连接池与慢查询分析面板。".to_string(),
    ];

    let theme = cx.theme();
    let mut notes_block = v_flex()
        .gap(px(10.))
        .child(
            div()
                .text_base()
                .font_semibold()
                .child(format!("{} 工作区", meta.kind.label())),
        )
        .child(
            div()
                .text_sm()
                .text_color(theme.muted_foreground)
                .child(summary(options)),
        );

    for note in notes {
        notes_block = notes_block.child(div().text_sm().text_color(theme.muted_foreground).child(note));
    }

    v_flex()
        .gap(px(16.))
        .child(
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("提示：后续将补充连接测试、历史操作等信息。"),
        )
        .child(config)
        .child(notes_block)
}

fn summary(options: &MySQLOptions) -> String {
    format!(
        "连接：{}@{}:{} / {}",
        options.username, options.host, options.port, options.database
    )
}
