use gpui::*;
use gpui_component::table::{Column, TableDelegate, TableState};

pub struct DataTable {
    col_defs: Vec<Column>,
    cols: Vec<SharedString>,
    rows: Vec<Vec<SharedString>>,
    loading: bool,
}

impl DataTable {
    pub fn new(
        cols: Vec<SharedString>,
        rows: Vec<Vec<SharedString>>,
    ) -> Self {
        let col_defs = cols
            .iter()
            .map(|name| Column::new(name.clone(), name.clone()))
            .collect();
        Self {
            col_defs,
            cols,
            rows,
            loading: false,
        }
    }

    pub fn build(
        self,
        window: &mut Window,
        cx: &mut App,
    ) -> Entity<TableState<Self>> {
        cx.new(|cx| {
            TableState::new(self, window, cx)
                .sortable(false)
                .col_movable(true)
                .col_resizable(true)
                .col_selectable(true)
                .row_selectable(true)
                .loop_selection(true)
        })
    }

    pub fn update_data(
        &mut self,
        cols: Vec<SharedString>,
        rows: Vec<Vec<SharedString>>,
    ) {
        self.col_defs = cols
            .iter()
            .map(|name| Column::new(name.clone(), name.clone()))
            .collect();
        self.cols = cols;
        self.rows = rows;
    }

    pub fn update_loading(
        &mut self,
        loading: bool,
    ) {
        self.loading = loading;
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
        _cx: &mut App,
    ) -> impl IntoElement {
        div()
            .size_full()
            .text_sm()
            .child(self.cols.get(col_ix).cloned().unwrap_or_default())
    }

    fn render_td(
        &self,
        row_ix: usize,
        col_ix: usize,
        _window: &mut Window,
        _cx: &mut App,
    ) -> impl IntoElement {
        let value = self
            .rows
            .get(row_ix)
            .and_then(|row| row.get(col_ix))
            .cloned()
            .unwrap_or_default();
        div().size_full().text_sm().child(value)
    }

    fn loading(
        &self,
        _cx: &App,
    ) -> bool {
        self.loading
    }
}
