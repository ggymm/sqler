use crate::app::SqlerApp;
use gpui::{div, px, AppContext, Axis, Context, Div, Entity, ParentElement, Styled, Window};
use gpui_component::form::Form;
use gpui_component::{
    form::form_field,
    input::{InputState, TextInput},
};

#[derive(Clone)]
pub struct RedisState {
    pub name: Entity<InputState>,
    pub host: Entity<InputState>,
    pub port: Entity<InputState>,
    pub username: Entity<InputState>,
    pub password: Entity<InputState>,
}

impl RedisState {
    pub fn new(
        window: &mut Window,
        cx: &mut Context<SqlerApp>,
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

pub fn render(state: &mut RedisState) -> Div {
    div().flex().flex_col().gap_4().child(
        Form::vertical()
            .layout(Axis::Horizontal)
            .label_width(px(80.))
            .child(
                form_field()
                    .label("数据源名称")
                    .child(TextInput::new(&state.name).cleanable()),
            )
            .child(
                form_field()
                    .label("主机")
                    .child(TextInput::new(&state.host).cleanable()),
            )
            .child(
                form_field()
                    .label("端口")
                    .child(TextInput::new(&state.port).cleanable()),
            )
            .child(
                form_field()
                    .label("用户名")
                    .child(TextInput::new(&state.username).cleanable()),
            )
            .child(
                form_field()
                    .label("密码")
                    .child(TextInput::new(&state.password).mask_toggle().cleanable()),
            ),
    )
}
