use iced::widget::{column, container, row, text};
use iced::{Background, Color, Element, Length, Shadow};

#[derive(Debug, Clone, Copy)]
pub enum NotificationKind {
    Info,
    Success,
    Warning,
    Error,
}

pub fn banner<'a, Message, Title>(
    kind: NotificationKind,
    title: Title,
    detail: Option<String>,
) -> Element<'a, Message>
where
    Title: Into<String>,
    Message: 'a,
{
    let style = kind.style();

    let mut content = column![text(title.into()).size(16).color(style.foreground)].spacing(6);

    if let Some(detail) = detail {
        if !detail.is_empty() {
            content = content.push(text(detail).size(13).color(style.foreground));
        }
    }

    container(row![content].spacing(12))
        .padding([12, 16])
        .width(Length::Fill)
        .style(move |_| container::Style {
            background: Some(Background::Color(style.background)),
            text_color: Some(style.foreground),
            border: iced::border::Border {
                color: style.border,
                width: 1.0,
                radius: 10.0.into(),
            },
            shadow: Shadow::default(),
        })
        .into()
}

struct NotificationStyle {
    background: Color,
    border: Color,
    foreground: Color,
}

impl NotificationKind {
    fn style(self) -> NotificationStyle {
        match self {
            NotificationKind::Info => NotificationStyle {
                background: Color::from_rgba8(0xd8, 0xe6, 0xff, 0.6),
                border: Color::from_rgb8(0x42, 0x82, 0xff),
                foreground: Color::from_rgb8(0x0b, 0x3d, 0x91),
            },
            NotificationKind::Success => NotificationStyle {
                background: Color::from_rgba8(0xd9, 0xf2, 0xe3, 0.65),
                border: Color::from_rgb8(0x2f, 0xa0, 0x5a),
                foreground: Color::from_rgb8(0x1f, 0x6b, 0x3b),
            },
            NotificationKind::Warning => NotificationStyle {
                background: Color::from_rgba8(0xff, 0xf4, 0xdc, 0.7),
                border: Color::from_rgb8(0xc7, 0x7c, 0x02),
                foreground: Color::from_rgb8(0x8a, 0x54, 0x00),
            },
            NotificationKind::Error => NotificationStyle {
                background: Color::from_rgba8(0xff, 0xe1, 0xe0, 0.7),
                border: Color::from_rgb8(0xd9, 0x3c, 0x3c),
                foreground: Color::from_rgb8(0x8b, 0x1e, 0x1e),
            },
        }
    }
}
