use gpui::{div, px, AnyElement, Context, IntoElement, ParentElement, Styled, Window};
use gpui::prelude::FluentBuilder;
use gpui_component::{
    button::{Button, ButtonVariants as _},
    h_flex,
    v_flex,
    ActiveTheme as _,
    StyledExt,
};

use super::{dialog, DatabaseKind, NewDataSourceState, SqlerApp};

pub(super) fn render_new_data_source_modal(
    state: &mut NewDataSourceState,
    _window: &mut Window,
    cx: &mut Context<SqlerApp>,
) -> AnyElement {
    let mut shell = v_flex().w(px(560.)).gap(px(20.)).bg(cx.theme().background).rounded(cx.theme().radius_lg).shadow_lg().p(px(24.));

    let header = render_header(state, cx);
    shell = shell.child(header);

    let body = if let Some(kind) = state.selected {
        render_form(kind, state, cx)
    } else {
        render_kind_selection(state, cx)
    };
    shell = shell.child(body);

    if state.selected.is_some() {
        shell = shell.child(render_footer(cx));
    }

    div()
        .absolute()
        .inset_0()
        .flex()
        .items_center()
        .justify_center()
        .bg(cx.theme().overlay)
        .child(shell)
        .into_any_element()
}

fn render_header(state: &mut NewDataSourceState, cx: &mut Context<SqlerApp>) -> gpui::Div {
    let title = match state.selected {
        Some(kind) => format!("配置 {} 数据源", kind.label()),
        None => "选择数据源类型".to_string(),
    };

    h_flex()
        .justify_between()
        .items_center()
        .child(
            h_flex()
                .gap(px(8.))
                .items_center()
                .child(div().text_lg().font_semibold().child(title))
                .when(state.selected.is_some(), |this| {
                    this.child(
                        Button::new("modal-back")
                            .text()
                            .label("返回类型列表")
                            .on_click(cx.listener(|this: &mut SqlerApp, _, _, cx| {
                                if let Some(state) = this.new_ds_modal.as_mut() {
                                    state.selected = None;
                                    cx.notify();
                                }
                            })),
                    )
                }),
        )
        .child(
            Button::new("modal-close")
                .text()
                .label("×")
                .on_click(cx.listener(|this, _, _, cx| {
                    this.hide_new_data_source_modal(cx);
                })),
        )
}

fn render_kind_selection(_state: &mut NewDataSourceState, cx: &mut Context<SqlerApp>) -> gpui::Div {
    let cards = DatabaseKind::all()
        .iter()
        .map(|kind| {
            Button::new(("modal-db-card", (*kind as u8) as usize))
                .ghost()
                .p(px(16.))
                .w(px(180.))
                .h(px(108.))
                .justify_start()
                .items_start()
                .flex_col()
                .gap(px(6.))
                .child(div().text_base().font_semibold().child(kind.label()))
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child(kind_description(*kind)),
                )
                .on_click(cx.listener({
                    let kind = *kind;
                    move |this: &mut SqlerApp, _, _, cx| {
                        if let Some(state) = this.new_ds_modal.as_mut() {
                            state.selected = Some(kind);
                            cx.notify();
                        }
                    }
                }))
                .into_any_element()
        })
        .collect::<Vec<_>>();

    v_flex()
        .gap(px(12.))
        .child(
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("请选择需要创建的数据源类型"),
        )
        .child(
            h_flex()
                .flex_wrap()
                .gap(px(12.))
                .children(cards),
        )
}

fn render_form(kind: DatabaseKind, state: &mut NewDataSourceState, cx: &mut Context<SqlerApp>) -> gpui::Div {
    match kind {
        DatabaseKind::Postgres => dialog::postgres::render(&mut state.postgres, cx),
        DatabaseKind::MySql => dialog::mysql::render(&mut state.mysql, cx),
        DatabaseKind::Sqlite => dialog::sqlite::render(&mut state.sqlite, cx),
        DatabaseKind::SqlServer => dialog::sqlserver::render(&mut state.sqlserver, cx),
    }
}

fn render_footer(cx: &mut Context<SqlerApp>) -> gpui::Div {
    h_flex()
        .justify_end()
        .gap(px(8.))
        .child(
            Button::new("modal-cancel")
                .ghost()
                .label("取消")
                .on_click(cx.listener(|this, _, _, cx| {
                    this.hide_new_data_source_modal(cx);
                })),
        )
        .child(
            Button::new("modal-save")
                .primary()
                .label("保存")
                .on_click(cx.listener(|this, _, _, cx| {
                    this.submit_new_data_source_modal(cx);
                })),
        )
}

fn kind_description(kind: DatabaseKind) -> &'static str {
    match kind {
        DatabaseKind::Postgres => "支持 Schema、SSL 等高级特性",
        DatabaseKind::MySql => "常用于业务库与分析库，默认 utf8mb4",
        DatabaseKind::Sqlite => "本地文件数据库，适合轻量级项目",
        DatabaseKind::SqlServer => "企业级数据库，支持实例/域账号",
    }
}
