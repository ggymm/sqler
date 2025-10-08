use iced::alignment::Horizontal;
use iced::widget::container::Style as ContainerStyle;
use iced::widget::svg::Handle as SvgHandle;
use iced::widget::{
    Rule, Stack, button, column, container, horizontal_space, row, scrollable, text, text_input,
};
use iced::{Alignment, Background, Color, Element, Length, Shadow, Task, Theme, Vector};

pub fn main() -> iced::Result {
    iced::application("Sqler", App::update, App::view).run()
}

#[derive(Debug)]
struct App {
    theme: ThemeMode,
    active_tab: ContentTab,
    connections: Vec<Connection>,
    selected_connection: Option<usize>,
    dialog: Option<DialogState>,
    next_connection_id: usize,
}

impl Default for App {
    fn default() -> Self {
        Self {
            theme: ThemeMode::Light,
            active_tab: ContentTab::Tables,
            connections: Vec::new(),
            selected_connection: None,
            dialog: None,
            next_connection_id: 1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ThemeMode {
    Light,
    Dark,
}

impl ThemeMode {
    fn toggle(&mut self) {
        *self = match self {
            ThemeMode::Light => ThemeMode::Dark,
            ThemeMode::Dark => ThemeMode::Light,
        };
    }

    fn palette(&self) -> Palette {
        match self {
            ThemeMode::Light => Palette {
                background: Color::from_rgb8(0xf7, 0xf8, 0xfb),
                surface: Color::WHITE,
                surface_muted: Color::from_rgb8(0xee, 0xf0, 0xf4),
                border: Color::from_rgb8(0xd9, 0xde, 0xe7),
                text: Color::from_rgb8(0x1f, 0x24, 0x2f),
                text_muted: Color::from_rgb8(0x58, 0x60, 0x72),
                accent: Color::from_rgb8(0x42, 0x82, 0xff),
                accent_text: Color::WHITE,
                accent_soft: Color::from_rgb8(0xd9, 0xe7, 0xff),
                overlay: Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 0.55,
                },
            },
            ThemeMode::Dark => Palette {
                background: Color::from_rgb8(0x18, 0x1c, 0x24),
                surface: Color::from_rgb8(0x21, 0x26, 0x31),
                surface_muted: Color::from_rgb8(0x29, 0x2f, 0x3d),
                border: Color::from_rgb8(0x35, 0x3c, 0x4a),
                text: Color::from_rgb8(0xf1, 0xf5, 0xff),
                text_muted: Color::from_rgb8(0x9e, 0xa6, 0xb9),
                accent: Color::from_rgb8(0x66, 0x9b, 0xff),
                accent_text: Color::WHITE,
                accent_soft: Color::from_rgb8(0x2a, 0x3b, 0x59),
                overlay: Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 0.6,
                },
            },
        }
    }
}

#[derive(Clone, Copy)]
struct Palette {
    background: Color,
    surface: Color,
    surface_muted: Color,
    border: Color,
    text: Color,
    text_muted: Color,
    accent: Color,
    accent_text: Color,
    accent_soft: Color,
    overlay: Color,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ContentTab {
    Tables,
    Queries,
    Functions,
    Users,
}

impl ContentTab {
    fn title(&self) -> &'static str {
        match self {
            ContentTab::Tables => "表",
            ContentTab::Queries => "查询",
            ContentTab::Functions => "函数",
            ContentTab::Users => "用户",
        }
    }

