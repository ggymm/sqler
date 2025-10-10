use iced::widget::{column, container, horizontal_space, row, scrollable, text, vertical_space};
use iced::{Alignment, Background, Element, Length, Shadow};
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

#[derive(Debug, Clone, Default)]
pub struct MysqlContentState {
    pub tables: LoadState<Vec<MysqlTable>>,
    pub processlist: LoadState<Vec<MysqlProcess>>,
    pub routines: LoadState<Vec<MysqlRoutine>>,
    pub users: LoadState<Vec<MysqlUser>>,
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

pub fn render(
    app: &App,
    tab: ContentTab,
    connection: &Connection,
    palette: Palette,
) -> Element<'static, Message> {
    let state = app.mysql_state(connection.id);

    match tab {
        ContentTab::Tables => tables_view(state.map(|s| &s.tables), connection, palette),
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
    state: Option<&LoadState<Vec<MysqlTable>>>,
    connection: &Connection,
    palette: Palette,
) -> Element<'static, Message> {
    match state {
        Some(LoadState::Loading) => loading_view("正在加载表信息…", palette),
        Some(LoadState::Error(err)) => error_view(err, palette),
        Some(LoadState::Ready(tables)) if tables.is_empty() => empty_view("当前数据库中没有表。", palette),
        Some(LoadState::Ready(tables)) => {
            let mut list = column![
                text(format!("数据库：{}", connection.summary()))
                    .size(13)
                    .color(palette.text_muted)
            ];

            for table in tables {
                list = list.push(table_row(table, palette));
            }

            scrollable(list.spacing(12)).into()
        }
        _ => idle_view(palette),
    }
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

fn table_row(
    table: &MysqlTable,
    palette: Palette,
) -> Element<'static, Message> {
    let mut body = column![
        row![
            text(table.name.clone()).size(16).color(palette.text),
            horizontal_space(),
            text(table.engine.clone().unwrap_or_else(|| "-".into()))
                .size(13)
                .color(palette.text_muted),
        ]
        .align_y(Alignment::Center),
        vertical_space().height(8),
        row![
            text(format!(
                "行数：{}",
                table.rows.map(|v| v.to_string()).unwrap_or_else(|| "-".into())
            ))
            .size(12)
            .color(palette.text_muted),
            horizontal_space(),
            text(table.updated.clone().unwrap_or_else(|| "最后更新：未知".into()))
                .size(12)
                .color(palette.text_muted),
        ]
        .spacing(12),
    ]
    .spacing(4);

    if let Some(comment) = table.comment.as_ref().filter(|c| !c.is_empty()) {
        body = body.push(vertical_space().height(8));
        body = body.push(text(comment.clone()).size(13).color(palette.text_muted));
    }

    container(body).padding(16).style(move |_| card_style(palette)).into()
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
