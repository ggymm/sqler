use iced::widget::svg::Handle as SvgHandle;
use iced::widget::{button, column, container, horizontal_space, row, scrollable, svg, text};
use iced::{Alignment, Background, Color, Element, Length, Shadow, Size, Theme};

use crate::comps::form::labeled_input;
use crate::comps::notification::{NotificationKind, banner as notify_banner};
use crate::comps::popup::modal_card;
use crate::driver::ConnectionParams;

use super::sidebar::{Connection, ConnectionConfig};
use super::{ConnectionStatus, ConnectionStatusInfo, DatabaseKind, Message, Palette};

pub fn modal_view(
    state: &NewConnectionDialog,
    palette: Palette,
    minimized: bool,
    window_size: Size,
) -> Element<'_, Message> {
    let title = match state {
        NewConnectionDialog::SelectingType => "新建连接",
        NewConnectionDialog::Editing(_) => "编辑连接",
    };

    if minimized {
        return modal_card(minimized_header(title, palette).into(), palette, [12.0, 16.0], 12.0)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into();
    }

    let layout = match state {
        NewConnectionDialog::SelectingType => selecting_type_layout(palette),
        NewConnectionDialog::Editing(form_state) => connection_form_layout(form_state, palette),
    };

    let header = window_header(title, palette);
    let footer = layout.footer;
    let body = scrollable(container(layout.main).width(Length::Fill).padding([0, 4])).height(Length::Fill);

    let content = column![header, body, footer]
        .spacing(18)
        .width(Length::Fill)
        .height(Length::Fill);

    let width = scale_dimension(window_size.width, 0.6, 360.0, 64.0);
    let height = scale_dimension(window_size.height, 0.6, 320.0, 80.0);

    let dialog_box = modal_card(content.into(), palette, 24.0, 16.0)
        .width(Length::Fixed(width))
        .height(Length::Fixed(height));

    container(dialog_box)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

pub fn connection_info_modal<'a>(
    info: &'a ConnectionStatusInfo,
    connection: Option<&'a Connection>,
    palette: Palette,
    window_size: Size,
) -> Element<'a, Message> {
    let (banner_kind, banner_title, banner_detail, show_retry) = match &info.status {
        ConnectionStatus::Connecting => (
            NotificationKind::Info,
            "正在连接",
            Some("正在尝试建立连接…".to_string()),
            true,
        ),
        ConnectionStatus::Success => (
            NotificationKind::Success,
            "连接成功",
            Some("连接已成功建立。".to_string()),
            false,
        ),
        ConnectionStatus::Failed(reason) => (NotificationKind::Error, "连接失败", Some(reason.clone()), true),
        ConnectionStatus::Details => (
            NotificationKind::Info,
            "连接信息",
            Some("查看当前连接配置。".to_string()),
            false,
        ),
    };

    let mut content = column![notify_banner(banner_kind, banner_title, banner_detail)].spacing(12);

    if let Some(connection) = connection {
        content = content.push(connection_details(connection, palette));
    } else {
        content = content.push(notify_banner(
            NotificationKind::Warning,
            "未找到该连接",
            Some("连接可能已被删除或不可用。".into()),
        ));
    }

    let close_button = button(text("关闭").color(palette.text))
        .padding([6, 16])
        .style(move |_, _| iced::widget::button::Style {
            background: Some(Background::Color(palette.surface_muted)),
            border: iced::border::Border {
                color: palette.border,
                width: 1.0,
                radius: 6.0.into(),
            },
            text_color: palette.text,
            shadow: Shadow::default(),
        })
        .on_press(Message::DismissConnectionStatus);

    let mut actions = row![close_button].spacing(12).align_y(Alignment::Center);

    if show_retry {
        actions = actions.push(
            button(text("重新连接").color(palette.accent_text))
                .padding([6, 18])
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
                            radius: 6.0.into(),
                        },
                        text_color: palette.accent_text,
                        shadow: Shadow::default(),
                    }
                })
                .on_press(Message::ActivateConnection(info.connection_id)),
        );
    }

    let modal_width = (window_size.width * 0.5).clamp(320.0, 720.0);
    let modal_height = (window_size.height * 0.5).clamp(220.0, 560.0);

    modal_card(column![content, actions].spacing(18).into(), palette, 24.0, 16.0)
        .width(Length::Shrink)
        .height(Length::Shrink)
        .max_width(modal_width)
        .max_height(modal_height)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

