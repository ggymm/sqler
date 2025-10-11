use iced::widget::{column, container, horizontal_space, row, scrollable, text, vertical_space};
use iced::{Alignment, Element};

use crate::app::{Message, Palette};

use super::{
    card_style, generic_toolbar_button, load_state_list_view, stack_section, LoadState, LoadStateMessages, MysqlUser,
};

pub(super) fn view(
    state: Option<&LoadState<Vec<MysqlUser>>>,
    palette: Palette,
) -> Element<'static, Message> {
    let actions = row![
        generic_toolbar_button("新增用户", Message::CreateUser, palette),
        generic_toolbar_button("编辑用户", Message::EditUser, palette),
        generic_toolbar_button("删除用户", Message::DeleteUser, palette),
        generic_toolbar_button("权限管理", Message::ManageUserPrivileges, palette),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let content = users_content(state, palette);
    stack_section(actions.into(), content)
}

fn users_content(
    state: Option<&LoadState<Vec<MysqlUser>>>,
    palette: Palette,
) -> Element<'static, Message> {
    load_state_list_view(
        state,
        palette,
        LoadStateMessages {
            loading: "正在加载数据库用户…",
            empty: "未找到任何 MySQL 用户。",
            idle: "请激活连接以查看用户列表。",
        },
        |users, palette| {
            let mut list = column![];
            for user in users {
                list = list.push(user_row(user, palette));
            }
            scrollable(list.spacing(12)).into()
        },
    )
}

fn user_row(
    user: &MysqlUser,
    palette: Palette,
) -> Element<'static, Message> {
    container(
        column![
            row![
                text(format!("{}@{}", user.name, user.host))
                    .size(15)
                    .color(palette.text),
                horizontal_space(),
                text(user.plugin.clone().unwrap_or_else(|| "-".into()))
                    .size(12)
                    .color(palette.text_muted),
            ]
            .align_y(Alignment::Center),
            vertical_space().height(6),
            row![
                text(
                    user.locked
                        .clone()
                        .filter(|v| !v.is_empty())
                        .map(|v| format!("状态：{}", v))
                        .unwrap_or_else(|| "状态：未知".into())
                )
                .size(12)
                .color(palette.text_muted),
                horizontal_space(),
                text(
                    user.password_changed
                        .clone()
                        .filter(|v| !v.is_empty())
                        .map(|v| format!("密码更新：{}", v))
                        .unwrap_or_else(|| "密码更新：未知".into())
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
