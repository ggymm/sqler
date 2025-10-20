use gpui::*;
use gpui_component::button::Button;
use gpui_component::button::ButtonVariants;
use gpui_component::h_flex;
use gpui_component::v_flex;
use gpui_component::ActiveTheme;
use gpui_component::Disableable;
use gpui_component::StyledExt;

use crate::app::NewDataSourceState;
use crate::app::SqlerApp;
use crate::DataSourceType;

pub mod mongodb;
pub mod mysql;
pub mod oracle;
pub mod postgres;
pub mod redis;
pub mod sqlite;
pub mod sqlserver;

pub struct CreateDataSourceWindow {
    state: NewDataSourceState,
    parent: WeakEntity<SqlerApp>,
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

        Self { state, parent }
    }

    fn clear_parent(
        &self,
        cx: &mut Context<Self>,
    ) {
        if let Some(parent) = self.parent.upgrade() {
            let _ = parent.update(cx, |app, cx| {
                app.clear_new_data_source_window();
                cx.notify();
            });
        }
    }

    fn back_to_selection(
        &mut self,
        cx: &mut Context<Self>,
    ) {
        if self.state.selected.take().is_some() {
            cx.notify();
        }
    }

    fn select_kind(
        &mut self,
        kind: DataSourceType,
        cx: &mut Context<Self>,
    ) {
        if self.state.selected != Some(kind) {
            self.state.selected = Some(kind);
            cx.notify();
        }
    }

    fn close_window(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.clear_parent(cx);
        window.remove_window();
    }

    fn submit(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.close_window(window, cx);
    }
}

impl Render for CreateDataSourceWindow {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let title = "新建数据源";

        let body = render_body(self, cx);
        let footer = render_footer(self, cx);

        v_flex()
            .size_full()
            .child(
                h_flex()
                    .justify_between()
                    .items_center()
                    .px(px(32.))
                    .py(px(20.))
                    .bg(cx.theme().tab_bar)
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .child(div().text_xl().font_semibold().child(title)),
            )
            .child(body)
            .child(footer)
            .into_any_element()
    }
}

fn render_body(
    view: &mut CreateDataSourceWindow,
    cx: &mut Context<CreateDataSourceWindow>,
) -> Stateful<gpui::Div> {
    let mut body_container = v_flex().flex_1().id("create-window-body").overflow_scroll();
    body_container.style().min_size.height = Some(Length::Definite(px(0.).into()));

    let content = match view.state.selected {
        Some(kind) => render_form_panel(kind, view, cx),
        None => render_type_selection(cx),
    };

    body_container.child(content)
}

fn render_type_selection(cx: &mut Context<CreateDataSourceWindow>) -> gpui::Div {
    let cards = DataSourceType::all()
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
                    div()
                        .flex_shrink_0()
                        .w(px(48.))
                        .h(px(48.))
                        .child(img(kind.image()).size_full()),
                )
                .child(
                    v_flex()
                        .flex_1()
                        .gap(px(4.))
                        .child(div().text_base().font_semibold().child(kind.label()))
                        .child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
                                .child(kind.description()),
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

    v_flex().px(px(32.)).py(px(24.)).gap(px(12.)).children(cards)
}

fn render_form_panel(
    kind: DataSourceType,
    view: &mut CreateDataSourceWindow,
    cx: &mut Context<CreateDataSourceWindow>,
) -> gpui::Div {
    v_flex()
        .gap(px(20.))
        .px(px(32.))
        .py(px(24.))
        .child(
            div()
                .text_base()
                .font_semibold()
                .child(format!("配置 {}", kind.label())),
        )
        .child(match kind {
            DataSourceType::MySQL => mysql::render(&mut view.state.mysql, cx),
            DataSourceType::Oracle => postgres::render(&mut view.state.postgres, cx),
            DataSourceType::SQLite => postgres::render(&mut view.state.postgres, cx),
            DataSourceType::SQLServer => postgres::render(&mut view.state.postgres, cx),
            DataSourceType::PostgreSQL => postgres::render(&mut view.state.postgres, cx),
            DataSourceType::Redis => postgres::render(&mut view.state.postgres, cx),
            DataSourceType::MongoDB => postgres::render(&mut view.state.postgres, cx),
        })
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
                .child(Button::new("modal-cancel").ghost().label("取消").on_click(cx.listener(
                    |this: &mut CreateDataSourceWindow, _, window, cx| {
                        this.close_window(window, cx);
                    },
                )))
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
