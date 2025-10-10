use iced::widget::scrollable::{Direction, Scrollbar};
use iced::widget::{button, column, container, horizontal_space, row, scrollable, text, vertical_space};
use iced::{Alignment, Background, Color, Element, Length, Shadow, Size};
use serde_json::Value as JsonValue;

use crate::app::{App, Connection, ContentTab, Message, Palette};
use crate::driver::{QueryPayload, QueryResponse};

pub const TABLES_SQL: &str = r#"
SELECT
    table_name AS name,
    engine,
    table_rows,
    IFNULL(table_comment, '') AS comment,
    IFNULL(DATE_FORMAT(update_time, '%Y-%m-%d %H:%i:%s'), '') AS updated
FROM information_schema.tables
WHERE table_schema = DATABASE()
ORDER BY table_name
LIMIT 200
"#;

pub const PROCESSLIST_SQL: &str = r#"
SELECT
    IFNULL(info, '') AS statement,
    TIME AS seconds,
    IFNULL(state, '') AS state,
    command
FROM information_schema.processlist
WHERE db = DATABASE()
ORDER BY TIME DESC
LIMIT 20
"#;

pub const ROUTINES_SQL: &str = r#"
SELECT
    routine_name AS name,
    routine_type AS kind,
    IFNULL(dtd_identifier, '') AS returns,
    IFNULL(security_type, '') AS security,
    IFNULL(DATE_FORMAT(created, '%Y-%m-%d %H:%i:%s'), '') AS created
FROM information_schema.routines
WHERE routine_schema = DATABASE()
ORDER BY routine_name
LIMIT 100
"#;

pub const USERS_SQL: &str = r#"
SELECT
    user AS name,
    host,
    IFNULL(plugin, '') AS plugin,
    IFNULL(account_locked, '') AS locked,
    IFNULL(DATE_FORMAT(password_last_changed, '%Y-%m-%d %H:%i:%s'), '') AS password_changed
FROM mysql.user
ORDER BY user, host
LIMIT 100
"#;

#[derive(Debug, Clone)]
pub enum LoadState<T> {
    Idle,
    Loading,
    Ready(T),
    Error(String),
}

impl<T> Default for LoadState<T> {
    fn default() -> Self {
        LoadState::Idle
    }
}

