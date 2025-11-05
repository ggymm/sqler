use gpui::{prelude::*, *};
use gpui_component::{
    dropdown::{Dropdown, DropdownState},
    form::{form_field, Form},
    input::{InputState, TextInput},
    IndexPath, Sizable, Size,
};

use crate::driver::RedisOptions;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RedisAuthMode {
    None,
    Password,
    UsernamePassword,
}

impl RedisAuthMode {
    fn all() -> &'static [RedisAuthMode] {
        &[
            RedisAuthMode::None,
            RedisAuthMode::Password,
            RedisAuthMode::UsernamePassword,
        ]
    }

    fn label(&self) -> &'static str {
        match self {
            RedisAuthMode::None => "无认证",
            RedisAuthMode::Password => "密码认证",
            RedisAuthMode::UsernamePassword => "账号密码认证",
        }
    }
}

pub struct RedisCreate {
    pub name: Entity<InputState>,
    pub host: Entity<InputState>,
    pub port: Entity<InputState>,
    pub auth: Entity<DropdownState<Vec<SharedString>>>,
    pub username: Entity<InputState>,
    pub password: Entity<InputState>,
}

impl RedisCreate {
    pub fn new(
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let auths: Vec<SharedString> = RedisAuthMode::all().iter().map(|mode| mode.label().into()).collect();

        Self {
            name: cx.new(|cx| InputState::new(window, cx).default_value("Redis数据源")),
            host: cx.new(|cx| InputState::new(window, cx).default_value("127.0.0.1")),
            port: cx.new(|cx| InputState::new(window, cx).default_value("6379")),
            auth: cx.new(|cx| DropdownState::new(auths, Some(IndexPath::new(0)), window, cx)),
            username: cx.new(|cx| InputState::new(window, cx)),
            password: cx.new(|cx| InputState::new(window, cx).masked(true)),
        }
    }

    pub fn options(
        &self,
        cx: &App,
    ) -> RedisOptions {
        let host = self.host.read(cx).value().to_string();
        let port = self.port.read(cx).value().to_string();
        let username = self.username.read(cx).value().to_string();
        let password = self.password.read(cx).value().to_string();

        RedisOptions {
            host,
            port: port.parse().unwrap_or(6379),
            username: if username.is_empty() { None } else { Some(username) },
            password: if password.is_empty() { None } else { Some(password) },
            use_tls: false,
        }
    }
}

impl Render for RedisCreate {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let auth = self.auth.read(cx).selected_value();

        let show_password = auth
            .as_ref()
            .map(|s| s.as_ref() == RedisAuthMode::Password.label())
            .unwrap_or(false);

        let show_username = auth
            .as_ref()
            .map(|s| s.as_ref() == RedisAuthMode::UsernamePassword.label())
            .unwrap_or(false);

        div().flex().flex_col().gap_4().child(
            Form::vertical()
                .layout(Axis::Horizontal)
                .with_size(Size::Large)
                .label_width(px(80.))
                .child(form_field().label("名称").child(TextInput::new(&self.name).cleanable()))
                .child(form_field().label("主机").child(TextInput::new(&self.host).cleanable()))
                .child(form_field().label("端口").child(TextInput::new(&self.port).cleanable()))
                .child(form_field().label("认证模式").child(Dropdown::new(&self.auth)))
                .when(show_username, |form| {
                    form.child(
                        form_field()
                            .label("账号")
                            .child(TextInput::new(&self.username).cleanable()),
                    )
                })
                .when(show_username || show_password, |form| {
                    form.child(
                        form_field()
                            .label("密码")
                            .child(TextInput::new(&self.password).mask_toggle().cleanable()),
                    )
                }),
        )
    }
}
