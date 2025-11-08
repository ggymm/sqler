use gpui::{prelude::*, *};
use gpui_component::{button::Button, ActiveTheme, StyledExt};

use crate::{
    app::{comps::DivExt, transfer::export::ExportTable, transfer::import::ImportTable, SqlerApp},
    model::DataSource,
};

mod export;
mod import;

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
    kind: Option<TransferType>,
    parent: WeakEntity<SqlerApp>,
    import: Entity<ImportTable>,
    output: Entity<ExportTable>,
}

impl TransferWindow {
    pub fn new(
        meta: DataSource,
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
            kind: None,
            parent,
            import: cx.new(|cx| ImportTable::new(meta, tables, window, cx)),
            output: cx.new(|cx| ExportTable::new(window, cx)),
        }
    }

    fn cancel(
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
}

impl Render for TransferWindow {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let kind = self.kind;

        let theme = cx.theme();
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
                    .child(div().text_xl().font_semibold().child(match kind {
                        None => "数据传输",
                        Some(TransferType::Import) => "数据导入",
                        Some(TransferType::Export) => "数据导出",
                    })),
            )
            .child(
                div().id("transfer-content").col_full().child(match kind {
                    None => div()
                        .p_6()
                        .gap_5()
                        .col_full()
                        .scrollable(Axis::Vertical)
                        .children(
                            TransferType::all()
                                .iter()
                                .map(|kind| {
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
                                        .id(("transfer-type-{}", *kind as u64))
                                        .hover(|this| this.bg(theme.list_hover))
                                        .child(
                                            div()
                                                .flex()
                                                .flex_1()
                                                .flex_col()
                                                .items_start()
                                                .justify_center()
                                                .child(div().text_base().font_semibold().child(kind.label()))
                                                .child(div().text_sm().child(kind.description())),
                                        )
                                        .on_click(cx.listener({
                                            move |this: &mut TransferWindow, _ev, _window, cx| {
                                                if this.kind != Some(*kind) {
                                                    this.kind = Some(*kind);
                                                    cx.notify();
                                                }
                                            }
                                        }))
                                        .into_any_element()
                                })
                                .collect::<Vec<_>>(),
                        )
                        .into_any_element(),
                    Some(TransferType::Import) => self.import.clone().into_any_element(),
                    Some(TransferType::Export) => self.output.clone().into_any_element(),
                }),
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
                    .when(kind.is_some(), |this| {
                        this.child(
                            Button::new("transfer-back")
                                .outline()
                                .label("上一步")
                                .on_click(cx.listener(|this: &mut TransferWindow, _ev, _window, cx| {
                                    if this.kind.take().is_some() {
                                        cx.notify();
                                    }
                                })),
                        )
                    })
                    .child(
                        Button::new("transfer-cancel")
                            .outline()
                            .label("取消")
                            .on_click(cx.listener(|this: &mut TransferWindow, _ev, window, cx| {
                                this.cancel(window, cx);
                            })),
                    ),
            )
    }
}
