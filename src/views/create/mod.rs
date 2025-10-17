pub mod postgres;
pub mod mysql;
pub mod sqlite;
pub mod sqlserver;

use gpui::{
    div,
    px,
    AnyElement,
    Context,
    IntoElement,
    Length,
    Overflow,
    ParentElement,
    Render,
    Styled,
    WeakEntity,
    Window,
};
use gpui::prelude::FluentBuilder;
use gpui_component::{
    button::{Button, ButtonVariants as _},
    h_flex,
    v_flex,
    ActiveTheme as _,
    Icon,
    Size,
    Sizable as _,
    StyledExt,
};
use gpui_component::Selectable as _;

use crate::views::{DatabaseKind, NewDataSourceState, SqlerApp};

pub struct CreateDataSourceWindow {
    parent: WeakEntity<SqlerApp>,
    state: NewDataSourceState,
}

impl CreateDataSourceWindow {
    pub fn new(
        state: NewDataSourceState,
        parent: WeakEntity<SqlerApp>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let parent_for_release = parent.clone();
        let _ = cx.on_release(move |_, app| {
            if let Some(parent) = parent_for_release.upgrade() {
                let _ = parent.update(app, |app, cx| {
                    app.clear_new_data_source_window();
                    cx.notify();
                });
            }
        });

        Self { parent, state }
    }

    fn clear_parent(&self, cx: &mut Context<Self>) {
        if let Some(parent) = self.parent.upgrade() {
            let _ = parent.update(cx, |app, cx| {
                app.clear_new_data_source_window();
                cx.notify();
            });
        }
    }

    fn close(&self, window: &mut Window, cx: &mut Context<Self>) {
        self.clear_parent(cx);
        window.remove_window();
    }

    fn back_to_selection(&mut self, cx: &mut Context<Self>) {
        if self.state.selected.take().is_some() {
            cx.notify();
        }
    }

    fn select_kind(&mut self, kind: DatabaseKind, cx: &mut Context<Self>) {
        if self.state.selected != Some(kind) {
            self.state.selected = Some(kind);
            cx.notify();
        }
    }

    fn submit(&self, window: &mut Window, cx: &mut Context<Self>) {
        self.clear_parent(cx);
        window.remove_window();
    }

    fn render_content(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let header = render_header(self, cx);
        let sidebar = render_kind_list(self, cx);
        let main_panel = render_main_panel(self, cx);

        v_flex()
            .size_full()
            .bg(cx.theme().background)
            .child(header)
            .child(
                h_flex()
                    .flex_1()
                    .bg(cx.theme().background)
                    .child(sidebar)
                    .child(main_panel),
            )
            .into_any_element()
    }
}

impl Render for CreateDataSourceWindow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.render_content(window, cx)
    }
}

fn render_header(
    view: &mut CreateDataSourceWindow,
    cx: &mut Context<CreateDataSourceWindow>,
) -> gpui::Div {
    let title = "新建数据源";
    let subtitle = match view.state.selected {
        Some(kind) => format!("当前配置：{}", kind.label()),
        None => "请选择要创建的数据源类型".to_string(),
    };

    h_flex()
        .justify_between()
        .items_center()
        .px(px(24.))
        .py(px(16.))
        .bg(cx.theme().tab_bar)
        .border_b_1()
        .border_color(cx.theme().border)
        .child(
            v_flex()
                .gap(px(4.))
                .child(div().text_lg().font_semibold().child(title))
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child(subtitle),
                ),
        )
        .child(
            h_flex()
                .gap(px(8.))
                .when(view.state.selected.is_some(), |this| {
                    this.child(
                        Button::new("modal-back")
                            .ghost()
                            .small()
                            .label("返回类型列表")
                            .on_click(cx.listener(|this: &mut CreateDataSourceWindow, _, _, cx| {
                                this.back_to_selection(cx);
                            })),
                    )
                })
                .child(
                    Button::new("modal-close")
                        .ghost()
                        .small()
                        .label("关闭")
                        .on_click(cx.listener(|this: &mut CreateDataSourceWindow, _, window, cx| {
                            this.close(window, cx);
                        })),
                ),
        )
}

