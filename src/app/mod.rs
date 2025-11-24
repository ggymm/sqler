use std::collections::HashMap;

use gpui::{prelude::*, *};
use gpui_component::{
    button::{Button, ButtonVariants},
    theme::{Theme, ThemeMode},
    ActiveTheme, Root, Sizable, Size,
};

use crate::{
    app::{
        comps::{comp_id, icon_close},
        create::CreateWindow,
        transfer::{ExportWindow, ImportWindow},
        workspace::WorkspaceState,
    },
    cache::CacheApp,
    model::DataSource,
};

mod comps;
mod create;
mod transfer;
mod workspace;

#[derive(Clone)]
pub enum WindowKind {
    Create(Option<DataSource>),
    Import(DataSource),
    Export(DataSource),
}

impl WindowKind {
    fn tag(&self) -> &'static str {
        match self {
            WindowKind::Create(_) => "create",
            WindowKind::Import(_) => "import",
            WindowKind::Export(_) => "export",
        }
    }
}

pub enum TabView {
    Home,
    Workspace(WorkspaceState),
}

pub struct TabState {
    pub id: String,
    pub view: TabView,
    pub title: SharedString,
    pub closable: bool,
}

impl TabState {
    fn home() -> Self {
        Self {
            id: "home".to_string(),
            view: TabView::Home,
            title: SharedString::from("首页"),
            closable: false,
        }
    }

    fn workspace(
        source: DataSource,
        window: &mut Window,
        cx: &mut Context<SqlerApp>,
    ) -> Self {
        let id = source.id.clone();
        let title = SharedString::from(source.name.clone());
        let workspace = WorkspaceState::new(source, window, cx);
        Self {
            id,
            view: TabView::Workspace(workspace),
            title,
            closable: true,
        }
    }
}

pub struct SqlerApp {
    pub tabs: Vec<TabState>,
    pub active_tab: String,

    pub cache: CacheApp,
    pub windows: HashMap<String, WindowHandle<Root>>,
}

impl SqlerApp {
    pub fn new(
        _window: &mut Window,
        cx: &mut Context<SqlerApp>,
    ) -> Self {
        Theme::change(ThemeMode::Light, None, cx);

        let cache = match CacheApp::init() {
            Ok(cache) => cache,
            Err(e) => panic!("{}", e),
        };

        Self {
            tabs: vec![TabState::home()],
            active_tab: "home".to_string(),

            cache,

            windows: HashMap::new(),
        }
    }

    pub fn close_tab(
        &mut self,
        tab_id: &str,
        cx: &mut Context<SqlerApp>,
    ) {
        if let Some(index) = self.tabs.iter().position(|tab| tab.id == tab_id) {
            if !self.tabs[index].closable {
                return;
            }
            self.tabs.remove(index);
            if self.tabs.is_empty() {
                return;
            }

            if self.active_tab == tab_id {
                let fallback = if index == 0 { 0 } else { index - 1 };
                self.active_tab = self.tabs[fallback].id.clone();
            }
            cx.notify();
        }
    }

    pub fn active_tab(
        &mut self,
        tab_id: &str,
        cx: &mut Context<SqlerApp>,
    ) {
        if self.tabs.iter().any(|tab| tab.id == tab_id) {
            self.active_tab = tab_id.to_string();
            cx.notify();
        }
    }

    pub fn create_tab(
        &mut self,
        tab_id: &str,
        window: &mut Window,
        cx: &mut Context<SqlerApp>,
    ) {
        if let Some(exist) = self.tabs.iter().find(|tab| {
            // rustfmt::skip
            matches!(&tab.view, TabView::Workspace(_)) && tab.id == tab_id
        }) {
            self.active_tab = exist.id.clone();
            cx.notify();
            return;
        }

        if let Some(source) = self.cache.sources().iter().find(|source| source.id == tab_id).cloned() {
            self.tabs.push(TabState::workspace(source, window, cx));
            self.active_tab = tab_id.to_string();
            cx.notify();
        }
    }

    pub fn toggle_theme(
        &mut self,
        window: &mut Window,
        cx: &mut Context<SqlerApp>,
    ) {
        let next_mode = if cx.theme().is_dark() {
            ThemeMode::Light
        } else {
            ThemeMode::Dark
        };
        Theme::change(next_mode, Some(window), cx);
        cx.notify();
    }

    pub fn close_window(
        &mut self,
        tag: &str,
    ) {
        self.windows.remove(tag);
    }

