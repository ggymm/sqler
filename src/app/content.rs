mod mysql;

use iced::widget::{column, container, scrollable, text};
use iced::{Background, Color, Element, Length, Shadow};

use super::{App, Connection, ContentTab, DatabaseKind, Message, Palette};

pub fn content(
    app: &App,
    palette: Palette,
) -> Element<'_, Message> {
    let title = text(app.active_tab().title()).size(22).color(palette.text);

    let intro = text(match app.active_tab() {
        ContentTab::Tables => "在此浏览所选数据库的表结构，并进行建表或结构修改。",
        ContentTab::Queries => "创建和管理查询，支持多标签页执行 SQL。",
        ContentTab::Functions => "查看数据库函数或存储过程，支持编辑与调试。",
        ContentTab::Users => "管理数据库用户、角色以及权限设置。",
    })
    .color(palette.text_muted)
    .size(15);

    let detail = current_connection_section(app, palette);

    column![
        container(column![title, intro, detail].spacing(12))
            .padding([18, 24])
            .style(move |_| container::Style {
                background: Some(Background::Color(palette.surface)),
                text_color: Some(palette.text),
                border: iced::border::Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 0.0.into(),
                },
                shadow: Shadow::default(),
            }),
        scrollable(
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
                }),
        )
        .height(Length::Fill),
    ]
    .spacing(12)
    .height(Length::Fill)
    .into()
}

fn current_connection_section(
    app: &App,
    palette: Palette,
) -> Element<'_, Message> {
    if let Some(selected) = app.selected_connection() {
        if let Some(connection) = app.connection(selected) {
            return column![
                text(format!("当前连接：{}", connection.name))
                    .color(palette.text)
                    .size(17),
                text(format!("连接信息：{}", connection.summary()))
                    .color(palette.text_muted)
                    .size(14),
            ]
            .spacing(6)
            .into();
        }
    }

    text("请选择或创建一个数据库连接以开始。")
        .color(palette.text_muted)
        .size(14)
        .into()
}

fn tab_body(
    app: &App,
    palette: Palette,
) -> Element<'static, Message> {
    let active_tab = app.active_tab();

    if let Some(connection) = app.selected_connection().and_then(|id| app.connection(id)) {
        match connection.kind {
            DatabaseKind::Mysql => mysql::render(active_tab, connection, palette),
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
