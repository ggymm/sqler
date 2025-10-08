use iced::widget::{column, text, text_input};
use iced::{Background, Element, Length};

use crate::app::Palette;

pub fn labeled_input<'a, Message, F>(
    label: &'a str,
    value: &'a str,
    palette: Palette,
    on_input: F,
) -> Element<'a, Message>
where
    Message: 'a + Clone,
    F: Fn(String) -> Message + 'a,
{
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
            .on_input(on_input),
    ]
    .spacing(6)
    .width(Length::Fill)
    .into()
}
