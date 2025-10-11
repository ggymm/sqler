use iced::widget::{column, row, text, vertical_space};
use iced::{Alignment, Element};

use crate::app::{Connection, Message, Palette};

use super::{generic_toolbar_button, stack_section};

pub(super) fn view(connection: &Connection, palette: Palette) -> Element<'static, Message> {
    let actions = row![
        generic_toolbar_button("新建查询", Message::NewSavedQuery, palette),
        generic_toolbar_button("删除查询", Message::DeleteSavedQuery, palette),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let summary = connection.summary();
    let content: Element<'static, Message> = column![
        text("查询列表").size(16).color(palette.text),
        text(format!("当前连接：{}", summary))
            .size(13)
            .color(palette.text_muted),
        vertical_space().height(12),
        text("暂无保存的查询。").size(13).color(palette.text_muted),
        text("点击“新建查询”以创建新的查询标签页。")
            .size(12)
            .color(palette.text_muted),
    ]
    .spacing(8)
    .into();

    stack_section(actions.into(), content)
}
