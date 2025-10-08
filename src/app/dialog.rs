use iced::widget::svg::Handle as SvgHandle;
use iced::widget::{button, column, container, horizontal_space, row, svg, text};
use iced::{Alignment, Background, Color, Element, Length, Shadow, Theme, Vector};

use crate::comps::form::labeled_input;

use super::{Connection, DatabaseKind};
use super::{Message, Palette};

#[derive(Debug, Clone)]
pub enum DialogState {
    NewConnection(NewConnectionDialog),
}

#[derive(Debug, Clone)]
pub enum NewConnectionDialog {
    SelectingType,
    Editing(ConnectionFormState),
}

#[derive(Debug, Clone)]
pub struct ConnectionFormState {
    pub form: ConnectionForm,
    pub error: Option<String>,
}

impl ConnectionFormState {
    pub fn new(kind: DatabaseKind) -> Self {
        Self {
            form: ConnectionForm::from_kind(kind),
            error: None,
        }
    }

    pub fn build_connection(
        &self,
        id: usize,
    ) -> Result<Connection, String> {
        self.form.build_connection(id)
    }

    pub fn clear_error(&mut self) {
        self.error = None;
    }
}

#[derive(Debug, Clone)]
pub enum ConnectionForm {
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
            DatabaseKind::Postgresql => ConnectionForm::Relational {
                kind,
                name: format!("{} 连接", kind.display_name()),
                host: "localhost".into(),
                port: kind.default_port().unwrap_or(5432).to_string(),
                database: "postgres".into(),
                username: "postgres".into(),
                password: String::new(),
            },
            DatabaseKind::Mysql => ConnectionForm::Relational {
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
            DatabaseKind::Sqlserver => ConnectionForm::Relational {
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
            DatabaseKind::Mongodb => ConnectionForm::Mongo {
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

    pub fn kind(&self) -> DatabaseKind {
        match self {
            ConnectionForm::Relational { kind, .. } => *kind,
            ConnectionForm::Sqlite { .. } => DatabaseKind::Sqlite,
            ConnectionForm::Mongo { .. } => DatabaseKind::Mongodb,
            ConnectionForm::Redis { .. } => DatabaseKind::Redis,
        }
    }

    pub fn update(
        &mut self,
        field: FormField,
        value: String,
    ) {
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

    fn build_connection(
        &self,
        id: usize,
    ) -> Result<Connection, String> {
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
                let port_num = port.trim().parse::<u16>().map_err(|_| "端口必须为数字".to_string())?;
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
                    kind: DatabaseKind::Mongodb,
                    summary: connection_string.trim().to_string(),
                })
            }
            ConnectionForm::Redis { name, host, port, .. } => {
                if name.trim().is_empty() {
                    return Err("请输入连接名称".into());
                }
                if host.trim().is_empty() {
                    return Err("请输入主机地址".into());
                }
                let port_num = port.trim().parse::<u16>().map_err(|_| "端口必须为数字".to_string())?;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormField {
    Name,
    Host,
    Port,
    Database,
    Username,
    Password,
    FilePath,
    ConnectionString,
}

pub fn modal_view(
    dialog: &DialogState,
    palette: Palette,
) -> Element<'_, Message> {
    match dialog {
        DialogState::NewConnection(state) => new_connection_modal(state, palette),
    }
}

fn new_connection_modal(
    dialog: &NewConnectionDialog,
    palette: Palette,
) -> Element<'_, Message> {
    let content = match dialog {
        NewConnectionDialog::SelectingType => connection_type_selector(palette),
        NewConnectionDialog::Editing(form_state) => connection_form(form_state, palette),
    };

    let modal = container(content)
        .width(Length::Fixed(540.0))
        .style(move |_| container::Style {
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
        .style(move |_| container::Style {
            background: Some(Background::Color(palette.overlay)),
            text_color: Some(palette.text),
            border: iced::border::Border::default(),
            shadow: Shadow::default(),
        })
        .into()
}

fn connection_type_selector(palette: Palette) -> Element<'static, Message> {
    let mut content = column![
        text("选择数据库类型").size(22).color(palette.text),
        text("请选择需要连接的数据库类型。").color(palette.text_muted).size(15),
    ]
    .spacing(12);

    for chunk in DatabaseKind::all().chunks(3) {
        let mut row_widget = row![].spacing(12).align_y(Alignment::Center);

        for kind in chunk {
            row_widget = row_widget.push(database_type_card(*kind, palette));
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

fn database_type_card(
    kind: DatabaseKind,
    palette: Palette,
) -> Element<'static, Message> {
    let icon = svg::<Theme>(SvgHandle::from_path(kind.icon_path()))
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

            button::Style {
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

fn connection_form(
    form_state: &ConnectionFormState,
    palette: Palette,
) -> Element<'_, Message> {
    let form = &form_state.form;
    let kind = form.kind();

    let mut content = column![
        row![
            svg::<Theme>(SvgHandle::from_path(kind.icon_path()))
                .width(36)
                .height(36),
            column![
                text(format!("配置 {}", kind.display_name()))
                    .color(palette.text)
                    .size(22),
                text("填写数据库连接信息。").color(palette.text_muted).size(15),
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
                .push(labeled_input("连接名称", name, palette, move |input| {
                    Message::UpdateFormField(FormField::Name, input)
                }))
                .push(labeled_input("主机", host, palette, move |input| {
                    Message::UpdateFormField(FormField::Host, input)
                }))
                .push(labeled_input("端口", port, palette, move |input| {
                    Message::UpdateFormField(FormField::Port, input)
                }))
                .push(labeled_input("数据库", database, palette, move |input| {
                    Message::UpdateFormField(FormField::Database, input)
                }))
                .push(labeled_input("用户名", username, palette, move |input| {
                    Message::UpdateFormField(FormField::Username, input)
                }))
                .push(labeled_input("密码", password, palette, move |input| {
                    Message::UpdateFormField(FormField::Password, input)
                }));
        }
        ConnectionForm::Sqlite { name, file_path } => {
            fields = fields
                .push(labeled_input("连接名称", name, palette, move |input| {
                    Message::UpdateFormField(FormField::Name, input)
                }))
                .push(labeled_input(
                    "数据库文件路径",
                    file_path,
                    palette,
                    move |input| Message::UpdateFormField(FormField::FilePath, input),
                ));
        }
        ConnectionForm::Mongo {
            name,
            connection_string,
        } => {
            fields = fields
                .push(labeled_input("连接名称", name, palette, move |input| {
                    Message::UpdateFormField(FormField::Name, input)
                }))
                .push(labeled_input(
                    "连接字符串",
                    connection_string,
                    palette,
                    move |input| Message::UpdateFormField(FormField::ConnectionString, input),
                ));
        }
        ConnectionForm::Redis {
            name,
            host,
            port,
            password,
        } => {
            fields = fields
                .push(labeled_input("连接名称", name, palette, move |input| {
                    Message::UpdateFormField(FormField::Name, input)
                }))
                .push(labeled_input("主机", host, palette, move |input| {
                    Message::UpdateFormField(FormField::Host, input)
                }))
                .push(labeled_input("端口", port, palette, move |input| {
                    Message::UpdateFormField(FormField::Port, input)
                }))
                .push(labeled_input("密码", password, palette, move |input| {
                    Message::UpdateFormField(FormField::Password, input)
                }));
        }
    }

    content = content.push(fields);

    if let Some(error) = &form_state.error {
        let error_color = Color::from_rgb8(0xff, 0x4d, 0x4f);
        content = content.push(
            container(text(error).color(error_color).size(14))
                .padding([8, 12])
                .style(move |_| container::Style {
                    background: Some(Background::Color(Color::from_rgba(0.98, 0.29, 0.3, 0.12))),
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
