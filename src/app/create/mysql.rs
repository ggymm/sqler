use gpui::*;
use gpui_component::form::form_field;
use gpui_component::form::v_form;
use gpui_component::input::InputState;
use gpui_component::input::TextInput;
use gpui_component::{v_flex, ActiveTheme as _};

use crate::app::create::CreateDataSourceWindow;
use crate::app::SqlerApp;

#[derive(Clone)]
pub struct MySQLState {
    pub name: Entity<InputState>,
    pub host: Entity<InputState>,
    pub port: Entity<InputState>,
    pub username: Entity<InputState>,
    pub password: Entity<InputState>,
    pub database: Entity<InputState>,
}

impl MySQLState {
    pub fn new(
        window: &mut Window,
        cx: &mut Context<SqlerApp>,
    ) -> Self {
        Self {
            name: cx.new(|cx| InputState::new(window, cx).default_value("MySQL数据源")),
            host: cx.new(|cx| InputState::new(window, cx).default_value("localhost")),
            port: cx.new(|cx| InputState::new(window, cx).default_value("3306")),
            username: cx.new(|cx| InputState::new(window, cx).default_value("root")),
            password: cx.new(|cx| InputState::new(window, cx).masked(true)),
            database: cx.new(|cx| InputState::new(window, cx)),
        }
    }
}

pub fn render(
    state: &mut MySQLState,
    cx: &Context<CreateDataSourceWindow>,
) -> Div {
    div()
        .flex()
        .flex_col()
        .gap_4()
        .child(
            v_form()
                .layout(Axis::Horizontal)
                .child(form_field().label("名称").child(TextInput::new(&state.name)))
                .child(form_field().label("主机").child(TextInput::new(&state.host)))
                .child(form_field().label("端口").child(TextInput::new(&state.port)))
                .child(form_field().label("用户名").child(TextInput::new(&state.username)))
                .child(
                    form_field()
                        .label("密码")
                        .child(TextInput::new(&state.password).mask_toggle()),
                )
                .child(form_field().label("数据库").child(TextInput::new(&state.database)))
        )
        .child(
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("提示：若连接云数据库，请确认安全组已放行当前机器 IP"),
        )
}
