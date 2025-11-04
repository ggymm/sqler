use gpui::{prelude::*, *};
use gpui_component::{button::Button, ActiveTheme, StyledExt};

use crate::{
    app::{comps::DivExt, SqlerApp},
    option::DataSource,
};

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
    parent: WeakEntity<SqlerApp>,

    import_config: Entity<import::ImportConfig>,
    output_config: Entity<output::OutputConfig>,
}

impl TransferWindow {
    pub fn new(
        datasource: DataSource,
        tables: Vec<SharedString>,
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
            parent,
            import_config: cx.new(|cx| import::ImportConfig::new(datasource, tables, window, cx)),
            output_config: cx.new(|cx| output::OutputConfig::new(window, cx)),
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
            cx.notify();
        }
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
            .child(div().flex_1().relative().overflow_hidden().child(match transfer_type {
                None => self.render_type_selection(&theme, cx).into_any_element(),
                Some(TransferType::Import) => self.import_config.clone().into_any_element(),
                Some(TransferType::Export) => self.output_config.clone().into_any_element(),
            }))
            .child(self.render_footer(&theme, transfer_type, cx))
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

    fn render_footer(
        &self,
        theme: &gpui_component::theme::Theme,
        transfer_type: Option<TransferType>,
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
                            this.deselect_type(cx);
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
    }
}
