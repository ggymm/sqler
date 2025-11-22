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
        let opts = opts.cloned().unwrap_or_default();
        let name_val = name.unwrap_or("SQLServer数据源").to_string();

        Self {
            name: cx.new(|cx| InputState::new(window, cx).default_value(&name_val)),
            host: cx.new(|cx| InputState::new(window, cx).default_value(&opts.host)),
            port: cx.new(|cx| InputState::new(window, cx).default_value(&opts.port.to_string())),
            username: cx.new(|cx| InputState::new(window, cx).default_value(&opts.username.unwrap_or_default())),
            password: cx.new(|cx| {
                InputState::new(window, cx)
                    .default_value(&opts.password.unwrap_or_default())
                    .masked(true)
            }),
            instance: cx.new(|cx| InputState::new(window, cx).default_value(&opts.instance.unwrap_or_default())),
            database: cx.new(|cx| InputState::new(window, cx).default_value(&opts.database)),
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
