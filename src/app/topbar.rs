use iced::widget::container;
use iced::widget::svg::Handle as SvgHandle;
use iced::widget::{Rule, button, horizontal_space, row, svg};
use iced::{Alignment, Background, Color, Element, Length, Theme};

use super::theme::{Palette, ThemeMode};
use super::{App, ContentTab, Message};

pub fn topbar(
    app: &App,
    palette: Palette,
) -> Element<'_, Message> {
    let divider = container(Rule::vertical(1).style(move |_| iced::widget::rule::Style {
        color: palette.border,
        width: 1,
        radius: 0.0.into(),
        fill_mode: iced::widget::rule::FillMode::Full,
    }))
    .height(Length::Fixed(28.0));

    let mut actions = row![
        icon_action_button(
            "assets/icons/new-conn.svg",
            "新建连接",
            Message::ShowNewConnectionDialog,
            palette,
        ),
        icon_action_button(
            "assets/icons/new-query.svg",
            "新建查询",
            Message::ShowNewQueryWorkspace,
            palette,
        ),
        divider,
    ]
    .spacing(12)
    .align_y(Alignment::Center);

    for tab in [
        ContentTab::Tables,
        ContentTab::Queries,
        ContentTab::Functions,
        ContentTab::Users,
    ] {
        actions = actions.push(icon_tab_button(tab, app.active_tab(), palette));
    }

    let theme_icon = match app.theme() {
        ThemeMode::Light => "assets/icons/theme-dark.svg",
        ThemeMode::Dark => "assets/icons/theme-light.svg",
    };

    row![
        actions,
        horizontal_space().width(Length::Fill),
        icon_action_button(theme_icon, "切换主题", Message::ToggleTheme, palette),
    ]
    .padding([12, 20])
    .align_y(Alignment::Center)
    .into()
}

fn icon_action_button<'a>(
    icon_path: &str,
    label: &'a str,
    message: Message,
    palette: Palette,
) -> Element<'a, Message> {
    let icon = svg::<Theme>(SvgHandle::from_path(icon_path)).width(24).height(24);

    button(
        row![icon, iced::widget::text(label).color(palette.text).size(16)]
            .spacing(8)
            .align_y(Alignment::Center),
    )
    .padding([8, 12])
    .style(move |_, status| {
        use iced::widget::button::Status;

        let mut style = button::Style::default();
        style.border = iced::border::Border {
            color: palette.border,
            width: 1.0,
            radius: 6.0.into(),
        };

        style.text_color = palette.text;
        style.background = Some(Background::Color(match status {
            Status::Hovered => palette.surface_muted,
            Status::Pressed => palette.surface,
            _ => Color::TRANSPARENT,
        }));

        style
    })
    .on_press(message)
    .into()
}

fn icon_tab_button(
    tab: ContentTab,
    active_tab: ContentTab,
    palette: Palette,
) -> Element<'static, Message> {
    let is_active = active_tab == tab;
    let icon = svg::<Theme>(SvgHandle::from_path(tab.icon_path())).width(24).height(24);
    let label = iced::widget::text(tab.title()).size(16);

    button(row![icon, label].spacing(8).align_y(Alignment::Center))
        .padding([8, 18])
        .style(move |_, status| {
            use iced::widget::button::Status;

            let mut style = button::Style::default();
            style.border = iced::border::Border {
                color: if is_active { palette.accent } else { palette.border },
                width: if is_active { 1.5 } else { 1.0 },
                radius: 8.0.into(),
            };

            let background = if is_active {
                palette.accent_soft
            } else if matches!(status, Status::Hovered) {
                palette.surface_muted
            } else {
                Color::TRANSPARENT
            };

            style.background = Some(Background::Color(background));
            style.text_color = if is_active { palette.accent } else { palette.text };

            style
        })
        .on_press(Message::SelectContentTab(tab))
        .into()
}