impl<T> LoadState<T> {
    pub fn should_load(&self) -> bool {
        matches!(self, LoadState::Idle | LoadState::Error(_))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableDisplayMode {
    List,
    Grid,
}

impl Default for TableDisplayMode {
    fn default() -> Self {
        TableDisplayMode::List
    }
}

const TABLE_ICON_PATH: &str = "assets/icons/table.svg";

#[derive(Debug, Clone, Default)]
pub struct MysqlContentState {
    pub tables: LoadState<Vec<MysqlTable>>,
    pub processlist: LoadState<Vec<MysqlProcess>>,
    pub routines: LoadState<Vec<MysqlRoutine>>,
    pub users: LoadState<Vec<MysqlUser>>,
    pub tables_mode: TableDisplayMode,
    pub selected_table: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct MysqlTable {
    pub name: String,
    pub engine: Option<String>,
    pub rows: Option<u64>,
    pub comment: Option<String>,
    pub updated: Option<String>,
}

#[derive(Debug, Clone)]
pub struct MysqlProcess {
    pub statement: String,
    pub seconds: Option<u64>,
    pub state: Option<String>,
    pub command: Option<String>,
}

#[derive(Debug, Clone)]
pub struct MysqlRoutine {
    pub name: String,
    pub kind: String,
    pub returns: Option<String>,
    pub security: Option<String>,
    pub created: Option<String>,
}

#[derive(Debug, Clone)]
pub struct MysqlUser {
    pub name: String,
    pub host: String,
    pub plugin: Option<String>,
    pub locked: Option<String>,
    pub password_changed: Option<String>,
}

const GRID_TILE_MIN_WIDTH: f32 = 240.0;
const GRID_TILE_HEIGHT: f32 = 120.0;
const GRID_TILE_SPACING: f32 = 16.0;
const GRID_VERTICAL_RESERVE: f32 = 220.0;

#[derive(Debug, Clone, Copy)]
pub enum TableMenuAction {
    Open,
    Design,
    Create,
    Delete,
    Import,
    Export,
}

pub fn render(
    app: &App,
    tab: ContentTab,
    connection: &Connection,
    palette: Palette,
) -> Element<'static, Message> {
    let state = app.mysql_state(connection.id);
    let viewport = app.window_size();

    match tab {
        ContentTab::Tables => {
            let mode = state.map(|s| s.tables_mode).unwrap_or_default();
            let selected = state.and_then(|s| s.selected_table);
            tables_view(
                connection.id,
                state.map(|s| &s.tables),
                connection,
                palette,
                mode,
                viewport,
                selected,
            )
        }
        ContentTab::Queries => processlist_view(state.map(|s| &s.processlist), palette),
        ContentTab::Functions => routines_view(state.map(|s| &s.routines), palette),
        ContentTab::Users => users_view(state.map(|s| &s.users), palette),
    }
}

pub fn parse_tables(response: QueryResponse) -> Result<Vec<MysqlTable>, String> {
    let (columns, rows) = expect_tabular(response)?;
    let idx_name = find_column(&columns, "name")?;
    let idx_engine = find_column(&columns, "engine").ok();
    let idx_rows = find_column(&columns, "table_rows")
        .ok()
        .or_else(|| find_column(&columns, "rows").ok());
    let idx_comment = find_column(&columns, "comment").ok();
    let idx_updated = find_column(&columns, "updated").ok();

    let mut result = Vec::new();
    for row in rows {
        let name = cell_string(row.get(idx_name));
        let engine = idx_engine
            .and_then(|i| row.get(i))
            .map(|v| cell_string(Some(v)))
            .filter(|v| !v.is_empty());
        let rows = idx_rows.and_then(|i| row.get(i)).and_then(cell_u64_opt);
        let comment = idx_comment
            .and_then(|i| row.get(i))
            .map(|v| cell_string(Some(v)))
            .filter(|v| !v.is_empty());
        let updated = idx_updated
            .and_then(|i| row.get(i))
            .map(|v| cell_string(Some(v)))
            .filter(|v| !v.is_empty());

        result.push(MysqlTable {
            name,
            engine,
            rows,
            comment,
            updated,
        });
    }

    Ok(result)
}

pub fn parse_processlist(response: QueryResponse) -> Result<Vec<MysqlProcess>, String> {
    let (columns, rows) = expect_tabular(response)?;
    let idx_statement = find_column(&columns, "statement")?;
    let idx_seconds = find_column(&columns, "seconds").ok();
    let idx_state = find_column(&columns, "state").ok();
    let idx_command = find_column(&columns, "command").ok();

    let mut result = Vec::new();
    for row in rows {
        let statement = cell_string(row.get(idx_statement));
        let seconds = idx_seconds.and_then(|i| row.get(i)).and_then(cell_u64_opt);
        let state = idx_state
            .and_then(|i| row.get(i))
            .map(|v| cell_string(Some(v)))
            .filter(|v| !v.is_empty());
        let command = idx_command
            .and_then(|i| row.get(i))
            .map(|v| cell_string(Some(v)))
            .filter(|v| !v.is_empty());

        result.push(MysqlProcess {
            statement,
            seconds,
            state,
            command,
        });
    }

    Ok(result)
}

pub fn parse_routines(response: QueryResponse) -> Result<Vec<MysqlRoutine>, String> {
    let (columns, rows) = expect_tabular(response)?;
    let idx_name = find_column(&columns, "name")?;
    let idx_kind = find_column(&columns, "kind")?;
    let idx_returns = find_column(&columns, "returns").ok();
    let idx_security = find_column(&columns, "security").ok();
    let idx_created = find_column(&columns, "created").ok();

    let mut result = Vec::new();
    for row in rows {
        let name = cell_string(row.get(idx_name));
        let kind = cell_string(row.get(idx_kind));
        let returns = idx_returns
            .and_then(|i| row.get(i))
            .map(|v| cell_string(Some(v)))
            .filter(|v| !v.is_empty());
        let security = idx_security
            .and_then(|i| row.get(i))
            .map(|v| cell_string(Some(v)))
            .filter(|v| !v.is_empty());
        let created = idx_created
            .and_then(|i| row.get(i))
            .map(|v| cell_string(Some(v)))
            .filter(|v| !v.is_empty());

        result.push(MysqlRoutine {
            name,
            kind,
            returns,
            security,
            created,
        });
    }

    Ok(result)
}

pub fn parse_users(response: QueryResponse) -> Result<Vec<MysqlUser>, String> {
    let (columns, rows) = expect_tabular(response)?;
    let idx_name = find_column(&columns, "name")?;
    let idx_host = find_column(&columns, "host")?;
    let idx_plugin = find_column(&columns, "plugin").ok();
    let idx_locked = find_column(&columns, "locked").ok();
    let idx_password_changed = find_column(&columns, "password_changed").ok();

    let mut result = Vec::new();
    for row in rows {
        let name = cell_string(row.get(idx_name));
        let host = cell_string(row.get(idx_host));
        let plugin = idx_plugin
            .and_then(|i| row.get(i))
            .map(|v| cell_string(Some(v)))
            .filter(|v| !v.is_empty());
        let locked = idx_locked
            .and_then(|i| row.get(i))
            .map(|v| cell_string(Some(v)))
            .filter(|v| !v.is_empty());
        let password_changed = idx_password_changed
            .and_then(|i| row.get(i))
            .map(|v| cell_string(Some(v)))
            .filter(|v| !v.is_empty());

        result.push(MysqlUser {
            name,
            host,
            plugin,
            locked,
            password_changed,
        });
    }

    Ok(result)
}

fn tables_view(
    connection_id: usize,
    state: Option<&LoadState<Vec<MysqlTable>>>,
    connection: &Connection,
    palette: Palette,
    mode: TableDisplayMode,
    viewport: Size,
    selected: Option<usize>,
) -> Element<'static, Message> {
    let toolbar = table_toolbar(connection_id, palette, mode);

    let body = match state {
        Some(LoadState::Loading) => loading_view("正在加载表信息…", palette),
        Some(LoadState::Error(err)) => error_view(err, palette),
        Some(LoadState::Ready(tables)) if tables.is_empty() => empty_view("当前数据库中没有表。", palette),
        Some(LoadState::Ready(tables)) => match mode {
            TableDisplayMode::List => table_list_view(connection_id, tables, connection, palette, selected),
            TableDisplayMode::Grid => table_grid_view(connection_id, tables, connection, palette, viewport, selected),
        },
        _ => idle_view(palette),
    };

    column![toolbar, body].spacing(16).into()
}

fn table_toolbar(
    connection_id: usize,
    palette: Palette,
    mode: TableDisplayMode,
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

    let mode_switch = row![
        mode_toggle_button("列表模式", connection_id, TableDisplayMode::List, mode, palette),
        mode_toggle_button("网格模式", connection_id, TableDisplayMode::Grid, mode, palette),
    ]
    .spacing(6)
    .align_y(Alignment::Center);

    row![actions, horizontal_space(), mode_switch]
        .align_y(Alignment::Center)
        .spacing(16)
        .into()
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

fn mode_toggle_button(
    label: &'static str,
    connection_id: usize,
    target: TableDisplayMode,
    current: TableDisplayMode,
    palette: Palette,
) -> Element<'static, Message> {
    let is_active = target == current;

    button(
        text(label)
            .size(13)
            .color(if is_active { palette.accent_text } else { palette.text }),
    )
    .padding([6, 14])
    .style(move |_, status| {
        use iced::widget::button::Status;

        let background = if is_active {
            palette.accent
        } else if matches!(status, Status::Hovered) {
            palette.surface_muted
        } else {
            Color::TRANSPARENT
        };

        iced::widget::button::Style {
            background: Some(Background::Color(background)),
            border: iced::border::Border {
                color: if is_active { palette.accent } else { palette.border },
                width: 1.0,
                radius: 8.0.into(),
            },
            text_color: if is_active { palette.accent_text } else { palette.text },
            shadow: Shadow::default(),
        }
    })
    .on_press(Message::MysqlChangeTableView(connection_id, target))
    .into()
}

fn table_list_view(
    connection_id: usize,
    tables: &[MysqlTable],
    connection: &Connection,
    palette: Palette,
    selected: Option<usize>,
) -> Element<'static, Message> {
    let mut list = column![].spacing(12);

