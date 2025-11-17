use gpui::{prelude::*, *};
use gpui_component::{
    button::Button,
    form::{field, Form},
    input::{Input, InputState},
    ActiveTheme, Sizable, Size, StyledExt,
};

use crate::app::{comps::DivExt, SqlerApp};

use super::TransferKind;

pub struct ExportWindow {
    parent: WeakEntity<SqlerApp>,
    format: Option<TransferKind>,
    file_path: Entity<InputState>,
    table_name: Entity<InputState>,
}

impl ExportWindow {
    pub fn new(
        parent: WeakEntity<SqlerApp>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let parent_for_release = parent.clone();
        let _ = cx.on_release(move |_, app| {
            if let Some(parent) = parent_for_release.upgrade() {
                let _ = parent.update(app, |app, cx| {
                    app.close_export_window();
                    cx.notify();
                });
            }
        });

        Self {
            parent,
            format: None,
            file_path: cx.new(|cx| InputState::new(window, cx)),
            table_name: cx.new(|cx| InputState::new(window, cx)),
        }
    }

    fn select_format(
        &mut self,
        format: TransferKind,
        cx: &mut Context<Self>,
    ) {
        self.format = Some(format);
        cx.notify();
    }

    fn render_form(
        &self,
        theme: &gpui_component::theme::Theme,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
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
                        field()
                            .label("源表名称")
                            .child(Input::new(&self.table_name).cleanable(true)),
                    )
                    .child(
                        field()
                            .label("文件路径")
                            .child(Input::new(&self.file_path).cleanable(true)),
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
                TransferKind::all()
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
                                move |this: &mut ExportWindow, _, _, cx| {
                                    this.select_format(fmt, cx);
                                }
                            }))
                            .into_any_element()
                    })
                    .collect::<Vec<_>>(),
            )
    }
}

impl Render for ExportWindow {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let theme = cx.theme().clone();

        div()
            .col_full()
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .px_8()
                    .py_5()
                    .bg(theme.secondary)
                    .border_b_1()
                    .border_color(theme.border)
                    .child(div().text_xl().font_semibold().child("数据导出")),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .size_full()
                    .child(div().flex_1().overflow_hidden().child(self.render_form(&theme, cx))),
            )
            .child(
                div()
                    .relative()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_end()
                    .px_8()
                    .py_5()
                    .gap_4()
                    .bg(theme.secondary)
                    .border_t_1()
                    .border_color(theme.border)
                    .child(
                        Button::new("transfer-cancel")
                            .outline()
                            .label("取消")
                            .on_click(cx.listener(|this: &mut ExportWindow, _, window, cx| {
                                if let Some(parent) = this.parent.upgrade() {
                                    let _ = parent.update(cx, |app, cx| {
                                        app.close_export_window();
                                        cx.notify();
                                    });
                                }
                                window.remove_window()
                            })),
                    ),
            )
    }
}
