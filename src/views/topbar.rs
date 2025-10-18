use gpui::prelude::FluentBuilder as _;
use gpui::InteractiveElement as _;
use gpui::StatefulInteractiveElement as _;
use gpui::{
    px, AnyElement, Context, IntoElement, Length, ParentElement, SharedString, Styled,
    TextOverflow, Window,
};
use gpui_component::{
    button::{Button, ButtonVariants as _},
    h_flex, ActiveTheme as _, Icon, Sizable as _, Size,
};

use crate::views::SqlerApp;

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
    let active = app.active_tab;

    let mut scroller =
        h_flex()
            .gap(px(6.))
            .px(px(4.))
            .flex_1()
            .min_w_0()
            .children(app.tabs.iter().map(move |tab| {
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
                            .text_left()
                            .whitespace_nowrap()
                            .overflow_hidden()
                            .text_overflow(TextOverflow::Truncate(Default::default()))
                            .child(tab.title.clone()),
                    );

                if tab.closable {
                    pill = pill.child(
                        Button::new(("close-tab", tab_id.raw()))
                            .ghost()
                            .compact()
                            .xsmall()
                            .tab_stop(false)
                            .icon(
                                Icon::default()
                                    .path("icons/close.svg")
                                    .with_size(Size::Small),
                            )
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.close_tab(tab_id, cx);
                            })),
                    );
                }

                {
                    let style = pill.style();
                    style.flex_grow = Some(0.);
                    style.flex_shrink = Some(1.);
                    style.flex_basis = Some(Length::Definite(px(240.).into()));
                    style.min_size.width = Some(Length::Definite(px(0.).into()));
                }

                pill.into_any_element()
            }));
    scroller.style().min_size.width = Some(Length::Definite(px(0.).into()));

    scroller.into_any_element()
}

fn controls(cx: &mut Context<SqlerApp>) -> gpui::Div {
    h_flex()
        .gap_5()
        .child(
            Button::new("header-new-source")
                .primary()
                .small()
                .label("新建数据源")
                .on_click(cx.listener(|this, _, window, cx| {
                    this.show_new_data_source_modal(window, cx);
                })),
        )
        .child(
            Button::new("toggle-theme")
                .ghost()
                .small()
                .label(if cx.theme().is_dark() {
                    "切换到亮色"
                } else {
                    "切换到暗色"
                })
                .on_click(cx.listener(|this, _, window, cx| {
                    this.toggle_theme(window, cx);
                })),
        )
}
