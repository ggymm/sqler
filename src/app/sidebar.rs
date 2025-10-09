use iced::alignment::Horizontal;
use iced::widget::mouse_area;
use iced::widget::svg::Handle as SvgHandle;
use iced::widget::{button, column, container, row, scrollable, svg, text};
use iced::{Alignment, Background, Color, Element, Length, Shadow, Theme};

use crate::cache::{self, StoredConnectionConfig};

use crate::driver::ConnectionParams;

use super::{Message, Palette};

use std::time::{Duration, Instant};

const DOUBLE_CLICK_THRESHOLD: Duration = Duration::from_millis(400);

#[derive(Debug, Default)]
pub struct ConnectionsState {
    entries: Vec<Connection>,
    selected: Option<usize>,
    next_id: usize,
    active: Option<usize>,
    last_click: Option<(usize, Instant)>,
}

impl ConnectionsState {
    pub fn new() -> Self {
        let mut state = Self {
            entries: Vec::new(),
            selected: None,
            next_id: 1,
            active: None,
            last_click: None,
        };

        if let Some(cache) = cache::load_connections() {
            for record in cache.connections {
                if let Some(kind) = DatabaseKind::from_cache_key(&record.kind) {
                    let config = record
                        .config
                        .and_then(|stored| ConnectionConfig::from_stored(kind, stored))
                        .unwrap_or_else(|| ConnectionConfig::from_summary(kind, record.summary.clone()));

                    state.entries.push(Connection {
                        id: record.id,
                        name: record.name,
                        kind,
                        config,
                    });
                    state.next_id = state.next_id.max(record.id.saturating_add(1));
                }
            }

            if let Some(selected) = cache.selected {
                if state.entries.iter().any(|conn| conn.id == selected) {
                    state.selected = Some(selected);
                }
            }

            state.next_id = state.next_id.max(cache.next_id);
            if state.next_id == 0 {
                state.next_id = state
                    .entries
                    .iter()
                    .map(|conn| conn.id)
                    .max()
                    .unwrap_or(0)
                    .saturating_add(1);
            }
        }

        state
    }

    pub fn list(&self) -> &[Connection] {
        &self.entries
    }

    pub fn selected(&self) -> Option<usize> {
        self.selected
    }

    pub fn select(
        &mut self,
        id: usize,
    ) -> bool {
        let now = Instant::now();
        let double_clicked = self
            .last_click
            .map(|(last_id, instant)| last_id == id && now.duration_since(instant) <= DOUBLE_CLICK_THRESHOLD)
            .unwrap_or(false);

        if self.entries.iter().any(|conn| conn.id == id) {
            self.selected = Some(id);
        }

        self.last_click = Some((id, now));
        self.persist();

        double_clicked
    }

    pub fn next_id(&self) -> usize {
        self.next_id
    }

    pub fn add(
        &mut self,
        connection: Connection,
    ) {
        self.selected = Some(connection.id);
        self.entries.push(connection);
        self.next_id += 1;
        self.persist();
    }

    pub fn update(
        &mut self,
        connection: Connection,
    ) {
        if let Some(existing) = self.entries.iter_mut().find(|conn| conn.id == connection.id) {
            *existing = connection;
            self.persist();
        }
    }

    pub fn remove(
        &mut self,
        id: usize,
    ) {
        self.entries.retain(|conn| conn.id != id);
        if self.selected == Some(id) {
            self.selected = None;
        }
        if self.active == Some(id) {
            self.active = None;
        }
        if self.last_click.map(|(last, _)| last == id).unwrap_or(false) {
            self.last_click = None;
        }
        self.persist();
    }

    pub fn find(
        &self,
        id: usize,
    ) -> Option<&Connection> {
        self.entries.iter().find(|conn| conn.id == id)
    }

    pub fn active(&self) -> Option<usize> {
        self.active
    }

    pub fn activate(
        &mut self,
        id: usize,
    ) {
        if self.entries.iter().any(|conn| conn.id == id) {
            self.active = Some(id);
            self.selected = Some(id);
            self.persist();
        }
    }

    pub fn deactivate(&mut self) {
        self.active = None;
        self.persist();
    }

    fn persist(&self) {
        let snapshot = crate::cache::ConnectionsCache {
            next_id: self.next_id,
            selected: self.selected,
            connections: self
                .entries
                .iter()
                .map(|conn| crate::cache::ConnectionRecord {
                    id: conn.id,
                    name: conn.name.clone(),
                    kind: conn.kind.cache_key().to_string(),
                    summary: conn.summary(),
                    config: Some(conn.config.to_stored()),
                })
                .collect(),
        };

        if let Err(err) = cache::save_connections(&snapshot) {
            eprintln!("保存连接缓存失败: {err}");
        }
    }
}

