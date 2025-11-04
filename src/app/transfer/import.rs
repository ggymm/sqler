use gpui::{prelude::*, *};
use gpui_component::{
    form::{form_field, Form},
    input::{InputState, TextInput},
    Sizable, Size,
};

use super::TransferFormat;

pub struct ImportConfig {
    pub file_path: Entity<InputState>,
    pub table_name: Entity<InputState>,
}

impl ImportConfig {
    pub fn new(
        format: TransferFormat,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let _ = format;
        Self {
            file_path: cx.new(|cx| InputState::new(window, cx)),
            table_name: cx.new(|cx| InputState::new(window, cx)),
        }
    }
}

impl Render for ImportConfig {
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
                .child(
                    form_field()
                        .label("File Path")
                        .child(TextInput::new(&self.file_path).cleanable()),
                )
                .child(
                    form_field()
                        .label("Table Name")
                        .child(TextInput::new(&self.table_name).cleanable()),
                ),
        )
    }
}
