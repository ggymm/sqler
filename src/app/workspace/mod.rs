use gpui::prelude::*;
use gpui::*;

use gpui_component::{
    button::{Button, ButtonVariants as _},
    h_flex, v_flex, ActiveTheme as _, InteractiveElementExt as _, Sizable, StyledExt,
};

use crate::app::{SqlerApp, TabKind};
use crate::option::{DataSource, DataSourceKind};

pub mod mongodb;
pub mod mysql;
pub mod oracle;
pub mod postgres;
pub mod redis;
pub mod sqlite;
pub mod sqlserver;

pub fn render(
    app: &mut SqlerApp,
    window: &mut Window,
    cx: &mut Context<SqlerApp>,
) -> AnyElement {
    if let Some(tab) = app.tabs.iter().find(|tab| tab.id == app.active_tab) {
        match &tab.kind {
            TabKind::Home => render_home(&app.saved_sources, window, cx).into_any_element(),
            TabKind::Workspace(meta) => match meta.kind {
                DataSourceKind::MySQL => mysql::render(meta, cx).into_any_element(),
                DataSourceKind::PostgreSQL => postgres::render(meta, cx).into_any_element(),
                DataSourceKind::SQLite => sqlite::render(meta, cx).into_any_element(),
                DataSourceKind::SQLServer => sqlserver::render(meta, cx).into_any_element(),
                DataSourceKind::Oracle => oracle::render(meta, cx).into_any_element(),
                DataSourceKind::Redis => redis::render(meta, cx).into_any_element(),
                DataSourceKind::MongoDB => mongodb::render(meta, cx).into_any_element(),
            },
        }
    } else {
        v_flex()
            .flex_1()
            .child(div().child("未找到可渲染的标签页"))
            .into_any_element()
    }
}

fn render_home(
    saved_sources: &[DataSource],
    window: &mut Window,
    cx: &mut Context<SqlerApp>,
) -> AnyElement {
    let mut source_list = v_flex()
        .px(px(20.))
        .py(px(16.))
        .gap(px(12.))
        .flex_1()
        .id("home-source-list")
        .overflow_scroll();
    source_list.style().min_size.height = Some(Length::Definite(px(0.).into()));
    let source_list = source_list.child(
        h_flex().flex_wrap().gap(px(12.)).children(
            saved_sources
                .iter()
                .map(|meta| render_data_source_card(meta, window, cx)),
        ),
    );

    let theme = cx.theme();

    let mut layout = v_flex().size_full().flex_1();
    {
        let style = layout.style();
        style.min_size.height = Some(Length::Definite(px(0.).into()));
        style.min_size.width = Some(Length::Definite(px(0.).into()));
    }

    layout
        .child(
            h_flex()
                .justify_between()
                .items_center()
                .px(px(20.))
                .py(px(16.))
                .border_b_1()
                .border_color(theme.border)
                .child(
                    v_flex()
                        .gap(px(4.))
                        .child(div().text_lg().font_semibold().child("数据源总览"))
                        .child(
                            div()
                                .text_sm()
                                .text_color(theme.muted_foreground)
                                .child("双击数据源以打开新标签页，统一管理和查询。"),
                        ),
                ),
        )
        .child(source_list)
        .into_any_element()
}

fn render_data_source_card(
    meta: &DataSource,
    _window: &mut Window,
    cx: &mut Context<SqlerApp>,
) -> AnyElement {
    let source_id = meta.id.clone();
    let icon_path = meta.kind.image();

    v_flex()
        .w(px(220.))
        .gap(px(8.))
        .p(px(14.))
        .rounded(px(8.))
        .bg(cx.theme().secondary)
        .border_1()
        .border_color(cx.theme().border)
        .cursor_pointer()
        .id(SharedString::from(format!("source-card-{}", source_id)))
        .hover(|this| this.bg(cx.theme().secondary_hover))
        .on_double_click(cx.listener(move |this, _, window, cx| {
            this.open_data_source_tab(&source_id, window, cx);
        }))
        .child(
            h_flex()
                .items_center()
                .gap(px(8.))
                .child(
                    div()
                        .flex_shrink_0()
                        .w(px(32.))
                        .h(px(32.))
                        .child(img(icon_path).size_full()),
                )
                .child(
                    div()
                        .flex_1()
                        .text_base()
                        .font_semibold()
                        .text_color(cx.theme().foreground)
                        .child(meta.name.clone()),
                ),
        )
        .child(
            Button::new(SharedString::from(format!("kind-chip-{}", meta.id)))
                .ghost()
                .xsmall()
                .tab_stop(false)
                .label(meta.kind.label()),
        )
        .child(
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child(meta.desc.clone()),
        )
        .into_any_element()
}
