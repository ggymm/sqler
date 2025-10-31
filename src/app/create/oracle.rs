use gpui::{prelude::*, *};
use gpui_component::{
    form::{form_field, v_form},
    input::{InputState, TextInput},
    v_flex, ActiveTheme,
};

pub struct OracleCreate {
    pub name: Entity<InputState>,
    pub host: Entity<InputState>,
    pub port: Entity<InputState>,
    pub username: Entity<InputState>,
    pub password: Entity<InputState>,
    pub database: Entity<InputState>,
}

impl OracleCreate {
    pub fn new(
        window: &mut Window,
        cx: &mut Context<Self>,
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

impl Render for OracleCreate {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        v_flex()
            .gap(px(12.))
            .child(
                v_form()
                    .gap(px(12.))
                    .child(form_field().label("数据源名称").child(TextInput::new(&self.name)))
                    .child(form_field().label("主机").child(TextInput::new(&self.host)))
                    .child(form_field().label("端口").child(TextInput::new(&self.port)))
                    .child(form_field().label("用户名").child(TextInput::new(&self.username)))
                    .child(
                        form_field()
                            .label("密码")
                            .child(TextInput::new(&self.password).mask_toggle()),
                    )
                    .child(form_field().label("数据库").child(TextInput::new(&self.database))),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child("提示：若连接云数据库，请确认安全组已放行当前机器 IP"),
            )
    }
}
