use gpui::{prelude::*, *};
use gpui_component::{
    form::{field, Form},
    input::{Input, InputState},
    Sizable, Size,
};

use crate::model::PostgresOptions;

pub struct PostgresCreate {
    pub name: Entity<InputState>,
    pub host: Entity<InputState>,
    pub port: Entity<InputState>,
    pub username: Entity<InputState>,
    pub password: Entity<InputState>,
    pub database: Entity<InputState>,
}

impl PostgresCreate {
    pub fn new(
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        Self {
            name: cx.new(|cx| InputState::new(window, cx).default_value("PostgreSQL数据源")),
            host: cx.new(|cx| InputState::new(window, cx).default_value("127.0.0.1")),
            port: cx.new(|cx| InputState::new(window, cx).default_value("5432")),
            username: cx.new(|cx| InputState::new(window, cx).default_value("postgres")),
            password: cx.new(|cx| InputState::new(window, cx).masked(true)),
            database: cx.new(|cx| InputState::new(window, cx)),
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
            port: port.parse().unwrap_or(5432),
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
        div().flex().flex_col().gap_4().child(
            Form::vertical()
                .layout(Axis::Horizontal)
                .with_size(Size::Large)
                .label_width(px(80.))
                .child(field().label("名称").child(Input::new(&self.name).cleanable(true)))
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
                ),
        )
    }
}
