use gpui::{prelude::*, *};
use gpui_component::{button::Button, ActiveTheme, Disableable, StyledExt};

use crate::app::{comps::DivExt, SqlerApp};

mod import;
mod output;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransferType {
    Import,
    Export,
}

impl TransferType {
    pub fn all() -> &'static [TransferType] {
        &[TransferType::Import, TransferType::Export]
    }

    pub fn label(&self) -> &'static str {
        match self {
            TransferType::Import => "数据导入",
            TransferType::Export => "数据导出",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            TransferType::Import => "从文件导入数据到数据库",
            TransferType::Export => "将数据库数据导出到文件",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransferFormat {
    Csv,
    Json,
    Sql,
}

impl TransferFormat {
    pub fn all() -> &'static [TransferFormat] {
        &[TransferFormat::Csv, TransferFormat::Json, TransferFormat::Sql]
    }

    pub fn label(&self) -> &'static str {
        match self {
            TransferFormat::Csv => "CSV",
            TransferFormat::Json => "JSON",
            TransferFormat::Sql => "SQL",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            TransferFormat::Csv => "逗号分隔值文件，适用于表格数据",
            TransferFormat::Json => "JSON 格式文件，适用于结构化数据",
            TransferFormat::Sql => "SQL 脚本文件，包含完整的建表和插入语句",
        }
    }
}

pub struct TransferWindow {
    transfer_type: Option<TransferType>,
    format: Option<TransferFormat>,
    parent: WeakEntity<SqlerApp>,

    import_config: Entity<import::ImportConfig>,
    output_config: Entity<output::OutputConfig>,
}

impl TransferWindow {
    pub fn new(
        parent: WeakEntity<SqlerApp>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let parent_for_release = parent.clone();
        let _ = cx.on_release(move |_, app| {
            if let Some(parent) = parent_for_release.upgrade() {
                let _ = parent.update(app, |app, cx| {
                    app.close_transfer_window();
                    cx.notify();
                });
            }
        });

        Self {
            transfer_type: None,
            format: None,
            parent,
            import_config: cx.new(|cx| import::ImportConfig::new(TransferFormat::Csv, window, cx)),
            output_config: cx.new(|cx| output::OutputConfig::new(TransferFormat::Csv, window, cx)),
        }
    }

    fn close_window(
        &self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(parent) = self.parent.upgrade() {
            let _ = parent.update(cx, |app, cx| {
                app.close_transfer_window();
                cx.notify();
            });
        }
    }

    fn select_type(
        &mut self,
        transfer_type: TransferType,
        cx: &mut Context<Self>,
    ) {
        if self.transfer_type != Some(transfer_type) {
            self.transfer_type = Some(transfer_type);
            cx.notify();
        }
    }

    fn deselect_type(
        &mut self,
        cx: &mut Context<Self>,
    ) {
        if self.transfer_type.take().is_some() {
            self.format = None;
            cx.notify();
        }
    }

    fn select_format(
        &mut self,
        format: TransferFormat,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.format != Some(format) {
            self.format = Some(format);
            self.import_config = cx.new(|cx| import::ImportConfig::new(format, window, cx));
            self.output_config = cx.new(|cx| output::OutputConfig::new(format, window, cx));
            cx.notify();
        }
    }

    fn deselect_format(
        &mut self,
        cx: &mut Context<Self>,
    ) {
        if self.format.take().is_some() {
            cx.notify();
        }
    }

    fn confirm(
        &self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.close_window(_window, cx);
    }
}

