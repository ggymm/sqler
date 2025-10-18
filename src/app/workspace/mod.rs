pub mod mysql;
pub mod postgres;
pub mod sqlite;
pub mod sqlserver;

use gpui::prelude::FluentBuilder as _;
use gpui::{
    div, img, px, AnyElement, Context, IntoElement, Length, ParentElement, SharedString, Styled,
    Window,
};
use gpui::StatefulInteractiveElement as _;
use gpui::InteractiveElement as _;
use gpui_component::{button::{Button, ButtonVariants as _}, form::{form_field, v_form}, h_flex, v_flex, ActiveTheme as _, Disableable as _, InteractiveElementExt as _, Selectable as _, Sizable, StyledExt};

use crate::app::{
    DataSourceMeta, DataSourceTabState, DatabaseKind, InnerTab, InnerTabId, SqlerApp, TabId,
    TabKind,
};

pub fn render_active(
    app: &mut SqlerApp,
    window: &mut Window,
    cx: &mut Context<SqlerApp>,
) -> AnyElement {
    if let Some(tab) = app.tabs.iter().find(|tab| tab.id == app.active_tab) {
        match &tab.kind {
            TabKind::Home => render_home(&app.saved_sources, window, cx).into_any_element(),
            TabKind::DataSource(state) => {
                render_data_source(tab.id, state, window, cx).into_any_element()
            }
        }
    } else {
        v_flex()
            .flex_1()
            .child(div().child("未找到可渲染的标签页"))
            .into_any_element()
    }
}

fn render_home(
    saved_sources: &[DataSourceMeta],
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

    v_flex()
        .size_full()
        .flex_1()
        .child(
            h_flex()
                .justify_between()
                .items_center()
                .px(px(20.))
                .py(px(16.))
                .bg(theme.tab_bar)
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
    meta: &DataSourceMeta,
    _window: &mut Window,
    cx: &mut Context<SqlerApp>,
) -> AnyElement {
    let source_id = meta.id;
    let icon_path = match meta.kind {
        DatabaseKind::Postgres => "icons/db/postgresql.svg",
        DatabaseKind::MySql => "icons/db/mysql.svg",
        DatabaseKind::Sqlite => "icons/db/sqlite.svg",
        DatabaseKind::SqlServer => "icons/db/sqlserver.svg",
    };

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
            this.open_data_source_tab(source_id, window, cx);
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
            Button::new(("kind-chip", source_id))
                .ghost()
                .xsmall()
                .tab_stop(false)
                .label(meta.kind.label()),
        )
        .child(
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child(meta.description.clone()),
        )
        .into_any_element()
}

fn render_data_source(
    tab_id: TabId,
    state: &DataSourceTabState,
    _window: &mut Window,
    cx: &mut Context<SqlerApp>,
) -> gpui::Div {
    let mut table_list = v_flex()
        .px(px(12.))
        .py(px(8.))
        .gap(px(6.))
        .flex_1()
        .id("workspace-table-list")
        .overflow_scroll();
    table_list.style().min_size.height = Some(Length::Definite(px(0.).into()));
    table_list = table_list.children(state.tables.iter().map(|table| {
        div()
            .px(px(10.))
            .py(px(6.))
            .rounded(px(4.))
            .hover(|this| this.bg(cx.theme().sidebar_accent))
            .child(table.clone())
    }));

    let mut left_panel = v_flex()
        .w(px(240.))
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
    }

    let mut right_panel = v_flex().flex_1();
    {
        let style = right_panel.style();
        style.min_size.height = Some(Length::Definite(px(0.).into()));
    }

    let mut detail_panel = v_flex()
        .flex_1()
        .id(SharedString::from(format!(
            "ds-detail-scroll-{}",
            tab_id.raw()
        )))
        .overflow_scroll();
    detail_panel.style().min_size.height = Some(Length::Definite(px(0.).into()));
    let detail_panel = detail_panel
        .px(px(24.))
        .py(px(20.))
        .child(data_source_detail(state, cx));

    let right_panel = right_panel
        .child(inner_tab_bar(
            tab_id,
            &state.inner_tabs,
            state.active_inner_tab,
            cx,
        ))
        .child(detail_panel);

    v_flex()
        .flex_1()
        .child(workspace_toolbar(tab_id, true, cx))
        .child(
            h_flex()
                .flex_1()
                .items_start()
                .child(left_panel)
                .child(right_panel),
        )
}

