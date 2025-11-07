use gpui::{prelude::*, *};
use gpui_component::{
    form::{form_field, Form},
    input::{InputState, TextInput},
    ActiveTheme, Sizable, Size, StyledExt,
};

use crate::app::comps::DivExt;

use super::TransferFormat;

pub struct OutputConfig {
    format: Option<TransferFormat>,
    file_path: Entity<InputState>,
    table_name: Entity<InputState>,
}

impl OutputConfig {
    pub fn new(
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        Self {
            format: None,
            file_path: cx.new(|cx| InputState::new(window, cx)),
            table_name: cx.new(|cx| InputState::new(window, cx)),
        }
    }

    fn select_format(
        &mut self,
        format: TransferFormat,
        cx: &mut Context<Self>,
    ) {
        self.format = Some(format);
        cx.notify();
    }
}

impl Render for OutputConfig {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let theme = cx.theme().clone();
        let selected_format = self.format;

        div()
            .p_6()
            .gap_5()
            .col_full()
            .scrollable(Axis::Vertical)
            .child(
                Form::vertical()
                    .layout(Axis::Horizontal)
                    .with_size(Size::Large)
                    .label_width(px(100.))
                    .child(
                        form_field()
                            .label("源表名称")
                            .child(TextInput::new(&self.table_name).cleanable()),
                    )
                    .child(
                        form_field()
                            .label("文件路径")
                            .child(TextInput::new(&self.file_path).cleanable()),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .mt_4()
                    .child(div().text_base().font_semibold().child("选择导出格式")),
            )
            .children(
                TransferFormat::all()
                    .iter()
                    .map(|fmt| {
                        let is_selected = selected_format == Some(*fmt);
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .p_4()
                            .gap_4()
                            .w_full()
                            .bg(theme.list)
                            .border_1()
                            .when(is_selected, |this| this.border_color(theme.primary))
                            .when(!is_selected, |this| this.border_color(theme.border))
                            .rounded_lg()
                            .cursor_pointer()
                            .id(("export-format-{}", *fmt as u64))
                            .hover(|this| this.bg(theme.list_hover))
                            .child(
                                div()
                                    .flex()
                                    .flex_1()
                                    .flex_col()
                                    .items_start()
                                    .gap_1()
                                    .child(div().text_base().font_semibold().child(fmt.label()))
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(theme.muted_foreground)
                                            .child(fmt.description()),
                                    ),
                            )
                            .on_click(cx.listener({
                                let fmt = *fmt;
                                move |this: &mut OutputConfig, _ev, _window, cx| {
                                    this.select_format(fmt, cx);
                                }
                            }))
                            .into_any_element()
                    })
                    .collect::<Vec<_>>(),
            )
    }
}
