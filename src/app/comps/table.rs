use gpui::*;
use gpui_component::{scroll::ScrollbarAxis, ActiveTheme as _, StyledExt};

use crate::app::comps::comp_id;

pub type TableRow = Vec<SharedString>;

#[derive(Clone)]
pub struct TableColumn {
    pub title: SharedString,
    pub width: Option<Pixels>,
    pub align_right: bool,
}

impl TableColumn {
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            width: None,
            align_right: false,
        }
    }

    pub fn width(mut self, width: impl Into<Pixels>) -> Self {
        self.width = Some(width.into());
        self
    }

    pub fn text_right(mut self) -> Self {
        self.align_right = true;
        self
    }
}

#[derive(Clone)]
pub struct TableData {
    pub columns: Vec<TableColumn>,
    pub rows: Vec<TableRow>,
}

impl TableData {
    pub fn new(columns: Vec<TableColumn>, rows: Vec<TableRow>) -> Self {
        Self { columns, rows }
    }
}

#[derive(Clone)]
pub struct DataTable {
    data: TableData,
}

impl DataTable {
    pub fn new(data: TableData) -> Self {
        Self { data }
    }

    pub fn set_data(&mut self, data: TableData) {
        self.data = data;
    }

    pub fn render<P>(&self, base_id: &str, cx: &mut Context<P>) -> AnyElement {
        let theme = cx.theme();
        let header = self.render_header(base_id, &theme);
        let body = self.render_body(base_id, &theme);

        div()
            .id(comp_id(["data-table", base_id]))
            .flex()
            .flex_col()
            .flex_1()
            .size_full()
            .min_w_0()
            .min_h_0()
            .bg(theme.secondary)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .min_w(px(0.))
                    .child(header)
                    .child(body),
            )
            .scrollable(ScrollbarAxis::Horizontal)
            .into_any_element()
    }

    fn render_header(
        &self,
        base_id: &str,
        theme: &gpui_component::ThemeColor,
    ) -> Div {
        let header_height = px(32.);
        let padding = px(12.);

        self.data.columns.iter().enumerate().fold(
            div()
                .flex()
                .flex_row()
                .min_w_0()
                .bg(theme.table_head)
                .border_b_1()
                .border_color(theme.border),
            |row, (col_ix, column)| {
                let mut cell = div()
                    .min_h(header_height)
                    .flex()
                    .items_center()
                    .px(padding)
                    .text_color(theme.table_head_foreground)
                    .font_semibold();

                if let Some(width) = column.width {
                    cell = cell.w(width);
                } else {
                    cell = cell.flex_1();
                }

                if column.align_right {
                    cell = cell.justify_end().text_right();
                }

                row.child(
                    cell.child(column.title.clone()).id(comp_id([
                        "data-table-header-cell",
                        base_id,
                        col_ix.to_string().as_str(),
                    ])),
                )
            },
        )
    }

    fn render_body(
        &self,
        base_id: &str,
        theme: &gpui_component::ThemeColor,
    ) -> Div {
        let row_height = px(32.);
        let padding = px(12.);

        let rows = self.data.rows.iter().enumerate().fold(
            div().flex().flex_col().gap(px(0.)),
            |body, (row_ix, row)| {
                let row_div = self
                    .data
                    .columns
                    .iter()
                    .enumerate()
                    .fold(div().flex().flex_row().min_w_0(), |row_div, (col_ix, column)| {
                        let mut cell = div()
                            .min_h(row_height)
                            .flex()
                            .items_center()
                            .px(padding)
                            .bg(theme.secondary)
                            .text_color(theme.foreground);

                        if let Some(width) = column.width {
                            cell = cell.w(width);
                        } else {
                            cell = cell.flex_1();
                        }

                        if column.align_right {
                            cell = cell.justify_end().text_right();
                        }

                        let value = row
                            .get(col_ix)
                            .cloned()
                            .unwrap_or_else(SharedString::default);

                        row_div.child(
                            cell.child(value).id(comp_id([
                                "data-table-cell",
                                base_id,
                                row_ix.to_string().as_str(),
                                col_ix.to_string().as_str(),
                            ])),
                        )
                    });

                body.child(
                    row_div.id(comp_id([
                        "data-table-row",
                        base_id,
                        row_ix.to_string().as_str(),
                    ])),
                )
            },
        );
        let vertical = rows.scrollable(ScrollbarAxis::Vertical);

        div()
            .flex()
            .flex_col()
            .flex_1()
            .min_h(px(0.))
            .child(vertical)
    }
}
