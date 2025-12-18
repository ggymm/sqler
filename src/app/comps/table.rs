use gpui::*;
use gpui_component::table::{Column, TableDelegate, TableState};

pub struct DataTable {
    cols: Vec<Column>,
    rows: Vec<Vec<SharedString>>,
    loading: bool,
}

fn calc_width(s: &str) -> usize {
    s.chars()
        .map(|c| {
            if c.is_uppercase() {
                12 // 大写字母宽度
            } else if c.is_ascii_alphabetic() || c.is_ascii_digit() {
                8 // 小写字母和数字宽度
            } else if c.is_ascii() {
                6 // 其他 ASCII 字符
            } else {
                16 // 中文等宽字符
            }
        })
        .sum()
}

impl DataTable {
    fn build_cols(
        cols: &[SharedString],
        rows: &[Vec<SharedString>],
    ) -> Vec<Column> {
        cols.iter()
            .enumerate()
            .map(|(col_ix, name)| {
                let col_width = calc_width(name);
                let row_width = rows
                    .iter()
                    .take(10)
                    .filter_map(|row| row.get(col_ix))
                    .map(|cell| calc_width(cell))
                    .max()
                    .unwrap_or(0);
                let max_width = col_width.max(row_width);
                let final_width = (max_width + 24).max(80).min(400) as f32;

                Column::new(name.clone(), name.clone()).width(px(final_width))
            })
            .collect()
    }

    pub fn new(
        cols: Vec<SharedString>,
        rows: Vec<Vec<SharedString>>,
        window: &mut Window,
        cx: &mut App,
    ) -> Entity<TableState<Self>> {
        let this = Self {
            cols: Self::build_cols(&cols, &rows),
            rows,
            loading: false,
        };
        cx.new(|cx| {
            TableState::new(this, window, cx)
                .sortable(false)
                .col_movable(true)
                .col_resizable(true)
                .col_selectable(true)
                .row_selectable(true)
                .loop_selection(true)
        })
    }

    pub fn get_data(
        &self,
        row_ix: usize,
    ) -> Vec<SharedString> {
        self.rows.get(row_ix).cloned().unwrap_or_default()
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
        &mut self,
        col_ix: usize,
        _window: &mut Window,
        _cx: &mut Context<TableState<Self>>,
    ) -> impl IntoElement {
        div()
            .size_full()
            .text_sm()
            .child(self.cols.get(col_ix).unwrap().name.clone())
    }

    fn render_td(
        &mut self,
        row_ix: usize,
        col_ix: usize,
        _window: &mut Window,
        _cx: &mut Context<TableState<Self>>,
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
