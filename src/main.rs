use gpui::prelude::*;
use gpui::{
    div, px, size, AnyElement, App, AppContext, Application, Bounds, Context, Entity, IntoElement,
    Render, SharedString, Styled, Window, WindowBounds, WindowOptions,
};
use gpui_component::{
    button::{Button, ButtonVariants as _},
    form::{form_field, v_form},
    h_flex,
    input::{InputState, TextInput},
    v_flex, ActiveTheme as _, Disableable as _, InteractiveElementExt as _, Root, Selectable as _,
    Sizable as _, StyledExt,
};

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

    fn render_tab_bar(&self, cx: &mut Context<SqlerApp>) -> Vec<AnyElement> {
        let active = self.active_tab;
        self.tabs
            .iter()
            .map(move |tab| {
                let tab_id = tab.id;
                let is_active = tab_id == active;

                let mut pill = h_flex()
                    .gap(px(6.))
                    .px(px(12.))
                    .py(px(6.))
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
                    .child(tab.title.clone());

                if tab.closable {
                    pill = pill.child(
                        Button::new(("close-tab", tab_id.raw()))
                            .text()
                            .tab_stop(false)
                            .label("x")
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.close_tab(tab_id, cx);
                            })),
                    );
                }

                pill.into_any_element()
            })
            .collect()
    }

    fn render_home(&self, window: &mut Window, cx: &mut Context<SqlerApp>) -> impl IntoElement {
        let theme = cx.theme();

        v_flex()
            .size_full()
            .child(
                h_flex()
                    .justify_between()
                    .items_center()
                    .px(px(20.))
                    .py(px(16.))
                    .bg(theme.tab_bar)
                    .border_b_1()
                    .border_color(theme.border)
                    .child(
                        v_flex()
                            .gap(px(4.))
                            .child(div().text_lg().font_semibold().child("数据源总览"))
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(theme.muted_foreground)
                                    .child("双击数据源以打开新标签页，统一管理和查询。"),
                            ),
                    )
                    .child(
                        Button::new("home-new-source")
                            .primary()
                            .small()
                            .label("新建数据源")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.open_new_data_source(window, cx);
                            })),
                    ),
            )
            .child(
                v_flex()
                    .px(px(20.))
                    .py(px(16.))
                    .gap(px(12.))
                    .flex_1()
                    .id("home-source-list")
                    .overflow_scroll()
                    .child(
                        h_flex().flex_wrap().gap(px(12.)).children(
                            self.saved_sources
                                .iter()
                                .map(|meta| self.render_data_source_card(meta, window, cx)),
                        ),
                    ),
            )
    }

    fn render_data_source_card(
        &self,
        meta: &DataSourceMeta,
        _window: &mut Window,
        cx: &mut Context<SqlerApp>,
    ) -> impl IntoElement {
        let source_id = meta.id;

        v_flex()
            .w(px(220.))
            .gap(px(8.))
            .p(px(14.))
            .rounded(px(8.))
            .bg(cx.theme().secondary)
            .border_1()
            .border_color(cx.theme().border)
            .cursor_pointer()
            .id(SharedString::from(format!("source-card-{}", source_id)))
            .hover(|this| this.bg(cx.theme().secondary_hover))
            .on_double_click(cx.listener(move |this, _, window, cx| {
                this.open_data_source_tab(source_id, window, cx);
            }))
            .child(
                div()
                    .text_base()
                    .font_semibold()
                    .text_color(cx.theme().foreground)
                    .child(meta.name.clone()),
            )
            .child(
                Button::new(("kind-chip", source_id))
                    .ghost()
                    .xsmall()
                    .tab_stop(false)
                    .label(meta.kind.label()),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(meta.description.clone()),
            )
    }

    fn render_workspace_toolbar(
        &self,
        tab_id: TabId,
        has_query: bool,
        cx: &mut Context<SqlerApp>,
    ) -> impl IntoElement {
        let buttons = [
            ("tab-config", "数据源配置", false),
            ("tab-new-query", "新建查询", !has_query),
            ("tab-import", "导入", true),
            ("tab-export", "导出", true),
        ];

        h_flex()
            .gap(px(8.))
            .px(px(16.))
            .py(px(10.))
            .bg(cx.theme().tab_bar)
            .border_b_1()
            .border_color(cx.theme().border)
            .children(buttons.into_iter().map(|(id, label, disabled)| {
                let button = Button::new((id, tab_id.raw())).ghost().small().label(label);

                if id == "tab-config" {
                    button
                        .selected(true)
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.set_active_inner_tab(tab_id, InnerTabId(0), cx);
                        }))
                } else {
                    button.disabled(disabled)
                }
            }))
    }

    fn render_inner_tab_bar(
        &self,
        tab_id: TabId,
        tabs: &[InnerTab],
        active: InnerTabId,
        cx: &mut Context<SqlerApp>,
    ) -> impl IntoElement {
        h_flex()
            .gap(px(6.))
            .px(px(16.))
            .py(px(8.))
            .border_b_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().tab_bar)
            .children(tabs.iter().map(move |tab| {
                let tab_id_inner = tab.id;
                let pill = h_flex()
                    .px(px(10.))
                    .py(px(6.))
                    .rounded(px(6.))
                    .cursor_pointer()
                    .id(SharedString::from(format!(
                        "inner-tab-{}-{}",
                        tab_id.raw(),
                        tab_id_inner.raw()
                    )))
                    .text_sm()
                    .when(tab_id_inner == active, |this| {
                        this.bg(cx.theme().tab_active)
                            .text_color(cx.theme().tab_active_foreground)
                    })
                    .when(tab_id_inner != active, |this| {
                        this.text_color(cx.theme().muted_foreground)
                    })
                    .child(tab.title.clone());

                pill.on_click(cx.listener(move |this, _, _, cx| {
                    this.set_active_inner_tab(tab_id, tab_id_inner, cx);
                }))
            }))
    }

    fn render_connection_form(
        &self,
        tab_id: TabId,
        state: &NewDataSourceState,
        cx: &mut Context<SqlerApp>,
    ) -> impl IntoElement {
        let selector = h_flex()
            .gap(px(8.))
            .children(DatabaseKind::all().iter().map(|kind| {
                let current = state.form.db_type == *kind;
                let button = Button::new(SharedString::from(format!(
                    "ds-kind-{}-{}",
                    tab_id.raw(),
                    kind.key()
                )))
                .small()
                .ghost()
                .label(kind.label())
                .selected(current);

                button.on_click(cx.listener({
                    let kind = *kind;
                    move |this, _, _, cx| {
                        this.set_database_kind(tab_id, kind, cx);
                    }
                }))
            }));

        let form = v_form()
            .gap(px(12.))
            .child(
                form_field()
                    .label("数据源名称")
                    .child(TextInput::new(&state.form.name)),
            )
            .child(
                form_field()
                    .label("主机")
                    .child(TextInput::new(&state.form.host)),
            )
            .child(
                form_field()
                    .label("端口")
                    .child(TextInput::new(&state.form.port)),
            )
            .child(
                form_field()
                    .label("用户名")
                    .child(TextInput::new(&state.form.username)),
            )
            .child(
                form_field()
                    .label("密码")
                    .child(TextInput::new(&state.form.password).mask_toggle()),
            )
            .child(
                form_field()
                    .label("数据库")
                    .child(TextInput::new(&state.form.database)),
            )
            .child(
                form_field()
                    .label("Schema")
                    .child(TextInput::new(&state.form.schema)),
            );

        v_flex()
            .gap(px(16.))
            .child(
                v_flex()
                    .gap(px(8.))
                    .child(div().text_sm().child("数据源类型"))
                    .child(selector),
            )
            .child(
                v_flex()
                    .gap(px(12.))
                    .child(div().text_sm().child("连接信息"))
                    .child(form),
            )
            .child(
                h_flex()
                    .gap(px(8.))
                    .justify_end()
                    .child(
                        Button::new(("test-connection", tab_id.raw()))
                            .ghost()
                            .label("测试连接")
                            .on_click(cx.listener(|_, _, _, cx| {
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new(("save-connection", tab_id.raw()))
                            .primary()
                            .label("保存")
                            .on_click(cx.listener(|_, _, _, cx| {
                                cx.notify();
                            })),
                    ),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child("提示：测试连接和保存将在后续实现。"),
            )
    }

    fn render_data_source_detail(
        &self,
        state: &DataSourceTabState,
        cx: &mut Context<SqlerApp>,
    ) -> impl IntoElement {
        let meta = &state.meta;
        let config = v_form()
            .gap(px(12.))
            .child(
                form_field()
                    .label("名称")
                    .child(div().child(meta.name.clone())),
            )
            .child(
                form_field()
                    .label("类型")
                    .child(div().child(meta.kind.label())),
            )
            .child(
                form_field()
                    .label("主机")
                    .child(div().child(meta.connection.host.clone())),
            )
            .child(
                form_field()
                    .label("端口")
                    .child(div().child(meta.connection.port.clone())),
            )
            .child(
                form_field()
                    .label("数据库")
                    .child(div().child(meta.connection.database.clone())),
            )
            .child(
                form_field()
                    .label("账号")
                    .child(div().child(meta.connection.username.clone())),
            )
            .child(
                form_field()
                    .label("描述")
                    .child(div().child(meta.description.clone())),
            );

        v_flex()
            .gap(px(16.))
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child("提示：后续将补充连接测试、历史操作等信息。"),
            )
            .child(config)
    }
}

impl Render for SqlerApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let tab_bar = h_flex()
            .gap(px(6.))
            .px(px(12.))
            .py(px(10.))
            .bg(cx.theme().background)
            .border_b_1()
            .border_color(cx.theme().border)
            .children(self.render_tab_bar(cx));

        let content = if let Some(tab) = self.tabs.iter().find(|tab| tab.id == self.active_tab) {
            let tab_id = tab.id;
            match &tab.kind {
                TabKind::Home => self.render_home(window, cx).into_any_element(),
                TabKind::NewDataSource(state) => {
                    let workspace = h_flex()
                        .flex_1()
                        .child(
                            v_flex()
                                .w(px(220.))
                                .bg(cx.theme().sidebar)
                                .border_r_1()
                                .border_color(cx.theme().border)
                                .child(
                                    v_flex()
                                        .gap(px(8.))
                                        .px(px(14.))
                                        .py(px(16.))
                                        .child(
                                            div()
                                                .text_sm()
                                                .text_color(cx.theme().muted_foreground)
                                                .child("连接成功后将展示表列表。"),
                                        )
                                        .child(div().text_sm().child("当前无可用数据。")),
                                ),
                        )
                        .child(
                            v_flex()
                                .flex_1()
                                .child(self.render_inner_tab_bar(
                                    tab_id,
                                    &state.inner_tabs,
                                    state.active_inner_tab,
                                    cx,
                                ))
                                .child(
                                    v_flex()
                                        .flex_1()
                                        .id(SharedString::from(format!(
                                            "new-ds-scroll-{}",
                                            tab_id.raw()
                                        )))
                                        .overflow_scroll()
                                        .px(px(24.))
                                        .py(px(18.))
                                        .child(self.render_connection_form(tab_id, state, cx)),
                                ),
                        );

                    v_flex()
                        .flex_1()
                        .child(self.render_workspace_toolbar(tab_id, false, cx))
                        .child(workspace)
                        .into_any_element()
                }
                TabKind::DataSource(state) => {
                    let workspace = h_flex()
                        .flex_1()
                        .child(
                            v_flex()
                                .w(px(220.))
                                .bg(cx.theme().sidebar)
                                .border_r_1()
                                .border_color(cx.theme().border)
                                .child(
                                    v_flex()
                                        .gap(px(4.))
                                        .px(px(14.))
                                        .py(px(16.))
                                        .child(
                                            div()
                                                .text_sm()
                                                .text_color(cx.theme().muted_foreground)
                                                .child("表列表"),
                                        )
                                        .children(state.tables.iter().map(|table| {
                                            div()
                                                .px(px(8.))
                                                .py(px(6.))
                                                .rounded(px(4.))
                                                .hover(|this| this.bg(cx.theme().sidebar_accent))
                                                .child(table.clone())
                                        })),
                                ),
                        )
                        .child(
                            v_flex()
                                .flex_1()
                                .child(self.render_inner_tab_bar(
                                    tab_id,
                                    &state.inner_tabs,
                                    state.active_inner_tab,
                                    cx,
                                ))
                                .child(
                                    v_flex()
                                        .flex_1()
                                        .id(SharedString::from(format!(
                                            "ds-detail-scroll-{}",
                                            tab_id.raw()
                                        )))
                                        .overflow_scroll()
                                        .px(px(24.))
                                        .py(px(18.))
                                        .child(self.render_data_source_detail(state, cx)),
                                ),
                        );

                    v_flex()
                        .flex_1()
                        .child(self.render_workspace_toolbar(tab_id, true, cx))
                        .child(workspace)
                        .into_any_element()
                }
            }
        } else {
            v_flex()
                .flex_1()
                .child(div().child("未找到可渲染的标签页"))
                .into_any_element()
        };

        v_flex().size_full().child(tab_bar).child(content)
    }
}

fn main() {
    let app = Application::new();

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
