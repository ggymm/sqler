use gpui::*;
use gpui_component::button::Button;
use gpui_component::button::ButtonVariants;
use gpui_component::h_flex;
use gpui_component::v_flex;
use gpui_component::ActiveTheme;
use gpui_component::Disableable;
use gpui_component::StyledExt;

use crate::app::SqlerApp;
use crate::DataSourceType;

pub mod mongodb;
pub mod mysql;
pub mod oracle;
pub mod postgres;
pub mod redis;
pub mod sqlite;
pub mod sqlserver;

#[derive(Clone)]
pub struct NewDataSourceState {
    pub selected: Option<DataSourceType>,

    pub mysql: mysql::MySQLState,
    pub oracle: oracle::OracleState,
    pub sqlite: sqlite::SqliteState,
    pub sqlserver: sqlserver::SqlServerState,
    pub postgres: postgres::PostgresState,
    pub redis: redis::RedisState,
    pub mongodb: mongodb::MongoDBState,
}

impl NewDataSourceState {
    pub fn new(
        window: &mut Window,
        cx: &mut Context<SqlerApp>,
    ) -> Self {
        Self {
            selected: None,

            mysql: mysql::MySQLState::new(window, cx),
            oracle: oracle::OracleState::new(window, cx),
            sqlite: sqlite::SqliteState::new(window, cx),
            sqlserver: sqlserver::SqlServerState::new(window, cx),
            postgres: postgres::PostgresState::new(window, cx),
            redis: redis::RedisState::new(window, cx),
            mongodb: mongodb::MongoDBState::new(window, cx),
        }
    }
}

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
        let theme = cx.theme().clone();

        let has_selection = self.state.selected.is_some();

        v_flex()
            .size_full()
            .child(
                h_flex()
                    .justify_between()
                    .items_center()
                    .px(px(32.))
                    .py(px(20.))
                    .bg(theme.background)
                    .border_b_1()
                    .border_color(theme.border)
                    .child(div().text_xl().font_semibold().child(title)),
            )
            .child(
                div()
                    .id("create-window")
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_w_0()
                    .min_h_0()
                    .overflow_scroll()
                    .child(match self.state.selected {
                        Some(kind) => div()
                            .flex()
                            .flex_col()
                            .px_8()
                            .py_6()
                            .gap_5()
                            .child(
                                div()
                                    .text_base()
                                    .font_semibold()
                                    .child(format!("配置 {}", kind.label())),
                            )
                            .child(match kind {
                                DataSourceType::MySQL => mysql::render(&mut self.state.mysql, cx),
                                DataSourceType::Oracle => oracle::render(&mut self.state.oracle, cx),
                                DataSourceType::SQLite => sqlite::render(&mut self.state.sqlite, cx),
                                DataSourceType::SQLServer => sqlserver::render(&mut self.state.sqlserver, cx),
                                DataSourceType::PostgreSQL => postgres::render(&mut self.state.postgres, cx),
                                DataSourceType::Redis => redis::render(&mut self.state.redis, cx),
                                DataSourceType::MongoDB => mongodb::render(&mut self.state.mongodb, cx),
                            }),
                        None => div().flex().flex_col().px(px(32.)).py(px(24.)).gap(px(12.)).children(
                            DataSourceType::all()
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
                                .collect::<Vec<_>>(),
                        ),
                    }),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_end()
                    .px_8()
                    .py_5()
                    .gap_4()
                    .border_t_1()
                    .border_color(theme.border)
                    .bg(theme.tab_bar)
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
                    ),
            )
            .into_any_element()
    }
}