#[derive(Debug, Clone)]
pub struct Connection {
    pub id: usize,
    pub name: String,
    pub kind: DatabaseKind,
    pub config: ConnectionConfig,
}

impl Connection {
    pub fn summary(&self) -> String {
        match &self.config {
            ConnectionConfig::Relational {
                host,
                port,
                database,
                username,
                ..
            } => format!("{username}@{host}:{port}/{database}"),
            ConnectionConfig::Sqlite { file_path } => file_path.clone(),
            ConnectionConfig::Mongo { connection_string } => connection_string.clone(),
            ConnectionConfig::Redis { host, port, .. } => format!("{host}:{port}"),
        }
    }

    pub fn to_params(&self) -> ConnectionParams {
        self.config.to_params(self.kind)
    }
}

#[derive(Debug, Clone)]
pub enum ConnectionConfig {
    Relational {
        host: String,
        port: u16,
        database: String,
        username: String,
        password: Option<String>,
    },
    Sqlite {
        file_path: String,
    },
    Mongo {
        connection_string: String,
    },
    Redis {
        host: String,
        port: u16,
        password: Option<String>,
    },
}

impl ConnectionConfig {
    pub fn from_stored(
        kind: DatabaseKind,
        stored: StoredConnectionConfig,
    ) -> Option<Self> {
        match (kind, stored) {
            (
                DatabaseKind::Postgresql | DatabaseKind::Mysql | DatabaseKind::Oracle | DatabaseKind::Sqlserver,
                StoredConnectionConfig::Relational {
                    host,
                    port,
                    database,
                    username,
                    password,
                },
            ) => Some(ConnectionConfig::Relational {
                host,
                port,
                database,
                username,
                password,
            }),
            (DatabaseKind::Sqlite, StoredConnectionConfig::Sqlite { file_path }) => {
                Some(ConnectionConfig::Sqlite { file_path })
            }
            (DatabaseKind::Mongodb, StoredConnectionConfig::Mongo { connection_string }) => {
                Some(ConnectionConfig::Mongo { connection_string })
            }
            (DatabaseKind::Redis, StoredConnectionConfig::Redis { host, port, password }) => {
                Some(ConnectionConfig::Redis { host, port, password })
            }
            (
                _,
                StoredConnectionConfig::Relational {
                    host,
                    port,
                    database,
                    username,
                    password,
                },
            ) => Some(ConnectionConfig::Relational {
                host,
                port,
                database,
                username,
                password,
            }),
            (_, StoredConnectionConfig::Sqlite { file_path }) => Some(ConnectionConfig::Sqlite { file_path }),
            (_, StoredConnectionConfig::Mongo { connection_string }) => {
                Some(ConnectionConfig::Mongo { connection_string })
            }
            (_, StoredConnectionConfig::Redis { host, port, password }) => {
                Some(ConnectionConfig::Redis { host, port, password })
            }
        }
    }

    pub fn to_stored(&self) -> StoredConnectionConfig {
        match self {
            ConnectionConfig::Relational {
                host,
                port,
                database,
                username,
                password,
            } => StoredConnectionConfig::Relational {
                host: host.clone(),
                port: *port,
                database: database.clone(),
                username: username.clone(),
                password: password.clone(),
            },
            ConnectionConfig::Sqlite { file_path } => StoredConnectionConfig::Sqlite {
                file_path: file_path.clone(),
            },
            ConnectionConfig::Mongo { connection_string } => StoredConnectionConfig::Mongo {
                connection_string: connection_string.clone(),
            },
            ConnectionConfig::Redis { host, port, password } => StoredConnectionConfig::Redis {
                host: host.clone(),
                port: *port,
                password: password.clone(),
            },
        }
    }

    pub fn from_summary(
        kind: DatabaseKind,
        summary: String,
    ) -> Self {
        match kind {
            DatabaseKind::Sqlite => ConnectionConfig::Sqlite { file_path: summary },
            DatabaseKind::Mongodb => ConnectionConfig::Mongo {
                connection_string: summary,
            },
            DatabaseKind::Redis => ConnectionConfig::Redis {
                host: summary,
                port: kind.default_port().unwrap_or(6379),
                password: None,
            },
            _ => ConnectionConfig::Relational {
                host: "localhost".into(),
                port: kind.default_port().unwrap_or(0),
                database: summary,
                username: "user".into(),
                password: None,
            },
        }
    }

