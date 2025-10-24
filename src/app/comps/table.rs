use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::input::{InputState, TextInput};
use gpui_component::table::{Column, ColumnSort, Table, TableDelegate};
use gpui_component::{ActiveTheme as _, Sizable, Size};

use super::comp_id;
use super::icon_search;

#[derive(Clone)]
pub struct StaticTableDelegate {
    columns: Vec<Column>,
    rows: Vec<Vec<SharedString>>,
    sort_state: Option<(usize, ColumnSort)>,
}

impl StaticTableDelegate {
    pub fn new(
        columns: Vec<Column>,
        rows: Vec<Vec<SharedString>>,
    ) -> Self {
        Self {
            columns,
            rows,
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
            .flex()
            .flex_row()
            .gap(px(12.))
            .h(px(36.))
            .id(("mysql-table-row", row_ix))
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
        div().text_sm().child(value)
    }
}

pub fn create_static_table<P>(
    window: &mut Window,
    cx: &mut Context<P>,
    columns: Vec<Column>,
    rows: Vec<Vec<SharedString>>,
) -> Entity<Table<StaticTableDelegate>> {
    let delegate = StaticTableDelegate::new(columns, rows);
    let table = cx.new(|cx| Table::new(delegate.clone(), window, cx));
    let _ = table.update(cx, |table, cx| {
        table.set_stripe(true, cx);
    });
    table
}

#[derive(Clone)]
pub struct DataTable {
    table: Entity<Table<StaticTableDelegate>>,
    search: Entity<InputState>,
}

impl DataTable {
    pub fn new<P>(
        window: &mut Window,
        cx: &mut Context<P>,
        columns: Vec<Column>,
        rows: Vec<Vec<SharedString>>,
    ) -> Self {
        let table = create_static_table(window, cx, columns, rows);
        let search = cx.new(|cx| InputState::new(window, cx).placeholder("搜索字段"));
        Self { table, search }
    }

    pub fn render<P>(
        &self,
        base_id: &str,
        cx: &mut Context<P>,
    ) -> Stateful<Div>
    where
        P: 'static,
    {
        let theme = cx.theme().clone();

        div()
            .id(comp_id(["data-table-container", base_id]))
            .flex()
            .flex_col()
            .size_full()
            .child(
                div()
                    .border_1()
                    .border_color(theme.border)
                    .rounded_lg()
                    .bg(theme.secondary)
                    .child(self.table.clone()),
            )
    }
}
