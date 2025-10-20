use gpui::*;
use gpui_component::button::Button;
use gpui_component::button::ButtonVariants;
use gpui_component::ActiveTheme;
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

    fn go_form(
        &mut self,
        kind: DataSourceType,
        cx: &mut Context<Self>,
    ) {
        if self.state.selected != Some(kind) {
            self.state.selected = Some(kind);
            cx.notify();
        }
    }

    fn back_select(
        &mut self,
        cx: &mut Context<Self>,
    ) {
        if self.state.selected.take().is_some() {
            cx.notify();
        }
    }

    fn close_window(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(parent) = self.parent.upgrade() {
            let _ = parent.update(cx, |app, cx| {
                app.clear_new_data_source_window();
                cx.notify();
            });
        }
        window.remove_window();
    }
}

impl Render for CreateDataSourceWindow {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let theme = cx.theme();
        let selected = self.state.selected;

        div()
            .flex()
            .flex_col()
            .size_full()
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .px_8()
                    .py_5()
                    .bg(theme.secondary)
                    .border_b_1()
                    .border_color(theme.border)
                    .child(div().text_xl().font_semibold().child("新建数据源")),
            )
            .child(
                div()
                    .id("datasource-create")
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_w_0()
                    .min_h_0()
                    .overflow_scroll()
                    .child(match selected {
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
                        None => div().flex().flex_col().px_8().py_6().gap_5().children(
                            DataSourceType::all()
                                .iter()
                                .map(|kind| {
                                    div()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .w_full()
                                        .h_20()
                                        .px_5()
                                        .py_4()
                                        .gap_4()
                                        .bg(theme.secondary)
                                        .border_1()
                                        .border_color(theme.border)
                                        .rounded_lg()
                                        .cursor_pointer()
                                        .id(("datasource-type-{}", *kind as u64))
                                        .hover(|this| this.bg(theme.secondary_hover))
                                        .child(div().w_12().h_12().child(img(kind.image()).size_full()))
                                        .child(
                                            div()
                                                .flex()
                                                .flex_col()
                                                .items_start()
                                                .gap_2()
                                                .flex_1()
                                                .child(div().text_base().font_semibold().child(kind.label()))
                                                .child(
                                                    div()
                                                        .text_sm()
                                                        .text_color(theme.secondary_foreground)
                                                        .child(kind.description()),
                                                ),
                                        )
                                        .on_click(cx.listener({
                                            let kind = *kind;
                                            move |this: &mut CreateDataSourceWindow, _ev, _window, cx| {
                                                this.go_form(kind, cx);
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
                    .justify_between()
                    .px_8()
                    .py_5()
                    .gap_4()
                    .bg(theme.secondary)
                    .border_t_1()
                    .border_color(theme.border)
                    .child(
                        Button::new("datasource-check-connection")
                            .ghost()
                            .label("测试连接")
                            .on_click(cx.listener(|_this: &mut CreateDataSourceWindow, _ev, _window, _cx| {
                                // TODO: 实现连接测试逻辑
                            })),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_4()
                            .child(
                                Button::new("datasource-create-back")
                                    .ghost()
                                    .label("上一步")
                                    .on_click(cx.listener(|this: &mut CreateDataSourceWindow, _ev, _window, cx| {
                                        this.back_select(cx);
                                    })),
                            )
                            .child(
                                Button::new("datasource-create-cancel")
                                    .ghost()
                                    .label("取消")
                                    .on_click(cx.listener(|this: &mut CreateDataSourceWindow, _ev, window, cx| {
                                        this.close_window(window, cx);
                                    })),
                            )
                            .child(Button::new("datasource-create-save").primary().label("保存").on_click(
                                cx.listener(|this: &mut CreateDataSourceWindow, _ev, window, cx| {
                                    this.close_window(window, cx);
                                }),
                            )),
                    ),
            )
            .into_any_element()
    }
}
