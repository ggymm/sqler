use iced::widget::{column, container, row, text};
use iced::{Alignment, Element, Length};

use crate::app::{Connection, ContentTab, Message, Palette};

use super::common::{
    card_style, centered_message, load_state_list_view, stack_section, LoadState, LoadStateMessages,
};
use crate::driver::{QueryPayload, QueryResponse};
use serde_json::Value as JsonValue;

#[derive(Debug, Clone, Default)]
pub struct RedisContentState {
    pub databases: LoadState<Vec<RedisDatabase>>,
}

#[derive(Debug, Clone)]
pub struct RedisDatabase {
    pub name: String,
    pub keys: u64,
    pub expires: u64,
    pub avg_ttl_ms: Option<u64>,
}

pub(super) fn render(
    state: Option<&RedisContentState>,
    tab: ContentTab,
    connection: &Connection,
    palette: Palette,
) -> Element<'static, Message> {
    match tab {
        ContentTab::Tables => database_view(state, connection, palette),
        _ => centered_message(
            [format!(
                "{} 的 {} 视图尚未实现。",
                connection.name,
                tab.title()
            )],
            palette,
        ),
    }
}

fn database_view(
    state: Option<&RedisContentState>,
    connection: &Connection,
    palette: Palette,
) -> Element<'static, Message> {
    let actions = row![text("Redis 数据库").size(16).color(palette.text)]
        .align_y(Alignment::Center);

    let summary = connection.summary();

    let content = load_state_list_view(
        state.map(|s| &s.databases),
        palette,
        LoadStateMessages {
            loading: "正在读取 Redis 数据库列表…",
            empty: "没有发现任何 Redis 数据库。",
            idle: "请选择 Redis 连接以加载数据库列表。",
        },
        move |databases, palette| {
            let mut list = column![text(format!("连接：{}", summary))
                .size(13)
                .color(palette.text_muted)]
            .spacing(12);

            for db in databases {
                list = list.push(database_row(db, palette));
            }

            container(list.spacing(8)).width(Length::Fill).into()
        },
    );

    stack_section(actions.into(), content)
}

fn database_row(
    db: &RedisDatabase,
    palette: Palette,
) -> Element<'static, Message> {
    let avg_ttl = db
        .avg_ttl_ms
        .map(|ms| format!("平均 TTL：{} ms", ms))
        .unwrap_or_else(|| "平均 TTL：-".into());

    container(
        column![
            text(db.name.clone())
                .size(15)
                .color(palette.text),
            row![
                text(format!("键数量：{}", db.keys))
                    .size(12)
                    .color(palette.text_muted),
                text(format!("有过期键：{}", db.expires))
                    .size(12)
                    .color(palette.text_muted),
                text(avg_ttl)
                    .size(12)
                    .color(palette.text_muted),
            ]
            .spacing(16),
        ]
        .spacing(6),
    )
    .padding(16)
    .style(move |_| card_style(palette))
    .into()
}

pub fn parse_databases(response: QueryResponse) -> Result<Vec<RedisDatabase>, String> {
    let (columns, rows) = expect_tabular(response)?;
    let idx_name = find_column(&columns, "database")?;
    let idx_keys = find_column(&columns, "keys")?;
    let idx_expires = find_column(&columns, "expires")?;
    let idx_avg_ttl = find_column(&columns, "avg_ttl").ok();

    let mut result = Vec::new();
    for row in rows {
        let name = cell_string(row.get(idx_name));
        let keys = cell_u64(row.get(idx_keys))?;
        let expires = cell_u64(row.get(idx_expires))?;
        let avg_ttl_ms = cell_u64_optional(idx_avg_ttl.and_then(|idx| row.get(idx)))?;

        result.push(RedisDatabase {
            name,
            keys,
            expires,
            avg_ttl_ms,
        });
    }

    Ok(result)
}

fn expect_tabular(response: QueryResponse) -> Result<(Vec<String>, Vec<Vec<JsonValue>>), String> {
    match response.payload {
        QueryPayload::Tabular { columns, rows } => Ok((columns, rows)),
    }
}

fn cell_string(value: Option<&JsonValue>) -> String {
    match value {
        Some(JsonValue::String(s)) => s.clone(),
        Some(JsonValue::Number(n)) => n.to_string(),
        Some(JsonValue::Bool(b)) => b.to_string(),
        _ => String::new(),
    }
}

fn cell_u64(value: Option<&JsonValue>) -> Result<u64, String> {
    let value = value.ok_or_else(|| "缺少数值字段".to_string())?;
    match value {
        JsonValue::Number(n) => n.as_u64().ok_or_else(|| "无法解析数字".into()),
        JsonValue::String(s) => s.parse::<u64>().map_err(|_| "无法解析数字".into()),
        _ => Err("无法解析数字".into()),
    }
}

fn cell_u64_optional(value: Option<&JsonValue>) -> Result<Option<u64>, String> {
    match value {
        None => Ok(None),
        Some(JsonValue::Number(n)) => n
            .as_u64()
            .map(Some)
            .ok_or_else(|| "无法解析数字".into()),
        Some(JsonValue::String(s)) if s.is_empty() => Ok(None),
        Some(JsonValue::String(s)) => s
            .parse::<u64>()
            .map(Some)
            .map_err(|_| "无法解析数字".into()),
        _ => Err("无法解析数字".into()),
    }
}

fn find_column(columns: &[String], name: &str) -> Result<usize, String> {
    columns
        .iter()
        .position(|c| c.eq_ignore_ascii_case(name))
        .ok_or_else(|| format!("结果集中缺少列：{}", name))
}
