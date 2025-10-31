use gpui::*;
use gpui_component::{
    form::{form_field, Form},
    input::{InputState, TextInput},
};

use crate::app::SqlerApp;

#[derive(Clone)]
pub struct SqliteState {
    pub name: Entity<InputState>,
    pub filepath: Entity<InputState>,
    pub password: Entity<InputState>,
}

impl SqliteState {
    pub fn new(
        window: &mut Window,
        cx: &mut Context<SqlerApp>,
    ) -> Self {
        Self {
            name: cx.new(|cx| InputState::new(window, cx)),
            filepath: cx.new(|cx| InputState::new(window, cx)),
            password: cx.new(|cx| InputState::new(window, cx).masked(true)),
        }
    }
}

pub fn render(state: &mut SqliteState) -> Div {
    div().flex().flex_col().gap_4().child(
        Form::vertical()
            .layout(Axis::Horizontal)
            .label_width(px(80.))
            .child(
                form_field()
                    .label("名称")
                    .child(TextInput::new(&state.name).cleanable()),
            )
            .child(
                form_field()
                    .label("文件路径")
                    .child(TextInput::new(&state.filepath).cleanable()),
            )
            .child(
                form_field()
                    .label("密码")
                    .child(TextInput::new(&state.password).mask_toggle().cleanable()),
            ),
    )
}
