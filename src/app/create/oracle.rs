use gpui::{prelude::*, *};
use gpui_component::{
    Sizable, Size,
    form::{Form, field},
    input::{Input, InputState},
};

use crate::model::OracleOptions;

pub struct OracleCreate {
    pub host: Entity<InputState>,
    pub port: Entity<InputState>,
    pub username: Entity<InputState>,
    pub password: Entity<InputState>,
    pub database: Entity<InputState>,
}

impl OracleCreate {
    pub fn new(
        opts: Option<&OracleOptions>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let opts = opts.cloned().unwrap_or_default();

        Self {
            host: cx.new(|cx| InputState::new(window, cx).default_value(&opts.host)),
            port: cx.new(|cx| InputState::new(window, cx).default_value(&opts.port.to_string())),
            username: cx.new(|cx| InputState::new(window, cx).default_value(&opts.username)),
            password: cx.new(|cx| {
                InputState::new(window, cx)
                    .default_value(&opts.password.unwrap_or_default())
                    .masked(true)
            }),
            database: cx.new(|cx| InputState::new(window, cx).default_value(&opts.address.value().to_string())),
        }
    }
}

impl Render for OracleCreate {
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
