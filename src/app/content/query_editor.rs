use iced::Element;

use super::{App, Message, Palette};
use super::common::surface_panel;

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

    surface_panel(
        "查询编辑器",
        [info, placeholder],
        palette,
    )
}
