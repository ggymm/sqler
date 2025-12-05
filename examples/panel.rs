use gpui::{prelude::*, *};
use gpui_component::{Root, init, scroll::ScrollbarShow, theme::Theme};

struct PanelExample {}

impl PanelExample {
    pub fn new(
        _window: &mut Window,
        _cx: &mut Context<PanelExample>,
    ) -> Self {
        Self {}
    }
}

impl Render for PanelExample {
    fn render(
        &mut self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div()
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
        theme.scrollbar_show = ScrollbarShow::Hover;

        let window_size = size(px(1280.), px(800.));
        let window_bounds = Bounds::centered(None, window_size, cx);
        cx.open_window(
            WindowOptions {
                kind: WindowKind::Floating,
                window_bounds: Some(WindowBounds::Windowed(window_bounds)),
                titlebar: Some(TitlebarOptions {
                    title: None,
                    appears_transparent: false,
                    traffic_light_position: None,
                }),
                ..Default::default()
            },
            |window, cx| {
                let view = cx.new(|cx| PanelExample::new(window, cx));
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
