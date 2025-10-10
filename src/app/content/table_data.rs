use iced::widget::{column, container, row, scrollable, text};
use iced::{Alignment, Background, Element, Length, Shadow};

use super::{App, LoadState, Message, MysqlTableData, Palette};

pub fn render_table_data(
    app: &App,
    palette: Palette,
    connection_id: usize,
    table_name: &str,
) -> Element<'static, Message> {
    let state = app
        .mysql_state(connection_id)
        .and_then(|s| s.table_data.get(table_name))
        .cloned()
        .unwrap_or(LoadState::Idle);

    match state {
        LoadState::Idle | LoadState::Loading => loading_view("正在加载表数据…", palette),
        LoadState::Error(err) => error_view(&err, palette),
        LoadState::Ready(data) => render_data_view(data, palette, table_name),
    }
}

fn render_data_view(
    data: MysqlTableData,
    palette: Palette,
    table_name: &str,
) -> Element<'static, Message> {
    if data.rows.is_empty() {
        return centered_message(
            vec![
                format!("{table_name} 暂无可展示的数据。"),
                "仅显示前 100 行数据。".into(),
            ],
            palette,
        );
    }

    let mut rows_view = column![header_row(&data.columns, palette)];

    for row in data.rows {
        rows_view = rows_view.push(data_row(&row, palette));
    }

    let note = text("仅显示前 100 行数据。").size(12).color(palette.text_muted);

    let content = column![scrollable(rows_view.spacing(8)).height(Length::Fill), note]
        .spacing(12)
        .height(Length::Fill);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_| container::Style {
            background: Some(Background::Color(palette.surface_muted)),
            text_color: Some(palette.text),
            border: iced::border::Border {
                color: palette.border,
                width: 1.0,
                radius: 8.0.into(),
            },
            shadow: Shadow::default(),
        })
        .into()
}

fn header_row(
    columns: &[String],
    palette: Palette,
) -> Element<'static, Message> {
    let mut header = row![];

    for column_name in columns {
        header = header.push(
            container(
                text(column_name.clone())
                    .size(13)
                    .color(palette.text)
                    .width(Length::Fill),
            )
            .padding([6, 10])
            .width(Length::FillPortion(1))
            .style(move |_| container::Style {
                background: Some(Background::Color(palette.surface)),
                text_color: Some(palette.text),
                border: iced::border::Border {
                    color: palette.border,
                    width: 1.0,
                    radius: 6.0.into(),
                },
                shadow: Shadow::default(),
            }),
        );
    }

    header.spacing(8).align_y(Alignment::Center).into()
}

fn data_row(
    row_data: &[String],
    palette: Palette,
) -> Element<'static, Message> {
    let mut view = row![];

    for cell in row_data {
        view = view.push(
            container(text(cell.clone()).size(13).color(palette.text).width(Length::Fill))
                .padding([6, 10])
                .width(Length::FillPortion(1))
                .style(move |_| container::Style {
                    background: Some(Background::Color(palette.surface)),
                    text_color: Some(palette.text),
                    border: iced::border::Border {
                        color: palette.border,
                        width: 1.0,
                        radius: 6.0.into(),
                    },
                    shadow: Shadow::default(),
                }),
        );
    }

    view.spacing(8).align_y(Alignment::Center).into()
}

fn loading_view(
    message: &'static str,
    palette: Palette,
) -> Element<'static, Message> {
    centered_message(vec![message.into()], palette)
}

fn error_view(
    message: &str,
    palette: Palette,
) -> Element<'static, Message> {
    centered_message(vec![format!("加载失败：{message}")], palette)
}

fn centered_message(
    lines: Vec<String>,
    palette: Palette,
) -> Element<'static, Message> {
    let mut content = column![];
    for line in lines {
        content = content.push(text(line).size(14).color(palette.text_muted));
    }

    container(content.spacing(6))
        .width(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}