    pub fn to_params(
        &self,
        kind: DatabaseKind,
    ) -> ConnectionParams {
        match self {
            ConnectionConfig::Relational {
                host,
                port,
                database,
                username,
                password,
            } => ConnectionParams {
                kind,
                host: Some(host.clone()),
                port: Some(*port),
                username: Some(username.clone()),
                password: password.clone(),
                database: Some(database.clone()),
                file_path: None,
                connection_string: None,
            },
            ConnectionConfig::Sqlite { file_path } => ConnectionParams {
                kind,
                host: None,
                port: None,
                username: None,
                password: None,
                database: None,
                file_path: Some(file_path.clone()),
                connection_string: None,
            },
            ConnectionConfig::Mongo { connection_string } => ConnectionParams {
                kind,
                host: None,
                port: None,
                username: None,
                password: None,
                database: None,
                file_path: None,
                connection_string: Some(connection_string.clone()),
            },
            ConnectionConfig::Redis { host, port, password } => ConnectionParams {
                kind,
                host: Some(host.clone()),
                port: Some(*port),
                username: None,
                password: password.clone(),
                database: None,
                file_path: None,
                connection_string: None,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseKind {
    Postgresql,
    Mysql,
    Sqlite,
    Mongodb,
    Oracle,
    Redis,
    Sqlserver,
}

impl DatabaseKind {
    pub fn all() -> &'static [DatabaseKind] {
        &[
            DatabaseKind::Postgresql,
            DatabaseKind::Mysql,
            DatabaseKind::Sqlite,
            DatabaseKind::Mongodb,
            DatabaseKind::Oracle,
            DatabaseKind::Redis,
            DatabaseKind::Sqlserver,
        ]
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            DatabaseKind::Postgresql => "PostgreSQL",
            DatabaseKind::Mysql => "MySQL",
            DatabaseKind::Sqlite => "SQLite",
            DatabaseKind::Mongodb => "MongoDB",
            DatabaseKind::Oracle => "Oracle",
            DatabaseKind::Redis => "Redis",
            DatabaseKind::Sqlserver => "SQL Server",
        }
    }

    pub fn icon_path(&self) -> &'static str {
        match self {
            DatabaseKind::Postgresql => "assets/icons/db/postgresql.svg",
            DatabaseKind::Mysql => "assets/icons/db/mysql.svg",
            DatabaseKind::Sqlite => "assets/icons/db/sqlite.svg",
            DatabaseKind::Mongodb => "assets/icons/db/mongodb.svg",
            DatabaseKind::Oracle => "assets/icons/db/oracle.svg",
            DatabaseKind::Redis => "assets/icons/db/redis.svg",
            DatabaseKind::Sqlserver => "assets/icons/db/sqlserver.svg",
        }
    }

    pub fn default_port(&self) -> Option<u16> {
        match self {
            DatabaseKind::Postgresql => Some(5432),
            DatabaseKind::Mysql => Some(3306),
            DatabaseKind::Sqlite => None,
            DatabaseKind::Mongodb => Some(27017),
            DatabaseKind::Oracle => Some(1521),
            DatabaseKind::Redis => Some(6379),
            DatabaseKind::Sqlserver => Some(1433),
        }
    }

    pub fn cache_key(&self) -> &'static str {
        match self {
            DatabaseKind::Postgresql => "postgresql",
            DatabaseKind::Mysql => "mysql",
            DatabaseKind::Sqlite => "sqlite",
            DatabaseKind::Mongodb => "mongodb",
            DatabaseKind::Oracle => "oracle",
            DatabaseKind::Redis => "redis",
            DatabaseKind::Sqlserver => "sqlserver",
        }
    }

    pub fn from_cache_key(key: &str) -> Option<Self> {
        let normalized = key.trim().to_ascii_lowercase();
        match normalized.as_str() {
            "postgresql" | "postgres" => Some(DatabaseKind::Postgresql),
            "mysql" => Some(DatabaseKind::Mysql),
            "sqlite" => Some(DatabaseKind::Sqlite),
            "mongodb" => Some(DatabaseKind::Mongodb),
            "oracle" => Some(DatabaseKind::Oracle),
            "redis" => Some(DatabaseKind::Redis),
            "sqlserver" | "sql_server" | "mssql" => Some(DatabaseKind::Sqlserver),
            _ => None,
        }
    }
}

