use gpui::{prelude::*, *};
use gpui_component::{
    Sizable, Size,
    form::{Form, field},
    input::{Input, InputState},
};

use crate::model::{MongoDBHost, MongoDBOptions};

pub struct MongoDBCreate {
    pub host: Entity<InputState>,
    pub port: Entity<InputState>,
    pub username: Entity<InputState>,
    pub password: Entity<InputState>,
    pub database: Entity<InputState>,
}

impl MongoDBCreate {
    pub fn new(
        opts: Option<&MongoDBOptions>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let opts = opts.cloned().unwrap_or_default();
        let first_host = opts.hosts.first().cloned().unwrap_or_default();

        Self {
            host: cx.new(|cx| InputState::new(window, cx).default_value(&first_host.host)),
            port: cx.new(|cx| InputState::new(window, cx).default_value(&first_host.port.to_string())),
            username: cx.new(|cx| InputState::new(window, cx).default_value(&opts.username.unwrap_or_default())),
            password: cx.new(|cx| {
                InputState::new(window, cx)
                    .default_value(&opts.password.unwrap_or_default())
                    .masked(true)
            }),
            database: cx.new(|cx| InputState::new(window, cx)),
        }
    }

    pub fn options(
        &self,
        cx: &App,
    ) -> MongoDBOptions {
        let host = self.host.read(cx).value().to_string();
        let port = self.port.read(cx).value().to_string();
        let username = self.username.read(cx).value().to_string();
        let password = self.password.read(cx).value().to_string();

        MongoDBOptions {
            connection_string: None,
            hosts: vec![MongoDBHost {
                host,
                port: port.parse().unwrap_or(27017),
            }],
            replica_set: None,
            auth_source: None,
            username: if username.is_empty() { None } else { Some(username) },
            password: if password.is_empty() { None } else { Some(password) },
            use_tls: false,
        }
    }
}

impl Render for MongoDBCreate {
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
