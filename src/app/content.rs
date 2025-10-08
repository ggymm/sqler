use iced::widget::{column, container, scrollable, text};
use iced::{Background, Color, Element, Length, Shadow};

use super::theme::Palette;
use super::{App, ContentTab, Message};

pub fn content_panel(
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
            .style(move |_| iced::widget::container::Style {
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
            container(tab_body(app.active_tab(), palette))
                .padding(24)
                .style(move |_| iced::widget::container::Style {
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
                text(format!("连接信息：{}", connection.summary))
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
    active_tab: ContentTab,
    palette: Palette,
) -> Element<'static, Message> {
    match active_tab {
        ContentTab::Tables => column![
            text("表管理功能即将上线").size(18).color(palette.text),
            text("这里将展示表列表、结构预览以及建模工具。")
                .color(palette.text_muted)
                .size(15),
        ]
        .spacing(8)
        .into(),
        ContentTab::Queries => column![
            text("查询编辑器").size(18).color(palette.text),
            text("支持多标签查询、语法高亮与自动完成等特性（开发中）。")
                .color(palette.text_muted)
                .size(15),
        ]
        .spacing(8)
        .into(),
        ContentTab::Functions => column![
            text("函数与存储过程").size(18).color(palette.text),
            text("管理数据库函数、触发器与存储过程，并支持分版本控制。")
                .color(palette.text_muted)
                .size(15),
        ]
        .spacing(8)
        .into(),
        ContentTab::Users => column![
            text("用户与权限").size(18).color(palette.text),
            text("在此配置数据库用户、角色以及权限策略。")
                .color(palette.text_muted)
                .size(15),
        ]
        .spacing(8)
        .into(),
    }
}
