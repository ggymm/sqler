use gpui::*;
use gpui::TextAlign;
use gpui_component::table::{Column, ColumnSort, Table, TableDelegate};
use gpui_component::{ActiveTheme as _, Size, StyleSized};

use super::comp_id;

pub type TableRow = Vec<SharedString>;

#[derive(Clone)]
pub struct StaticTableDelegate {
    columns: Vec<Column>,
    rows: Vec<TableRow>,
    size: Size,
    sort_state: Option<(usize, ColumnSort)>,
}

impl StaticTableDelegate {
    pub fn new(
        columns: Vec<Column>,
        rows: Vec<TableRow>,
    ) -> Self {
        Self {
            columns,
            rows,
            size: Size::Small,
            sort_state: None,
        }
    }

    fn sort_rows(
        &mut self,
        col_ix: usize,
        sort: ColumnSort,
    ) {
        match sort {
            ColumnSort::Ascending => {
                self.rows.sort_by(|a, b| a.get(col_ix).cmp(&b.get(col_ix)));
            }
            ColumnSort::Descending => {
                self.rows.sort_by(|a, b| b.get(col_ix).cmp(&a.get(col_ix)));
            }
            ColumnSort::Default => {}
        }
        self.sort_state = Some((col_ix, sort));
    }
}

impl TableDelegate for StaticTableDelegate {
    fn columns_count(
        &self,
        _cx: &App,
    ) -> usize {
        self.columns.len()
    }

    fn rows_count(
        &self,
        _cx: &App,
    ) -> usize {
        self.rows.len()
    }

    fn column(
        &self,
        col_ix: usize,
        _cx: &App,
    ) -> &Column {
        &self.columns[col_ix]
    }

    fn perform_sort(
        &mut self,
        col_ix: usize,
        sort: ColumnSort,
        _window: &mut Window,
        _cx: &mut Context<Table<Self>>,
    ) {
        self.sort_rows(col_ix, sort);
    }

    fn render_tr(
        &self,
        row_ix: usize,
        _window: &mut Window,
        _cx: &mut Context<Table<Self>>,
    ) -> Stateful<Div> {
        div()
            .id(comp_id([format!("data-table-row-{}", row_ix)]))
            .flex()
            .flex_row()
            .h(self.size.table_row_height())
    }

    fn render_td(
        &self,
        row_ix: usize,
        col_ix: usize,
        _window: &mut Window,
        _cx: &mut Context<Table<Self>>,
    ) -> impl IntoElement {
        let value = self
            .rows
            .get(row_ix)
            .and_then(|row| row.get(col_ix))
            .cloned()
            .unwrap_or_default();

        let column = self.columns.get(col_ix);
        let mut cell = div()
            .table_cell_size(self.size)
            .flex()
            .flex_row()
            .items_center()
            .px_3()
            .overflow_hidden()
            .whitespace_nowrap()
            .child(value);

        if let Some(column) = column {
            if column.align == TextAlign::Right {
                cell = cell.justify_end().text_right();
            }
        }

        cell.id(comp_id([format!("data-table-cell-{}-{}", row_ix, col_ix)]))
    }
}

pub fn create_static_table<P>(
    window: &mut Window,
    cx: &mut Context<P>,
    columns: Vec<Column>,
    rows: Vec<TableRow>,
) -> Entity<Table<StaticTableDelegate>> {
    let delegate = StaticTableDelegate::new(columns, rows);
    let table = cx.new(|cx| Table::new(delegate, window, cx));
    let _ = table.update(cx, |table, cx| {
        table.set_stripe(true, cx);
        table.set_size(Size::Small, cx);
        table.delegate_mut().size = Size::Small;
    });
    table
}

#[derive(Clone)]
pub struct DataTable {
    table: Entity<Table<StaticTableDelegate>>,
}

impl DataTable {
    pub fn new<P>(
        window: &mut Window,
        cx: &mut Context<P>,
        columns: Vec<Column>,
        rows: Vec<TableRow>,
    ) -> Self {
        let table = create_static_table(window, cx, columns, rows);
        Self { table }
    }

    pub fn render<P>(
        &self,
        base_id: &str,
        cx: &mut Context<P>,
    ) -> Stateful<Div>
    where
        P: 'static,
    {
        let theme = cx.theme();

        div()
            .id(comp_id(["data-table-container", base_id]))
            .flex()
            .flex_col()
            .flex_1()
            .size_full()
            .min_w_0()
            .min_h_0()
            .bg(theme.secondary)
            .overflow_hidden()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_w_0()
                    .min_h_0()
                    .child(self.table.clone()),
            )
    }
}
