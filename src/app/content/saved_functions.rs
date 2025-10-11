use iced::Element;

use super::{App, Message, Palette};
use super::common::surface_panel;

pub fn render_saved_functions(
    app: &App,
    palette: Palette,
    connection_id: usize,
) -> Element<'static, Message> {
    let summary = app
        .connection(connection_id)
        .map(|conn| conn.summary())
        .unwrap_or_else(|| "未知连接".into());

    surface_panel(
        "保存的函数",
        [
            format!("连接：{}", summary),
            "函数列表页面仍在开发中。".into(),
        ],
        palette,
    )
}
