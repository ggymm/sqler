use iced::widget::{Stack, column, container, row};
use iced::{Background, Element, Length, Shadow, Task};

mod connections;
mod content;
mod dialog;
mod theme;
mod top_bar;

pub use connections::{Connection, ConnectionsState, DatabaseKind};
pub use dialog::{ConnectionFormState, DialogState, FormField, NewConnectionDialog};
pub use theme::Palette;

use connections::sidebar;
use content::content_panel;
use dialog::modal_view;
use theme::ThemeMode;
use top_bar::top_bar;

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

    pub fn connection(&self, id: usize) -> Option<&Connection> {
        self.connections.find(id)
    }
}

pub fn update(app: &mut App, message: Message) -> Task<Message> {
    match message {
        Message::ToggleTheme => {
            app.theme.toggle();
        }
        Message::SelectContentTab(tab) => {
            app.active_tab = tab;
        }
        Message::ShowNewConnectionDialog => {
            app.dialog = Some(DialogState::NewConnection(
                NewConnectionDialog::SelectingType,
            ));
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
            if let Some(DialogState::NewConnection(NewConnectionDialog::Editing(form_state))) =
                &mut app.dialog
            {
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
                                app.dialog = Some(DialogState::NewConnection(
                                    NewConnectionDialog::Editing(form_state),
                                ));
                            }
                        }
                    }
                    NewConnectionDialog::SelectingType => {
                        app.dialog = Some(DialogState::NewConnection(
                            NewConnectionDialog::SelectingType,
                        ));
                    }
                }
            }
        }
    }

    Task::none()
}

pub fn view(app: &App) -> Element<'_, Message> {
    let palette = app.palette();

    let content = column![top_bar(app, palette), body(app, palette)]
        .spacing(0)
        .height(Length::Fill);

    let base = container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_| iced::widget::container::Style {
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

fn body(app: &App, palette: Palette) -> Element<'_, Message> {
    row![
        container(sidebar(&app.connections, palette))
            .width(Length::Fixed(260.0))
            .style(move |_| iced::widget::container::Style {
                background: Some(Background::Color(palette.surface)),
                border: iced::border::Border {
                    color: palette.border,
                    width: 1.0,
                    radius: 0.0.into(),
                },
                text_color: Some(palette.text),
                shadow: Shadow::default(),
            }),
        container(content_panel(app, palette))
            .width(Length::Fill)
            .style(move |_| iced::widget::container::Style {
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
