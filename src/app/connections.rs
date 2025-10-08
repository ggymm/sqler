use iced::alignment::Horizontal;
use iced::widget::svg::Handle as SvgHandle;
use iced::widget::{button, column, container, row, scrollable, svg, text};
use iced::{Alignment, Background, Color, Element, Length, Shadow, Theme};

use super::Message;
use super::theme::Palette;

#[derive(Debug, Default)]
pub struct ConnectionsState {
    entries: Vec<Connection>,
    selected: Option<usize>,
    next_id: usize,
}

impl ConnectionsState {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            selected: None,
            next_id: 1,
        }
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
    ) {
        if self.entries.iter().any(|conn| conn.id == id) {
            self.selected = Some(id);
        }
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
    }

    pub fn find(
        &self,
        id: usize,
    ) -> Option<&Connection> {
        self.entries.iter().find(|conn| conn.id == id)
    }
}

#[derive(Debug, Clone)]
pub struct Connection {
    pub id: usize,
    pub name: String,
    pub kind: DatabaseKind,
    pub summary: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseKind {
    PostgreSql,
    MySql,
    Sqlite,
    MongoDb,
    Oracle,
    Redis,
    SqlServer,
}

impl DatabaseKind {
    pub fn all() -> &'static [DatabaseKind] {
        &[
            DatabaseKind::PostgreSql,
            DatabaseKind::MySql,
            DatabaseKind::Sqlite,
            DatabaseKind::MongoDb,
            DatabaseKind::Oracle,
            DatabaseKind::Redis,
            DatabaseKind::SqlServer,
        ]
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            DatabaseKind::PostgreSql => "PostgreSQL",
            DatabaseKind::MySql => "MySQL",
            DatabaseKind::Sqlite => "SQLite",
            DatabaseKind::MongoDb => "MongoDB",
            DatabaseKind::Oracle => "Oracle",
            DatabaseKind::Redis => "Redis",
            DatabaseKind::SqlServer => "SQL Server",
        }
    }

    pub fn icon_path(&self) -> &'static str {
        match self {
            DatabaseKind::PostgreSql => "assets/icons/db/postgresql.svg",
            DatabaseKind::MySql => "assets/icons/db/mysql.svg",
            DatabaseKind::Sqlite => "assets/icons/db/sqlite.svg",
            DatabaseKind::MongoDb => "assets/icons/db/mongodb.svg",
            DatabaseKind::Oracle => "assets/icons/db/oracle.svg",
            DatabaseKind::Redis => "assets/icons/db/redis.svg",
            DatabaseKind::SqlServer => "assets/icons/db/sqlserver.svg",
        }
    }

    pub fn default_port(&self) -> Option<u16> {
        match self {
            DatabaseKind::PostgreSql => Some(5432),
            DatabaseKind::MySql => Some(3306),
            DatabaseKind::Sqlite => None,
            DatabaseKind::MongoDb => Some(27017),
            DatabaseKind::Oracle => Some(1521),
            DatabaseKind::Redis => Some(6379),
            DatabaseKind::SqlServer => Some(1433),
        }
    }
}

pub fn sidebar(
    connections: &ConnectionsState,
    palette: Palette,
) -> Element<'_, Message> {
    let header = container(text("连接管理").color(palette.text).size(18))
        .padding([12, 16])
        .style(move |_| iced::widget::container::Style {
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
            column = column.push(connection_item(connection, connections.selected(), palette));
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

fn connection_item<'a>(
    connection: &'a Connection,
    selected: Option<usize>,
    palette: Palette,
) -> Element<'a, Message> {
    let is_selected = selected == Some(connection.id);

    let icon = svg::<Theme>(SvgHandle::from_path(connection.kind.icon_path()))
        .width(28)
        .height(28);

    let name = text(&connection.name)
        .color(if is_selected { palette.accent } else { palette.text })
        .size(16);

    let summary = text(&connection.summary)
        .color(if is_selected {
            palette.accent
        } else {
            palette.text_muted
        })
        .size(13);

    let content = column![name, summary].spacing(4).width(Length::Fill);

    button(row![icon, content].spacing(12).align_y(Alignment::Center))
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

            iced::widget::button::Style {
                background: Some(Background::Color(background)),
                border: iced::border::Border {
                    color: border_color,
                    width: if is_selected { 1.5 } else { 1.0 },
                    radius: 8.0.into(),
                },
                text_color: palette.text,
                shadow: Shadow::default(),
            }
        })
        .on_press(Message::SelectConnection(connection.id))
        .into()
}
