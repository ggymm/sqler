use iced::widget::{Space, Stack, column, container, row};
use iced::{Background, Color, Element, Font, Length, Shadow, Size, Subscription, Task, Theme, window};

mod content;
mod dialog;
mod sidebar;
mod topbar;

use content::content;
use dialog::{ConnectionFormState, FormField, NewConnectionDialog, connection_info_modal, modal_view};
#[allow(unused_imports)]
pub use sidebar::ConnectionConfig;
use sidebar::sidebar;
pub use sidebar::{Connection, ConnectionsState, DatabaseKind};
use topbar::topbar;

use crate::driver::DriverRegistry;

#[derive(Debug)]
pub struct App {
    theme: ThemeMode,
    active_tab: ContentTab,
    connections: ConnectionsState,
    dialog: Option<NewConnectionDialog>,
    dialog_minimized: bool,
    drivers: DriverRegistry,
    active_connection: Option<usize>,
    connection_status: Option<ConnectionStatusInfo>,
    context_menu: Option<usize>,
    window_size: Size,
}

impl Default for App {
    fn default() -> Self {
        Self {
            theme: ThemeMode::Light,
            active_tab: ContentTab::Tables,
            connections: ConnectionsState::new(),
            dialog: None,
            dialog_minimized: false,
            drivers: DriverRegistry::new(),
            active_connection: None,
            connection_status: None,
            context_menu: None,
            window_size: Size::new(1280.0, 800.0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConnectionStatusInfo {
    pub connection_id: usize,
    pub status: ConnectionStatus,
}

impl ConnectionStatusInfo {
    pub fn connecting(connection_id: usize) -> Self {
        Self {
            connection_id,
            status: ConnectionStatus::Connecting,
        }
    }

    pub fn success(connection_id: usize) -> Self {
        Self {
            connection_id,
            status: ConnectionStatus::Success,
        }
    }

    pub fn failed(
        connection_id: usize,
        reason: String,
    ) -> Self {
        Self {
            connection_id,
            status: ConnectionStatus::Failed(reason),
        }
    }

    pub fn details(connection_id: usize) -> Self {
        Self {
            connection_id,
            status: ConnectionStatus::Details,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ConnectionStatus {
    Connecting,
    Success,
    Failed(String),
    Details,
}

impl App {
    pub fn palette(&self) -> Palette {
        self.theme.palette()
    }

    pub fn theme(&self) -> ThemeMode {
        self.theme
    }

    pub fn active_tab(&self) -> ContentTab {
        self.active_tab
    }

    pub fn selected_connection(&self) -> Option<usize> {
        self.connections.selected()
    }

    pub fn connection(
        &self,
        id: usize,
    ) -> Option<&Connection> {
        self.connections.find(id)
    }

    pub fn window_size(&self) -> Size {
        self.window_size
    }

    #[allow(dead_code)]
    pub fn active_connection(&self) -> Option<&Connection> {
        self.active_connection.and_then(|id| self.connections.find(id))
    }
}

pub fn update(
    app: &mut App,
    message: Message,
) -> Task<Message> {
    match message {
        Message::ToggleTheme => app.theme.toggle(),
        Message::SelectContentTab(tab) => app.active_tab = tab,
        Message::ShowNewConnectionDialog => {
            app.dialog = Some(NewConnectionDialog::SelectingType);
            app.dialog_minimized = false;
        }
        Message::ShowNewQueryWorkspace => {
            app.active_tab = ContentTab::Queries;
        }
        Message::SelectConnection(id) => {
            let double_clicked = app.connections.select(id);
            app.context_menu = None;

            if double_clicked {
                return Task::done(Message::ActivateConnection(id));
            }
        }
        Message::ActivateConnection(id) => {
            app.context_menu = None;

            if let Some(connection) = app.connections.find(id) {
                let params = connection.to_params();
                app.connection_status = Some(ConnectionStatusInfo::connecting(id));

                let task = app
                    .drivers
                    .test_connection(params)
                    .map(move |result| Message::ConnectionActivationFinished(id, result.map_err(|e| e.to_string())));

                return task;
            } else {
                app.connection_status = Some(ConnectionStatusInfo::failed(id, "连接不存在".into()));
            }
        }
        Message::OpenConnectionContextMenu(id) => {
            app.context_menu = Some(id);
        }
        Message::CancelDialog => {
            app.dialog = None;
            app.dialog_minimized = false;
        }
        Message::NewConnectionTypeSelected(kind) => {
            app.dialog = Some(NewConnectionDialog::Editing(ConnectionFormState::new(kind)));
            app.dialog_minimized = false;
        }
        Message::BackToConnectionTypeSelection => {
            if let Some(NewConnectionDialog::Editing(_)) = app.dialog {
                app.dialog = Some(NewConnectionDialog::SelectingType);
                app.dialog_minimized = false;
            }
        }
        Message::UpdateFormField(field, value) => {
            if let Some(NewConnectionDialog::Editing(form_state)) = &mut app.dialog {
                form_state.clear_error();
                form_state.form.update(field, value);
            }
        }
        Message::SubmitNewConnection => {
            if let Some(NewConnectionDialog::Editing(form_state)) = app.dialog.take() {
                let next_id = app.connections.next_id();
                let is_edit = form_state.existing_id.is_some();

                match form_state.build_connection(next_id) {
                    Ok(connection) => {
                        if is_edit {
                            app.connections.update(connection);
                        } else {
                            app.connections.add(connection);
                        }
                        app.dialog = None;
                        app.dialog_minimized = false;
                    }
                    Err(error) => {
                        let mut state = form_state;
                        state.error = Some(error);
                        app.dialog = Some(NewConnectionDialog::Editing(state));
                    }
                }
            }
        }
        Message::MinimizeDialog => {
            if app.dialog.is_some() {
                app.dialog_minimized = true;
            }
        }
        Message::RestoreDialog => {
            if app.dialog.is_some() {
                app.dialog_minimized = false;
            }
        }
        Message::WindowResized(_id, size) => {
            app.window_size = size;
        }
        Message::TestConnection => {
            if let Some(NewConnectionDialog::Editing(form_state)) = &mut app.dialog {
                match form_state.form.to_params() {
                    Ok(params) => {
                        form_state.testing = true;
                        form_state.test_result = None;

                        let task = app
                            .drivers
                            .test_connection(params)
                            .map(|result| Message::TestConnectionFinished(result.map_err(|e| e.to_string())));

                        return task;
                    }
                    Err(error) => {
                        form_state.test_result = Some(Err(error));
                    }
                }
            }
        }
        Message::TestConnectionFinished(result) => {
            if let Some(NewConnectionDialog::Editing(form_state)) = &mut app.dialog {
                form_state.testing = false;
                form_state.test_result = Some(result);
                app.dialog_minimized = false;
            }
        }
        Message::ConnectionActivationFinished(id, result) => match result {
            Ok(()) => {
                app.connections.activate(id);
                app.active_connection = Some(id);
                app.connection_status = Some(ConnectionStatusInfo::success(id));
            }
            Err(error) => {
                if app.active_connection == Some(id) {
                    app.active_connection = None;
                    app.connections.deactivate();
                }
                app.connection_status = Some(ConnectionStatusInfo::failed(id, error));
            }
        },
        Message::ViewConnection(id) => {
            app.context_menu = None;
            if app.connections.find(id).is_some() {
                app.connection_status = Some(ConnectionStatusInfo::details(id));
            } else {
                app.connection_status = Some(ConnectionStatusInfo::failed(id, "连接不存在".into()));
            }
        }
        Message::EditConnection(id) => {
            app.context_menu = None;
            if let Some(connection) = app.connections.find(id) {
                let form_state = ConnectionFormState::from_connection(connection);
                app.dialog = Some(NewConnectionDialog::Editing(form_state));
                app.dialog_minimized = false;
            } else {
                app.connection_status = Some(ConnectionStatusInfo::failed(id, "连接不存在".into()));
            }
        }
        Message::DeleteConnection(id) => {
            app.context_menu = None;
            app.connections.remove(id);
            if app.active_connection == Some(id) {
                app.active_connection = None;
                app.connection_status = None;
            }
            if app.connection_status.as_ref().map(|info| info.connection_id) == Some(id) {
                app.connection_status = None;
            }
        }
        Message::DismissConnectionStatus => {
            app.connection_status = None;
        }
    }

    Task::none()
}

pub fn view(app: &App) -> Element<'_, Message> {
    let palette = app.palette();

    let base: Element<'_, Message> = container(
        column![topbar(app, palette), body(app, palette, app.context_menu)]
            .spacing(0)
            .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(move |_| container::Style {
        background: Some(Background::Color(palette.background)),
        text_color: Some(palette.text),
        border: iced::border::Border::default(),
        shadow: Shadow::default(),
    })
    .into();

    let mut stack = Stack::new().width(Length::Fill).height(Length::Fill).push(base);

    if let Some(info) = &app.connection_status {
        let overlay = container(Space::with_width(Length::Fill).height(Length::Fill))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_| container::Style {
                background: Some(Background::Color(palette.overlay)),
                text_color: None,
                border: iced::border::Border::default(),
                shadow: Shadow::default(),
            });

        let connection = app.connections.find(info.connection_id);

        stack = stack
            .push(overlay)
            .push(connection_info_modal(info, connection, palette));
    }

    if let Some(dialog) = &app.dialog {
        let overlay = container(Space::with_width(Length::Fill).height(Length::Fill))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_| container::Style {
                background: Some(Background::Color(palette.overlay)),
                text_color: None,
                border: iced::border::Border::default(),
                shadow: Shadow::default(),
            });

        stack = stack
            .push(overlay)
            .push(modal_view(dialog, palette, app.dialog_minimized, app.window_size()));
    }

    stack.into()
}

fn body(
    app: &App,
    palette: Palette,
    context_menu: Option<usize>,
) -> Element<'_, Message> {
    row![
        container(sidebar(&app.connections, palette, context_menu))
            .width(Length::Fixed(260.0))
            .style(move |_| container::Style {
                background: Some(Background::Color(palette.surface)),
                border: iced::border::Border {
                    color: palette.border,
                    width: 1.0,
                    radius: 0.0.into(),
                },
                text_color: Some(palette.text),
                shadow: Shadow::default(),
            }),
        container(content(app, palette))
            .width(Length::Fill)
            .style(move |_| container::Style {
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

pub fn subscription(_app: &App) -> Subscription<Message> {
    window::resize_events().map(|(id, size)| Message::WindowResized(id, size))
}

pub fn theme(app: &App) -> Theme {
    match app.theme {
        ThemeMode::Dark => Theme::Dark,
        ThemeMode::Light => Theme::Light,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentTab {
    Tables,
    Queries,
    Functions,
    Users,
}

impl ContentTab {
    pub fn title(&self) -> &'static str {
        match self {
            ContentTab::Tables => "表",
            ContentTab::Queries => "查询",
            ContentTab::Functions => "函数",
            ContentTab::Users => "用户",
        }
    }

    pub fn icon_path(&self) -> &'static str {
        match self {
            ContentTab::Tables => "assets/icons/table.svg",
            ContentTab::Queries => "assets/icons/query.svg",
            ContentTab::Functions => "assets/icons/function.svg",
            ContentTab::Users => "assets/icons/user.svg",
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    WindowResized(window::Id, Size),
    ToggleTheme,
    SelectContentTab(ContentTab),
    ShowNewConnectionDialog,
    ShowNewQueryWorkspace,
    SelectConnection(usize),
    ActivateConnection(usize),
    OpenConnectionContextMenu(usize),
    ViewConnection(usize),
    EditConnection(usize),
    DeleteConnection(usize),
    CancelDialog,
    NewConnectionTypeSelected(DatabaseKind),
    BackToConnectionTypeSelection,
    UpdateFormField(FormField, String),
    SubmitNewConnection,
    MinimizeDialog,
    RestoreDialog,
    TestConnection,
    TestConnectionFinished(Result<(), String>),
    ConnectionActivationFinished(usize, Result<(), String>),
    DismissConnectionStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    Dark,
    Light,
}

impl ThemeMode {
    pub fn toggle(&mut self) {
        *self = match self {
            ThemeMode::Dark => ThemeMode::Light,
            ThemeMode::Light => ThemeMode::Dark,
        };
    }

    pub fn palette(&self) -> Palette {
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
pub struct Palette {
    pub background: Color,
    pub surface: Color,
    pub surface_muted: Color,
    pub border: Color,
    pub text: Color,
    pub text_muted: Color,
    pub accent: Color,
    pub accent_text: Color,
    pub accent_soft: Color,
    pub overlay: Color,
}

pub fn default_font() -> Font {
    if cfg!(target_os = "macos") {
        Font::with_name("PingFang SC")
    } else if cfg!(target_os = "windows") {
        Font::with_name("Microsoft YaHei UI")
    } else if cfg!(any(target_os = "android", target_os = "linux")) {
        Font::with_name("Noto Sans CJK SC")
    } else if cfg!(target_os = "ios") {
        Font::with_name("PingFang TC")
    } else {
        Font::with_name("Noto Sans CJK SC")
    }
}