fn workspace_toolbar(tab_id: TabId, has_query: bool, cx: &mut Context<SqlerApp>) -> gpui::Div {
    let buttons = [
        ("tab-config", "数据源配置", false),
        ("tab-new-query", "新建查询", !has_query),
        ("tab-import", "导入", true),
        ("tab-export", "导出", true),
    ];

    h_flex()
        .gap(px(8.))
        .px(px(16.))
        .py(px(10.))
        .bg(cx.theme().tab_bar)
        .border_b_1()
        .border_color(cx.theme().border)
        .children(buttons.into_iter().map(|(id, label, disabled)| {
            let button = Button::new((id, tab_id.raw())).ghost().small().label(label);

            if id == "tab-config" {
                button
                    .selected(true)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.set_active_inner_tab(tab_id, InnerTabId(0), cx);
                    }))
            } else {
                button.disabled(disabled)
            }
        }))
}

fn inner_tab_bar(
    tab_id: TabId,
    tabs: &[InnerTab],
    active: InnerTabId,
    cx: &mut Context<SqlerApp>,
) -> gpui::Div {
    h_flex()
        .gap(px(6.))
        .px(px(16.))
        .py(px(8.))
        .border_b_1()
        .border_color(cx.theme().border)
        .bg(cx.theme().tab_bar)
        .children(tabs.iter().map(move |tab| {
            let tab_id_inner = tab.id;
            let pill = h_flex()
                .px(px(10.))
                .py(px(6.))
                .rounded(px(6.))
                .cursor_pointer()
                .id(SharedString::from(format!(
                    "inner-tab-{}-{}",
                    tab_id.raw(),
                    tab_id_inner.raw()
                )))
                .text_sm()
                .when(tab_id_inner == active, |this| {
                    this.bg(cx.theme().tab_active)
                        .text_color(cx.theme().tab_active_foreground)
                })
                .when(tab_id_inner != active, |this| {
                    this.text_color(cx.theme().muted_foreground)
                })
                .child(tab.title.clone());

            pill.on_click(cx.listener(move |this, _, _, cx| {
                this.set_active_inner_tab(tab_id, tab_id_inner, cx);
            }))
        }))
}

fn data_source_detail(state: &DataSourceTabState, cx: &mut Context<SqlerApp>) -> gpui::Div {
    let meta = &state.meta;
    let config = v_form()
        .gap(px(12.))
        .child(
            form_field()
                .label("名称")
                .child(div().child(meta.name.clone())),
        )
        .child(
            form_field()
                .label("类型")
                .child(div().child(meta.kind.label())),
        )
        .child(
            form_field()
                .label("主机")
                .child(div().child(meta.connection.host.clone())),
        )
        .child(
            form_field()
                .label("端口")
                .child(div().child(meta.connection.port.clone())),
        )
        .child(
            form_field()
                .label("数据库")
                .child(div().child(meta.connection.database.clone())),
        )
        .child(
            form_field()
                .label("账号")
                .child(div().child(meta.connection.username.clone())),
        )
        .child(
            form_field()
                .label("描述")
                .child(div().child(meta.description.clone())),
        );

    let workspace_view = render_workspace_body(meta, cx);

    v_flex()
        .gap(px(16.))
        .child(
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("提示：后续将补充连接测试、历史操作等信息。"),
        )
        .child(config)
        .child(workspace_view)
}

fn render_workspace_body(meta: &DataSourceMeta, cx: &mut Context<SqlerApp>) -> gpui::Div {
    match meta.kind {
        DatabaseKind::Postgres => postgres::render(meta.kind, meta, cx),
        DatabaseKind::MySql => mysql::render(meta.kind, meta, cx),
        DatabaseKind::Sqlite => sqlite::render(meta.kind, meta, cx),
        DatabaseKind::SqlServer => sqlserver::render(meta.kind, meta, cx),
    }
}

pub fn render_common_workspace(
    kind: DatabaseKind,
    meta: &DataSourceMeta,
    notes: Vec<String>,
    cx: &mut Context<SqlerApp>,
) -> gpui::Div {
    let connection = &meta.connection;
    let theme = cx.theme();

    let mut section = v_flex()
        .gap(px(10.))
        .child(
            div()
                .text_base()
                .font_semibold()
                .child(format!("{} 工作区", kind.label())),
        )
        .child(
            div()
                .text_sm()
                .text_color(theme.muted_foreground)
                .child(format!(
                    "连接：{}@{}:{} / {}",
                    connection.username, connection.host, connection.port, connection.database,
                )),
        );

    for note in notes {
        section = section.child(
            div()
                .text_sm()
                .text_color(theme.muted_foreground)
                .child(note),
        );
    }

    section
}
