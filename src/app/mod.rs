use iced::widget::{Stack, column, container, row};
use iced::{Background, Color, Element, Font, Length, Shadow, Task};

mod sidebar;
mod content;
mod dialog;
mod topbar;

pub use sidebar::{Connection, ConnectionsState, DatabaseKind};
pub use dialog::{ConnectionFormState, DialogState, FormField, NewConnectionDialog};

use sidebar::sidebar;
use content::content;
use dialog::modal_view;
use topbar::topbar;

#[derive(Debug)]
pub struct App {
    theme: ThemeMode,
    active_tab: ContentTab,
    connections: ConnectionsState,
    dialog: Option<DialogState>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            theme: ThemeMode::Light,
            active_tab: ContentTab::Tables,
            connections: ConnectionsState::new(),
            dialog: None,
        }
    }
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

    pub fn dialog(&self) -> Option<&DialogState> {
        self.dialog.as_ref()
    }

    pub fn connection(
        &self,
        id: usize,
    ) -> Option<&Connection> {
        self.connections.find(id)
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    Dark,
    Light,
}

impl ThemeMode {
    pub fn toggle(&mut self) {
        *self = match self {
           ThemeMode::Dark =>ThemeMode::Light,
           ThemeMode::Light =>ThemeMode::Dark,
        };
    }

    pub fn palette(&self) -> crate::app::Palette {
        match self {
           ThemeMode::Light => crate::app::Palette {
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
           ThemeMode::Dark => crate::app::Palette {
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

pub fn update(
    app: &mut App,
    message: Message,
) -> Task<Message> {
    match message {
        Message::ToggleTheme => {
            app.theme.toggle();
        }
        Message::SelectContentTab(tab) => {
            app.active_tab = tab;
        }
        Message::ShowNewConnectionDialog => {
            app.dialog = Some(DialogState::NewConnection(NewConnectionDialog::SelectingType));
        }
        Message::ShowNewQueryWorkspace => {
            app.active_tab = ContentTab::Queries;
        }
        Message::SelectConnection(id) => {
            app.connections.select(id);
        }
        Message::CancelDialog => {
            app.dialog = None;
        }
        Message::NewConnectionTypeSelected(kind) => {
            let next = NewConnectionDialog::Editing(ConnectionFormState::new(kind));

            match &mut app.dialog {
                Some(DialogState::NewConnection(dialog)) => {
                    *dialog = next;
                }
                _ => {
                    app.dialog = Some(DialogState::NewConnection(next));
                }
            }
        }
        Message::BackToConnectionTypeSelection => {
            if let Some(DialogState::NewConnection(dialog)) = &mut app.dialog {
                *dialog = NewConnectionDialog::SelectingType;
            }
        }
        Message::UpdateFormField(field, value) => {
            if let Some(DialogState::NewConnection(NewConnectionDialog::Editing(form_state))) = &mut app.dialog {
                form_state.clear_error();
                form_state.form.update(field, value);
            }
        }
        Message::SubmitNewConnection => {
            if let Some(DialogState::NewConnection(dialog)) = app.dialog.take() {
                match dialog {
                    NewConnectionDialog::Editing(mut form_state) => {
                        let id = app.connections.next_id();
                        match form_state.build_connection(id) {
                            Ok(connection) => {
                                app.connections.add(connection);
                                app.dialog = None;
                            }
                            Err(error) => {
                                form_state.error = Some(error);
                                app.dialog = Some(DialogState::NewConnection(NewConnectionDialog::Editing(form_state)));
                            }
                        }
                    }
                    NewConnectionDialog::SelectingType => {
                        app.dialog = Some(DialogState::NewConnection(NewConnectionDialog::SelectingType));
                    }
                }
            }
        }
    }

    Task::none()
}

pub fn view(app: &App) -> Element<'_, Message> {
    let palette = app.palette();

    let content = column![topbar(app, palette), body(app, palette)]
        .spacing(0)
        .height(Length::Fill);

    let base = container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_| container::Style {
            background: Some(Background::Color(palette.background)),
            text_color: Some(palette.text),
            border: iced::border::Border::default(),
            shadow: Shadow::default(),
        })
        .into();

    if let Some(dialog_state) = app.dialog() {
        Stack::new()
            .width(Length::Fill)
            .height(Length::Fill)
            .push(base)
            .push(modal_view(dialog_state, palette))
            .into()
    } else {
        base
    }
}

fn body(
    app: &App,
    palette: Palette,
) -> Element<'_, Message> {
    row![
        container(sidebar(&app.connections, palette))
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
