use gpui::{prelude::*, *};
use gpui_component::{
    Root, StyledExt, init,
    scroll::{ScrollableElement, ScrollbarShow},
    theme::Theme,
};

struct GridExample {}

impl GridExample {
    pub fn new(
        _window: &mut Window,
        _cx: &mut Context<GridExample>,
    ) -> Self {
        Self {}
    }
}

impl Render for GridExample {
    fn render(
        &mut self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> impl IntoElement {
        // 预定义颜色数组
        let colors = vec![
            hsla(0.0 / 360.0, 0.7, 0.6, 1.0),   // 红色
            hsla(30.0 / 360.0, 0.7, 0.6, 1.0),  // 橙色
            hsla(60.0 / 360.0, 0.7, 0.6, 1.0),  // 黄色
            hsla(120.0 / 360.0, 0.7, 0.6, 1.0), // 绿色
            hsla(180.0 / 360.0, 0.7, 0.6, 1.0), // 青色
            hsla(240.0 / 360.0, 0.7, 0.6, 1.0), // 蓝色
            hsla(270.0 / 360.0, 0.7, 0.6, 1.0), // 紫色
            hsla(300.0 / 360.0, 0.7, 0.6, 1.0), // 品红
            hsla(15.0 / 360.0, 0.8, 0.5, 1.0),  // 深橙
            hsla(45.0 / 360.0, 0.8, 0.5, 1.0),  // 金黄
            hsla(90.0 / 360.0, 0.6, 0.5, 1.0),  // 草绿
            hsla(150.0 / 360.0, 0.6, 0.5, 1.0), // 翠绿
        ];

        div()
            .overflow_scrollbar()
            .child(
                div()
                    .id("grid")
                    .size_full()
                    .grid()
                    .grid_cols(4)
                    .p_4()
                    .gap_4()
                    .children((0..20).map(|i| {
                        let color = colors[i % colors.len()];
                        div()
                            .id(("grid-item", i))
                            .flex()
                            .items_center()
                            .justify_center()
                            .w_full()
                            .h(px(150.))
                            .rounded_lg()
                            .bg(color)
                            .border_1()
                            .border_color(hsla(0.0, 0.0, 1.0, 0.1))
                            .shadow_md()
                            .child(
                                div()
                                    .text_xl()
                                    .font_bold()
                                    .text_color(hsla(0.0, 0.0, 1.0, 1.0))
                                    .child(format!("项目 {}", i + 1)),
                            )
                    })),
            )
    }
}

fn main() {
    let app = Application::new();
    app.run(|cx: &mut App| {
        init(cx);

        cx.activate(true);
        cx.on_window_closed(|cx| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();

        let theme = Theme::global_mut(cx);
        theme.scrollbar_show = ScrollbarShow::Always;

        let window_size = size(px(1280.), px(800.));
        let window_bounds = Bounds::centered(None, window_size, cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(window_bounds)),
                titlebar: Some(TitlebarOptions {
                    title: None,
                    appears_transparent: false,
                    traffic_light_position: None,
                }),
                ..Default::default()
            },
            |window, cx| {
                let view = cx.new(|cx| GridExample::new(window, cx));
                cx.new(|cx| Root::new(view, window, cx))
            },
        )
        .expect("failed to open window")
        .update(cx, |_, window, _| {
            window.activate_window();
        })
        .expect("failed to update window");
    });
}