    fn icon_path(&self) -> &'static str {
        match self {
            ContentTab::Tables => "assets/icons/table.svg",
            ContentTab::Queries => "assets/icons/query.svg",
            ContentTab::Functions => "assets/icons/function.svg",
            ContentTab::Users => "assets/icons/user.svg",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FormField {
    Name,
    Host,
    Port,
    Database,
    Username,
    Password,
    FilePath,
    ConnectionString,
}

#[derive(Debug, Clone)]
enum DialogState {
    NewConnection(NewConnectionDialog),
}

#[derive(Debug, Clone)]
enum NewConnectionDialog {
    SelectingType,
    Editing(ConnectionFormState),
}

#[derive(Debug, Clone)]
struct ConnectionFormState {
    form: ConnectionForm,
    error: Option<String>,
}

impl ConnectionFormState {
    fn new(kind: DatabaseKind) -> Self {
        Self {
            form: ConnectionForm::from_kind(kind),
            error: None,
        }
    }

    fn build_connection(&self, id: usize) -> Result<Connection, String> {
        self.form.build_connection(id)
    }

    fn clear_error(&mut self) {
        self.error = None;
    }
}

#[derive(Debug, Clone)]
enum ConnectionForm {
    Relational {
        kind: DatabaseKind,
        name: String,
        host: String,
        port: String,
        database: String,
        username: String,
        password: String,
    },
    Sqlite {
        name: String,
        file_path: String,
    },
    Mongo {
        name: String,
        connection_string: String,
    },
    Redis {
        name: String,
        host: String,
        port: String,
        password: String,
    },
}

impl ConnectionForm {
    fn from_kind(kind: DatabaseKind) -> Self {
        match kind {
            DatabaseKind::PostgreSql => ConnectionForm::Relational {
                kind,
                name: format!("{} 连接", kind.display_name()),
                host: "localhost".into(),
                port: kind.default_port().unwrap_or(5432).to_string(),
                database: "postgres".into(),
                username: "postgres".into(),
                password: String::new(),
            },
            DatabaseKind::MySql => ConnectionForm::Relational {
                kind,
                name: format!("{} 连接", kind.display_name()),
                host: "localhost".into(),
                port: kind.default_port().unwrap_or(3306).to_string(),
                database: "mysql".into(),
                username: "root".into(),
                password: String::new(),
            },
            DatabaseKind::Oracle => ConnectionForm::Relational {
                kind,
                name: format!("{} 连接", kind.display_name()),
                host: "localhost".into(),
                port: kind.default_port().unwrap_or(1521).to_string(),
                database: "xe".into(),
                username: "system".into(),
                password: String::new(),
            },
            DatabaseKind::SqlServer => ConnectionForm::Relational {
                kind,
                name: format!("{} 连接", kind.display_name()),
                host: "localhost".into(),
                port: kind.default_port().unwrap_or(1433).to_string(),
                database: "master".into(),
                username: "sa".into(),
                password: String::new(),
            },
            DatabaseKind::Sqlite => ConnectionForm::Sqlite {
                name: "SQLite 连接".into(),
                file_path: "./database.sqlite3".into(),
            },
            DatabaseKind::MongoDb => ConnectionForm::Mongo {
                name: "MongoDB 连接".into(),
                connection_string: "mongodb://localhost:27017".into(),
            },
            DatabaseKind::Redis => ConnectionForm::Redis {
                name: "Redis 连接".into(),
                host: "localhost".into(),
                port: kind.default_port().unwrap_or(6379).to_string(),
                password: String::new(),
            },
        }
    }

    fn kind(&self) -> DatabaseKind {
        match self {
            ConnectionForm::Relational { kind, .. } => *kind,
            ConnectionForm::Sqlite { .. } => DatabaseKind::Sqlite,
            ConnectionForm::Mongo { .. } => DatabaseKind::MongoDb,
            ConnectionForm::Redis { .. } => DatabaseKind::Redis,
        }
    }

    fn update(&mut self, field: FormField, value: String) {
        match self {
            ConnectionForm::Relational {
                name,
                host,
                port,
                database,
                username,
                password,
                ..
            } => match field {
                FormField::Name => *name = value,
                FormField::Host => *host = value,
                FormField::Port => *port = value,
                FormField::Database => *database = value,
                FormField::Username => *username = value,
                FormField::Password => *password = value,
                _ => {}
            },
            ConnectionForm::Sqlite { name, file_path } => match field {
                FormField::Name => *name = value,
                FormField::FilePath => *file_path = value,
                _ => {}
            },
            ConnectionForm::Mongo {
                name,
                connection_string,
            } => match field {
                FormField::Name => *name = value,
                FormField::ConnectionString => *connection_string = value,
                _ => {}
            },
            ConnectionForm::Redis {
                name,
                host,
                port,
                password,
            } => match field {
                FormField::Name => *name = value,
                FormField::Host => *host = value,
                FormField::Port => *port = value,
                FormField::Password => *password = value,
                _ => {}
            },
        }
    }

    fn build_connection(&self, id: usize) -> Result<Connection, String> {
        match self {
            ConnectionForm::Relational {
                kind,
                name,
                host,
                port,
                database,
                username,
                ..
            } => {
                if name.trim().is_empty() {
                    return Err("请输入连接名称".into());
                }
                if host.trim().is_empty() {
                    return Err("请输入主机地址".into());
                }
                let port_num = port
                    .trim()
                    .parse::<u16>()
                    .map_err(|_| "端口必须为数字".to_string())?;
                if username.trim().is_empty() {
                    return Err("请输入用户名".into());
                }
                if database.trim().is_empty() {
                    return Err("请输入数据库名".into());
                }

                Ok(Connection {
                    id,
                    name: name.trim().to_string(),
                    kind: *kind,
                    summary: format!("{username}@{host}:{port_num}/{database}"),
                })
            }
            ConnectionForm::Sqlite { name, file_path } => {
                if name.trim().is_empty() {
                    return Err("请输入连接名称".into());
                }
                if file_path.trim().is_empty() {
                    return Err("请输入数据库文件路径".into());
                }

                Ok(Connection {
                    id,
                    name: name.trim().to_string(),
                    kind: DatabaseKind::Sqlite,
                    summary: file_path.trim().to_string(),
                })
            }
            ConnectionForm::Mongo {
                name,
                connection_string,
            } => {
                if name.trim().is_empty() {
                    return Err("请输入连接名称".into());
                }
                if connection_string.trim().is_empty() {
                    return Err("请输入连接字符串".into());
                }

                Ok(Connection {
                    id,
                    name: name.trim().to_string(),
                    kind: DatabaseKind::MongoDb,
                    summary: connection_string.trim().to_string(),
                })
            }
            ConnectionForm::Redis {
                name, host, port, ..
            } => {
                if name.trim().is_empty() {
                    return Err("请输入连接名称".into());
                }
                if host.trim().is_empty() {
                    return Err("请输入主机地址".into());
                }
                let port_num = port
                    .trim()
                    .parse::<u16>()
                    .map_err(|_| "端口必须为数字".to_string())?;

                Ok(Connection {
                    id,
                    name: name.trim().to_string(),
                    kind: DatabaseKind::Redis,
                    summary: format!("{host}:{port_num}"),
                })
            }
        }
    }
}

#[derive(Debug, Clone)]
struct Connection {
    id: usize,
    name: String,
    kind: DatabaseKind,
    summary: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DatabaseKind {
    PostgreSql,
    MySql,
    Sqlite,
    MongoDb,
    Oracle,
    Redis,
    SqlServer,
}

impl DatabaseKind {
    fn all() -> &'static [DatabaseKind] {
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

    fn display_name(&self) -> &'static str {
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

    fn icon_path(&self) -> &'static str {
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

    fn default_port(&self) -> Option<u16> {
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

#[derive(Debug, Clone)]
enum Message {
    ToggleTheme,
    SelectContentTab(ContentTab),
    ShowNewConnectionDialog,
    ShowNewQueryWorkspace,
    SelectConnection(usize),
    CancelDialog,
    NewConnectionTypeSelected(DatabaseKind),
    BackToConnectionTypeSelection,
    UpdateFormField(FormField, String),
    SubmitNewConnection,
}

impl App {
    fn palette(&self) -> Palette {
        self.theme.palette()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ToggleTheme => {
                self.theme.toggle();
            }
            Message::SelectContentTab(tab) => {
                self.active_tab = tab;
            }
            Message::ShowNewConnectionDialog => {
                self.dialog = Some(DialogState::NewConnection(
                    NewConnectionDialog::SelectingType,
                ));
            }
            Message::ShowNewQueryWorkspace => {
                self.active_tab = ContentTab::Queries;
            }
            Message::SelectConnection(id) => {
                if self.connections.iter().any(|conn| conn.id == id) {
                    self.selected_connection = Some(id);
                }
            }
            Message::CancelDialog => {
                self.dialog = None;
            }
            Message::NewConnectionTypeSelected(kind) => {
                if let Some(DialogState::NewConnection(dialog)) = &mut self.dialog {
                    *dialog = NewConnectionDialog::Editing(ConnectionFormState::new(kind));
                } else {
                    self.dialog = Some(DialogState::NewConnection(NewConnectionDialog::Editing(
                        ConnectionFormState::new(kind),
                    )));
                }
            }
            Message::BackToConnectionTypeSelection => {
                if let Some(DialogState::NewConnection(dialog)) = &mut self.dialog {
                    *dialog = NewConnectionDialog::SelectingType;
                }
            }
            Message::UpdateFormField(field, value) => {
                if let Some(DialogState::NewConnection(NewConnectionDialog::Editing(form_state))) =
                    &mut self.dialog
                {
                    form_state.clear_error();
                    form_state.form.update(field, value);
                }
            }
            Message::SubmitNewConnection => {
                if let Some(DialogState::NewConnection(dialog)) = self.dialog.take() {
                    match dialog {
                        NewConnectionDialog::Editing(mut form_state) => {
                            match form_state.build_connection(self.next_connection_id) {
                                Ok(connection) => {
                                    self.next_connection_id += 1;
                                    self.selected_connection = Some(connection.id);
                                    self.connections.push(connection);
                                    self.dialog = None;
                                }
                                Err(error) => {
                                    form_state.error = Some(error);
                                    self.dialog = Some(DialogState::NewConnection(
                                        NewConnectionDialog::Editing(form_state),
                                    ));
                                }
                            }
                        }
                        NewConnectionDialog::SelectingType => {
                            self.dialog = Some(DialogState::NewConnection(
                                NewConnectionDialog::SelectingType,
                            ));
                        }
                    }
                }
            }
        }

        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let palette = self.palette();

        let content = column![self.view_top_bar(), self.view_body(),]
            .spacing(0)
            .height(Length::Fill);

        let base = container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_| ContainerStyle {
                background: Some(Background::Color(palette.background)),
                text_color: Some(palette.text),
                border: iced::border::Border::default(),
                shadow: Shadow::default(),
            })
            .into();

        if let Some(dialog) = &self.dialog {
            Stack::new()
                .width(Length::Fill)
                .height(Length::Fill)
                .push(base)
                .push(self.view_dialog(dialog))
                .into()
        } else {
            base
        }
    }

    fn view_top_bar(&self) -> Element<'_, Message> {
        let palette = self.palette();

        let divider = container(Rule::vertical(1).style(move |_| iced::widget::rule::Style {
            color: palette.border,
            width: 1,
            radius: 0.0.into(),
            fill_mode: iced::widget::rule::FillMode::Full,
        }))
        .height(Length::Fixed(28.0));

        let mut actions = row![
            self.icon_action_button(
                "assets/icons/new-conn.svg",
                "新建连接",
                Message::ShowNewConnectionDialog
            ),
            self.icon_action_button(
                "assets/icons/new-query.svg",
                "新建查询",
                Message::ShowNewQueryWorkspace
            ),
            divider,
        ]
        .spacing(12)
        .align_y(Alignment::Center);

        for tab in [
            ContentTab::Tables,
            ContentTab::Queries,
            ContentTab::Functions,
            ContentTab::Users,
        ] {
            actions = actions.push(self.icon_tab_button(tab));
        }

        let theme_icon = match self.theme {
            ThemeMode::Light => "assets/icons/theme-dark.svg",
            ThemeMode::Dark => "assets/icons/theme-light.svg",
        };

        row![
            actions,
            horizontal_space().width(Length::Fill),
            self.icon_action_button(theme_icon, "切换主题", Message::ToggleTheme),
        ]
        .padding([12, 20])
        .align_y(Alignment::Center)
        .into()
    }

    fn icon_action_button<'a>(
        &self,
        icon_path: &str,
        label: &'a str,
        message: Message,
    ) -> Element<'a, Message> {
        let palette = self.palette();
        let icon = iced::widget::svg::<Theme>(SvgHandle::from_path(icon_path))
            .width(24)
            .height(24);
        let text_color = palette.text;

        button(
            row![icon, text(label).color(text_color).size(16)]
                .spacing(8)
                .align_y(Alignment::Center),
        )
        .padding([8, 12])
        .style(move |_, status| {
            use iced::widget::button::Status;

            let mut style = iced::widget::button::Style::default();
            style.border = iced::border::Border {
                color: palette.border,
                width: 1.0,
                radius: 6.0.into(),
            };

            style.text_color = palette.text;
            style.background = Some(Background::Color(match status {
                Status::Hovered => palette.surface_muted,
                Status::Pressed => palette.surface,
                _ => Color::TRANSPARENT,
            }));

            style
        })
        .on_press(message)
        .into()
    }

