use gpui::{prelude::*, *};
use gpui_component::{
    form::{field, Form},
    input::{Input, InputState},
    Sizable, Size,
};

use crate::model::SQLServerOptions;

pub struct SQLServerCreate {
    pub name: Entity<InputState>,
    pub host: Entity<InputState>,
    pub port: Entity<InputState>,
    pub username: Entity<InputState>,
    pub password: Entity<InputState>,
    pub instance: Entity<InputState>,
    pub database: Entity<InputState>,
}

impl SQLServerCreate {
    pub fn new(
        name: Option<&str>,
        opts: Option<&SQLServerOptions>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let (name_val, host_val, port_val, username_val, password_val, instance_val, database_val) = match opts {
            Some(opts) => (
                name.unwrap_or("SQLServer数据源").to_string(),
                opts.host.clone(),
                opts.port.to_string(),
                opts.username.clone().unwrap_or_default(),
                opts.password.clone().unwrap_or_default(),
                opts.instance.clone().unwrap_or_default(),
                opts.database.clone(),
            ),
            None => (
                "SQLServer数据源".to_string(),
                "127.0.0.1".to_string(),
                "1433".to_string(),
                "sa".to_string(),
                String::new(),
                String::new(),
                String::new(),
            ),
        };

        Self {
            name: cx.new(|cx| InputState::new(window, cx).default_value(&name_val)),
            host: cx.new(|cx| InputState::new(window, cx).default_value(&host_val)),
            port: cx.new(|cx| InputState::new(window, cx).default_value(&port_val)),
            username: cx.new(|cx| InputState::new(window, cx).default_value(&username_val)),
            password: cx.new(|cx| InputState::new(window, cx).default_value(&password_val).masked(true)),
            instance: cx.new(|cx| InputState::new(window, cx).default_value(&instance_val)),
            database: cx.new(|cx| InputState::new(window, cx).default_value(&database_val)),
        }
    }
}

impl Render for SQLServerCreate {
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
                        .label("实例名")
                        .child(Input::new(&self.instance).cleanable(true)),
                )
                .child(
                    field()
                        .label("数据库")
                        .child(Input::new(&self.database).cleanable(true)),
                ),
        )
    }
}
