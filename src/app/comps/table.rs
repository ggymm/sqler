use gpui::*;
use gpui_component::table::Column;
use gpui_component::table::Table;
use gpui_component::table::TableDelegate;
use gpui_component::Sizable;
use gpui_component::Size;

pub struct DataTable {
    col_defs: Vec<Column>,
    cols: Vec<SharedString>,
    rows: Vec<Vec<SharedString>>,
}

impl DataTable {
    pub fn new(
        cols: Vec<SharedString>,
        rows: Vec<Vec<SharedString>>,
    ) -> Self {
        let col_defs = cols.iter().map(|name| Column::new(name.to_string(), "")).collect();
        Self { col_defs, cols, rows }
    }

    pub fn build(
        self,
        window: &mut Window,
        cx: &mut App,
    ) -> Entity<Table<Self>> {
        cx.new(|cx| {
            Table::new(self, window, cx)
                .with_size(Size::Small)
                .border(true)
                .stripe(false)
                .sortable(false)
                .col_movable(true)
                .col_resizable(true)
                .col_selectable(true)
                .row_selectable(true)
                .loop_selection(true)
                .scrollbar_visible(true, true)
        })
    }

    pub fn columns(&self) -> &[SharedString] {
        &self.cols
    }

    pub fn update_data(
        &mut self,
        cols: Vec<SharedString>,
        rows: Vec<Vec<SharedString>>,
    ) {
        self.col_defs = cols.iter().map(|name| Column::new(name.to_string(), "")).collect();
        self.cols = cols;
        self.rows = rows;
    }
}

impl TableDelegate for DataTable {
    fn columns_count(
        &self,
        _cx: &App,
    ) -> usize {
        self.cols.len()
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
        &self.col_defs[col_ix]
    }

    fn render_th(
        &self,
        col_ix: usize,
        _window: &mut Window,
        _cx: &mut Context<Table<Self>>,
    ) -> impl IntoElement {
        div()
            .size_full()
            .child(self.cols.get(col_ix).cloned().unwrap_or_default())
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
        div().size_full().child(value)
    }
}