    for (index, table) in tables.iter().enumerate() {
        list = list.push(table_list_item(
            connection_id,
            index,
            table,
            palette,
            selected == Some(index),
        ));
    }

    scrollable(list.spacing(10)).into()
}

fn table_grid_view(
    connection_id: usize,
    tables: &[MysqlTable],
    _connection: &Connection,
    palette: Palette,
    viewport: Size,
    selected: Option<usize>,
) -> Element<'static, Message> {
    let padding_compensation = 48.0; // parent container horizontal padding
    let available_height = (viewport.height - GRID_VERTICAL_RESERVE).max(GRID_TILE_HEIGHT);
    let rows_per_column = (available_height / (GRID_TILE_HEIGHT + GRID_TILE_SPACING))
        .floor()
        .max(1.0) as usize;

    let mut column_elements: Vec<Element<'static, Message>> = Vec::new();

    for (col_index, chunk) in tables.chunks(rows_per_column).enumerate() {
        let mut column_view = column![].spacing(GRID_TILE_SPACING);
        for (row_idx, table) in chunk.iter().enumerate() {
            let global_index = col_index * rows_per_column + row_idx;
            column_view = column_view.push(table_grid_item(
                connection_id,
                global_index,
                table,
                palette,
                selected == Some(global_index),
            ));
        }
        let column_element: Element<'static, Message> = container(column_view)
            .width(Length::Fixed(GRID_TILE_MIN_WIDTH))
            .height(Length::Fixed(available_height))
            .into();
        column_elements.push(column_element);
    }

    let column_count = column_elements.len().max(1);

    let mut row_view = row![].spacing(GRID_TILE_SPACING).align_y(Alignment::Start);
    for column_element in column_elements {
        row_view = row_view.push(column_element);
    }

    let content_width = (GRID_TILE_MIN_WIDTH + GRID_TILE_SPACING) * column_count as f32;
    let available_width = (viewport.width - padding_compensation).max(GRID_TILE_MIN_WIDTH);
    let horizontal_scroll = content_width > available_width;

    let grid: Element<'static, Message> = if horizontal_scroll {
        let scroll = scrollable(row_view)
            .direction(Direction::Horizontal(Scrollbar::new()))
            .width(Length::Fill);

        container(scroll).width(Length::Fill).height(Length::Fill).into()
    } else {
        container(row_view).width(Length::Fill).height(Length::Fill).into()
    };

    grid.into()
}

