use iced::widget::{column, container, text};
use iced::{Background, Element, Length, Shadow};

use super::{App, Message, Palette};

pub fn render_saved_queries(
    app: &App,
    palette: Palette,
    connection_id: usize,
) -> Element<'static, Message> {
    let summary = app
        .connection(connection_id)
        .map(|conn| conn.summary())
        .unwrap_or_else(|| "未知连接".into());

    container(
        column![
            text("保存的查询").size(18).color(palette.text),
            text(format!("连接：{}", summary)).size(13).color(palette.text_muted),
            text("暂未实现查询管理界面。").size(13).color(palette.text_muted),
        ]
        .spacing(12),
    )
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
