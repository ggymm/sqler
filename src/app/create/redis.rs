use gpui::{prelude::*, *};
use gpui_component::{
    form::{form_field, Form},
    input::{InputState, TextInput},
};

pub struct RedisCreate {
    pub name: Entity<InputState>,
    pub host: Entity<InputState>,
    pub port: Entity<InputState>,
    pub username: Entity<InputState>,
    pub password: Entity<InputState>,
}

impl RedisCreate {
    pub fn new(
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        Self {
            name: cx.new(|cx| InputState::new(window, cx).default_value("Redis数据源")),
            host: cx.new(|cx| InputState::new(window, cx).default_value("127.0.0.1")),
            port: cx.new(|cx| InputState::new(window, cx).default_value("6379")),
            username: cx.new(|cx| InputState::new(window, cx).default_value("default")),
            password: cx.new(|cx| InputState::new(window, cx).masked(true)),
        }
    }
}

impl Render for RedisCreate {
    fn render(
        &mut self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div().flex().flex_col().gap_4().child(
            Form::vertical()
                .layout(Axis::Horizontal)
                .label_width(px(80.))
                .child(form_field().label("名称").child(TextInput::new(&self.name).cleanable()))
                .child(form_field().label("主机").child(TextInput::new(&self.host).cleanable()))
                .child(form_field().label("端口").child(TextInput::new(&self.port).cleanable()))
                .child(
                    form_field()
                        .label("账号")
                        .child(TextInput::new(&self.username).cleanable()),
                )
                .child(
                    form_field()
                        .label("密码")
                        .child(TextInput::new(&self.password).mask_toggle().cleanable()),
                ),
        )
    }
}
