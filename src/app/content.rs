mod common;
mod overview;
mod redis;
mod query_editor;
mod saved_functions;
mod saved_queries;
mod table_data;

pub use common::{LoadState, LoadState as MysqlLoadState};
pub use redis::{RedisContentState, RedisDatabase, parse_databases as parse_redis_databases};
pub use overview::{
    MysqlContentState, MysqlProcess, MysqlRoutine, MysqlTable, MysqlTableData, MysqlUser, PROCESSLIST_SQL,
    ROUTINES_SQL, TABLES_SQL, TableDataPreferences, TableMenuAction, USERS_SQL, parse_processlist, parse_routines,
    parse_table_data, parse_tables, parse_users,
};

use iced::Element;
use iced::widget::{column, text};

use super::{App, Connection, ContentTab, DatabaseKind, Message, Palette, WorkspaceTab, WorkspaceTabKind};

pub fn content(
    app: &App,
    palette: Palette,
    tab: &WorkspaceTab,
) -> Element<'static, Message> {
    match &tab.kind {
        WorkspaceTabKind::Overview => overview_tab(app, palette),
        WorkspaceTabKind::TableData {
            connection_id,
            table_name,
        } => table_data::render_table_data(app, palette, *connection_id, table_name),
        WorkspaceTabKind::QueryEditor {
            connection_id,
            initial_sql,
        } => query_editor::render_query_editor(app, palette, *connection_id, initial_sql.clone()),
        WorkspaceTabKind::SavedQueryList { connection_id } => {
            saved_queries::render_saved_queries(app, palette, *connection_id)
        }
        WorkspaceTabKind::SavedFunctionList { connection_id } => {
            saved_functions::render_saved_functions(app, palette, *connection_id)
        }
    }
}

fn overview_tab(
    app: &App,
    palette: Palette,
) -> Element<'static, Message> {
    let active_tab = app.active_tab();

    if let Some(connection) = app.selected_connection().and_then(|id| app.connection(id)) {
        match connection.kind {
            DatabaseKind::Mysql => overview::render(app, active_tab, connection, palette),
            DatabaseKind::Redis => {
                let state = app.redis_state(connection.id);
                redis::render(state, active_tab, connection, palette)
            }
            other => unsupported_database(other, active_tab, palette, connection),
        }
    } else {
        no_connection_selected(active_tab, palette)
    }
}

fn no_connection_selected(
    active_tab: ContentTab,
    palette: Palette,
) -> Element<'static, Message> {
    column![
        text(format!("请选择连接以查看{}。", active_tab.title()))
            .size(18)
            .color(palette.text),
        text("在左侧连接列表中双击一个连接或创建新的连接。")
            .color(palette.text_muted)
            .size(15),
    ]
    .spacing(8)
    .into()
}

fn unsupported_database(
    kind: DatabaseKind,
    active_tab: ContentTab,
    palette: Palette,
    connection: &Connection,
) -> Element<'static, Message> {
    let name = connection.name.clone();
    column![
        text(format!("{} 的{}视图暂未就绪。", name, active_tab.title()))
            .size(18)
            .color(palette.text),
        text(format!(
            "我们正在为 {} 数据库补充 {} 功能。",
            kind.display_name(),
            active_tab.title()
        ))
        .color(palette.text_muted)
        .size(15),
    ]
    .spacing(8)
    .into()
}
