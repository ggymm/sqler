use gpui::*;
use gpui_component::table::{Column, TableDelegate, TableState};

pub struct DataTable {
    cols: Vec<Column>,
    rows: Vec<Vec<SharedString>>,
    loading: bool,
}

impl DataTable {
    fn build_cols(
        cols: &[SharedString],
        rows: &[Vec<SharedString>],
    ) -> Vec<Column> {
        cols.iter()
            .enumerate()
            .map(|(col_ix, name)| {
                let col_len = name.chars().count();
                let row_len = rows
                    .iter()
                    .take(10)
                    .filter_map(|row| row.get(col_ix))
                    .map(|cell| cell.chars().count())
                    .max()
                    .unwrap_or(0);
                let max_len = col_len.max(row_len);
                let col_width = (max_len * 8 + 16).max(80).min(400) as f32;

                Column::new(name.clone(), name.clone()).width(px(col_width))
            })
            .collect()
    }

    pub fn new(
        cols: Vec<SharedString>,
        rows: Vec<Vec<SharedString>>,
    ) -> Self {
        Self {
            cols: Self::build_cols(&cols, &rows),
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
        self.cols = Self::build_cols(&cols, &rows);
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
        &self.cols[col_ix]
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
            .child(self.cols.get(col_ix).unwrap().name.clone())
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
