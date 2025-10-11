use iced::widget::{button, column, container, text};
use iced::{Background, Color, Element, Length, Shadow};

use crate::app::{Message, Palette};

#[derive(Debug, Clone)]
pub enum LoadState<T> {
    Idle,
    Loading,
    Ready(T),
    Error(String),
}

impl<T> Default for LoadState<T> {
    fn default() -> Self {
        LoadState::Idle
    }
}

impl<T> LoadState<T> {
    pub fn should_load(&self) -> bool {
        matches!(self, LoadState::Idle | LoadState::Error(_))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LoadStateMessages {
    pub loading: &'static str,
    pub empty: &'static str,
    pub idle: &'static str,
}

pub fn generic_toolbar_button(
    label: &'static str,
    message: Message,
    palette: Palette,
) -> Element<'static, Message> {
    button(text(label).size(14).color(palette.text))
        .padding([6, 12])
        .style(move |_, status| {
            use iced::widget::button::Status;

            let background = match status {
                Status::Hovered => palette.surface_muted,
                Status::Pressed => palette.surface,
                _ => Color::TRANSPARENT,
            };

            iced::widget::button::Style {
                background: Some(Background::Color(background)),
                border: iced::border::Border {
                    color: palette.border,
                    width: 1.0,
                    radius: 6.0.into(),
                },
                text_color: palette.text,
                shadow: Shadow::default(),
            }
        })
        .on_press(message)
        .into()
}

pub fn stack_section(
    actions: Element<'static, Message>,
    content: Element<'static, Message>,
) -> Element<'static, Message> {
    column![actions, content].spacing(16).into()
}

pub fn centered_message<I, S>(
    lines: I,
    palette: Palette,
) -> Element<'static, Message>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let mut body = column![];
    for line in lines.into_iter() {
        body = body.push(text(line.into()).size(13).color(palette.text_muted));
    }

    container(body.spacing(6))
        .width(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

pub fn load_state_list_view<'a, T>(
    state: Option<&'a LoadState<Vec<T>>>,
    palette: Palette,
    messages: LoadStateMessages,
    ready: impl Fn(&'a [T], Palette) -> Element<'static, Message>,
) -> Element<'static, Message> {
    match state {
        Some(LoadState::Loading) => loading_view(messages.loading, palette),
        Some(LoadState::Error(err)) => error_view(err, palette),
        Some(LoadState::Ready(items)) if items.is_empty() => centered_message([messages.empty], palette),
        Some(LoadState::Ready(items)) => ready(items, palette),
        _ => idle_view(messages.idle, palette),
    }
}

pub fn loading_view(
    message: &'static str,
    palette: Palette,
) -> Element<'static, Message> {
    container(text(message).size(14).color(palette.text_muted))
        .width(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

pub fn error_view(
    message: &str,
    palette: Palette,
) -> Element<'static, Message> {
    container(text(format!("加载失败：{message}")).size(14).color(palette.accent))
        .width(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

pub fn empty_view(
    message: &'static str,
    palette: Palette,
) -> Element<'static, Message> {
    container(text(message).size(14).color(palette.text_muted))
        .width(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

pub fn idle_view(
    message: &'static str,
    palette: Palette,
) -> Element<'static, Message> {
    container(text(message).size(14).color(palette.text_muted))
        .width(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

pub fn card_style(palette: Palette) -> container::Style {
    container::Style {
        background: Some(Background::Color(palette.surface)),
        text_color: Some(palette.text),
        border: iced::border::Border {
            color: palette.border,
            width: 1.0,
            radius: 12.0.into(),
        },
        shadow: Shadow::default(),
    }
}

pub fn surface_panel<I, S>(
    title: &str,
    lines: I,
    palette: Palette,
) -> Element<'static, Message>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let mut column = column![text(title.to_owned()).size(18).color(palette.text)];
    for line in lines {
        column = column.push(text(line.into()).size(13).color(palette.text_muted));
    }

    container(column.spacing(12))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_| container::Style {
            background: Some(Background::Color(palette.surface)),
            text_color: Some(palette.text),
            border: iced::border::Border {
                color: palette.border,
                width: 1.0,
                radius: 8.0.into(),
            },
            shadow: Shadow::default(),
        })
        .padding(16)
        .into()
}
