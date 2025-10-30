use std::collections::HashMap;

use gpui::{prelude::*, *};
use gpui_component::{
    button::{Button, ButtonVariants},
    theme::{Theme, ThemeMode},
    ActiveTheme, Icon, Root, Sizable, Size,
};
use serde_json::json;
use uuid::Uuid;

use crate::{
    app::{
        create::{CreateState, CreateWindow},
        workspace::WorkspaceState,
    },
    cache::CacheApp,
    option::{DataSource, DataSourceKind, DataSourceOptions, MySQLOptions},
};

mod comps;
mod create;
mod workspace;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct TabId(u64);

impl TabId {
    pub fn raw(self) -> u64 {
        self.0
    }

    pub fn next(c: &mut u64) -> Self {
        let id = *c;
        *c += 1;
        TabId(id)
    }
}

pub enum TabView {
    Home,
    Workspace(WorkspaceState),
}

pub struct TabState {
    pub id: TabId,
    pub view: TabView,
    pub title: SharedString,
    pub closable: bool,
}

impl TabState {
    fn home(id: TabId) -> Self {
        Self {
            id,
            view: TabView::Home,
            title: SharedString::from("首页"),
            closable: false,
        }
    }

    fn data_source(
        id: TabId,
        meta: DataSource,
        window: &mut Window,
        cx: &mut Context<SqlerApp>,
    ) -> Self {
        let title = SharedString::from(meta.name.clone());
        let workspace = WorkspaceState::new(meta, window, cx);
        Self {
            id,
            view: TabView::Workspace(workspace),
            title,
            closable: true,
        }
    }

    fn is_data_source(
        &self,
        id: &str,
    ) -> bool {
        matches!(&self.view, TabView::Workspace(state) if state.id() == id)
    }
}

pub struct SqlerApp {
    pub tabs: Vec<TabState>,
    pub next_tab: u64,
    pub active_tab: TabId,
    pub cache: CacheApp,
    pub create_window: Option<WindowHandle<Root>>,

    pub sources: Vec<DataSource>,
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

        let saved_sources = seed_sources();
        let mut next_tab = 1;
        let home_id = TabId::next(&mut next_tab);

        Self {
            tabs: vec![TabState::home(home_id)],
            active_tab: home_id,
            next_tab,

            cache,
            sources: saved_sources,
            create_window: None,
        }
    }

    pub fn show_new_data_source_modal(
        &mut self,
        window: &mut Window,
        cx: &mut Context<SqlerApp>,
    ) {
        if let Some(handle) = &self.create_window {
            let _ = handle.update(cx, |_, modal_window, _| {
                modal_window.activate_window();
            });
            return;
        }

        let state = CreateState::new(window, cx);
        let parent = cx.weak_entity();
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                None,
                size(px(640.), px(560.)),
                cx,
            ))),
            kind: WindowKind::Floating,
            is_resizable: true,
            is_movable: true,
            is_minimizable: false,
            ..Default::default()
        };

        match cx.open_window(options, move |modal_window, app_cx| {
            let parent = parent.clone();
            let view = app_cx.new(|cx| CreateWindow::new(state, parent.clone(), modal_window, cx));
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

    pub fn clear_new_data_source_window(&mut self) {
        self.create_window = None;
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

    pub fn open_data_source_tab(
        &mut self,
        id: &str,
        window: &mut Window,
        cx: &mut Context<SqlerApp>,
    ) {
        if let Some(existing) = self.tabs.iter().find(|tab| tab.is_data_source(id)) {
            self.active_tab = existing.id;
            cx.notify();
            return;
        }

        if let Some(meta) = self.sources.iter().find(|meta| meta.id == id).cloned() {
            let id = TabId::next(&mut self.next_tab);
            self.tabs.push(TabState::data_source(id, meta, window, cx));
            self.active_tab = id;
            cx.notify();
        }
    }

    pub fn close_tab(
        &mut self,
        tab_id: TabId,
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
                self.active_tab = self.tabs[fallback].id;
            }
            cx.notify();
        }
    }

    pub fn set_active_tab(
        &mut self,
        tab_id: TabId,
        cx: &mut Context<SqlerApp>,
    ) {
        if self.tabs.iter().any(|tab| tab.id == tab_id) {
            self.active_tab = tab_id;
            cx.notify();
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
        let active = self.active_tab;
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
                                let tab_id = tab.id;
                                let tab_active = tab_id == active;

                                let mut item = div()
                                    .id(("main-tab-{}", tab_id.raw()))
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
                                        this.set_active_tab(tab_id, cx);
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
                                        Button::new(("close-tab", tab_id.raw()))
                                            .ghost()
                                            .xsmall()
                                            .compact()
                                            .tab_stop(false)
                                            .icon(Icon::default().path("icons/close.svg").with_size(Size::Small))
                                            .on_click(cx.listener(move |this, _, _, cx| {
                                                this.close_tab(tab_id, cx);
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
                                    this.show_new_data_source_modal(window, cx);
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
        desc: "用于测试应用功能的测试数据库".to_string(),
        kind: DataSourceKind::MySQL,
        options: DataSourceOptions::MySQL(MySQLOptions {
            host: "127.0.0.1".into(),
            port: 3306,
            username: "root".into(),
            password: Some("root".into()),
            database: "qnt_robot_prod".into(),
            charset: Some("utf8mb4".into()),
            use_tls: false,
        }),
        extras: Some(HashMap::from([(
            "tables".to_string(),
            json!(["orders", "order_items", "users", "regions"]),
        )])),
    }]
}