fn render_kind_list(
    view: &mut CreateDataSourceWindow,
    cx: &mut Context<CreateDataSourceWindow>,
) -> gpui::Div {
    let items = DatabaseKind::all()
        .iter()
        .map(|kind| {
            let selected = view.state.selected == Some(*kind);

            Button::new(("modal-db-card", (*kind as u8) as usize))
                .ghost()
                .justify_start()
                .items_start()
                .p(px(14.))
                .gap(px(14.))
                .w_full()
                .selected(selected)
                        .child(
                            Icon::default()
                                .path(kind_icon_path(*kind))
                                .with_size(if selected { Size::Large } else { Size::Medium })
                                .view(cx),
                        )
                .child(
                    v_flex()
                        .gap(px(4.))
                        .child(div().text_base().font_semibold().child(kind.label()))
                        .child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
                                .whitespace_normal()
                                .child(kind_description(*kind)),
                        ),
                )
                .on_click(cx.listener({
                    let kind = *kind;
                    move |this: &mut CreateDataSourceWindow, _, _, cx| {
                        this.select_kind(kind, cx);
                    }
                }))
                .into_any_element()
        })
        .collect::<Vec<_>>();

    v_flex()
        .w(px(240.))
        .gap(px(12.))
        .px(px(16.))
        .py(px(16.))
        .bg(cx.theme().secondary)
        .border_r_1()
        .border_color(cx.theme().border)
        .child(
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("数据源类型"),
        )
        .child(v_flex().gap(px(8.)).children(items))
}

fn render_form(
    kind: DatabaseKind,
    state: &mut NewDataSourceState,
    cx: &mut Context<CreateDataSourceWindow>,
) -> gpui::Div {
    match kind {
        DatabaseKind::Postgres => postgres::render(&mut state.postgres, cx),
        DatabaseKind::MySql => mysql::render(&mut state.mysql, cx),
        DatabaseKind::Sqlite => sqlite::render(&mut state.sqlite, cx),
        DatabaseKind::SqlServer => sqlserver::render(&mut state.sqlserver, cx),
    }
}

fn render_main_panel(
    view: &mut CreateDataSourceWindow,
    cx: &mut Context<CreateDataSourceWindow>,
) -> gpui::Div {
    let mut panel = v_flex()
        .flex_1()
        .gap(px(16.))
        .px(px(24.))
        .py(px(24.));

    match view.state.selected {
        Some(kind) => {
            panel = panel.child(
                div()
                    .text_base()
                    .font_semibold()
                    .child(format!("配置 {}", kind.label())),
            );
            let mut form_container = v_flex().flex_1();
            form_container.style().min_size.height = Some(Length::Definite(px(0.).into()));
            form_container.style().overflow.y = Some(Overflow::Scroll);
            let form_container = form_container.child(render_form(kind, &mut view.state, cx));

            panel = panel
                .child(
                    v_flex()
                        .flex_1()
                        .gap(px(16.))
                        .child(form_container),
                )
                .child(render_footer(cx));
        }
        None => {
            panel = panel.child(
                v_flex()
                    .flex_1()
                    .items_center()
                    .justify_center()
                    .gap(px(12.))
                    .child(div().text_base().font_semibold().child("请选择左侧的数据源类型"))
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child("选择后将显示对应的连接配置表单。"),
                    ),
            );
        }
    }

    panel
}

fn render_footer(cx: &mut Context<CreateDataSourceWindow>) -> gpui::Div {
    h_flex()
        .justify_end()
        .gap(px(8.))
        .child(
            Button::new("modal-cancel")
                .ghost()
                .label("取消")
                .on_click(cx.listener(|this: &mut CreateDataSourceWindow, _, window, cx| {
                    this.close(window, cx);
                })),
        )
        .child(
            Button::new("modal-save")
                .primary()
                .label("保存")
                .on_click(cx.listener(|this: &mut CreateDataSourceWindow, _, window, cx| {
                    this.submit(window, cx);
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

fn kind_icon_path(kind: DatabaseKind) -> &'static str {
    match kind {
        DatabaseKind::Postgres => "icons/db/postgresql.svg",
        DatabaseKind::MySql => "icons/db/mysql.svg",
        DatabaseKind::Sqlite => "icons/db/sqlite.svg",
        DatabaseKind::SqlServer => "icons/db/sqlserver.svg",
    }
}
