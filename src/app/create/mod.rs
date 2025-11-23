use gpui::{prelude::*, *};
use gpui_component::{button::Button, ActiveTheme, StyledExt};

use crate::{
    app::{comps::DivExt, SqlerApp},
    driver::check_connection,
    model::{DataSource, DataSourceKind, DataSourceOptions},
};

mod mongodb;
mod mysql;
mod oracle;
mod postgres;
mod redis;
mod sqlite;
mod sqlserver;

#[derive(Clone, Debug)]
pub enum CreateStatus {
    Testing,
    Error(String),
    Success(String),
}

pub struct CreateWindow {
    kind: Option<DataSourceKind>,
    parent: WeakEntity<SqlerApp>,
    status: Option<CreateStatus>,
    update_id: Option<String>,

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
        source: Option<&DataSource>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let parent_for_release = parent.clone();
        let _ = cx.on_release(move |_, app| {
            if let Some(parent) = parent_for_release.upgrade() {
                let _ = parent.update(app, |app, _| {
                    app.close_window("create");
                });
            }
        });

        let kind;
        let name;
        let update_id;
        let mut mysql_opts = None;
        let mut sqlite_opts = None;
        let mut postgres_opts = None;
        let mut oracle_opts = None;
        let mut sqlserver_opts = None;
        let mut redis_opts = None;
        let mut mongodb_opts = None;
        if let Some(s) = source.as_ref() {
            kind = Some(s.kind);
            name = Some(s.name.as_str());
            update_id = Some(s.id.clone());
            match &s.options {
                DataSourceOptions::MySQL(opts) => mysql_opts = Some(opts),
                DataSourceOptions::SQLite(opts) => sqlite_opts = Some(opts),
                DataSourceOptions::Postgres(opts) => postgres_opts = Some(opts),
                DataSourceOptions::Oracle(opts) => oracle_opts = Some(opts),
                DataSourceOptions::SQLServer(opts) => sqlserver_opts = Some(opts),
                DataSourceOptions::Redis(opts) => redis_opts = Some(opts),
                DataSourceOptions::MongoDB(opts) => mongodb_opts = Some(opts),
            }
        } else {
            kind = None;
            name = None;
            update_id = None;
        }

        Self {
            kind,
            parent,
            status: None,
            update_id,

            mysql: cx.new(|cx| mysql::MySQLCreate::new(name, mysql_opts, window, cx)),
            oracle: cx.new(|cx| oracle::OracleCreate::new(name, oracle_opts, window, cx)),
            sqlite: cx.new(|cx| sqlite::SQLiteCreate::new(name, sqlite_opts, window, cx)),
            sqlserver: cx.new(|cx| sqlserver::SQLServerCreate::new(name, sqlserver_opts, window, cx)),
            postgres: cx.new(|cx| postgres::PostgresCreate::new(name, postgres_opts, window, cx)),
            redis: cx.new(|cx| redis::RedisCreate::new(name, redis_opts, window, cx)),
            mongodb: cx.new(|cx| mongodb::MongoDBCreate::new(name, mongodb_opts, window, cx)),
        }
    }

    fn cancel(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(parent) = self.parent.upgrade() {
            let _ = parent.update(cx, |app, _| {
                app.close_window("create");
            });
        }
        window.remove_window();
    }

    fn check_conn(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(kind) = self.kind else {
            self.status = Some(CreateStatus::Error("请先选择数据源类型".to_string()));
            cx.notify();
            return;
        };

        let options = match kind {
            DataSourceKind::MySQL => DataSourceOptions::MySQL(self.mysql.read(cx).options(cx)),
            DataSourceKind::SQLite => DataSourceOptions::SQLite(self.sqlite.read(cx).options(cx)),
            DataSourceKind::Postgres => DataSourceOptions::Postgres(self.postgres.read(cx).options(cx)),
            DataSourceKind::Oracle => {
                self.status = Some(CreateStatus::Error("Oracle 驱动暂未实现".to_string()));
                cx.notify();
                return;
            }
            DataSourceKind::SQLServer => {
                self.status = Some(CreateStatus::Error("SQL Server 驱动暂未实现".to_string()));
                cx.notify();
                return;
            }
            DataSourceKind::Redis => DataSourceOptions::Redis(self.redis.read(cx).options(cx)),
            DataSourceKind::MongoDB => DataSourceOptions::MongoDB(self.mongodb.read(cx).options(cx)),
        };

        self.status = Some(CreateStatus::Testing);
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
                            this.status = Some(CreateStatus::Success("连接成功".to_string()));
                        }
                        Err(e) => {
                            this.status = Some(CreateStatus::Error(format!("{}", e)));
                        }
                    }
                    cx.notify();
                });
            });
        })
        .detach();
    }

    fn create_conn(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(kind) = self.kind else {
            self.status = Some(CreateStatus::Error("请先选择数据源类型".to_string()));
            cx.notify();
            return;
        };

        // 构建
        let name = match kind {
            DataSourceKind::MySQL => self.mysql.read(cx).name.read(cx).value().to_string(),
            DataSourceKind::SQLite => self.sqlite.read(cx).name.read(cx).value().to_string(),
            DataSourceKind::Postgres => self.postgres.read(cx).name.read(cx).value().to_string(),
            DataSourceKind::Oracle => self.oracle.read(cx).name.read(cx).value().to_string(),
            DataSourceKind::SQLServer => self.sqlserver.read(cx).name.read(cx).value().to_string(),
            DataSourceKind::Redis => self.redis.read(cx).name.read(cx).value().to_string(),
            DataSourceKind::MongoDB => self.mongodb.read(cx).name.read(cx).value().to_string(),
        };
        let options = match kind {
            DataSourceKind::MySQL => DataSourceOptions::MySQL(self.mysql.read(cx).options(cx)),
            DataSourceKind::SQLite => DataSourceOptions::SQLite(self.sqlite.read(cx).options(cx)),
            DataSourceKind::Postgres => DataSourceOptions::Postgres(self.postgres.read(cx).options(cx)),
            DataSourceKind::Oracle => {
                self.status = Some(CreateStatus::Error("Oracle 驱动暂未实现".to_string()));
                cx.notify();
                return;
            }
            DataSourceKind::SQLServer => {
                self.status = Some(CreateStatus::Error("SQL Server 驱动暂未实现".to_string()));
                cx.notify();
                return;
            }
            DataSourceKind::Redis => DataSourceOptions::Redis(self.redis.read(cx).options(cx)),
            DataSourceKind::MongoDB => DataSourceOptions::MongoDB(self.mongodb.read(cx).options(cx)),
        };

        // 保存
        let result = if let Some(parent) = self.parent.upgrade() {
            parent.update(cx, |app, cx| {
                if let Some(id) = &self.update_id {
                    // 编辑模式：更新现有数据源
                    if let Some(source) = app.cache.sources_mut().iter_mut().find(|s| &s.id == id) {
                        source.name = name;
                        source.kind = kind;
                        source.options = options;
                    }
                } else {
                    // 新建模式：添加新数据源
                    let source = DataSource::new(name, kind, options);
                    app.cache.sources_mut().push(source);
                }
                let result = app.cache.sources_update();

                if result.is_ok() {
                    cx.notify();
                }

                result
            })
        } else {
            return;
        };
        match result {
            Ok(()) => {
                self.cancel(window, cx);
            }
            Err(e) => {
                self.status = Some(CreateStatus::Error(format!("保存失败: {}", e)));
                cx.notify();
            }
        }
    }
}

