use gpui::*;
use gpui_component::{button::Button, ActiveTheme, StyledExt};

use crate::{
    app::{comps::DivExt, SqlerApp},
    option::DataSourceKind,
};

pub mod mongodb;
pub mod mysql;
pub mod oracle;
pub mod postgres;
pub mod redis;
pub mod sqlite;
pub mod sqlserver;

#[derive(Clone)]
pub struct CreateState {
    pub selected: Option<DataSourceKind>,

    pub mysql: mysql::MySQLState,
    pub oracle: oracle::OracleState,
    pub sqlite: sqlite::SqliteState,
    pub sqlserver: sqlserver::SqlServerState,
    pub postgres: postgres::PostgresState,
    pub redis: redis::RedisState,
    pub mongodb: mongodb::MongoDBState,
}

impl CreateState {
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

pub struct CreateWindow {
    state: CreateState,
    parent: WeakEntity<SqlerApp>,
}

impl CreateWindow {
    pub fn new(
        state: CreateState,
        parent: WeakEntity<SqlerApp>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let parent_for_release = parent.clone();
        let _ = cx.on_release(move |_, app| {
            if let Some(parent) = parent_for_release.upgrade() {
                let _ = parent.update(app, |app, cx| {
                    app.close_create_window();
                    cx.notify();
                });
            }
        });

        Self { state, parent }
    }

    fn deselect(
        &mut self,
        cx: &mut Context<Self>,
    ) {
        if self.state.selected.take().is_some() {
            cx.notify();
        }
    }

    fn selected(
        &mut self,
        kind: DataSourceKind,
        cx: &mut Context<Self>,
    ) {
        if self.state.selected != Some(kind) {
            self.state.selected = Some(kind);
            cx.notify();
        }
    }

    fn check_conn(
        &mut self,
        _cx: &mut Context<Self>,
    ) -> bool {
        if let Some(_selected) = self.state.selected.take() {}

        true
    }

    fn close_window(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(parent) = self.parent.upgrade() {
            let _ = parent.update(cx, |app, cx| {
                app.close_create_window();
                cx.notify();
            });
        }
        window.remove_window();
    }
}

impl Render for CreateWindow {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let theme = cx.theme();
        let selected = self.state.selected;

        div()
            .col_full()
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
                    .child(match selected {
                        Some(kind) => div().text_xl().font_semibold().child(format!("配置 {}", kind.label())),
                        None => div().text_xl().font_semibold().child("新建数据源"),
                    }),
            )
            .child(
                div().id("datasource-create").col_full().child(match selected {
                    Some(kind) => div()
                        .p_6()
                        .gap_5()
                        .col_full()
                        .scrollable(Axis::Vertical)
                        .child(match kind {
                            DataSourceKind::MySQL => mysql::render(&mut self.state.mysql),
                            DataSourceKind::Oracle => oracle::render(&mut self.state.oracle, cx),
                            DataSourceKind::SQLite => sqlite::render(&mut self.state.sqlite, cx),
                            DataSourceKind::SQLServer => sqlserver::render(&mut self.state.sqlserver, cx),
                            DataSourceKind::PostgreSQL => postgres::render(&mut self.state.postgres, cx),
                            DataSourceKind::Redis => redis::render(&mut self.state.redis),
                            DataSourceKind::MongoDB => mongodb::render(&mut self.state.mongodb, cx),
                        }),
                    None => div().p_6().gap_5().col_full().scrollable(Axis::Vertical).children(
                        DataSourceKind::all()
                            .iter()
                            .map(|kind| {
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .w_full()
                                    .h_20()
                                    .p_4()
                                    .gap_4()
                                    .bg(theme.secondary)
                                    .border_1()
                                    .border_color(theme.border)
                                    .rounded_lg()
                                    .cursor_pointer()
                                    .id(("datasource-type-{}", *kind as u64))
                                    .hover(|this| this.bg(theme.secondary_hover))
                                    .child(div().w_12().h_12().child(img(kind.image()).size_full().rounded_lg()))
                                    .child(
                                        div()
                                            .flex()
                                            .flex_1()
                                            .flex_col()
                                            .items_start()
                                            .justify_center()
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
                                        move |this: &mut CreateWindow, _ev, _window, cx| {
                                            this.selected(kind, cx);
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
                            .outline()
                            .label("测试连接")
                            .on_click(cx.listener(|this: &mut CreateWindow, _ev, _window, cx| {
                                this.check_conn(cx);
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
                                    .outline()
                                    .label("上一步")
                                    .on_click(cx.listener(|this: &mut CreateWindow, _ev, _window, cx| {
                                        this.deselect(cx);
                                    })),
                            )
                            .child(
                                Button::new("datasource-create-cancel")
                                    .outline()
                                    .label("取消")
                                    .on_click(cx.listener(|this: &mut CreateWindow, _ev, window, cx| {
                                        this.close_window(window, cx);
                                    })),
                            )
                            .child(
                                Button::new("datasource-create-confirm")
                                    .outline()
                                    .label("确认")
                                    .on_click(cx.listener(|this: &mut CreateWindow, _ev, window, cx| {
                                        this.close_window(window, cx);
                                    })),
                            ),
                    ),
            )
            .into_any_element()
    }
}
