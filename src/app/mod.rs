use gpui::{prelude::*, *};
use gpui_component::{
    button::{Button, ButtonVariants},
    theme::{Theme, ThemeMode},
    ActiveTheme, Root, Sizable, Size,
};
use uuid::Uuid;

use crate::{
    app::{
        comps::{comp_id, icon_close},
        create::CreateWindow,
        transfer::TransferWindow,
        workspace::WorkspaceState,
    },
    cache::CacheApp,
    model::{DataSource, DataSourceKind, DataSourceOptions, MySQLOptions},
};

mod comps;
mod create;
mod transfer;
mod workspace;

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

    fn is_workspace(
        &self,
        id: &str,
    ) -> bool {
        matches!(&self.view, TabView::Workspace(_)) && self.id == id
    }

    fn workspace(
        meta: DataSource,
        window: &mut Window,
        cx: &mut Context<SqlerApp>,
    ) -> Self {
        let id = meta.id.clone();
        let title = SharedString::from(meta.name.clone());
        let workspace = WorkspaceState::new(meta, window, cx);
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
    pub sources: Vec<DataSource>,

    pub create_window: Option<WindowHandle<Root>>,
    pub transfer_window: Option<WindowHandle<Root>>,
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

        let sources = seed_sources();

        Self {
            tabs: vec![TabState::home()],
            active_tab: "home".to_string(),

            cache,
            sources,

            create_window: None,
            transfer_window: None,
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
        if let Some(existing) = self.tabs.iter().find(|tab| tab.is_workspace(tab_id)) {
            self.active_tab = existing.id.clone();
            cx.notify();
            return;
        }

        if let Some(meta) = self.sources.iter().find(|meta| meta.id == tab_id).cloned() {
            self.tabs.push(TabState::workspace(meta, window, cx));
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

    pub fn close_create_window(&mut self) {
        self.create_window = None;
    }

    pub fn display_create_window(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<SqlerApp>,
    ) {
        if let Some(handle) = &self.create_window {
            let _ = handle.update(cx, |_, create_window, _| {
                create_window.activate_window();
            });
            return;
        }

        let wsize = size(px(640.), px(560.));
        let options = WindowOptions {
            kind: WindowKind::Floating,
            window_bounds: Some(WindowBounds::Windowed(Bounds::centered(None, wsize, cx))),
            // window_min_size: Some(gpui::Size {
            //     width: wsize.width,
            //     height: wsize.height,
            // }),
            is_minimizable: false,
            ..Default::default()
        };

        let parent = cx.weak_entity();
        match cx.open_window(options, move |modal_window, app_cx| {
            let parent = parent.clone();
            let view = app_cx.new(|cx| CreateWindow::new(parent.clone(), modal_window, cx));
            app_cx.new(|cx| Root::new(view.into(), modal_window, cx))
        }) {
            Ok(handle) => {
                let _ = handle.update(cx, |_, modal_window, _| {
                    modal_window.set_window_title("新建数据源");
                });
                self.create_window = Some(handle);
            }
            Err(err) => {
                eprintln!("failed to open create data source window: {err:?}");
            }
        }
    }

    pub fn close_transfer_window(&mut self) {
        self.transfer_window = None;
    }

    pub fn display_transfer_window(
        &mut self,
        datasource: DataSource,
        tables: Vec<SharedString>,
        _window: &mut Window,
        cx: &mut Context<SqlerApp>,
    ) {
        if let Some(handle) = &self.transfer_window {
            let _ = handle.update(cx, |_, transfer_window, _| {
                transfer_window.activate_window();
            });
            return;
        }

        let wsize = size(px(640.), px(480.));
        let options = WindowOptions {
            kind: WindowKind::Floating,
            window_bounds: Some(WindowBounds::Windowed(Bounds::centered(None, wsize, cx))),
            is_minimizable: false,
            ..Default::default()
        };

        let parent = cx.weak_entity();
        match cx.open_window(options, move |modal_window, app_cx| {
            let parent = parent.clone();
            let view = app_cx.new(|cx| TransferWindow::new(datasource, tables, parent.clone(), modal_window, cx));
            app_cx.new(|cx| Root::new(view.into(), modal_window, cx))
        }) {
            Ok(handle) => {
                let _ = handle.update(cx, |_, modal_window, _| {
                    modal_window.set_window_title("数据传输");
                });
                self.transfer_window = Some(handle);
            }
            Err(err) => {
                eprintln!("failed to open transfer window: {err:?}");
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

                                let tab_id_for_click = tab_id.clone();
                                let tab_id_for_close = tab_id.clone();

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
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.active_tab(&tab_id_for_click, cx);
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
                                            .on_click(cx.listener(move |this, _, _, cx| {
                                                this.close_tab(&tab_id_for_close, cx);
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
                            .child(Button::new("header-new-source").outline().label("新建数据源").on_click(
                                cx.listener(|this, _, window, cx| {
                                    this.display_create_window(window, cx);
                                }),
                            ))
                            .child(
                                Button::new("toggle-theme")
                                    .outline()
                                    .label(if theme.is_dark() {
                                        "切换到亮色"
                                    } else {
                                        "切换到暗色"
                                    })
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

fn seed_sources() -> Vec<DataSource> {
    vec![DataSource {
        id: Uuid::new_v4().to_string(),
        name: "测试数据库".to_string(),
        kind: DataSourceKind::MySQL,
        options: DataSourceOptions::MySQL(MySQLOptions {
            host: "127.0.0.1".into(),
            port: 3306,
            username: "root".into(),
            password: "root".into(),
            database: "qnt_robot_prod".into(),
            use_tls: false,
        }),
    }]
}
