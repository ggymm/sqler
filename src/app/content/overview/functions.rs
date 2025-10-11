use iced::widget::{column, container, horizontal_space, row, scrollable, text, vertical_space};
use iced::{Alignment, Element};

use crate::app::{Message, Palette};

use super::{
    card_style, generic_toolbar_button, load_state_list_view, stack_section, LoadState, LoadStateMessages,
    MysqlRoutine,
};

pub(super) fn view(
    state: Option<&LoadState<Vec<MysqlRoutine>>>,
    palette: Palette,
) -> Element<'static, Message> {
    let actions = row![
        generic_toolbar_button("新建函数", Message::NewFunction, palette),
        generic_toolbar_button("删除函数", Message::DeleteFunction, palette),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let content = routines_content(state, palette);
    stack_section(actions.into(), content)
}

fn routines_content(
    state: Option<&LoadState<Vec<MysqlRoutine>>>,
    palette: Palette,
) -> Element<'static, Message> {
    load_state_list_view(
        state,
        palette,
        LoadStateMessages {
            loading: "正在加载函数与存储过程…",
            empty: "当前库尚未定义函数或存储过程。",
            idle: "请激活连接以查看函数列表。",
        },
        |routines, palette| {
            let mut list = column![];
            for routine in routines {
                list = list.push(routine_row(routine, palette));
            }
            scrollable(list.spacing(12)).into()
        },
    )
}

fn routine_row(
    routine: &MysqlRoutine,
    palette: Palette,
) -> Element<'static, Message> {
    container(
        column![
            row![
                text(routine.name.clone()).size(15).color(palette.text),
                horizontal_space(),
                text(routine.kind.clone()).size(12).color(palette.text_muted),
            ]
            .align_y(Alignment::Center),
            vertical_space().height(6),
            row![
                text(
                    routine
                        .returns
                        .clone()
                        .filter(|v| !v.is_empty())
                        .unwrap_or_else(|| "无返回".into())
                )
                .size(12)
                .color(palette.text_muted),
                horizontal_space(),
                text(
                    routine
                        .security
                        .clone()
                        .filter(|v| !v.is_empty())
                        .unwrap_or_else(|| "-".into())
                )
                .size(12)
                .color(palette.text_muted),
                horizontal_space(),
                text(
                    routine
                        .created
                        .clone()
                        .filter(|v| !v.is_empty())
                        .unwrap_or_else(|| "-".into())
                )
                .size(12)
                .color(palette.text_muted),
            ]
            .spacing(12),
        ]
        .spacing(4),
    )
    .padding(16)
    .style(move |_| card_style(palette))
    .into()
}
