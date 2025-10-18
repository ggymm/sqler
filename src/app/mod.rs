use gpui::prelude::FluentBuilder;
use gpui::{
    div, AnyElement, AppContext as _, InteractiveElement, Length, ParentElement,
    StatefulInteractiveElement, Styled, TextOverflow,
};
use gpui::{
    px, size, Bounds, Context, IntoElement, Render, SharedString, Window, WindowBounds,
    WindowHandle, WindowKind, WindowOptions,
};
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::{h_flex, Icon, Root, Sizable, Size};
use gpui_component::{
    theme::{Theme, ThemeMode},
    ActiveTheme as _,
};

mod comps;
mod create;
mod workspace;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct TabId(u64);

impl TabId {
    pub fn next(counter: &mut u64) -> Self {
        let id = *counter;
        *counter += 1;
        TabId(id)
    }

    pub fn raw(self) -> u64 {
        self.0
    }
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum DatabaseKind {
    Postgres,
    MySql,
    Sqlite,
    SqlServer,
}

impl DatabaseKind {
    pub fn label(&self) -> &'static str {
        match self {
            DatabaseKind::Postgres => "Postgres",
            DatabaseKind::MySql => "MySQL",
            DatabaseKind::Sqlite => "SQLite",
            DatabaseKind::SqlServer => "SQL Server",
        }
    }

    pub fn all() -> &'static [DatabaseKind] {
        &[
            DatabaseKind::Postgres,
            DatabaseKind::MySql,
            DatabaseKind::Sqlite,
            DatabaseKind::SqlServer,
        ]
    }
}

#[derive(Clone)]
pub struct ConnectionPreset {
    pub host: SharedString,
    pub port: SharedString,
    pub database: SharedString,
    pub username: SharedString,
}

#[derive(Clone)]
pub struct DataSourceMeta {
    pub id: u64,
    pub name: SharedString,
    pub kind: DatabaseKind,
    pub description: SharedString,
    pub connection: ConnectionPreset,
    pub tables: Vec<SharedString>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct InnerTabId(u64);

impl InnerTabId {
    pub fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum InnerTabKind {
    Config,
}

#[derive(Clone)]
pub struct InnerTab {
    pub id: InnerTabId,
    pub title: SharedString,
    _kind: InnerTabKind,
    _closable: bool,
}

impl InnerTab {
    pub fn config() -> Self {
        Self {
            id: InnerTabId(0),
            title: SharedString::from("配置"),
            _kind: InnerTabKind::Config,
            _closable: false,
        }
    }
}

#[derive(Clone)]
pub struct NewDataSourceState {
    pub selected: Option<DatabaseKind>,
    pub postgres: create::postgres::PostgresState,
    pub mysql: create::mysql::MySqlState,
    pub sqlite: create::sqlite::SqliteState,
    pub sqlserver: create::sqlserver::SqlServerState,
}

impl NewDataSourceState {
    pub fn new(window: &mut Window, cx: &mut Context<SqlerApp>) -> Self {
        Self {
            selected: None,
            postgres: create::postgres::PostgresState::new(window, cx),
            mysql: create::mysql::MySqlState::new(window, cx),
            sqlite: create::sqlite::SqliteState::new(window, cx),
            sqlserver: create::sqlserver::SqlServerState::new(window, cx),
        }
    }
}

#[derive(Clone)]
pub struct DataSourceTabState {
    pub meta: DataSourceMeta,
    pub inner_tabs: Vec<InnerTab>,
    pub active_inner_tab: InnerTabId,
    pub tables: Vec<SharedString>,
}

impl DataSourceTabState {
    fn new(meta: DataSourceMeta) -> Self {
        let tables = meta.tables.clone();
        Self {
            meta,
            inner_tabs: vec![InnerTab::config()],
            active_inner_tab: InnerTabId(0),
            tables,
        }
    }
}

pub enum TabKind {
    Home,
    DataSource(DataSourceTabState),
}

pub struct TabState {
    pub id: TabId,
    pub title: SharedString,
    pub closable: bool,
    pub kind: TabKind,
}

impl TabState {
    fn home(id: TabId) -> Self {
        Self {
            id,
            title: SharedString::from("首页"),
            closable: false,
            kind: TabKind::Home,
        }
    }

    fn data_source(id: TabId, meta: DataSourceMeta) -> Self {
        let title = meta.name.clone();
        Self {
            id,
            title,
            closable: true,
            kind: TabKind::DataSource(DataSourceTabState::new(meta)),
        }
    }

    fn is_data_source(&self, id: u64) -> bool {
        matches!(&self.kind, TabKind::DataSource(state) if state.meta.id == id)
    }
}

pub struct SqlerApp {
    pub tabs: Vec<TabState>,
    pub active_tab: TabId,
    pub next_tab_id: u64,
    pub saved_sources: Vec<DataSourceMeta>,
    pub new_ds_window: Option<WindowHandle<Root>>,
}

impl SqlerApp {
    pub fn new(_window: &mut Window, _cx: &mut Context<SqlerApp>) -> Self {
        let saved_sources = seed_sources();
        let mut next_tab_id = 1;
        let home_id = TabId::next(&mut next_tab_id);

        Self {
            tabs: vec![TabState::home(home_id)],
            active_tab: home_id,
            next_tab_id,
            saved_sources,
            new_ds_window: None,
        }
    }

