use crate::app::create::CreateWindow;
use crate::app::SqlerApp;
use gpui::{div, px, AppContext, Context, Entity, ParentElement, Styled, Window};
use gpui_component::form::{form_field, v_form};
use gpui_component::input::{InputState, TextInput};
use gpui_component::{v_flex, ActiveTheme};

#[derive(Clone)]
pub struct MongoDBState {
    pub name: Entity<InputState>,
    pub host: Entity<InputState>,
    pub port: Entity<InputState>,
    pub username: Entity<InputState>,
    pub password: Entity<InputState>,
    pub database: Entity<InputState>,
}

impl MongoDBState {
    pub fn new(
        window: &mut Window,
        cx: &mut Context<SqlerApp>,
    ) -> Self {
        Self {
            name: cx.new(|cx| InputState::new(window, cx).placeholder("数据源名称")),
            host: cx.new(|cx| {
                InputState::new(window, cx)
                    .placeholder("主机地址，例如：127.0.0.1")
                    .default_value("127.0.0.1")
            }),
            port: cx.new(|cx| {
                InputState::new(window, cx)
                    .placeholder("端口，例如：3306")
                    .default_value("3306")
            }),
            username: cx.new(|cx| {
                InputState::new(window, cx)
                    .placeholder("用户名，例如：root")
                    .default_value("root")
            }),
            password: cx.new(|cx| InputState::new(window, cx).placeholder("密码").masked(true)),
            database: cx.new(|cx| InputState::new(window, cx).placeholder("数据库名称，例如：analytics")),
        }
    }
}

pub fn render(
    state: &mut MongoDBState,
    cx: &Context<CreateWindow>,
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
                .child(form_field().label("数据库").child(TextInput::new(&state.database))),
        )
        .child(
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("提示：若连接云数据库，请确认安全组已放行当前机器 IP"),
        )
}
