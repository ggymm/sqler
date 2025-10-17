use gpui::prelude::FluentBuilder as _;
use gpui::{
    px, AnyElement, Context, InteractiveElement, IntoElement, Length, ParentElement, SharedString,
    Styled, Window,
};
use gpui::StatefulInteractiveElement as _;
use gpui_component::{
    button::{Button, ButtonVariants as _},
    h_flex,
    ActiveTheme as _,
    Icon,
    Size,
    Sizable as _,
};

use crate::SqlerApp;

pub fn render(app: &mut SqlerApp, _window: &mut Window, cx: &mut Context<SqlerApp>) -> gpui::Div {
    let tabs = tab_scroller(app, cx);
    let tabs_container = gpui::div().flex_1().min_w_0().child(tabs);
    let controls = controls(cx);

    h_flex()
        .w_full()
        .items_center()
        .gap(px(12.))
        .px(px(12.))
        .py(px(10.))
        .bg(cx.theme().background)
        .border_b_1()
        .border_color(cx.theme().border)
        .child(tabs_container)
        .child(controls)
}

fn tab_scroller(app: &mut SqlerApp, cx: &mut Context<SqlerApp>) -> AnyElement {
    let mut scroller = h_flex()
        .gap(px(6.))
        .px(px(4.))
        .flex_1()
        .min_w_0()
        .children(tab_buttons(app, cx));
    scroller.style().min_size.width = Some(Length::Definite(px(0.).into()));

    scroller.into_any_element()
}

fn tab_buttons(app: &mut SqlerApp, cx: &mut Context<SqlerApp>) -> Vec<AnyElement> {
    let active = app.active_tab;

    app.tabs
        .iter()
        .map(move |tab| {
            let tab_id = tab.id;
            let is_active = tab_id == active;

            let mut pill = h_flex()
                .gap(px(6.))
                .px(px(12.))
                .py(px(6.))
                .items_center()
                .rounded_tl(px(6.))
                .rounded_tr(px(6.))
                .cursor_pointer()
                .id(SharedString::from(format!("main-tab-{}", tab_id.raw())))
                .when(is_active, |this| {
                    this.bg(cx.theme().tab_active)
                        .text_color(cx.theme().tab_active_foreground)
                        .border_1()
                        .border_color(cx.theme().border)
                })
                .when(!is_active, |this| {
                    this.text_color(cx.theme().muted_foreground)
                        .border_1()
                        .border_color(cx.theme().border)
                        .bg(cx.theme().tab_bar)
                })
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.set_active_tab(tab_id, cx);
                }))
                .child(
                    gpui::div()
                        .flex_1()
                        .min_w_0()
                        .text_ellipsis()
                        .text_left()
                        .child(tab.title.clone()),
                );

            if tab.closable {
                pill = pill.child(
                    Button::new(("close-tab", tab_id.raw()))
                        .ghost()
                        .compact()
                        .xsmall()
                        .tab_stop(false)
                        .icon(Icon::default().path("icons/close.svg").with_size(Size::Small))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.close_tab(tab_id, cx);
                        })),
                );
            }

            {
                let style = pill.style();
                style.flex_grow = Some(1.);
                style.flex_shrink = Some(1.);
                style.min_size.width = Some(Length::Definite(px(0.).into()));
            }

            pill.into_any_element()
        })
        .collect()
}

fn controls(cx: &mut Context<SqlerApp>) -> gpui::Div {
    let new_source = Button::new("header-new-source")
        .primary()
        .small()
        .label("新建数据源")
        .on_click(cx.listener(|this, _, window, cx| {
            this.open_new_data_source(window, cx);
        }));

    let theme_toggle = Button::new("toggle-theme")
        .ghost()
        .small()
        .label(if cx.theme().is_dark() {
            "切换到亮色"
        } else {
            "切换到暗色"
        })
        .on_click(cx.listener(|this, _, window, cx| {
            this.toggle_theme(window, cx);
        }));

    h_flex()
        .gap(px(8.))
        .child(new_source)
        .child(theme_toggle)
}