pub fn sidebar(
    connections: &ConnectionsState,
    palette: Palette,
    context_menu: Option<usize>,
) -> Element<'_, Message> {
    let header = container(text("连接管理").color(palette.text).size(18))
        .padding([12, 16])
        .style(move |_| container::Style {
            background: Some(Background::Color(palette.surface_muted)),
            text_color: Some(palette.text),
            border: iced::border::Border {
                color: palette.border,
                width: 0.0,
                radius: 0.0.into(),
            },
            shadow: Shadow::default(),
        });

    let list = if connections.list().is_empty() {
        empty_state(palette)
    } else {
        let mut column = column![];

        for connection in connections.list() {
            column = column.push(connection_item(
                connection,
                connections.selected(),
                connections.active(),
                context_menu,
                palette,
            ));
        }

        scrollable(column.spacing(8).padding([12, 10]))
            .height(Length::Fill)
            .into()
    };

    column![header, list].spacing(0).height(Length::Fill).into()
}

fn empty_state(palette: Palette) -> Element<'static, Message> {
    let hint = text("还没有连接，点击顶部的“新建连接”开始吧。")
        .color(palette.text_muted)
        .size(14)
        .width(Length::Fill)
        .align_x(Horizontal::Center);

    let empty_panel = column![
        svg::<Theme>(SvgHandle::from_path("assets/icons/add.svg"))
            .width(48)
            .height(48),
        hint,
    ]
    .align_x(Alignment::Center)
    .spacing(12);

    container(empty_panel)
        .padding(24)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

fn connection_item(
    connection: &Connection,
    selected: Option<usize>,
    active: Option<usize>,
    context_menu: Option<usize>,
    palette: Palette,
) -> Element<'_, Message> {
    let is_selected = selected == Some(connection.id);
    let is_active = active == Some(connection.id);
    let menu_open = context_menu == Some(connection.id);

    let icon = svg::<Theme>(SvgHandle::from_path(connection.kind.icon_path()))
        .width(28)
        .height(28);

    let name = text(&connection.name)
        .color(if is_selected { palette.accent } else { palette.text })
        .size(16);

    let summary = text(connection.summary())
        .color(if is_selected {
            palette.accent
        } else {
            palette.text_muted
        })
        .size(13);

    let content = column![name, summary].spacing(4).width(Length::Fill);

    let mut row_content = row![icon, content].spacing(12).align_y(Alignment::Center);

    if is_active {
        let badge = container(text("已连接").size(12).color(palette.accent_text))
            .padding([2, 8])
            .style(move |_| container::Style {
                background: Some(Background::Color(palette.accent)),
                text_color: Some(palette.accent_text),
                border: iced::border::Border::default(),
                shadow: Shadow::default(),
            });
        row_content = row_content.push(badge);
    }

    let button = button(row_content)
        .width(Length::Fill)
        .padding([10, 14])
        .style(move |_, status| {
            use iced::widget::button::Status;

            let background = if is_selected {
                palette.accent_soft
            } else if matches!(status, Status::Hovered) {
                palette.surface_muted
            } else {
                Color::TRANSPARENT
            };

            let border_color = if is_selected { palette.accent } else { palette.border };

            button::Style {
                background: Some(Background::Color(background)),
                border: iced::border::Border {
                    color: border_color,
                    width: if is_selected { 1.5 } else { 1.0 },
                    radius: 8.0.into(),
                },
                text_color: palette.text,
                shadow: Shadow::default(),
            }
        });

    let mut column = column![
        mouse_area(button)
            .on_press(Message::SelectConnection(connection.id))
            .on_right_press(Message::OpenConnectionContextMenu(connection.id))
    ]
    .spacing(6);

    if menu_open {
        column = column.push(context_menu_widget(connection.id, palette));
    }

    column.into()
}

fn context_menu_widget(
    id: usize,
    palette: Palette,
) -> Element<'static, Message> {
    let menu_button = |label: &'static str, message: Message| -> Element<'static, Message> {
        let button = button(text(label).color(palette.text))
            .padding([6, 12])
            .style(move |_: &Theme, _| iced::widget::button::Style {
                background: Some(Background::Color(palette.surface_muted)),
                border: iced::border::Border {
                    color: palette.border,
                    width: 1.0,
                    radius: 6.0.into(),
                },
                text_color: palette.text,
                shadow: Shadow::default(),
            })
            .on_press(message);

        Element::from(button)
    };

    let menu = column![
        menu_button("查看", Message::ViewConnection(id)),
        menu_button("编辑", Message::EditConnection(id)),
        menu_button("删除", Message::DeleteConnection(id)),
    ]
    .spacing(6);

    container(menu)
        .padding(12)
        .width(Length::Fill)
        .style(move |_| container::Style {
            background: Some(Background::Color(palette.surface)),
            text_color: Some(palette.text),
            border: iced::border::Border {
                color: palette.border,
                width: 1.0,
                radius: 10.0.into(),
            },
            shadow: Shadow::default(),
        })
        .into()
}