    fn icon_tab_button(&self, tab: ContentTab) -> Element<'_, Message> {
        let palette = self.palette();
        let is_active = self.active_tab == tab;
        let icon = iced::widget::svg::<Theme>(SvgHandle::from_path(tab.icon_path()))
            .width(24)
            .height(24);
        let label = text(tab.title()).size(16);

        button(row![icon, label].spacing(8).align_y(Alignment::Center))
            .padding([8, 18])
            .style(move |_, status| {
                use iced::widget::button::Status;

                let mut style = iced::widget::button::Style::default();
                style.border = iced::border::Border {
                    color: if is_active {
                        palette.accent
                    } else {
                        palette.border
                    },
                    width: if is_active { 1.5 } else { 1.0 },
                    radius: 8.0.into(),
                };

                let background = if is_active {
                    palette.accent_soft
                } else if matches!(status, Status::Hovered) {
                    palette.surface_muted
                } else {
                    Color::TRANSPARENT
                };

                style.background = Some(Background::Color(background));
                style.text_color = if is_active {
                    palette.accent
                } else {
                    palette.text
                };

                style
            })
            .on_press(Message::SelectContentTab(tab))
            .into()
    }

    fn view_body(&self) -> Element<'_, Message> {
        let palette = self.palette();
        let left_panel = self.view_connections_panel();
        let content_panel = self.view_content_panel();

