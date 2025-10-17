use gpui::{
    px, size, App, AppContext, Application, AssetSource, Bounds, Context, Entity, IntoElement,
    Render, Result, SharedString, Window, WindowBounds, WindowOptions,
};
use gpui_component::{input::InputState, ActiveTheme as _, Root, theme::{Theme, ThemeMode}};
use std::{borrow::Cow, fs::read, path::PathBuf};

mod comps;
mod views;

struct FsAssets;

impl AssetSource for FsAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if path.is_empty() {
            return Ok(None);
        }

        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let full = manifest_dir.join("assets").join(path);

        match read(full) {
            Ok(data) => Ok(Some(Cow::Owned(data))),
            Err(_) => Ok(None),
        }
    }

    fn list(&self, _path: &str) -> Result<Vec<SharedString>> {
        Ok(Vec::new())
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct TabId(u64);

impl TabId {
    fn next(counter: &mut u64) -> Self {
        let id = *counter;
        *counter += 1;
        TabId(id)
    }

    fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum DatabaseKind {
    Postgres,
    MySql,
    Sqlite,
    SqlServer,
}

impl DatabaseKind {
    fn all() -> &'static [DatabaseKind] {
        &[
            DatabaseKind::Postgres,
            DatabaseKind::MySql,
            DatabaseKind::Sqlite,
            DatabaseKind::SqlServer,
        ]
    }

    fn label(&self) -> &'static str {
        match self {
            DatabaseKind::Postgres => "Postgres",
            DatabaseKind::MySql => "MySQL",
            DatabaseKind::Sqlite => "SQLite",
            DatabaseKind::SqlServer => "SQL Server",
        }
    }

    fn key(&self) -> &'static str {
        match self {
            DatabaseKind::Postgres => "postgres",
            DatabaseKind::MySql => "mysql",
            DatabaseKind::Sqlite => "sqlite",
            DatabaseKind::SqlServer => "sqlserver",
        }
    }
}

#[derive(Clone)]
struct ConnectionPreset {
    host: SharedString,
    port: SharedString,
    database: SharedString,
    username: SharedString,
}

#[derive(Clone)]
struct DataSourceMeta {
    id: u64,
    name: SharedString,
    kind: DatabaseKind,
    description: SharedString,
    connection: ConnectionPreset,
    tables: Vec<SharedString>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct InnerTabId(u64);

#[derive(Clone, Copy, PartialEq, Eq)]
enum InnerTabKind {
    Config,
}

#[derive(Clone)]
struct InnerTab {
    id: InnerTabId,
    title: SharedString,
    _kind: InnerTabKind,
    _closable: bool,
}

impl InnerTabId {
    fn raw(self) -> u64 {
        self.0
    }
}

impl InnerTab {
    fn config() -> Self {
        Self {
            id: InnerTabId(0),
            title: SharedString::from("配置"),
            _kind: InnerTabKind::Config,
            _closable: false,
        }
    }
}

struct ConnectionForm {
    name: Entity<InputState>,
    host: Entity<InputState>,
    port: Entity<InputState>,
    username: Entity<InputState>,
    password: Entity<InputState>,
    database: Entity<InputState>,
    schema: Entity<InputState>,
    db_type: DatabaseKind,
}

impl ConnectionForm {
    fn new(window: &mut Window, cx: &mut Context<SqlerApp>) -> Self {
        Self {
            name: cx.new(|cx| {
                InputState::new(window, cx).placeholder("输入数据源名称，例如：线上生产库")
            }),
            host: cx.new(|cx| {
                InputState::new(window, cx)
                    .placeholder("主机地址，例如：127.0.0.1")
                    .default_value("127.0.0.1")
            }),
            port: cx.new(|cx| {
                InputState::new(window, cx)
                    .placeholder("端口，例如：5432")
                    .default_value("5432")
            }),
            username: cx.new(|cx| {
                InputState::new(window, cx)
                    .placeholder("用户名，例如：admin")
                    .default_value("postgres")
            }),
            password: cx.new(|cx| InputState::new(window, cx).placeholder("密码").masked(true)),
            database: cx
                .new(|cx| InputState::new(window, cx).placeholder("数据库名称，例如：prod_db")),
            schema: cx.new(|cx| InputState::new(window, cx).placeholder("模式/Schema，可选")),
            db_type: DatabaseKind::Postgres,
        }
    }
}

struct NewDataSourceState {
    form: ConnectionForm,
    inner_tabs: Vec<InnerTab>,
    active_inner_tab: InnerTabId,
}

impl NewDataSourceState {
    fn new(window: &mut Window, cx: &mut Context<SqlerApp>) -> Self {
        Self {
            form: ConnectionForm::new(window, cx),
            inner_tabs: vec![InnerTab::config()],
            active_inner_tab: InnerTabId(0),
        }
    }
}

struct DataSourceTabState {
    meta: DataSourceMeta,
    inner_tabs: Vec<InnerTab>,
    active_inner_tab: InnerTabId,
    tables: Vec<SharedString>,
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

enum TabKind {
    Home,
    NewDataSource(NewDataSourceState),
    DataSource(DataSourceTabState),
}

struct TabState {
    id: TabId,
    title: SharedString,
    closable: bool,
    kind: TabKind,
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

