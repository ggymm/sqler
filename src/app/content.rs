mod mysql;

pub use mysql::{
    LoadState as MysqlLoadState, MysqlContentState, MysqlProcess, MysqlRoutine, MysqlTable, MysqlTableData,
    MysqlTableTab, MysqlUser, PROCESSLIST_SQL, ROUTINES_SQL, TABLES_SQL, TableMenuAction, USERS_SQL, parse_processlist,
    parse_routines, parse_table_data, parse_tables, parse_users,
};

use iced::widget::{column, container, text};
use iced::{Background, Color, Element, Length, Shadow};

use super::{App, Connection, ContentTab, DatabaseKind, Message, Palette};

pub fn content(
    app: &App,
    palette: Palette,
) -> Element<'_, Message> {
    column![
        container(tab_body(app, palette))
            .padding(24)
            .style(move |_| container::Style {
                background: Some(Background::Color(palette.surface)),
                text_color: Some(palette.text),
                border: iced::border::Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 0.0.into(),
                },
                shadow: Shadow::default(),
            })
            .height(Length::Fill),
    ]
    .spacing(12)
    .height(Length::Fill)
    .into()
}

fn tab_body(
    app: &App,
    palette: Palette,
) -> Element<'static, Message> {
    let active_tab = app.active_tab();

    if let Some(connection) = app.selected_connection().and_then(|id| app.connection(id)) {
        match connection.kind {
            DatabaseKind::Mysql => mysql::render(app, active_tab, connection, palette),
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