fn processlist_view(
    state: Option<&LoadState<Vec<MysqlProcess>>>,
    palette: Palette,
) -> Element<'static, Message> {
    match state {
        Some(LoadState::Loading) => loading_view("正在加载最近查询…", palette),
        Some(LoadState::Error(err)) => error_view(err, palette),
        Some(LoadState::Ready(records)) if records.is_empty() => empty_view("尚未捕获到执行中的 SQL。", palette),
        Some(LoadState::Ready(records)) => {
            let mut list = column![];

            for item in records {
                list = list.push(process_row(item, palette));
            }

            scrollable(list.spacing(12)).into()
        }
        _ => idle_view(palette),
    }
}

fn routines_view(
    state: Option<&LoadState<Vec<MysqlRoutine>>>,
    palette: Palette,
) -> Element<'static, Message> {
    match state {
        Some(LoadState::Loading) => loading_view("正在加载函数与存储过程…", palette),
        Some(LoadState::Error(err)) => error_view(err, palette),
        Some(LoadState::Ready(routines)) if routines.is_empty() => {
            empty_view("当前库尚未定义函数或存储过程。", palette)
        }
        Some(LoadState::Ready(routines)) => {
            let mut list = column![];
            for routine in routines {
                list = list.push(routine_row(routine, palette));
            }
            scrollable(list.spacing(12)).into()
        }
        _ => idle_view(palette),
    }
}

