use gpui::{prelude::*, *};
use gpui_component::{
    form::{field, Form},
    input::{Input, InputState},
    select::{Select, SelectState},
    IndexPath, Sizable, Size,
};

use crate::model::RedisOptions;

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
    pub auth: Entity<SelectState<Vec<SharedString>>>,
    pub username: Entity<InputState>,
    pub password: Entity<InputState>,
}

impl RedisCreate {
    pub fn new(
        name: Option<&str>,
        opts: Option<&RedisOptions>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let auths: Vec<SharedString> = RedisAuthMode::all().iter().map(|mode| mode.label().into()).collect();
        let opts = opts.cloned().unwrap_or_default();
        let name_val = name.unwrap_or("Redis数据源").to_string();

        Self {
            name: cx.new(|cx| InputState::new(window, cx).default_value(&name_val)),
            host: cx.new(|cx| InputState::new(window, cx).default_value(&opts.host)),
            port: cx.new(|cx| InputState::new(window, cx).default_value(&opts.port)),
            auth: cx.new(|cx| SelectState::new(auths, Some(IndexPath::new(0)), window, cx)),
            username: cx.new(|cx| InputState::new(window, cx).default_value(&opts.username.unwrap_or_default())),
            password: cx.new(|cx| {
                InputState::new(window, cx)
                    .default_value(&opts.password.unwrap_or_default())
                    .masked(true)
            }),
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
            port,
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
                .child(field().label("名称").child(Input::new(&self.name).cleanable(true)))
                .child(field().label("主机").child(Input::new(&self.host).cleanable(true)))
                .child(field().label("端口").child(Input::new(&self.port).cleanable(true)))
                .child(field().label("认证模式").child(Select::new(&self.auth)))
                .when(show_username, |form| {
                    form.child(field().label("账号").child(Input::new(&self.username).cleanable(true)))
                })
                .when(show_username || show_password, |form| {
                    form.child(
                        field()
                            .label("密码")
                            .child(Input::new(&self.password).mask_toggle().cleanable(true)),
                    )
                }),
        )
    }
}
