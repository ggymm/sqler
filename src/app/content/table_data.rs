use std::fmt;

use iced::widget::{column, horizontal_space, pick_list, row, text, text_input};
use iced::{Alignment, Element, Length};

use crate::comps::table::{TableColumn, TableRow, data_table};

use super::{App, LoadState, Message, MysqlTableData, Palette, TableDataPreferences};
use super::common::{centered_message, error_view, loading_view};

const PAGE_SIZE_OPTIONS: &[usize] = &[50, 100, 200, 500, 1000];
const MAX_CELL_CHARS: usize = 512;

#[derive(Debug, Clone, PartialEq, Eq)]
struct SortChoice {
    label: String,
    index: Option<usize>,
}

impl fmt::Display for SortChoice {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        write!(f, "{}", self.label)
    }
}

pub fn render_table_data(
    app: &App,
    palette: Palette,
    connection_id: usize,
    table_name: &str,
) -> Element<'static, Message> {
    let state = app.mysql_state(connection_id);

    let data_state = state
        .and_then(|s| s.table_data.get(table_name))
        .cloned()
        .unwrap_or(LoadState::Idle);

    let prefs = state
        .and_then(|s| s.table_prefs.get(table_name))
        .cloned()
        .unwrap_or_default();

    match data_state {
        LoadState::Idle | LoadState::Loading => loading_view("正在加载表数据…", palette),
        LoadState::Error(err) => error_view(&err, palette),
        LoadState::Ready(data) => render_data_view(palette, connection_id, table_name, data, prefs),
    }
}

fn render_data_view(
    palette: Palette,
    connection_id: usize,
    table_name: &str,
    data: MysqlTableData,
    prefs: TableDataPreferences,
) -> Element<'static, Message> {
    let filter_value = prefs.filter.trim().to_lowercase();
    let mut rows: Vec<&Vec<String>> = data.rows.iter().collect();

    if !filter_value.is_empty() {
        rows.retain(|row| row.iter().any(|cell| cell.to_lowercase().contains(&filter_value)));
    }

    if let Some(index) = prefs.sort_column.filter(|idx| *idx < data.columns.len()) {
        rows.sort_by(|a, b| {
            let left = a.get(index).map(|s| s.as_str()).unwrap_or("");
            let right = b.get(index).map(|s| s.as_str()).unwrap_or("");
            left.cmp(right)
        });
    }

    let page_size = prefs.page_size.max(1);
    let displayed: Vec<TableRow> = rows
        .into_iter()
        .take(page_size)
        .map(|row| {
            let sanitized = row.iter().map(|cell| sanitize_cell(cell)).collect();
            TableRow::new(sanitized)
        })
        .collect();

    let columns: Vec<TableColumn> = data
        .columns
        .iter()
        .map(|title| TableColumn::new(title.clone(), 160.0))
        .collect();

    if displayed.is_empty() {
        return centered_message(
            [
                format!("{} 暂无匹配的数据。", table_name),
                format!("当前页大小：{} 行。", page_size),
            ],
            palette,
        );
    }

    let sort_options = build_sort_choices(&data.columns);
    let current_sort = sort_options
        .iter()
        .find(|item| item.index == prefs.sort_column)
        .cloned()
        .unwrap_or_else(|| sort_options[0].clone());

    let page_options: Vec<usize> = PAGE_SIZE_OPTIONS.iter().cloned().collect();
    let current_page_size = page_options
        .iter()
        .cloned()
        .find(|size| *size == prefs.page_size)
        .unwrap_or(100);

    let filter_table_key = table_name.to_string();
    let sort_table_key = table_name.to_string();
    let page_table_key = table_name.to_string();

    let filter_input = text_input("输入关键字过滤…", prefs.filter.as_str())
        .size(13)
        .padding([6, 10])
        .width(Length::Fixed(220.0))
        .on_input(move |value| Message::MysqlTableDataFilterChanged(connection_id, filter_table_key.clone(), value));

    let sort_pick = pick_list(
        sort_options.clone(),
        Some(current_sort.clone()),
        move |choice: SortChoice| {
            Message::MysqlTableDataSortChanged(connection_id, sort_table_key.clone(), choice.index)
        },
    )
    .placeholder("默认顺序")
    .text_size(13);

    let page_pick = pick_list(page_options.clone(), Some(current_page_size), move |size: usize| {
        Message::MysqlTableDataPageSizeChanged(connection_id, page_table_key.clone(), size)
    })
    .text_size(13);

    let controls = row![
        text("筛选").size(12).color(palette.text_muted),
        filter_input,
        horizontal_space(),
        text("排序").size(12).color(palette.text_muted),
        sort_pick,
        horizontal_space(),
        text("每页行数").size(12).color(palette.text_muted),
        page_pick,
    ]
    .spacing(12)
    .align_y(Alignment::Center);

    let table_element: Element<'static, Message> = data_table(
        connection_id,
        table_name.to_string(),
        prefs.scroll_x,
        prefs.scroll_y,
        columns,
        displayed,
        palette,
    )
    .into();

    column![controls, table_element].spacing(16).into()
}

fn build_sort_choices(columns: &[String]) -> Vec<SortChoice> {
    let mut result = Vec::with_capacity(columns.len() + 1);
    result.push(SortChoice {
        label: "默认顺序".into(),
        index: None,
    });

    for (idx, title) in columns.iter().enumerate() {
        result.push(SortChoice {
            label: title.clone(),
            index: Some(idx),
        });
    }

    result
}

fn sanitize_cell(value: &str) -> String {
    let mut sanitized = value.replace(&['\n', '\r'][..], " ");

    if sanitized.len() > MAX_CELL_CHARS {
        sanitized.truncate(MAX_CELL_CHARS);
        sanitized.push('…');
    }

    sanitized
}
