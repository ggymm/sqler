use iced::widget::{button, column, container, text};
use iced::{Background, Color, Element, Length, Shadow};

use crate::app::Palette;

pub fn context_menu_button<'a, Message: 'a + Clone>(
    label: &'static str,
    message: Message,
    palette: Palette,
) -> Element<'a, Message> {
    button(text(label).size(14).color(palette.text))
        .width(Length::Fill)
        .padding([8, 12])
        .style(move |_, status| {
            use iced::widget::button::Status;

            let background = match status {
                Status::Hovered => palette.surface_muted,
                Status::Pressed => palette.surface_muted,
                _ => Color::TRANSPARENT,
            };

            let mut style = iced::widget::button::Style::default();
            style.background = Some(Background::Color(background));
            style.text_color = palette.text;
            style.border = iced::border::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 6.0.into(),
            };
            style.shadow = Shadow::default();

            style
        })
        .on_press(message)
        .into()
}

pub fn context_menu<'a, Message: 'a, I>(
    items: I,
    palette: Palette,
) -> container::Container<'a, Message>
where
    I: IntoIterator<Item = Element<'a, Message>>,
{
    let content = items
        .into_iter()
        .fold(column![].spacing(6), |column, item| column.push(item));

    container(content)
        .padding(6)
        .width(Length::Fixed(168.0))
        .style(move |_| container::Style {
            background: Some(Background::Color(palette.surface)),
            text_color: Some(palette.text),
            border: iced::border::Border {
                color: palette.border,
                width: 1.0,
                radius: 10.0.into(),
            },
            shadow: Shadow {
                color: Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 0.15,
                },
                blur_radius: 12.0,
                offset: iced::Vector::new(0.0, 6.0),
            },
        })
}