impl Render for CreateWindow {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let kind = self.kind;
        let status = self.status.clone();

        let theme = cx.theme();
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
                    .child(div().text_xl().font_semibold().child(match kind {
                        None => "新建数据源".to_string(),
                        Some(kind) => format!("配置 {}", kind.label()),
                    })),
            )
            .child(
                div().id("datasource-create").col_full().child(match kind {
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
                                        move |this: &mut CreateWindow, _, _, cx| {
                                            if this.kind != Some(*kind) {
                                                this.kind = Some(*kind);
                                                this.status = None;
                                                cx.notify();
                                            }
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
                            .label("测试连接")
                            .outline()
                            .on_click(cx.listener(|this: &mut CreateWindow, _, window, cx| {
                                this.check_conn(window, cx);
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
                                    .label("上一步")
                                    .outline()
                                    .on_click(cx.listener(|this: &mut CreateWindow, _, _, cx| {
                                        if this.kind.take().is_some() {
                                            cx.notify();
                                        }
                                    })),
                            )
                            .child(
                                Button::new("datasource-create-cancel")
                                    .label("取消")
                                    .outline()
                                    .on_click(cx.listener(|this: &mut CreateWindow, _, window, cx| {
                                        this.cancel(window, cx);
                                    })),
                            )
                            .child(
                                Button::new("datasource-create-confirm")
                                    .label("保存")
                                    .outline()
                                    .on_click(cx.listener(|this: &mut CreateWindow, _, window, cx| {
                                        this.create_conn(window, cx);
                                    })),
                            ),
                    )
                    .children(status.as_ref().map(|s| {
                        let (bg, fg, message) = match s {
                            CreateStatus::Testing => (theme.info, theme.info_foreground, "测试连接...".to_string()),
                            CreateStatus::Success(msg) => (theme.success, theme.success_foreground, msg.clone()),
                            CreateStatus::Error(msg) => (theme.danger, theme.danger_foreground, msg.clone()),
                        };

                        div()
                            .flex()
                            .items_center()
                            .p_2()
                            .h(px(36.))
                            .w_full()
                            .top(px(-36.))
                            .left_0()
                            .absolute()
                            .bg(bg)
                            .text_color(fg)
                            .child(div().text_sm().overflow_hidden().whitespace_nowrap().child(message))
                    })),
            )
    }
}
