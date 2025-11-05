use gpui::{prelude::*, *};
use gpui_component::{button::Button, ActiveTheme, StyledExt};

use crate::{
    app::{comps::DivExt, SqlerApp},
    driver::{check_connection, DataSourceKind, DataSourceOptions},
};

pub mod mongodb;
pub mod mysql;
pub mod oracle;
pub mod postgres;
pub mod redis;
pub mod sqlite;
pub mod sqlserver;

#[derive(Clone, Debug)]
pub enum ConnectionStatus {
    None,
    Testing,
    Success(String),
    Error(String),
}

pub struct CreateWindow {
    kind: Option<DataSourceKind>,
    parent: WeakEntity<SqlerApp>,
    status: ConnectionStatus,

    mysql: Entity<mysql::MySQLCreate>,
    oracle: Entity<oracle::OracleCreate>,
    sqlite: Entity<sqlite::SQLiteCreate>,
    sqlserver: Entity<sqlserver::SQLServerCreate>,
    postgres: Entity<postgres::PostgresCreate>,
    redis: Entity<redis::RedisCreate>,
    mongodb: Entity<mongodb::MongoDBCreate>,
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
            kind: None,
            parent,
            status: ConnectionStatus::None,

            mysql: cx.new(|cx| mysql::MySQLCreate::new(window, cx)),
            oracle: cx.new(|cx| oracle::OracleCreate::new(window, cx)),
            sqlite: cx.new(|cx| sqlite::SQLiteCreate::new(window, cx)),
            sqlserver: cx.new(|cx| sqlserver::SQLServerCreate::new(window, cx)),
            postgres: cx.new(|cx| postgres::PostgresCreate::new(window, cx)),
            redis: cx.new(|cx| redis::RedisCreate::new(window, cx)),
            mongodb: cx.new(|cx| mongodb::MongoDBCreate::new(window, cx)),
        }
    }

    fn deselect(
        &mut self,
        cx: &mut Context<Self>,
    ) {
        if self.kind.take().is_some() {
            cx.notify();
        }
    }

    fn selected(
        &mut self,
        kind: DataSourceKind,
        cx: &mut Context<Self>,
    ) {
        if self.kind != Some(kind) {
            self.kind = Some(kind);
            self.status = ConnectionStatus::None;
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
                app.close_create_window();
                cx.notify();
            });
        }
        window.remove_window();
    }

    fn check_connection(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(kind) = self.kind else {
            self.status = ConnectionStatus::Error("请先选择数据源类型".to_string());
            cx.notify();
            return;
        };

        let options = match kind {
            DataSourceKind::MySQL => DataSourceOptions::MySQL(self.mysql.read(cx).options(cx)),
            DataSourceKind::SQLite => DataSourceOptions::SQLite(self.sqlite.read(cx).options(cx)),
            DataSourceKind::Postgres => DataSourceOptions::Postgres(self.postgres.read(cx).options(cx)),
            DataSourceKind::Oracle => {
                self.status = ConnectionStatus::Error("Oracle 驱动暂未实现".to_string());
                cx.notify();
                return;
            }
            DataSourceKind::SQLServer => {
                self.status = ConnectionStatus::Error("SQL Server 驱动暂未实现".to_string());
                cx.notify();
                return;
            }
            DataSourceKind::Redis => DataSourceOptions::Redis(self.redis.read(cx).options(cx)),
            DataSourceKind::MongoDB => DataSourceOptions::MongoDB(self.mongodb.read(cx).options(cx)),
        };

        self.status = ConnectionStatus::Testing;
        cx.notify();

        cx.spawn_in(window, async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move { check_connection(&options) })
                .await;

            let _ = cx.update(|_, cx| {
                let _ = this.update(cx, |this, cx| {
                    match result {
                        Ok(_) => {
                            this.status = ConnectionStatus::Success("连接成功".to_string());
                        }
                        Err(e) => {
                            this.status = ConnectionStatus::Error(format!("{}", e));
                        }
                    }
                    cx.notify();
                });
            });
        })
        .detach();
    }

    fn create_connection(
        &mut self,
        _cx: &mut Context<Self>,
    ) {
        // 保存方法
    }
}

impl Render for CreateWindow {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let theme = cx.theme();
        let selected = self.kind;
        let status = self.status.clone();

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
                            DataSourceKind::SQLite => self.sqlite.clone().into_any_element(),
                            DataSourceKind::Postgres => self.postgres.clone().into_any_element(),
                            DataSourceKind::Oracle => self.oracle.clone().into_any_element(),
                            DataSourceKind::SQLServer => self.sqlserver.clone().into_any_element(),
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
                    .relative()
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
                            .on_click(cx.listener(|this: &mut CreateWindow, _ev, window, cx| {
                                this.check_connection(window, cx);
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
                    )
                    .when(!matches!(status, ConnectionStatus::None), |this| {
                        this.child(
                            div()
                                .flex()
                                .items_center()
                                .p_2()
                                .h(px(36.))
                                .w_full()
                                .top(px(-36.))
                                .left_0()
                                .absolute()
                                .when(matches!(status, ConnectionStatus::Testing), |this| {
                                    this.bg(theme.info).text_color(theme.info_foreground).child(
                                        div()
                                            .text_sm()
                                            .overflow_hidden()
                                            .whitespace_nowrap()
                                            .child("正在测试连接..."),
                                    )
                                })
                                .when(matches!(status, ConnectionStatus::Success(_)), |this| {
                                    this.bg(theme.success).text_color(theme.success_foreground).child(
                                        div().text_sm().overflow_hidden().whitespace_nowrap().child(
                                            if let ConnectionStatus::Success(msg) = &status {
                                                msg.clone()
                                            } else {
                                                "连接成功".to_string()
                                            },
                                        ),
                                    )
                                })
                                .when(matches!(status, ConnectionStatus::Error(_)), |this| {
                                    this.bg(theme.danger).text_color(theme.danger_foreground).child(
                                        div().text_sm().overflow_hidden().whitespace_nowrap().child(
                                            if let ConnectionStatus::Error(msg) = &status {
                                                msg.clone()
                                            } else {
                                                "连接失败".to_string()
                                            },
                                        ),
                                    )
                                }),
                        )
                    }),
            )
            .into_any_element()
    }
}