    pub fn create_window(
        &mut self,
        kind: WindowKind,
        cx: &mut Context<SqlerApp>,
    ) {
        let tag = kind.tag();

        if let Some(handle) = self.windows.get(tag) {
            if handle.update(cx, |_, window, _| {
                window.activate_window();
            }).is_ok() {
                return;
            }
            self.windows.remove(tag);
        }

        let title = match kind {
            WindowKind::Create(Some(_)) => "编辑数据源",
            WindowKind::Create(None) => "新建数据源",
            WindowKind::Import(_) => "数据导入",
            WindowKind::Export(_) => "数据导出",
        };
        let bounds = Bounds {
            size: size(px(1280.), px(720.)),
            origin: point(px(0.), px(0.)),
        };
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            is_minimizable: false,
            ..Default::default()
        };
        let parent = cx.weak_entity();
        let result = cx.open_window(options, move |window, cx| {
            let view: AnyView = match kind {
                WindowKind::Create(source) => cx
                    .new(|cx| {
                        // rustfmt::skip
                        CreateWindow::new(parent.clone(), source.as_ref(), window, cx)
                    })
                    .into(),
                WindowKind::Import(source) => cx
                    .new(|cx| {
                        // rustfmt::skip
                        ImportWindow::new(parent.clone(), source.clone(), window, cx)
                    })
                    .into(),
                WindowKind::Export(source) => cx
                    .new(|cx| {
                        // rustfmt::skip
                        ExportWindow::new(parent.clone(), source.clone(), window, cx)
                    })
                    .into(),
            };
            cx.new(|cx| Root::new(view, window, cx))
        });
        match result {
            Ok(handle) => {
                let _ = handle.update(cx, |_, modal_window, _| {
                    modal_window.set_window_title(title);
                });
                self.windows.insert(tag.to_string(), handle);
            }
            Err(err) => {
                eprintln!("failed to open window: {err:?}");
            }
        }
    }
}

impl Render for SqlerApp {
    fn render(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let theme = cx.theme();
        let active = &self.active_tab;
        div()
            .flex()
            .flex_col()
            .relative()
            .size_full()
            .min_w_0()
            .min_h_0()
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .p_2()
                    .gap_4()
                    .border_b_1()
                    .border_color(theme.border)
                    .child(
                        div()
                            .flex()
                            .flex_1()
                            .flex_row()
                            .px_2()
                            .gap_2()
                            .min_w_0()
                            .children(self.tabs.iter().map(|tab| {
                                let tab_id = tab.id.clone();
                                let tab_active = &tab_id == active;

                                let mut item = div()
                                    .id(comp_id(["main-tab", &tab_id]))
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .justify_center()
                                    .px_3()
                                    .py_1()
                                    .gap_2()
                                    .border_1()
                                    .border_color(theme.border)
                                    .rounded_lg()
                                    .cursor_pointer()
                                    .when(tab_active, |this| {
                                        this.bg(theme.tab_active).text_color(theme.tab_active_foreground)
                                    })
                                    .when(!tab_active, |this| {
                                        this.bg(theme.tab_bar).text_color(theme.muted_foreground)
                                    })
                                    .on_click(cx.listener({
                                        let tab_id = tab_id.clone();
                                        move |this, _, _, cx| {
                                            this.active_tab(&tab_id, cx);
                                        }
                                    }))
                                    .child(
                                        div()
                                            .flex_1()
                                            .min_w_0()
                                            .overflow_hidden()
                                            .whitespace_nowrap()
                                            .child(tab.title.clone()),
                                    );

                                if tab.closable {
                                    item = item.child(
                                        Button::new(comp_id(["close-tab", &tab_id]))
                                            .ghost()
                                            .xsmall()
                                            .compact()
                                            .tab_stop(false)
                                            .icon(icon_close().with_size(Size::Small))
                                            .on_click(cx.listener({
                                                let tab_id = tab_id.clone();
                                                move |this, _, _, cx| {
                                                    this.close_tab(&tab_id, cx);
                                                }
                                            })),
                                    );
                                }

                                {
                                    let style = item.style();
                                    style.flex_grow = Some(0.);
                                    style.flex_shrink = Some(1.);
                                    style.flex_basis = Some(Length::Definite(px(200.).into()));
                                    style.min_size.width = Some(Length::Definite(px(0.).into()));
                                }

                                item.into_any_element()
                            })),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap_5()
                            .child(Button::new("header-new-source").label("新建数据源").outline().on_click(
                                cx.listener(|this, _, _, cx| {
                                    this.create_window(WindowKind::Create(None), cx);
                                }),
                            ))
                            .child(
                                Button::new("toggle-theme")
                                    .label(if theme.is_dark() {
                                        "切换到亮色"
                                    } else {
                                        "切换到暗色"
                                    })
                                    .outline()
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.toggle_theme(window, cx);
                                    })),
                            ),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .size_full()
                    .min_w_0()
                    .min_h_0()
                    .child(workspace::render(self, window, cx)),
            )
            .into_any_element()
    }
}