        row![
            container(left_panel)
                .width(Length::Fixed(260.0))
                .style(move |_| ContainerStyle {
                    background: Some(Background::Color(palette.surface)),
                    border: iced::border::Border {
                        color: palette.border,
                        width: 1.0,
                        radius: 0.0.into(),
                    },
                    text_color: Some(palette.text),
                    shadow: Shadow::default(),
                }),
            container(content_panel)
                .width(Length::Fill)
                .style(move |_| ContainerStyle {
                    background: Some(Background::Color(palette.surface)),
                    border: iced::border::Border {
                        color: palette.border,
                        width: 1.0,
                        radius: 0.0.into(),
                    },
                    text_color: Some(palette.text),
                    shadow: Shadow::default(),
                }),
        ]
        .height(Length::Fill)
        .into()
    }

    fn view_connections_panel(&self) -> Element<'_, Message> {
        let palette = self.palette();

        let header = container(text("连接管理").color(palette.text).size(18))
            .padding([12, 16])
            .style(move |_| ContainerStyle {
                background: Some(Background::Color(palette.surface_muted)),
                text_color: Some(palette.text),
                border: iced::border::Border {
                    color: palette.border,
                    width: 0.0,
                    radius: 0.0.into(),
                },
                shadow: Shadow::default(),
            });

        let list: Element<'_, Message> = if self.connections.is_empty() {
            let hint = text("还没有连接，点击顶部的“新建连接”开始吧。")
                .color(palette.text_muted)
                .size(14)
                .width(Length::Fill)
                .align_x(Horizontal::Center);

            let empty_panel = column![
                iced::widget::svg::<Theme>(SvgHandle::from_path("assets/icons/add.svg"),)
                    .width(48)
                    .height(48),
                hint,
            ]
            .align_x(Alignment::Center)
            .spacing(12);

            let empty_element: Element<'_, Message> = empty_panel.into();

            container(empty_element)
                .padding(24)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .into()
        } else {
            let mut column = column![];

            for connection in &self.connections {
                let is_selected = self.selected_connection == Some(connection.id);

                let icon =
                    iced::widget::svg::<Theme>(SvgHandle::from_path(connection.kind.icon_path()))
                        .width(28)
                        .height(28);

                let name = text(&connection.name)
                    .color(if is_selected {
                        palette.accent
                    } else {
                        palette.text
                    })
                    .size(16);

                let summary = text(&connection.summary)
                    .color(if is_selected {
                        palette.accent
                    } else {
                        palette.text_muted
                    })
                    .size(13);

                let content = column![name, summary].spacing(4).width(Length::Fill);

                column = column.push(
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

                            let border_color = if is_selected {
                                palette.accent
                            } else {
                                palette.border
                            };

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
                        .on_press(Message::SelectConnection(connection.id)),
                );
            }

            scrollable(column.spacing(8).padding([12, 10]))
                .height(Length::Fill)
                .into()
        };

        column![header, list].spacing(0).height(Length::Fill).into()
    }

    fn view_content_panel(&self) -> Element<'_, Message> {
        let palette = self.palette();

        let title = text(self.active_tab.title()).size(22).color(palette.text);

        let intro = text(match self.active_tab {
            ContentTab::Tables => "在此浏览所选数据库的表结构，并进行建表或结构修改。",
            ContentTab::Queries => "创建和管理查询，支持多标签页执行 SQL。",
            ContentTab::Functions => "查看数据库函数或存储过程，支持编辑与调试。",
            ContentTab::Users => "管理数据库用户、角色以及权限设置。",
        })
        .color(palette.text_muted)
        .size(15);

        let no_connection_hint = || {
            text("请选择或创建一个数据库连接以开始。")
                .color(palette.text_muted)
                .size(14)
                .into()
        };

        let detail: Element<'_, Message> = if let Some(selected_id) = self.selected_connection {
            if let Some(connection) = self.connections.iter().find(|c| c.id == selected_id) {
                column![
                    text(format!("当前连接：{}", connection.name))
                        .color(palette.text)
                        .size(17),
                    text(format!("连接信息：{}", connection.summary))
                        .color(palette.text_muted)
                        .size(14),
                ]
                .spacing(6)
                .into()
            } else {
                no_connection_hint()
            }
        } else {
            no_connection_hint()
        };

        column![
            container(column![title, intro, detail].spacing(12))
                .padding([18, 24])
                .style(move |_| ContainerStyle {
                    background: Some(Background::Color(palette.surface)),
                    text_color: Some(palette.text),
                    border: iced::border::Border {
                        color: Color::TRANSPARENT,
                        width: 0.0,
                        radius: 0.0.into(),
                    },
                    shadow: Shadow::default(),
                }),
            scrollable(container(self.view_tab_body()).padding(24).style(move |_| {
                ContainerStyle {
                    background: Some(Background::Color(palette.surface)),
                    text_color: Some(palette.text),
                    border: iced::border::Border {
                        color: Color::TRANSPARENT,
                        width: 0.0,
                        radius: 0.0.into(),
                    },
                    shadow: Shadow::default(),
                }
            }),)
            .height(Length::Fill),
        ]
        .spacing(12)
        .height(Length::Fill)
        .into()
    }

    fn view_tab_body(&self) -> Element<'_, Message> {
        let palette = self.palette();

        match self.active_tab {
            ContentTab::Tables => column![
                text("表管理功能即将上线").size(18).color(palette.text),
                text("这里将展示表列表、结构预览以及建模工具。")
                    .color(palette.text_muted)
                    .size(15),
            ]
            .spacing(8)
            .into(),
            ContentTab::Queries => column![
                text("查询编辑器").size(18).color(palette.text),
                text("支持多标签查询、语法高亮与自动完成等特性（开发中）。")
                    .color(palette.text_muted)
                    .size(15),
            ]
            .spacing(8)
            .into(),
            ContentTab::Functions => column![
                text("函数与存储过程").size(18).color(palette.text),
                text("管理数据库函数、触发器与存储过程，并支持分版本控制。")
                    .color(palette.text_muted)
                    .size(15),
            ]
            .spacing(8)
            .into(),
            ContentTab::Users => column![
                text("用户与权限").size(18).color(palette.text),
                text("在此配置数据库用户、角色以及权限策略。")
                    .color(palette.text_muted)
                    .size(15),
            ]
            .spacing(8)
            .into(),
        }
    }

    fn view_dialog<'a>(&'a self, dialog: &'a DialogState) -> Element<'a, Message> {
        match dialog {
            DialogState::NewConnection(state) => self.view_new_connection_dialog(state),
        }
    }

    fn view_new_connection_dialog<'a>(
        &'a self,
        dialog: &'a NewConnectionDialog,
    ) -> Element<'a, Message> {
        let palette = self.palette();

        let modal_content = match dialog {
            NewConnectionDialog::SelectingType => self.view_connection_type_selector(),
            NewConnectionDialog::Editing(form_state) => self.view_connection_form(form_state),
        };

        let modal = container(modal_content)
            .width(Length::Fixed(540.0))
            .style(move |_| ContainerStyle {
                background: Some(Background::Color(palette.surface)),
                text_color: Some(palette.text),
                border: iced::border::Border {
                    color: palette.border,
                    width: 1.0,
                    radius: 16.0.into(),
                },
                shadow: Shadow {
                    color: Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 0.35,
                    },
                    blur_radius: 24.0,
                    offset: Vector::new(0.0, 12.0),
                },
            })
            .padding(24);

        container(modal)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(move |_| ContainerStyle {
                background: Some(Background::Color(palette.overlay)),
                text_color: Some(palette.text),
                border: iced::border::Border::default(),
                shadow: Shadow::default(),
            })
            .into()
    }

    fn view_connection_type_selector(&self) -> Element<'_, Message> {
        let palette = self.palette();

        let mut content = column![
            text("选择数据库类型").size(22).color(palette.text),
            text("请选择需要连接的数据库类型。")
                .color(palette.text_muted)
                .size(15),
        ]
        .spacing(12);

        for chunk in DatabaseKind::all().chunks(3) {
            let mut row_widget = row![].spacing(12).align_y(Alignment::Center);

            for kind in chunk {
                row_widget = row_widget.push(self.database_type_card(*kind));
            }

            content = content.push(row_widget);
        }

        content
            .push(
                row![
                    horizontal_space().width(Length::Fill),
                    button(text("取消").color(palette.text))
                        .padding([8, 18])
                        .style(move |_, _| iced::widget::button::Style {
                            background: Some(Background::Color(palette.surface_muted)),
                            border: iced::border::Border {
                                color: palette.border,
                                width: 1.0,
                                radius: 8.0.into(),
                            },
                            text_color: palette.text,
                            shadow: Shadow::default(),
                        })
                        .on_press(Message::CancelDialog),
                ]
                .spacing(12)
                .align_y(Alignment::Center),
            )
            .spacing(20)
            .into()
    }

    fn database_type_card(&self, kind: DatabaseKind) -> Element<'_, Message> {
        let palette = self.palette();
        let icon = iced::widget::svg::<Theme>(SvgHandle::from_path(kind.icon_path()))
            .width(48)
            .height(48);

        let name = text(kind.display_name()).color(palette.text).size(17);

        button(column![icon, name].spacing(12).align_x(Alignment::Center))
            .width(Length::FillPortion(1))
            .padding(18)
            .style(move |_, status| {
                use iced::widget::button::Status;

                let background = match status {
                    Status::Hovered => palette.surface_muted,
                    Status::Pressed => palette.surface,
                    _ => palette.surface,
                };

                iced::widget::button::Style {
                    background: Some(Background::Color(background)),
                    border: iced::border::Border {
                        color: palette.border,
                        width: 1.0,
                        radius: 14.0.into(),
                    },
                    text_color: palette.text,
                    shadow: Shadow::default(),
                }
            })
            .on_press(Message::NewConnectionTypeSelected(kind))
            .into()
    }

    fn view_connection_form<'a>(
        &self,
        form_state: &'a ConnectionFormState,
    ) -> Element<'a, Message> {
        let palette = self.palette();
        let form = &form_state.form;
        let kind = form.kind();

        let mut content = column![
            row![
                iced::widget::svg::<Theme>(SvgHandle::from_path(kind.icon_path()))
                    .width(36)
                    .height(36),
                column![
                    text(format!("配置 {}", kind.display_name()))
                        .color(palette.text)
                        .size(22),
                    text("填写数据库连接信息。")
                        .color(palette.text_muted)
                        .size(15),
                ]
                .spacing(6),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
        ]
        .spacing(18);

        let mut fields = column![].spacing(12);

        match form {
            ConnectionForm::Relational {
                name,
                host,
                port,
                database,
                username,
                password,
                ..
            } => {
                fields = fields
                    .push(self.labeled_input("连接名称", name, FormField::Name))
                    .push(self.labeled_input("主机", host, FormField::Host))
                    .push(self.labeled_input("端口", port, FormField::Port))
                    .push(self.labeled_input("数据库", database, FormField::Database))
                    .push(self.labeled_input("用户名", username, FormField::Username))
                    .push(self.labeled_input("密码", password, FormField::Password));
            }
            ConnectionForm::Sqlite { name, file_path } => {
                fields = fields
                    .push(self.labeled_input("连接名称", name, FormField::Name))
                    .push(self.labeled_input("数据库文件路径", file_path, FormField::FilePath));
            }
            ConnectionForm::Mongo {
                name,
                connection_string,
            } => {
                fields = fields
                    .push(self.labeled_input("连接名称", name, FormField::Name))
                    .push(self.labeled_input(
                        "连接字符串",
                        connection_string,
                        FormField::ConnectionString,
                    ));
            }
            ConnectionForm::Redis {
                name,
                host,
                port,
                password,
            } => {
                fields = fields
                    .push(self.labeled_input("连接名称", name, FormField::Name))
                    .push(self.labeled_input("主机", host, FormField::Host))
                    .push(self.labeled_input("端口", port, FormField::Port))
                    .push(self.labeled_input("密码", password, FormField::Password));
            }
        }

        content = content.push(fields);

        if let Some(error) = &form_state.error {
            let error_color = Color::from_rgb8(0xff, 0x4d, 0x4f);
            content = content.push(
                container(text(error).color(error_color).size(14))
                    .padding([8, 12])
                    .style(move |_| ContainerStyle {
                        background: Some(Background::Color(Color::from_rgba(
                            0.98, 0.29, 0.3, 0.12,
                        ))),
                        text_color: Some(error_color),
                        border: iced::border::Border {
                            color: error_color,
                            width: 1.0,
                            radius: 8.0.into(),
                        },
                        shadow: Shadow::default(),
                    }),
            );
        }

        content
            .push(
                row![
                    button(text("返回").color(palette.text))
                        .padding([8, 18])
                        .style(move |_, _| iced::widget::button::Style {
                            background: Some(Background::Color(palette.surface_muted)),
                            border: iced::border::Border {
                                color: palette.border,
                                width: 1.0,
                                radius: 8.0.into(),
                            },
                            text_color: palette.text,
                            shadow: Shadow::default(),
                        })
                        .on_press(Message::BackToConnectionTypeSelection),
                    horizontal_space().width(Length::Fill),
                    button(text("取消").color(palette.text))
                        .padding([8, 18])
                        .style(move |_, _| iced::widget::button::Style {
                            background: Some(Background::Color(palette.surface_muted)),
                            border: iced::border::Border {
                                color: palette.border,
                                width: 1.0,
                                radius: 8.0.into(),
                            },
                            text_color: palette.text,
                            shadow: Shadow::default(),
                        })
                        .on_press(Message::CancelDialog),
                    button(text("保存连接").color(palette.accent_text))
                        .padding([8, 20])
                        .style(move |_, status| {
                            use iced::widget::button::Status;

                            let background = match status {
                                Status::Hovered => palette.accent.scale_alpha(0.9),
                                Status::Pressed => palette.accent.scale_alpha(1.0),
                                _ => palette.accent,
                            };

                            iced::widget::button::Style {
                                background: Some(Background::Color(background)),
                                border: iced::border::Border {
                                    color: background,
                                    width: 1.0,
                                    radius: 8.0.into(),
                                },
                                text_color: palette.accent_text,
                                shadow: Shadow::default(),
                            }
                        })
                        .on_press(Message::SubmitNewConnection),
                ]
                .spacing(12)
                .align_y(Alignment::Center),
            )
            .spacing(20)
            .into()
    }

    fn labeled_input<'a>(
        &self,
        label: &'a str,
        value: &'a str,
        field: FormField,
    ) -> Element<'a, Message> {
        let palette = self.palette();

        column![
            text(label).color(palette.text_muted).size(14),
            text_input("", value)
                .padding([10, 12])
                .style(move |_, _| iced::widget::text_input::Style {
                    background: Background::Color(palette.surface_muted),
                    border: iced::border::Border {
                        color: palette.border,
                        width: 1.0,
                        radius: 6.0.into(),
                    },
                    icon: palette.text_muted,
                    placeholder: palette.text_muted,
                    selection: palette.accent,
                    value: palette.text,
                })
                .on_input(move |input| Message::UpdateFormField(field, input)),
        ]
        .spacing(6)
        .into()
    }
}
