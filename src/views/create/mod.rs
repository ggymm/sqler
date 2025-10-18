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
    InteractiveElement as _,
    Length,
    Overflow,
    ParentElement,
    Render,
    SharedString,
    StatefulInteractiveElement as _,
    Styled,
    WeakEntity,
    Window,
};
use gpui_component::{
    button::{Button, ButtonVariants as _},
    h_flex,
    v_flex,
    ActiveTheme as _,
    Disableable as _,
    Icon,
    Size,
    Sizable as _,
    StyledExt,
};

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

    fn close_window(&self, window: &mut Window, cx: &mut Context<Self>) {
        self.clear_parent(cx);
        window.remove_window();
    }

    fn submit(&self, window: &mut Window, cx: &mut Context<Self>) {
        self.close_window(window, cx);
    }

    fn render_content(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let header = render_header(cx);
        let body = render_body(self, cx);
        let footer = render_footer(self, cx);

        v_flex()
            .size_full()
            .bg(cx.theme().background)
            .child(header)
            .child(body)
            .child(footer)
            .into_any_element()
    }
}

impl Render for CreateDataSourceWindow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.render_content(window, cx)
    }
}

fn render_header(cx: &mut Context<CreateDataSourceWindow>) -> gpui::Div {
    let title = "新建数据源";

    h_flex()
        .justify_between()
        .items_center()
        .px(px(32.))
        .py(px(20.))
        .bg(cx.theme().tab_bar)
        .border_b_1()
        .border_color(cx.theme().border)
        .child(
            div()
                .text_xl()
                .font_semibold()
                .child(title)
        )
}

fn render_body(
    view: &mut CreateDataSourceWindow,
    cx: &mut Context<CreateDataSourceWindow>,
) -> gpui::Div {
    let mut body_container = v_flex().flex_1();
    body_container.style().min_size.height = Some(Length::Definite(px(0.).into()));
    body_container.style().overflow.y = Some(Overflow::Scroll);

    let content = match view.state.selected {
        Some(kind) => render_form_panel(kind, view, cx),
        None => render_type_selection(cx),
    };

    body_container.child(content)
}

fn render_type_selection(cx: &mut Context<CreateDataSourceWindow>) -> gpui::Div {
    let cards = DatabaseKind::all()
        .iter()
        .map(|kind| {
            h_flex()
                .w_full()
                .h(px(80.))
                .items_center()
                .gap(px(16.))
                .px(px(20.))
                .py(px(16.))
                .bg(cx.theme().secondary)
                .border_1()
                .border_color(cx.theme().border)
                .rounded(px(8.))
                .cursor_pointer()
                .id(SharedString::from(format!("type-card-{}", (*kind as u8))))
                .hover(|this| this.bg(cx.theme().secondary_hover))
                .child(
                    Icon::default()
                        .path(kind_icon_path(*kind))
                        .with_size(Size::Large)
                        .view(cx),
                )
                .child(
                    v_flex()
                        .flex_1()
                        .gap(px(4.))
                        .child(
                            div()
                                .text_base()
                                .font_semibold()
                                .child(kind.label())
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
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
        .px(px(32.))
        .py(px(24.))
        .gap(px(12.))
        .children(cards)
}

fn render_form_panel(
    kind: DatabaseKind,
    view: &mut CreateDataSourceWindow,
    cx: &mut Context<CreateDataSourceWindow>,
) -> gpui::Div {
    v_flex()
        .gap(px(20.))
        .px(px(32.))
        .py(px(24.))
        .child(div().text_base().font_semibold().child(format!("配置 {}", kind.label())))
        .child(render_form(kind, &mut view.state, cx))
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

fn render_footer(
    view: &mut CreateDataSourceWindow,
    cx: &mut Context<CreateDataSourceWindow>,
) -> gpui::Div {
    let has_selection = view.state.selected.is_some();

    h_flex()
        .justify_end()
        .gap(px(12.))
        .px(px(32.))
        .py(px(20.))
        .border_t_1()
        .border_color(cx.theme().border)
        .bg(cx.theme().tab_bar)
        .child(
            Button::new("modal-test-connection")
                .ghost()
                .label("测试连接")
                .disabled(!has_selection)
                .on_click(cx.listener(|_this: &mut CreateDataSourceWindow, _, _window, _cx| {
                    // TODO: 实现连接测试逻辑
                })),
        )
        .child(
            h_flex()
                .gap(px(12.))
                .child(
                    Button::new("modal-back")
                        .ghost()
                        .disabled(!has_selection)
                        .label("上一步")
                        .on_click(cx.listener(|this: &mut CreateDataSourceWindow, _, _, cx| {
                            this.back_to_selection(cx);
                        })),
                )
                .child(
                    Button::new("modal-cancel")
                        .ghost()
                        .label("取消")
                        .on_click(cx.listener(|this: &mut CreateDataSourceWindow, _, window, cx| {
                            this.close_window(window, cx);
                        })),
                )
                .child(
                    Button::new("modal-save")
                        .primary()
                        .disabled(!has_selection)
                        .label("保存")
                        .on_click(cx.listener(|this: &mut CreateDataSourceWindow, _, window, cx| {
                            this.submit(window, cx);
                        })),
                ),
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
