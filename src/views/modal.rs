use gpui::{div, px, AnyElement, Context, IntoElement, ParentElement, Styled, Window};
use gpui_component::{
    button::{Button, ButtonVariants as _},
    form::form_field,
    h_flex,
    input::TextInput,
    v_flex,
    ActiveTheme as _,
    StyledExt,
};

use super::{NewDataSourceState, SqlerApp};

pub(super) fn render_new_data_source_modal(
    state: &mut NewDataSourceState,
    _window: &mut Window,
    cx: &mut Context<SqlerApp>,
) -> AnyElement {
    let header = h_flex()
        .justify_between()
        .items_center()
        .child(div().text_lg().font_semibold().child("新建数据源"))
        .child(
            Button::new("modal-close")
                .text()
                .label("×")
                .on_click(cx.listener(|this, _, _, cx| {
                    this.hide_new_data_source_modal(cx);
                })),
        );

    let content = v_flex()
        .gap(px(12.))
        .child(
            form_field()
                .label("数据源名称")
                .child(TextInput::new(&state.form.name)),
        )
        .child(
            form_field()
                .label("主机")
                .child(TextInput::new(&state.form.host)),
        )
        .child(
            form_field()
                .label("端口")
                .child(TextInput::new(&state.form.port)),
        )
        .child(
            form_field()
                .label("用户名")
                .child(TextInput::new(&state.form.username)),
        )
        .child(
            form_field()
                .label("密码")
                .child(TextInput::new(&state.form.password).mask_toggle()),
        )
        .child(
            form_field()
                .label("数据库")
                .child(TextInput::new(&state.form.database)),
        )
        .child(
            form_field()
                .label("Schema")
                .child(TextInput::new(&state.form.schema)),
        );

    let footer = h_flex()
        .justify_end()
        .gap(px(8.))
        .child(
            Button::new("modal-cancel")
                .ghost()
                .label("取消")
                .on_click(cx.listener(|this, _, _, cx| {
                    this.hide_new_data_source_modal(cx);
                })),
        )
        .child(
            Button::new("modal-save")
                .primary()
                .label("保存")
                .on_click(cx.listener(|this, _, _, cx| {
                    this.submit_new_data_source_modal(cx);
                })),
        );

    div()
        .absolute()
        .inset_0()
        .flex()
        .items_center()
        .justify_center()
        .bg(cx.theme().overlay)
        .child(
            v_flex()
                .w(px(560.))
                .gap(px(20.))
                .bg(cx.theme().background)
                .rounded(cx.theme().radius_lg)
                .shadow_lg()
                .p(px(24.))
                .child(header)
                .child(content)
                .child(footer),
        )
        .into_any_element()
}
