use gpui::{div, px, Context, ParentElement, Styled, Window};
use gpui::{AppContext as _, Entity};
use gpui_component::{
    form::{form_field, v_form},
    input::InputState,
    input::TextInput,
    v_flex, ActiveTheme as _,
};

use crate::app::{create::CreateWindow, SqlerApp};

#[derive(Clone)]
pub struct PostgresState {
    pub name: Entity<InputState>,
    pub host: Entity<InputState>,
    pub port: Entity<InputState>,
    pub username: Entity<InputState>,
    pub password: Entity<InputState>,
    pub database: Entity<InputState>,
    pub schema: Entity<InputState>,
}

impl PostgresState {
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
                    .placeholder("端口，例如：5432")
                    .default_value("5432")
            }),
            username: cx.new(|cx| {
                InputState::new(window, cx)
                    .placeholder("用户名，例如：postgres")
                    .default_value("postgres")
            }),
            password: cx.new(|cx| InputState::new(window, cx).placeholder("密码").masked(true)),
            database: cx.new(|cx| InputState::new(window, cx).placeholder("数据库名称，例如：sqler")),
            schema: cx.new(|cx| InputState::new(window, cx).placeholder("Schema，可选")),
        }
    }
}

pub fn render(
    state: &mut PostgresState,
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
                .child(form_field().label("数据库").child(TextInput::new(&state.database)))
                .child(form_field().label("Schema").child(TextInput::new(&state.schema))),
        )
        .child(
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("提示：Schema 留空时将使用默认 search_path"),
        )
}
