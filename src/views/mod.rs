use gpui::{Context, IntoElement, Render, SharedString, Window};
use gpui_component::{theme::{Theme, ThemeMode}, ActiveTheme as _, ActiveTheme};

pub(crate) mod content;
pub(crate) mod dialog;
pub(crate) mod dialog_view;
pub(crate) mod topbar;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct TabId(u64);

impl TabId {
    pub(crate) fn next(counter: &mut u64) -> Self {
        let id = *counter;
        *counter += 1;
        TabId(id)
    }

    pub(crate) fn raw(self) -> u64 {
        self.0
    }
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) enum DatabaseKind {
    Postgres,
    MySql,
    Sqlite,
    SqlServer,
}

impl DatabaseKind {
    pub(crate) fn label(&self) -> &'static str {
        match self {
            DatabaseKind::Postgres => "Postgres",
            DatabaseKind::MySql => "MySQL",
            DatabaseKind::Sqlite => "SQLite",
            DatabaseKind::SqlServer => "SQL Server",
        }
    }

    pub(crate) fn all() -> &'static [DatabaseKind] {
        &[
            DatabaseKind::Postgres,
            DatabaseKind::MySql,
            DatabaseKind::Sqlite,
            DatabaseKind::SqlServer,
        ]
    }
}

#[derive(Clone)]
pub(crate) struct ConnectionPreset {
    pub host: SharedString,
    pub port: SharedString,
    pub database: SharedString,
    pub username: SharedString,
}

#[derive(Clone)]
pub(crate) struct DataSourceMeta {
    pub id: u64,
    pub name: SharedString,
    pub kind: DatabaseKind,
    pub description: SharedString,
    pub connection: ConnectionPreset,
    pub tables: Vec<SharedString>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct InnerTabId(u64);

impl InnerTabId {
    pub(crate) fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum InnerTabKind {
    Config,
}

#[derive(Clone)]
pub(crate) struct InnerTab {
    pub id: InnerTabId,
    pub title: SharedString,
    _kind: InnerTabKind,
    _closable: bool,
}

impl InnerTab {
    pub(crate) fn config() -> Self {
        Self {
            id: InnerTabId(0),
            title: SharedString::from("配置"),
            _kind: InnerTabKind::Config,
            _closable: false,
        }
    }
}

#[derive(Clone)]
pub(crate) struct NewDataSourceState {
    pub selected: Option<DatabaseKind>,
    pub postgres: dialog::postgres::PostgresState,
    pub mysql: dialog::mysql::MySqlState,
    pub sqlite: dialog::sqlite::SqliteState,
    pub sqlserver: dialog::sqlserver::SqlServerState,
}

impl NewDataSourceState {
    pub(crate) fn new(window: &mut Window, cx: &mut Context<SqlerApp>) -> Self {
        Self {
            selected: None,
            postgres: dialog::postgres::PostgresState::new(window, cx),
            mysql: dialog::mysql::MySqlState::new(window, cx),
            sqlite: dialog::sqlite::SqliteState::new(window, cx),
            sqlserver: dialog::sqlserver::SqlServerState::new(window, cx),
        }
    }
}

#[derive(Clone)]
pub(crate) struct DataSourceTabState {
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

pub(crate) enum TabKind {
    Home,
    DataSource(DataSourceTabState),
}

pub(crate) struct TabState {
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
    pub(crate) tabs: Vec<TabState>,
    pub(crate) active_tab: TabId,
    pub(crate) next_tab_id: u64,
    pub(crate) saved_sources: Vec<DataSourceMeta>,
    pub(crate) new_ds_modal: Option<NewDataSourceState>,
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
            new_ds_modal: None,
        }
    }

    pub(crate) fn show_new_data_source_modal(
        &mut self,
        window: &mut Window,
        cx: &mut Context<SqlerApp>,
    ) {
        self.new_ds_modal = Some(NewDataSourceState::new(window, cx));
        cx.notify();
    }

    pub(crate) fn hide_new_data_source_modal(&mut self, cx: &mut Context<SqlerApp>) {
        if self.new_ds_modal.take().is_some() {
            cx.notify();
        }
    }

    pub(crate) fn submit_new_data_source_modal(&mut self, cx: &mut Context<SqlerApp>) {
        if self.new_ds_modal.take().is_some() {
            cx.notify();
        }
    }

    pub(crate) fn toggle_theme(&mut self, window: &mut Window, cx: &mut Context<SqlerApp>) {
        let next_mode = if cx.theme().is_dark() {
            ThemeMode::Light
        } else {
            ThemeMode::Dark
        };
        Theme::change(next_mode, Some(window), cx);
        cx.notify();
    }

    pub(crate) fn open_data_source_tab(
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

    pub(crate) fn close_tab(&mut self, tab_id: TabId, cx: &mut Context<SqlerApp>) {
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

    pub(crate) fn set_active_tab(&mut self, tab_id: TabId, cx: &mut Context<SqlerApp>) {
        if self.tabs.iter().any(|tab| tab.id == tab_id) {
            self.active_tab = tab_id;
            cx.notify();
        }
    }

    pub(crate) fn set_active_inner_tab(
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
        content::render_root(self, window, cx)
    }
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
