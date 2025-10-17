use gpui::{div, px, Context, ParentElement, Styled, Window};
use gpui::{AppContext as _, Entity};
use gpui_component::{
    form::{form_field, v_form},
    input::InputState,
    input::TextInput,
    v_flex,
    ActiveTheme as _,
};

use crate::views::{create::CreateDataSourceWindow, SqlerApp};

#[derive(Clone)]
pub struct SqlServerState {
    pub name: Entity<InputState>,
    pub host: Entity<InputState>,
    pub port: Entity<InputState>,
    pub username: Entity<InputState>,
    pub password: Entity<InputState>,
    pub instance: Entity<InputState>,
    pub database: Entity<InputState>,
}

impl SqlServerState {
    pub fn new(window: &mut Window, cx: &mut Context<SqlerApp>) -> Self {
        Self {
            name: cx.new(|cx| InputState::new(window, cx).placeholder("数据源名称")),
            host: cx.new(|cx| {
                InputState::new(window, cx)
                    .placeholder("主机地址，例如：127.0.0.1")
                    .default_value("127.0.0.1")
            }),
            port: cx.new(|cx| {
                InputState::new(window, cx)
                    .placeholder("端口，例如：1433")
                    .default_value("1433")
            }),
            username: cx.new(|cx| InputState::new(window, cx).placeholder("用户名")),
            password: cx.new(|cx| InputState::new(window, cx).placeholder("密码").masked(true)),
            instance: cx.new(|cx| InputState::new(window, cx).placeholder("实例名，可选")),
            database: cx.new(|cx| InputState::new(window, cx).placeholder("数据库名称")),
        }
    }
}

pub fn render(
    state: &mut SqlServerState,
    cx: &mut Context<CreateDataSourceWindow>,
) -> gpui::Div {
    v_flex()
        .gap(px(12.))
        .child(
            v_form()
                .gap(px(12.))
                .child(form_field().label("数据源名称").child(TextInput::new(&state.name)))
                .child(form_field().label("主机").child(TextInput::new(&state.host)))
                .child(form_field().label("端口").child(TextInput::new(&state.port)))
                .child(form_field().label("用户名").child(TextInput::new(&state.username)))
                .child(
                    form_field()
                        .label("密码")
                        .child(TextInput::new(&state.password).mask_toggle()),
                )
                .child(form_field().label("实例名").child(TextInput::new(&state.instance)))
                .child(form_field().label("数据库").child(TextInput::new(&state.database))),
        )
        .child(
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("提示：如启用 Windows 身份验证，请在用户名字段输入 `domain\\user`"),
        )
}