impl Render for TransferWindow {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let theme = cx.theme().clone();
        let transfer_type = self.transfer_type;
        let format = self.format;

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(theme.background)
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
                    .child(div().text_lg().font_semibold().child("数据传输")),
            )
            .child(
                div()
                    .flex_1()
                    .relative()
                    .overflow_hidden()
                    .child(match (transfer_type, format) {
                        (None, _) => self.render_type_selection(&theme, cx).into_any_element(),
                        (Some(_), None) => self.render_format_selection(&theme, cx).into_any_element(),
                        (Some(TransferType::Import), Some(_)) => div()
                            .p_6()
                            .gap_5()
                            .col_full()
                            .scrollable(Axis::Vertical)
                            .child(self.import_config.clone())
                            .into_any_element(),
                        (Some(TransferType::Export), Some(_)) => div()
                            .p_6()
                            .gap_5()
                            .col_full()
                            .scrollable(Axis::Vertical)
                            .child(self.output_config.clone())
                            .into_any_element(),
                    }),
            )
            .child(self.render_footer(&theme, transfer_type, format, cx))
    }
}

impl TransferWindow {
    fn render_type_selection(
        &self,
        theme: &gpui_component::theme::Theme,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div().p_6().gap_5().col_full().scrollable(Axis::Vertical).children(
            TransferType::all()
                .iter()
                .map(|typ| {
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .p_4()
                        .gap_4()
                        .h_20()
                        .w_full()
                        .bg(theme.list)
                        .border_1()
                        .border_color(theme.border)
                        .rounded_lg()
                        .cursor_pointer()
                        .id(("transfer-type-{}", *typ as u64))
                        .hover(|this| this.bg(theme.list_hover))
                        .child(
                            div()
                                .flex()
                                .flex_1()
                                .flex_col()
                                .items_start()
                                .justify_center()
                                .child(div().text_base().font_semibold().child(typ.label()))
                                .child(div().text_sm().child(typ.description())),
                        )
                        .on_click(cx.listener({
                            let typ = *typ;
                            move |this: &mut TransferWindow, _ev, _window, cx| {
                                this.select_type(typ, cx);
                            }
                        }))
                        .into_any_element()
                })
                .collect::<Vec<_>>(),
        )
    }

    fn render_format_selection(
        &self,
        theme: &gpui_component::theme::Theme,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div().p_6().gap_5().col_full().scrollable(Axis::Vertical).children(
            TransferFormat::all()
                .iter()
                .map(|fmt| {
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .p_4()
                        .gap_4()
                        .h_20()
                        .w_full()
                        .bg(theme.list)
                        .border_1()
                        .border_color(theme.border)
                        .rounded_lg()
                        .cursor_pointer()
                        .id(("transfer-format-{}", *fmt as u64))
                        .hover(|this| this.bg(theme.list_hover))
                        .child(
                            div()
                                .flex()
                                .flex_1()
                                .flex_col()
                                .items_start()
                                .justify_center()
                                .child(div().text_base().font_semibold().child(fmt.label()))
                                .child(div().text_sm().child(fmt.description())),
                        )
                        .on_click(cx.listener({
                            let fmt = *fmt;
                            move |this: &mut TransferWindow, _ev, window, cx| {
                                this.select_format(fmt, window, cx);
                            }
                        }))
                        .into_any_element()
                })
                .collect::<Vec<_>>(),
        )
    }

    fn render_footer(
        &self,
        theme: &gpui_component::theme::Theme,
        transfer_type: Option<TransferType>,
        format: Option<TransferFormat>,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
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
            .when(transfer_type.is_some(), |this| {
                this.child(
                    Button::new("transfer-back")
                        .outline()
                        .label("上一步")
                        .on_click(cx.listener(|this: &mut TransferWindow, _ev, _window, cx| {
                            if this.format.is_some() {
                                this.deselect_format(cx);
                            } else {
                                this.deselect_type(cx);
                            }
                        })),
                )
            })
            .child(
                Button::new("transfer-cancel")
                    .outline()
                    .label("取消")
                    .on_click(cx.listener(|this: &mut TransferWindow, _ev, window, cx| {
                        this.close_window(window, cx);
                    })),
            )
            .child(
                Button::new("transfer-confirm")
                    .outline()
                    .label("确认")
                    .disabled(transfer_type.is_none() || format.is_none())
                    .on_click(cx.listener(|this: &mut TransferWindow, _ev, window, cx| {
                        this.confirm(window, cx);
                    })),
            )
    }
}
