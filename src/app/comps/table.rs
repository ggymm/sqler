use gpui::*;
use gpui_component::table::{Column, Table, TableDelegate};

pub struct DataTable {
    headers: Vec<SharedString>,
    rows: Vec<Vec<SharedString>>,
}

impl DataTable {
    pub fn new(
        headers: Vec<SharedString>,
        rows: Vec<Vec<SharedString>>,
    ) -> Self {
        Self { headers, rows }
    }

    pub fn build(
        self,
        window: &mut Window,
        cx: &mut App,
    ) -> Entity<Table<Self>> {
        cx.new(|cx| {
            Table::new(self, window, cx)
                .border(true)
                .stripe(false)
                .sortable(false)
                .col_movable(true)
                .col_resizable(true)
                .loop_selection(true)
                .scrollbar_visible(true, true)
        })
    }

    pub fn update_data(
        &mut self,
        headers: Vec<SharedString>,
        rows: Vec<Vec<SharedString>>,
    ) {
        self.headers = headers;
        self.rows = rows;
    }
}

impl TableDelegate for DataTable {
    fn columns_count(
        &self,
        _cx: &App,
    ) -> usize {
        self.headers.len()
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
        static COLUMNS: std::sync::OnceLock<Vec<Column>> = std::sync::OnceLock::new();
        COLUMNS.get_or_init(|| {
            (0..100)
                .map(|i| Column::new(i.to_string(), "").width(px(100.)))
                .collect()
        });
        &COLUMNS.get().unwrap()[col_ix]
    }

    fn render_th(
        &self,
        col_ix: usize,
        _window: &mut Window,
        _cx: &mut Context<Table<Self>>,
    ) -> impl IntoElement {
        div()
            .size_full()
            .child(self.headers.get(col_ix).cloned().unwrap_or_default())
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
