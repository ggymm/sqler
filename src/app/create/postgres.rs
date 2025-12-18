use gpui::{prelude::*, *};
use gpui_component::{
    Sizable, Size,
    form::{Form, field},
    input::{Input, InputState},
};

use crate::model::PostgresOptions;

pub struct PostgresCreate {
    pub host: Entity<InputState>,
    pub port: Entity<InputState>,
    pub username: Entity<InputState>,
    pub password: Entity<InputState>,
    pub database: Entity<InputState>,
}

impl PostgresCreate {
    pub fn new(
        opts: Option<&PostgresOptions>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let opts = opts.cloned().unwrap_or_default();

        Self {
            host: cx.new(|cx| InputState::new(window, cx).default_value(&opts.host)),
            port: cx.new(|cx| InputState::new(window, cx).default_value(&opts.port)),
            username: cx.new(|cx| InputState::new(window, cx).default_value(&opts.username)),
            password: cx.new(|cx| InputState::new(window, cx).default_value(&opts.password).masked(true)),
            database: cx.new(|cx| InputState::new(window, cx).default_value(&opts.database)),
        }
    }

    pub fn options(
        &self,
        cx: &App,
    ) -> PostgresOptions {
        let host = self.host.read(cx).value().to_string();
        let port = self.port.read(cx).value().to_string();
        let username = self.username.read(cx).value().to_string();
        let password = self.password.read(cx).value().to_string();
        let database = self.database.read(cx).value().to_string();

        PostgresOptions {
            host,
            port,
            username,
            password,
            database,
            use_tls: false,
        }
    }
}

impl Render for PostgresCreate {
    fn render(
        &mut self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> impl IntoElement {
        Form::vertical()
            .layout(Axis::Horizontal)
            .with_size(Size::Large)
            .label_width(px(80.))
            .child(field().label("主机").child(Input::new(&self.host).cleanable(true)))
            .child(field().label("端口").child(Input::new(&self.port).cleanable(true)))
            .child(field().label("账号").child(Input::new(&self.username).cleanable(true)))
            .child(
                field()
                    .label("密码")
                    .child(Input::new(&self.password).mask_toggle().cleanable(true)),
            )
            .child(
                field()
                    .label("数据库")
                    .child(Input::new(&self.database).cleanable(true)),
            )
    }
}