    fn new_data_source(id: TabId, window: &mut Window, cx: &mut Context<SqlerApp>) -> Self {
        Self {
            id,
            title: SharedString::from("新建数据源"),
            closable: true,
            kind: TabKind::NewDataSource(NewDataSourceState::new(window, cx)),
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
        matches!(
            &self.kind,
            TabKind::DataSource(state) if state.meta.id == id
        )
    }
}

struct SqlerApp {
    tabs: Vec<TabState>,
    active_tab: TabId,
    next_tab_id: u64,
    saved_sources: Vec<DataSourceMeta>,
}

impl SqlerApp {
    fn new(_window: &mut Window, _cx: &mut Context<SqlerApp>) -> Self {
        let saved_sources = vec![
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
        ];

        let mut next_tab_id = 1;
        let home_id = TabId::next(&mut next_tab_id);

        Self {
            tabs: vec![TabState::home(home_id)],
            active_tab: home_id,
            next_tab_id,
            saved_sources,
        }
    }

    fn open_new_data_source(&mut self, window: &mut Window, cx: &mut Context<SqlerApp>) {
        let id = TabId::next(&mut self.next_tab_id);
        self.tabs.push(TabState::new_data_source(id, window, cx));
        self.active_tab = id;
        cx.notify();
    }

    fn toggle_theme(&mut self, window: &mut Window, cx: &mut Context<SqlerApp>) {
        let next_mode = if cx.theme().is_dark() {
            ThemeMode::Light
        } else {
            ThemeMode::Dark
        };
        Theme::change(next_mode, Some(window), cx);
        cx.notify();
    }

    fn open_data_source_tab(
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

    fn close_tab(&mut self, tab_id: TabId, cx: &mut Context<SqlerApp>) {
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

    fn set_active_tab(&mut self, tab_id: TabId, cx: &mut Context<SqlerApp>) {
        if self.tabs.iter().any(|tab| tab.id == tab_id) {
            self.active_tab = tab_id;
            cx.notify();
        }
    }

    fn set_active_inner_tab(
        &mut self,
        tab_id: TabId,
        inner_id: InnerTabId,
        cx: &mut Context<SqlerApp>,
    ) {
        if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == tab_id) {
            match &mut tab.kind {
                TabKind::NewDataSource(state) => state.active_inner_tab = inner_id,
                TabKind::DataSource(state) => state.active_inner_tab = inner_id,
                TabKind::Home => {}
            }
            cx.notify();
        }
    }

    fn set_database_kind(&mut self, tab_id: TabId, kind: DatabaseKind, cx: &mut Context<SqlerApp>) {
        if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == tab_id) {
            if let TabKind::NewDataSource(state) = &mut tab.kind {
                state.form.db_type = kind;
                cx.notify();
            }
        }
    }

}

impl Render for SqlerApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        views::main::render(self, window, cx)
    }
}

fn main() {
    let app = Application::new().with_assets(FsAssets);

    app.run(|cx: &mut App| {
        gpui_component::init(cx);
        cx.activate(true);

        let window_bounds = Bounds::centered(None, size(px(1200.), px(800.)), cx);

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(window_bounds)),
                ..Default::default()
            },
            |window, cx| {
                let view = cx.new(|cx| SqlerApp::new(window, cx));
                cx.new(|cx| Root::new(view.into(), window, cx))
            },
        )
        .expect("failed to open window")
        .update(cx, |_, window, _| {
            window.set_window_title("Sqler");
            window.activate_window();
        })
        .unwrap();
    });
}