struct DialogSections<'a> {
    main: Element<'a, Message>,
    footer: Element<'a, Message>,
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
    pub testing: bool,
    pub test_result: Option<Result<(), String>>,
    pub existing_id: Option<usize>,
}

impl ConnectionFormState {
    pub fn new(kind: DatabaseKind) -> Self {
        Self {
            form: ConnectionForm::from_kind(kind),
            error: None,
            testing: false,
            test_result: None,
            existing_id: None,
        }
    }

    pub fn from_connection(connection: &Connection) -> Self {
        Self {
            form: ConnectionForm::from_connection(connection),
            error: None,
            testing: false,
            test_result: None,
            existing_id: Some(connection.id),
        }
    }

    pub fn build_connection(
        &self,
        id: usize,
    ) -> Result<super::Connection, String> {
        self.form.build_connection(self.existing_id.unwrap_or(id))
    }

    pub fn clear_error(&mut self) {
        self.error = None;
        self.test_result = None;
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

    fn from_connection(connection: &Connection) -> Self {
        match &connection.config {
            ConnectionConfig::Relational {
                host,
                port,
                database,
                username,
                password,
            } => ConnectionForm::Relational {
                kind: connection.kind,
                name: connection.name.clone(),
                host: host.clone(),
                port: port.to_string(),
                database: database.clone(),
                username: username.clone(),
                password: password.clone().unwrap_or_default(),
            },
            ConnectionConfig::Sqlite { file_path } => ConnectionForm::Sqlite {
                name: connection.name.clone(),
                file_path: file_path.clone(),
            },
            ConnectionConfig::Mongo { connection_string } => ConnectionForm::Mongo {
                name: connection.name.clone(),
                connection_string: connection_string.clone(),
            },
            ConnectionConfig::Redis { host, port, password } => ConnectionForm::Redis {
                name: connection.name.clone(),
                host: host.clone(),
                port: port.to_string(),
                password: password.clone().unwrap_or_default(),
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

    pub fn to_params(&self) -> Result<ConnectionParams, String> {
        match self {
            ConnectionForm::Relational {
                kind,
                host,
                port,
                username,
                password,
                database,
                ..
            } => {
                let host = host.trim();
                if host.is_empty() {
                    return Err("请输入主机地址".into());
                }

                let port = port.trim().parse::<u16>().map_err(|_| "端口必须为数字".to_string())?;

                let username = username.trim();
                if username.is_empty() {
                    return Err("请输入用户名".into());
                }

                let database = database.trim();

                Ok(ConnectionParams {
                    kind: *kind,
                    host: Some(host.to_string()),
                    port: Some(port),
                    username: Some(username.to_string()),
                    password: if password.is_empty() {
                        None
                    } else {
                        Some(password.clone())
                    },
                    database: if database.is_empty() {
                        None
                    } else {
                        Some(database.to_string())
                    },
                    file_path: None,
                    connection_string: None,
                })
            }
            ConnectionForm::Sqlite { file_path, .. } => {
                let path = file_path.trim();
                if path.is_empty() {
                    return Err("请输入数据库文件路径".into());
                }

                Ok(ConnectionParams {
                    kind: DatabaseKind::Sqlite,
                    host: None,
                    port: None,
                    username: None,
                    password: None,
                    database: None,
                    file_path: Some(path.to_string()),
                    connection_string: None,
                })
            }
            ConnectionForm::Mongo { connection_string, .. } => {
                let conn = connection_string.trim();
                if conn.is_empty() {
                    return Err("请输入连接字符串".into());
                }

                Ok(ConnectionParams {
                    kind: DatabaseKind::Mongodb,
                    host: None,
                    port: None,
                    username: None,
                    password: None,
                    database: None,
                    file_path: None,
                    connection_string: Some(conn.to_string()),
                })
            }
            ConnectionForm::Redis {
                host, port, password, ..
            } => {
                let host = host.trim();
                if host.is_empty() {
                    return Err("请输入主机地址".into());
                }

                let port = port.trim().parse::<u16>().map_err(|_| "端口必须为数字".to_string())?;

                Ok(ConnectionParams {
                    kind: DatabaseKind::Redis,
                    host: Some(host.to_string()),
                    port: Some(port),
                    username: None,
                    password: if password.is_empty() {
                        None
                    } else {
                        Some(password.clone())
                    },
                    database: None,
                    file_path: None,
                    connection_string: None,
                })
            }
        }
    }

    fn build_connection(
        &self,
        id: usize,
    ) -> Result<super::Connection, String> {
        match self {
            ConnectionForm::Relational {
                kind,
                name,
                host,
                port,
                database,
                username,
                password,
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

                let config = ConnectionConfig::Relational {
                    host: host.trim().to_string(),
                    port: port_num,
                    database: database.trim().to_string(),
                    username: username.trim().to_string(),
                    password: if password.trim().is_empty() {
                        None
                    } else {
                        Some(password.trim().to_string())
                    },
                };

                Ok(super::Connection {
                    id,
                    name: name.trim().to_string(),
                    kind: *kind,
                    config,
                })
            }
            ConnectionForm::Sqlite { name, file_path } => {
                if name.trim().is_empty() {
                    return Err("请输入连接名称".into());
                }
                if file_path.trim().is_empty() {
                    return Err("请输入数据库文件路径".into());
                }

                let config = ConnectionConfig::Sqlite {
                    file_path: file_path.trim().to_string(),
                };

                Ok(super::Connection {
                    id,
                    name: name.trim().to_string(),
                    kind: DatabaseKind::Sqlite,
                    config,
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

                let config = ConnectionConfig::Mongo {
                    connection_string: connection_string.trim().to_string(),
                };

                Ok(super::Connection {
                    id,
                    name: name.trim().to_string(),
                    kind: DatabaseKind::Mongodb,
                    config,
                })
            }
            ConnectionForm::Redis {
                name,
                host,
                port,
                password,
            } => {
                if name.trim().is_empty() {
                    return Err("请输入连接名称".into());
                }
                if host.trim().is_empty() {
                    return Err("请输入主机地址".into());
                }
                let port_num = port.trim().parse::<u16>().map_err(|_| "端口必须为数字".to_string())?;

                let config = ConnectionConfig::Redis {
                    host: host.trim().to_string(),
                    port: port_num,
                    password: if password.trim().is_empty() {
                        None
                    } else {
                        Some(password.trim().to_string())
                    },
                };

                Ok(super::Connection {
                    id,
                    name: name.trim().to_string(),
                    kind: DatabaseKind::Redis,
                    config,
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

fn selecting_type_layout(palette: Palette) -> DialogSections<'static> {
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

    let footer = row![
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
    .align_y(Alignment::Center);

    DialogSections {
        main: content.spacing(20).into(),
        footer: container(footer).width(Length::Fill).align_x(Alignment::End).into(),
    }
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

fn connection_form_layout<'a>(
    form_state: &'a ConnectionFormState,
    palette: Palette,
) -> DialogSections<'a> {
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

    if form_state.testing {
        content = content.push(text("正在测试连接...").color(palette.text_muted).size(14));
    } else if let Some(result) = &form_state.test_result {
        let feedback = match result {
            Ok(_) => text("连接成功").color(Color::from_rgb8(0x1f, 0xaa, 0x5c)).size(14),
            Err(err) => text(err).color(Color::from_rgb8(0xff, 0x4d, 0x4f)).size(14),
        };

        content = content.push(feedback);
    }

    let mut test_button = button(text("测试连接").color(palette.text))
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
        });

    if !form_state.testing {
        test_button = test_button.on_press(Message::TestConnection);
    }

    let footer = row![
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
        test_button,
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
    .align_y(Alignment::Center);

    DialogSections {
        main: content.spacing(18).into(),
        footer: container(footer).width(Length::Fill).align_x(Alignment::End).into(),
    }
}

fn connection_details(
    connection: &Connection,
    palette: Palette,
) -> Element<'static, Message> {
    let mut details = column![
        text(format!("连接名称：{}", connection.name))
            .color(palette.text)
            .size(14),
        text(format!("数据库类型：{}", connection.kind.display_name()))
            .color(palette.text)
            .size(14),
    ]
    .spacing(6);

    match &connection.config {
        ConnectionConfig::Relational {
            host,
            port,
            database,
            username,
            password,
        } => {
            details = details
                .push(text(format!("主机：{}", host)).color(palette.text_muted).size(13))
                .push(text(format!("端口：{}", port)).color(palette.text_muted).size(13))
                .push(text(format!("数据库：{}", database)).color(palette.text_muted).size(13))
                .push(text(format!("用户名：{}", username)).color(palette.text_muted).size(13));

            if let Some(password) = password {
                let masked = if password.is_empty() {
                    "(未设置)".to_string()
                } else {
                    "******".to_string()
                };
                details = details.push(text(format!("密码：{}", masked)).color(palette.text_muted).size(13));
            }
        }
        ConnectionConfig::Sqlite { file_path } => {
            details = details.push(
                text(format!("文件路径：{}", file_path))
                    .color(palette.text_muted)
                    .size(13),
            );
        }
        ConnectionConfig::Mongo { connection_string } => {
            details = details.push(
                text(format!("连接字符串：{}", connection_string))
                    .color(palette.text_muted)
                    .size(13),
            );
        }
        ConnectionConfig::Redis { host, port, password } => {
            details = details
                .push(text(format!("主机：{}", host)).color(palette.text_muted).size(13))
                .push(text(format!("端口：{}", port)).color(palette.text_muted).size(13));

            if let Some(password) = password {
                let masked = if password.is_empty() {
                    "(未设置)".to_string()
                } else {
                    "******".to_string()
                };
                details = details.push(text(format!("密码：{}", masked)).color(palette.text_muted).size(13));
            }
        }
    }

    details.into()
}

fn window_header(
    title: &str,
    palette: Palette,
) -> Element<'_, Message> {
    row![
        text(title).color(palette.text).size(20),
        horizontal_space().width(Length::Fill),
        window_controls(palette, false),
    ]
    .align_y(Alignment::Center)
    .into()
}

fn minimized_header(
    title: &str,
    palette: Palette,
) -> Element<'_, Message> {
    row![
        text(title).color(palette.text).size(18),
        horizontal_space().width(Length::Fill),
        window_controls(palette, true),
    ]
    .align_y(Alignment::Center)
    .into()
}

fn window_controls(
    palette: Palette,
    minimized: bool,
) -> Element<'static, Message> {
    let minimize_label = if minimized { "□" } else { "–" };
    let minimize_message = if minimized {
        Message::RestoreDialog
    } else {
        Message::MinimizeDialog
    };

    row![
        control_button(minimize_label, palette).on_press(minimize_message),
        control_button("×", palette).on_press(Message::CancelDialog),
    ]
    .spacing(6)
    .into()
}

fn scale_dimension(
    size: f32,
    ratio: f32,
    minimum: f32,
    padding: f32,
) -> f32 {
    let mut value = (size * ratio).max(minimum);
    let available = (size - padding).max(minimum);
    if size > padding {
        value = value.min(available);
    }
    value
}

fn control_button(
    label: &'static str,
    palette: Palette,
) -> iced::widget::Button<'static, Message> {
    button(text(label).size(16).color(palette.text))
        .padding([4, 10])
        .style(move |_, _| iced::widget::button::Style {
            background: Some(Background::Color(palette.surface_muted)),
            border: iced::border::Border {
                color: palette.border,
                width: 1.0,
                radius: 10.0.into(),
            },
            text_color: palette.text,
            shadow: Shadow::default(),
        })
}
