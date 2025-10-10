use iced::widget::{column, container, text};
use iced::{Background, Element, Length, Shadow};

use super::{App, Message, Palette};

pub fn render_query_editor(
    app: &App,
    palette: Palette,
    connection_id: Option<usize>,
    initial_sql: Option<String>,
) -> Element<'static, Message> {
    let info = match connection_id
        .and_then(|id| app.connection(id))
        .map(|conn| conn.summary())
    {
        Some(summary) => format!("当前连接：{}", summary),
        None => "未选择连接，编辑器处于脱机模式。".into(),
    };

    let placeholder = initial_sql.unwrap_or_else(|| "-- TODO: 在此编写 SQL 查询".into());

    container(
        column![
            text("查询编辑器").size(18).color(palette.text),
            text(info).size(13).color(palette.text_muted),
            text(placeholder).size(13).color(palette.text_muted),
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
