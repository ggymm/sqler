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

pub struct CreateWindow {
    selected: Option<DataSourceKind>,

    mysql: Entity<mysql::MySQLCreate>,
    oracle: Entity<oracle::OracleCreate>,
    sqlite: Entity<sqlite::SQLiteCreate>,
    sqlserver: Entity<sqlserver::SQLServerCreate>,
    postgres: Entity<postgres::PostgresCreate>,
    redis: Entity<redis::RedisCreate>,
    mongodb: Entity<mongodb::MongoDBCreate>,

    parent: WeakEntity<SqlerApp>,
}

impl CreateWindow {
    pub fn new(
        parent: WeakEntity<SqlerApp>,
        window: &mut Window,
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

        Self {
            selected: None,

            mysql: cx.new(|cx| mysql::MySQLCreate::new(window, cx)),
            oracle: cx.new(|cx| oracle::OracleCreate::new(window, cx)),
            sqlite: cx.new(|cx| sqlite::SQLiteCreate::new(window, cx)),
            sqlserver: cx.new(|cx| sqlserver::SQLServerCreate::new(window, cx)),
            postgres: cx.new(|cx| postgres::PostgresCreate::new(window, cx)),
            redis: cx.new(|cx| redis::RedisCreate::new(window, cx)),
            mongodb: cx.new(|cx| mongodb::MongoDBCreate::new(window, cx)),

            parent,
        }
    }

    fn deselect(
        &mut self,
        cx: &mut Context<Self>,
    ) {
        if self.selected.take().is_some() {
            cx.notify();
        }
    }

    fn selected(
        &mut self,
        kind: DataSourceKind,
        cx: &mut Context<Self>,
    ) {
        if self.selected != Some(kind) {
            self.selected = Some(kind);
            cx.notify();
        }
    }

    fn check_conn(
        &mut self,
        _cx: &mut Context<Self>,
    ) -> bool {
        if let Some(_selected) = self.selected.take() {}

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
        let selected = self.selected;

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
                            DataSourceKind::MySQL => self.mysql.clone().into_any_element(),
                            DataSourceKind::Oracle => self.oracle.clone().into_any_element(),
                            DataSourceKind::SQLite => self.sqlite.clone().into_any_element(),
                            DataSourceKind::SQLServer => self.sqlserver.clone().into_any_element(),
                            DataSourceKind::PostgreSQL => self.postgres.clone().into_any_element(),
                            DataSourceKind::Redis => self.redis.clone().into_any_element(),
                            DataSourceKind::MongoDB => self.mongodb.clone().into_any_element(),
                        }),
                    None => div().p_6().gap_5().col_full().scrollable(Axis::Vertical).children(
                        DataSourceKind::all()
                            .iter()
                            .map(|kind| {
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .p_4()
                                    .gap_4()
                                    .h_20()
                                    .w_full()
                                    .bg(theme.list)
                                    .border_1()
                                    .border_color(theme.border)
                                    .rounded_lg()
                                    .cursor_pointer()
                                    .id(("datasource-type-{}", *kind as u64))
                                    .hover(|this| this.bg(theme.list_hover))
                                    .child(div().w_12().h_12().child(img(kind.image()).size_full().rounded_lg()))
                                    .child(
                                        div()
                                            .flex()
                                            .flex_1()
                                            .flex_col()
                                            .items_start()
                                            .justify_center()
                                            .child(div().text_base().font_semibold().child(kind.label()))
                                            .child(div().text_sm().child(kind.description())),
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
