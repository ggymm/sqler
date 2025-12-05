use gpui::{prelude::*, *};
use gpui_component::{
    Sizable, Size,
    form::{Form, field},
    input::{Input, InputState},
};

use crate::model::MySQLOptions;

pub struct MySQLCreate {
    pub host: Entity<InputState>,
    pub port: Entity<InputState>,
    pub username: Entity<InputState>,
    pub password: Entity<InputState>,
    pub database: Entity<InputState>,
}

impl MySQLCreate {
    pub fn new(
        opts: Option<&MySQLOptions>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let opts = opts.cloned().unwrap_or_default();

        Self {
            host: cx.new(|cx| InputState::new(window, cx).default_value(&opts.host)),
            port: cx.new(|cx| InputState::new(window, cx).default_value(&opts.port.to_string())),
            username: cx.new(|cx| InputState::new(window, cx).default_value(&opts.username)),
            password: cx.new(|cx| InputState::new(window, cx).default_value(&opts.password).masked(true)),
            database: cx.new(|cx| InputState::new(window, cx).default_value(&opts.database)),
        }
    }

    pub fn options(
        &self,
        cx: &App,
    ) -> MySQLOptions {
        MySQLOptions {
            host: self.host.read(cx).value().to_string(),
            port: self.port.read(cx).value().to_string(),
            username: self.username.read(cx).value().to_string(),
            password: self.password.read(cx).value().to_string(),
            database: self.database.read(cx).value().to_string(),
            use_tls: false,
        }
    }
}

impl Render for MySQLCreate {
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