    pub fn show_new_data_source_modal(&mut self, window: &mut Window, cx: &mut Context<SqlerApp>) {
        if let Some(handle) = &self.new_ds_window {
            let _ = handle.update(cx, |_, modal_window, _| {
                modal_window.activate_window();
            });
            return;
        }

        let state = NewDataSourceState::new(window, cx);
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
            let view = app_cx.new(|cx| {
                create::CreateDataSourceWindow::new(state, parent.clone(), modal_window, cx)
            });
            app_cx.new(|cx| Root::new(view.into(), modal_window, cx))
        }) {
            Ok(handle) => {
                let _ = handle.update(cx, |_, modal_window, _| {
                    modal_window.set_window_title("新建数据源");
                });
                self.new_ds_window = Some(handle);
            }
            Err(err) => {
                eprintln!("failed to open create data source window: {err:?}");
            }
        }
    }

    pub fn clear_new_data_source_window(&mut self) {
        self.new_ds_window = None;
    }

    pub fn toggle_theme(&mut self, window: &mut Window, cx: &mut Context<SqlerApp>) {
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
        source_id: u64,
        _window: &mut Window,
        cx: &mut Context<SqlerApp>,
    ) {
        if let Some(existing) = self.tabs.iter().find(|tab| tab.is_data_source(source_id)) {
            self.active_tab = existing.id;
            cx.notify();
            return;
        }

        if let Some(meta) = self
            .saved_sources
            .iter()
            .find(|meta| meta.id == source_id)
            .cloned()
        {
            let id = TabId::next(&mut self.next_tab_id);
            self.tabs.push(TabState::data_source(id, meta));
            self.active_tab = id;
            cx.notify();
        }
    }

    pub fn close_tab(&mut self, tab_id: TabId, cx: &mut Context<SqlerApp>) {
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

    pub fn set_active_tab(&mut self, tab_id: TabId, cx: &mut Context<SqlerApp>) {
        if self.tabs.iter().any(|tab| tab.id == tab_id) {
            self.active_tab = tab_id;
            cx.notify();
        }
    }

    pub fn set_active_inner_tab(
        &mut self,
        tab_id: TabId,
        inner_id: InnerTabId,
        cx: &mut Context<SqlerApp>,
    ) {
        if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == tab_id) {
            if let TabKind::DataSource(state) = &mut tab.kind {
                state.active_inner_tab = inner_id;
            }
            cx.notify();
        }
    }
}

impl Render for SqlerApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .relative()
            .size_full()
            .min_w_0()
            .min_h_0()
            .child(render_head(self, window, cx))
            .child(
                div()
                    .size_full()
                    .min_w_0()
                    .min_h_0()
                    .child(workspace::render_active(self, window, cx)),
            )
            .into_any_element()
    }
}

pub fn render_head(
    app: &mut SqlerApp,
    _window: &mut Window,
    cx: &mut Context<SqlerApp>,
) -> gpui::Div {
    let active = app.active_tab;
    let mut tabs =
        h_flex()
            .gap(px(6.))
            .px(px(4.))
            .flex_1()
            .min_w_0()
            .children(app.tabs.iter().map(|tab| {
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
    tabs.style().min_size.width = Some(Length::Definite(px(0.).into()));

    let tabs_container = div().flex_1().min_w_0().child(tabs);
    let controls = h_flex()
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
        );

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

fn seed_sources() -> Vec<DataSourceMeta> {
    vec![
        DataSourceMeta {
            id: 1,
            name: SharedString::from("生产库"),
            kind: DatabaseKind::Postgres,
            description: SharedString::from("线上订单主库"),
            connection: ConnectionPreset {
                host: SharedString::from("10.10.12.5"),
                port: SharedString::from("5432"),
                database: SharedString::from("order_prod"),
                username: SharedString::from("svc_order"),
            },
            tables: vec![
                SharedString::from("orders"),
                SharedString::from("order_items"),
                SharedString::from("users"),
                SharedString::from("regions"),
            ],
        },
        DataSourceMeta {
            id: 2,
            name: SharedString::from("BI 分析库"),
            kind: DatabaseKind::MySql,
            description: SharedString::from("数仓汇总使用"),
            connection: ConnectionPreset {
                host: SharedString::from("10.60.1.10"),
                port: SharedString::from("3306"),
                database: SharedString::from("dw_report"),
                username: SharedString::from("reporter"),
            },
            tables: vec![
                SharedString::from("daily_metrics"),
                SharedString::from("marketing_channels"),
                SharedString::from("product_sku"),
            ],
        },
        DataSourceMeta {
            id: 3,
            name: SharedString::from("测试环境"),
            kind: DatabaseKind::Sqlite,
            description: SharedString::from("本地调试用"),
            connection: ConnectionPreset {
                host: SharedString::from("local"),
                port: SharedString::from("0"),
                database: SharedString::from("sqler-dev"),
                username: SharedString::from("dev"),
            },
            tables: vec![SharedString::from("sample_jobs")],
        },
    ]
}
