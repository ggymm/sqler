use iced::widget::{Space, container};
use iced::{Background, Color, Element, Length, Shadow};

use crate::app::Palette;

pub fn overlay_backdrop<'a, Message: 'a>(palette: Palette) -> Element<'a, Message> {
    container(Space::with_width(Length::Fill).height(Length::Fill))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_| container::Style {
            background: Some(Background::Color(palette.overlay)),
            text_color: None,
            border: iced::border::Border::default(),
            shadow: Shadow::default(),
        })
        .into()
}

pub fn modal_card<'a, Message: 'a>(
    content: Element<'a, Message>,
    palette: Palette,
    padding: impl Into<iced::Padding>,
    radius: f32,
) -> container::Container<'a, Message> {
    container(content).padding(padding).style(move |_| container::Style {
        background: Some(Background::Color(palette.surface)),
        text_color: Some(palette.text),
        border: iced::border::Border {
            color: palette.border,
            width: 1.0,
            radius: radius.into(),
        },
        shadow: Shadow {
            color: Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.35,
            },
            blur_radius: 24.0,
            offset: iced::Vector::new(0.0, 12.0),
        },
    })
}