fn users_view(
    state: Option<&LoadState<Vec<MysqlUser>>>,
    palette: Palette,
) -> Element<'static, Message> {
    match state {
        Some(LoadState::Loading) => loading_view("正在加载数据库用户…", palette),
        Some(LoadState::Error(err)) => error_view(err, palette),
        Some(LoadState::Ready(users)) if users.is_empty() => empty_view("未找到任何 MySQL 用户。", palette),
        Some(LoadState::Ready(users)) => {
            let mut list = column![];
            for user in users {
                list = list.push(user_row(user, palette));
            }
            scrollable(list.spacing(12)).into()
        }
        _ => idle_view(palette),
    }
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

    let mut info =
        column![
            text(table.name.clone())
                .size(15)
                .color(if selected { palette.accent } else { palette.text }),
        ]
        .spacing(4)
        .width(Length::Fill);

    if let Some(comment) = table.comment.as_ref().filter(|c| !c.is_empty()) {
        info = info.push(text(comment.clone()).size(12).color(palette.text_muted));
    }

    let engine = table.engine.clone().unwrap_or_else(|| "-".into());
    let rows = table.rows.map(|v| v.to_string()).unwrap_or_else(|| "未知".into());

    info = info.push(
        text(format!("{} • {} 行", engine, rows))
            .size(12)
            .color(palette.text_muted),
    );

    button(row![icon, info].spacing(12).align_y(Alignment::Center))
        .padding([10, 14])
        .width(Length::Fill)
        .style(move |_, status| table_button_style(palette, selected, status, 10.0))
        .on_press(Message::MysqlSelectTable(connection_id, index))
        .into()
}

fn table_grid_item(
    connection_id: usize,
    index: usize,
    table: &MysqlTable,
    palette: Palette,
    selected: bool,
) -> Element<'static, Message> {
    let icon = iced::widget::svg::Svg::<iced::Theme>::from_path(TABLE_ICON_PATH)
        .width(28)
        .height(28);

    let label = text(table.name.clone())
        .size(14)
        .color(if selected { palette.accent } else { palette.text })
        .width(Length::Fill);

    let content = row![icon, label]
        .spacing(12)
        .align_y(Alignment::Center)
        .width(Length::Fill);

    button(content)
        .padding(16)
        .width(Length::Fixed(GRID_TILE_MIN_WIDTH))
        .style(move |_, status| table_button_style(palette, selected, status, 12.0))
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

fn process_row(
    process: &MysqlProcess,
    palette: Palette,
) -> Element<'static, Message> {
    container(
        column![
            text(process.statement.trim().to_string())
                .size(14)
                .color(palette.text)
                .width(Length::Fill),
            vertical_space().height(6),
            row![
                text(format!(
                    "已运行：{}s",
                    process.seconds.map(|s| s.to_string()).unwrap_or_else(|| "0".into())
                ))
                .size(12)
                .color(palette.text_muted),
                horizontal_space(),
                text(process.state.clone().unwrap_or_else(|| "-".into()))
                    .size(12)
                    .color(palette.text_muted),
                horizontal_space(),
                text(process.command.clone().unwrap_or_else(|| "-".into()))
                    .size(12)
                    .color(palette.text_muted),
            ]
            .spacing(12),
        ]
        .spacing(4),
    )
    .padding(16)
    .style(move |_| card_style(palette))
    .into()
}

fn routine_row(
    routine: &MysqlRoutine,
    palette: Palette,
) -> Element<'static, Message> {
    container(
        column![
            row![
                text(routine.name.clone()).size(15).color(palette.text),
                horizontal_space(),
                text(routine.kind.clone()).size(12).color(palette.text_muted),
            ]
            .align_y(Alignment::Center),
            vertical_space().height(6),
            row![
                text(
                    routine
                        .returns
                        .clone()
                        .filter(|v| !v.is_empty())
                        .unwrap_or_else(|| "无返回".into())
                )
                .size(12)
                .color(palette.text_muted),
                horizontal_space(),
                text(
                    routine
                        .security
                        .clone()
                        .filter(|v| !v.is_empty())
                        .unwrap_or_else(|| "-".into())
                )
                .size(12)
                .color(palette.text_muted),
                horizontal_space(),
                text(
                    routine
                        .created
                        .clone()
                        .filter(|v| !v.is_empty())
                        .unwrap_or_else(|| "-".into())
                )
                .size(12)
                .color(palette.text_muted),
            ]
            .spacing(12),
        ]
        .spacing(4),
    )
    .padding(16)
    .style(move |_| card_style(palette))
    .into()
}

