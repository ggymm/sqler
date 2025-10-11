use iced::widget::{button, column, container, horizontal_space, row, scrollable, text, text_input};
use iced::{Alignment, Background, Color, Element, Length, Shadow};

use crate::app::{Connection, Message, Palette};

use super::{
    LoadState, MysqlContentState, MysqlTable, TABLE_ICON_PATH, TableMenuAction, empty_view, error_view, idle_view,
    loading_view,
};

pub(super) fn view(
    connection_id: usize,
    state: Option<&MysqlContentState>,
    connection: &Connection,
    palette: Palette,
) -> Element<'static, Message> {
    let filter = state.map(|s| s.table_filter.as_str()).unwrap_or("");
    let toolbar = table_toolbar(connection_id, palette, filter);

    let selected = state.and_then(|s| s.selected_table);
    let load_state = state.map(|s| &s.tables);

    let body = match load_state {
        Some(LoadState::Loading) => loading_view("正在加载表信息…", palette),
        Some(LoadState::Error(err)) => error_view(err, palette),
        Some(LoadState::Ready(tables)) if tables.is_empty() => empty_view("当前数据库中没有表。", palette),
        Some(LoadState::Ready(tables)) => {
            let filtered = filter_tables(tables, filter);

            if filtered.is_empty() {
                column![
                    text("未找到匹配的表。").size(14).color(palette.text_muted),
                    text("尝试调整搜索关键字或清除筛选。")
                        .size(13)
                        .color(palette.text_muted),
                ]
                .spacing(6)
                .into()
            } else {
                table_list_view(connection_id, &filtered, connection, palette, selected)
            }
        }
        _ => idle_view(palette),
    };

    column![toolbar, body].spacing(16).into()
}

fn table_toolbar(
    connection_id: usize,
    palette: Palette,
    filter: &str,
) -> Element<'static, Message> {
    let actions = row![
        toolbar_action_button("打开表", connection_id, TableMenuAction::Open, palette),
        toolbar_action_button("设计表", connection_id, TableMenuAction::Design, palette),
        toolbar_action_button("新建表", connection_id, TableMenuAction::Create, palette),
        toolbar_action_button("删除表", connection_id, TableMenuAction::Delete, palette),
        toolbar_action_button("导入向导", connection_id, TableMenuAction::Import, palette),
        toolbar_action_button("导出向导", connection_id, TableMenuAction::Export, palette),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let search = text_input("搜索表…", filter)
        .size(13)
        .padding([6, 10])
        .on_input(move |value| Message::MysqlFilterTables(connection_id, value));

    row![
        actions,
        horizontal_space(),
        container(search)
            .width(Length::Fixed(220.0))
            .style(move |_| container::Style {
                background: Some(Background::Color(palette.surface)),
                text_color: Some(palette.text),
                border: iced::border::Border {
                    color: palette.border,
                    width: 1.0,
                    radius: 8.0.into(),
                },
                shadow: Shadow::default(),
            }),
    ]
    .align_y(Alignment::Center)
    .spacing(16)
    .into()
}

fn filter_tables<'a>(
    tables: &'a [MysqlTable],
    filter: &str,
) -> Vec<(usize, &'a MysqlTable)> {
    let needle = filter.trim().to_lowercase();
    if needle.is_empty() {
        return tables.iter().enumerate().collect();
    }

    tables
        .iter()
        .enumerate()
        .filter(|(_, table)| table_matches_filter(table, &needle))
        .collect()
}

fn table_matches_filter(
    table: &MysqlTable,
    needle: &str,
) -> bool {
    if needle.is_empty() {
        return true;
    }

    let name = table.name.to_lowercase();
    if name.contains(needle) {
        return true;
    }

    if table
        .comment
        .as_deref()
        .map(|c| c.to_lowercase().contains(needle))
        .unwrap_or(false)
    {
        return true;
    }

    table
        .engine
        .as_deref()
        .map(|e| e.to_lowercase().contains(needle))
        .unwrap_or(false)
}

fn table_list_view(
    connection_id: usize,
    tables: &[(usize, &MysqlTable)],
    connection: &Connection,
    palette: Palette,
    selected: Option<usize>,
) -> Element<'static, Message> {
    let header = text(format!("数据库：{}", connection.summary()))
        .size(13)
        .color(palette.text_muted);

    let mut list = column![header].spacing(12);

    for (index, table) in tables {
        list = list.push(table_list_item(
            connection_id,
            *index,
            table,
            palette,
            selected == Some(*index),
        ));
    }

    scrollable(list.spacing(10)).into()
}

fn table_list_item(
    connection_id: usize,
    index: usize,
    table: &MysqlTable,
    palette: Palette,
    selected: bool,
) -> Element<'static, Message> {
    let icon = iced::widget::svg::Svg::<iced::Theme>::from_path(TABLE_ICON_PATH)
        .width(24)
        .height(24);

    let engine = table.engine.clone().unwrap_or_else(|| "-".into());
    let rows = table.rows.map(|v| v.to_string()).unwrap_or_else(|| "未知".into());

    let info = column![
        text(table.name.clone())
            .size(15)
            .color(if selected { palette.accent } else { palette.text }),
        table
            .comment
            .as_ref()
            .filter(|c| !c.is_empty())
            .map(|comment| text(comment.clone()).size(12).color(palette.text_muted))
            .unwrap_or_else(|| text("-").size(12).color(palette.text_muted)),
        text(format!("{} • {} 行", engine, rows))
            .size(12)
            .color(palette.text_muted),
    ]
    .spacing(4)
    .width(Length::Fill);

    button(row![icon, info].spacing(16).align_y(Alignment::Center))
        .padding([10, 14])
        .width(Length::Fill)
        .style(move |_, status| table_button_style(palette, selected, status, 10.0))
        .on_press(Message::MysqlSelectTable(connection_id, index))
        .into()
}

fn table_button_style(
    palette: Palette,
    selected: bool,
    status: iced::widget::button::Status,
    radius: f32,
) -> iced::widget::button::Style {
    let background = if selected {
        palette.accent_soft
    } else {
        match status {
            iced::widget::button::Status::Hovered => palette.surface_muted,
            iced::widget::button::Status::Pressed => palette.surface,
            _ => palette.surface,
        }
    };

    iced::widget::button::Style {
        background: Some(Background::Color(background)),
        border: iced::border::Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: radius.into(),
        },
        text_color: if selected { palette.accent } else { palette.text },
        shadow: Shadow::default(),
    }
}

fn toolbar_action_button(
    label: &'static str,
    connection_id: usize,
    action: TableMenuAction,
    palette: Palette,
) -> Element<'static, Message> {
    button(text(label).size(14).color(palette.text))
        .padding([6, 12])
        .style(move |_, status| {
            use iced::widget::button::Status;

            let background = match status {
                Status::Hovered => palette.surface_muted,
                Status::Pressed => palette.surface,
                _ => Color::TRANSPARENT,
            };

            iced::widget::button::Style {
                background: Some(Background::Color(background)),
                border: iced::border::Border {
                    color: palette.border,
                    width: 1.0,
                    radius: 6.0.into(),
                },
                text_color: palette.text,
                shadow: Shadow::default(),
            }
        })
        .on_press(Message::MysqlTableMenuAction(connection_id, action))
        .into()
}
