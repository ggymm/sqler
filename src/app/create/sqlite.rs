use gpui::{div, px, Context, ParentElement, Styled, Window};
use gpui::{AppContext as _, Entity};
use gpui_component::{
    form::{form_field, v_form},
    input::InputState,
    input::TextInput,
    v_flex, ActiveTheme as _,
};

use crate::app::{create::CreateDataSourceWindow, SqlerApp};

#[derive(Clone)]
pub struct SqliteState {
    pub name: Entity<InputState>,
    pub file_path: Entity<InputState>,
    pub password: Entity<InputState>,
}

impl SqliteState {
    pub fn new(
        window: &mut Window,
        cx: &mut Context<SqlerApp>,
    ) -> Self {
        Self {
            name: cx.new(|cx| InputState::new(window, cx).placeholder("数据源名称")),
            file_path: cx.new(|cx| InputState::new(window, cx).placeholder("数据库文件路径，例如：/data/db.sqlite")),
            password: cx.new(|cx| InputState::new(window, cx).placeholder("密码，可选").masked(true)),
        }
    }
}

pub fn render(
    state: &mut SqliteState,
    cx: &mut Context<CreateDataSourceWindow>,
) -> gpui::Div {
    v_flex()
        .gap(px(12.))
        .child(
            v_form()
                .gap(px(12.))
                .child(form_field().label("数据源名称").child(TextInput::new(&state.name)))
                .child(form_field().label("文件路径").child(TextInput::new(&state.file_path)))
                .child(
                    form_field()
                        .label("密码")
                        .child(TextInput::new(&state.password).mask_toggle()),
                ),
        )
        .child(
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("提示：路径支持相对/绝对地址，请确保应用具备读写权限"),
        )
}