fn user_row(
    user: &MysqlUser,
    palette: Palette,
) -> Element<'static, Message> {
    container(
        column![
            row![
                text(format!("{}@{}", user.name, user.host))
                    .size(15)
                    .color(palette.text),
                horizontal_space(),
                text(user.plugin.clone().unwrap_or_else(|| "-".into()))
                    .size(12)
                    .color(palette.text_muted),
            ]
            .align_y(Alignment::Center),
            vertical_space().height(6),
            row![
                text(
                    user.locked
                        .clone()
                        .filter(|v| !v.is_empty())
                        .map(|v| format!("状态：{}", v))
                        .unwrap_or_else(|| "状态：未知".into())
                )
                .size(12)
                .color(palette.text_muted),
                horizontal_space(),
                text(
                    user.password_changed
                        .clone()
                        .filter(|v| !v.is_empty())
                        .map(|v| format!("密码更新：{}", v))
                        .unwrap_or_else(|| "密码更新：未知".into())
                )
                .size(12)
                .color(palette.text_muted),
            ]
            .spacing(12),
        ]
        .spacing(4),
    )
    .padding(16)
    .style(move |_| card_style(palette))
    .into()
}

fn loading_view(
    message: &'static str,
    palette: Palette,
) -> Element<'static, Message> {
    container(text(message).size(14).color(palette.text_muted))
        .width(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

fn error_view(
    message: &str,
    palette: Palette,
) -> Element<'static, Message> {
    container(text(format!("加载失败：{}", message)).size(14).color(palette.accent))
        .width(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

fn empty_view(
    message: &'static str,
    palette: Palette,
) -> Element<'static, Message> {
    container(text(message).size(14).color(palette.text_muted))
        .width(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

fn idle_view(palette: Palette) -> Element<'static, Message> {
    container(text("请激活连接以加载 MySQL 数据。").size(14).color(palette.text_muted))
        .width(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

fn card_style(palette: Palette) -> container::Style {
    container::Style {
        background: Some(Background::Color(palette.surface)),
        text_color: Some(palette.text),
        border: iced::border::Border {
            color: palette.border,
            width: 1.0,
            radius: 12.0.into(),
        },
        shadow: Shadow::default(),
    }
}

fn expect_tabular(response: QueryResponse) -> Result<(Vec<String>, Vec<Vec<JsonValue>>), String> {
    match response.payload {
        QueryPayload::Tabular { columns, rows } => Ok((columns, rows)),
        QueryPayload::Documents { .. } => Err("期望表格数据，但收到文档结果".into()),
    }
}

fn find_column(
    columns: &[String],
    name: &str,
) -> Result<usize, String> {
    columns
        .iter()
        .position(|c| c.eq_ignore_ascii_case(name))
        .ok_or_else(|| format!("结果集中缺少列：{}", name))
}

fn cell_string(value: Option<&JsonValue>) -> String {
    match value {
        Some(JsonValue::String(s)) => s.clone(),
        Some(JsonValue::Number(n)) => {
            if let Some(i) = n.as_i64() {
                i.to_string()
            } else if let Some(u) = n.as_u64() {
                u.to_string()
            } else if let Some(f) = n.as_f64() {
                if (f - f.trunc()).abs() < f64::EPSILON {
                    format!("{:.0}", f)
                } else {
                    f.to_string()
                }
            } else {
                String::new()
            }
        }
        Some(JsonValue::Bool(b)) => b.to_string(),
        Some(JsonValue::Null) | None => String::new(),
        Some(other) => other.to_string(),
    }
}

fn cell_u64_opt(value: &JsonValue) -> Option<u64> {
    match value {
        JsonValue::Number(n) => n.as_u64().or_else(|| n.as_i64().map(|v| v.max(0) as u64)),
        JsonValue::String(s) => s.parse::<u64>().ok(),
        _ => None,
    }
}
