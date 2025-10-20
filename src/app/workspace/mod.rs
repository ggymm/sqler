use gpui::prelude::*;
use gpui::*;

use gpui_component::{
    button::{Button, ButtonVariants as _},
    form::{form_field, v_form},
    h_flex, v_flex, ActiveTheme as _, Disableable as _, InteractiveElementExt as _, Selectable as _, Sizable,
    StyledExt,
};

use crate::app::{DataSourceTabState, InnerTab, InnerTabId, SqlerApp, TabId, TabKind};
use crate::option::{OracleAddress, SQLServerAuth, SslMode, StoredOptions};
use crate::DataSourceMeta;
use crate::DataSourceType;

pub mod mongodb;
pub mod mysql;
pub mod oracle;
pub mod postgres;
pub mod redis;
pub mod sqlite;
pub mod sqlserver;

pub fn render(
    app: &mut SqlerApp,
    window: &mut Window,
    cx: &mut Context<SqlerApp>,
) -> AnyElement {
    if let Some(tab) = app.tabs.iter().find(|tab| tab.id == app.active_tab) {
        match &tab.kind {
            TabKind::Home => render_home(&app.saved_sources, window, cx).into_any_element(),
            TabKind::DataSource(state) => render_data_source(tab.id, state, window, cx).into_any_element(),
        }
    } else {
        v_flex()
            .flex_1()
            .child(div().child("未找到可渲染的标签页"))
            .into_any_element()
    }
}

fn render_home(
    saved_sources: &[DataSourceMeta],
    window: &mut Window,
    cx: &mut Context<SqlerApp>,
) -> AnyElement {
    let mut source_list = v_flex()
        .px(px(20.))
        .py(px(16.))
        .gap(px(12.))
        .flex_1()
        .id("home-source-list")
        .overflow_scroll();
    source_list.style().min_size.height = Some(Length::Definite(px(0.).into()));
    let source_list = source_list.child(
        h_flex().flex_wrap().gap(px(12.)).children(
            saved_sources
                .iter()
                .map(|meta| render_data_source_card(meta, window, cx)),
        ),
    );

    let theme = cx.theme();

    let mut layout = v_flex().size_full().flex_1();
    {
        let style = layout.style();
        style.min_size.height = Some(Length::Definite(px(0.).into()));
        style.min_size.width = Some(Length::Definite(px(0.).into()));
    }

    layout
        .child(
            h_flex()
                .justify_between()
                .items_center()
                .px(px(20.))
                .py(px(16.))
                .border_1()
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
                ),
        )
        .child(source_list)
        .into_any_element()
}

fn render_data_source_card(
    meta: &DataSourceMeta,
    _window: &mut Window,
    cx: &mut Context<SqlerApp>,
) -> AnyElement {
    let source_id = meta.id;
    let icon_path = meta.kind.image();

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
            h_flex()
                .items_center()
                .gap(px(8.))
                .child(
                    div()
                        .flex_shrink_0()
                        .w(px(32.))
                        .h(px(32.))
                        .child(img(icon_path).size_full()),
                )
                .child(
                    div()
                        .flex_1()
                        .text_base()
                        .font_semibold()
                        .text_color(cx.theme().foreground)
                        .child(meta.name.clone()),
                ),
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
                .child(meta.desc.clone()),
        )
        .into_any_element()
}

fn render_data_source(
    tab_id: TabId,
    state: &DataSourceTabState,
    _window: &mut Window,
    cx: &mut Context<SqlerApp>,
) -> gpui::Div {
    let mut table_list = v_flex()
        .px(px(12.))
        .py(px(8.))
        .gap(px(6.))
        .flex_1()
        .id("workspace-table-list")
        .overflow_scroll();
    {
        let style = table_list.style();
        style.min_size.height = Some(Length::Definite(px(0.).into()));
        style.min_size.width = Some(Length::Definite(px(0.).into()));
    }
    table_list = table_list.children(state.tables.iter().map(|table| {
        div()
            .px(px(10.))
            .py(px(6.))
            .rounded(px(4.))
            .hover(|this| this.bg(cx.theme().sidebar_accent))
            .child(table.clone())
    }));

    let mut left_panel = v_flex()
        .w(px(240.))
        .flex_shrink_0()
        .bg(cx.theme().sidebar)
        .border_r_1()
        .border_color(cx.theme().border)
        .child(
            div()
                .px(px(16.))
                .py(px(18.))
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("表列表"),
        )
        .child(table_list);
    {
        let style = left_panel.style();
        style.min_size.height = Some(Length::Definite(px(0.).into()));
        style.min_size.width = Some(Length::Definite(px(0.).into()));
    }

    let mut right_panel = v_flex().flex_1().size_full();
    {
        let style = right_panel.style();
        style.min_size.height = Some(Length::Definite(px(0.).into()));
        style.min_size.width = Some(Length::Definite(px(0.).into()));
    }

    let mut detail_panel = v_flex()
        .flex_1()
        .id(SharedString::from(format!("ds-detail-scroll-{}", tab_id.raw())))
        .overflow_scroll();
    {
        let style = detail_panel.style();
        style.min_size.height = Some(Length::Definite(px(0.).into()));
        style.min_size.width = Some(Length::Definite(px(0.).into()));
    }
    let detail_panel = detail_panel
        .px(px(24.))
        .py(px(20.))
        .child(data_source_detail(state, cx));

    let right_panel = right_panel
        .child(inner_tab_bar(tab_id, &state.inner_tabs, state.active_inner_tab, cx))
        .child(detail_panel);

    let mut content_row = h_flex().flex_1().size_full();
    {
        let style = content_row.style();
        style.align_items = Some(AlignItems::Stretch);
        style.min_size.height = Some(Length::Definite(px(0.).into()));
        style.min_size.width = Some(Length::Definite(px(0.).into()));
    }
    let content_row = content_row.child(left_panel).child(right_panel);

    let mut root = v_flex().flex_1().size_full();
    {
        let style = root.style();
        style.min_size.height = Some(Length::Definite(px(0.).into()));
        style.min_size.width = Some(Length::Definite(px(0.).into()));
    }

    root.child(workspace_toolbar(tab_id, true, cx)).child(content_row)
}

fn workspace_toolbar(
    tab_id: TabId,
    has_query: bool,
    cx: &mut Context<SqlerApp>,
) -> gpui::Div {
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
                button.selected(true).on_click(cx.listener(move |this, _, _, cx| {
                    this.set_active_inner_tab(tab_id, InnerTabId(0), cx);
                }))
            } else {
                button.disabled(disabled)
            }
        }))
}

fn inner_tab_bar(
    tab_id: TabId,
    tabs: &[InnerTab],
    active: InnerTabId,
    cx: &mut Context<SqlerApp>,
) -> gpui::Div {
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

fn data_source_detail(
    state: &DataSourceTabState,
    cx: &mut Context<SqlerApp>,
) -> gpui::Div {
    let meta = &state.meta;
    let mut config = v_form()
        .gap(px(12.))
        .child(form_field().label("名称").child(div().child(meta.name.clone())))
        .child(form_field().label("类型").child(div().child(meta.kind.label())));

    for (label, value) in connection_details(meta) {
        config = config.child(form_field().label(label).child(div().child(value)));
    }

    config = config.child(form_field().label("描述").child(div().child(meta.desc.clone())));

    let workspace_view = render_workspace_body(meta, cx);

    v_flex()
        .gap(px(16.))
        .child(
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("提示：后续将补充连接测试、历史操作等信息。"),
        )
        .child(config)
        .child(workspace_view)
}

fn render_workspace_body(
    meta: &DataSourceMeta,
    cx: &mut Context<SqlerApp>,
) -> gpui::Div {
    match meta.kind {
        DataSourceType::MySQL => mysql::render(meta.kind, meta, cx),
        DataSourceType::Oracle => mysql::render(meta.kind, meta, cx),
        DataSourceType::SQLite => mysql::render(meta.kind, meta, cx),
        DataSourceType::SQLServer => mysql::render(meta.kind, meta, cx),
        DataSourceType::PostgreSQL => postgres::render(meta.kind, meta, cx),
        DataSourceType::Redis => mysql::render(meta.kind, meta, cx),
        DataSourceType::MongoDB => mysql::render(meta.kind, meta, cx),
    }
}

fn connection_details(meta: &DataSourceMeta) -> Vec<(&'static str, SharedString)> {
    match &meta.options {
        StoredOptions::MySQL(opts) => {
            let mut fields = vec![
                ("主机", SharedString::from(opts.host.clone())),
                ("端口", SharedString::from(opts.port.to_string())),
                ("数据库", SharedString::from(opts.database.clone())),
                ("账号", SharedString::from(opts.username.clone())),
            ];
            if let Some(charset) = &opts.charset {
                fields.push(("字符集", SharedString::from(charset.clone())));
            }
            if opts.password.is_some() {
                fields.push(("密码", SharedString::from("已设置")));
            }
            if opts.use_tls {
                fields.push(("TLS", SharedString::from("开启")));
            }
            fields
        }
        StoredOptions::PostgreSQL(opts) => {
            let mut fields = vec![
                ("主机", SharedString::from(opts.host.clone())),
                ("端口", SharedString::from(opts.port.to_string())),
                ("数据库", SharedString::from(opts.database.clone())),
                ("账号", SharedString::from(opts.username.clone())),
            ];
            if let Some(schema) = &opts.schema {
                fields.push(("Schema", SharedString::from(schema.clone())));
            }
            if let Some(mode) = opts.ssl_mode {
                let mode_str = match mode {
                    SslMode::Disable => "Disable",
                    SslMode::Prefer => "Prefer",
                    SslMode::Require => "Require",
                };
                fields.push(("SSL 模式", SharedString::from(mode_str)));
            }
            if opts.password.is_some() {
                fields.push(("密码", SharedString::from("已设置")));
            }
            fields
        }
        StoredOptions::SQLite(opts) => {
            let mut fields = vec![("文件路径", SharedString::from(opts.file_path.clone()))];
            fields.push((
                "访问模式",
                SharedString::from(if opts.read_only { "只读" } else { "读写" }),
            ));
            if opts.password.is_some() {
                fields.push(("密码", SharedString::from("已设置")));
            }
            fields
        }
        StoredOptions::SQLServer(opts) => {
            let mut fields = vec![
                ("主机", SharedString::from(opts.host.clone())),
                ("端口", SharedString::from(opts.port.to_string())),
                ("数据库", SharedString::from(opts.database.clone())),
                (
                    "认证模式",
                    SharedString::from(match opts.auth {
                        SQLServerAuth::SqlPassword => "SQL 密码",
                        SQLServerAuth::Integrated => "集成认证",
                    }),
                ),
            ];
            if let Some(instance) = &opts.instance {
                fields.push(("实例名", SharedString::from(instance.clone())));
            }
            if let Some(username) = &opts.username {
                fields.push(("账号", SharedString::from(username.clone())));
            }
            if opts.password.is_some() {
                fields.push(("密码", SharedString::from("已设置")));
            }
            fields
        }
        StoredOptions::Oracle(opts) => {
            let address = match &opts.address {
                OracleAddress::ServiceName(name) => format!("ServiceName: {}", name),
                OracleAddress::Sid(sid) => format!("SID: {}", sid),
            };
            let mut fields = vec![
                ("主机", SharedString::from(opts.host.clone())),
                ("端口", SharedString::from(opts.port.to_string())),
                ("地址", SharedString::from(address)),
                ("账号", SharedString::from(opts.username.clone())),
            ];
            if opts.wallet_path.is_some() {
                fields.push(("钱包", SharedString::from("已配置")));
            }
            if opts.password.is_some() {
                fields.push(("密码", SharedString::from("已设置")));
            }
            fields
        }
        StoredOptions::Redis(opts) => {
            let mut fields = vec![
                ("主机", SharedString::from(opts.host.clone())),
                ("端口", SharedString::from(opts.port.to_string())),
                ("数据库索引", SharedString::from(opts.db.to_string())),
                (
                    "TLS",
                    SharedString::from(if opts.use_tls { "开启" } else { "关闭" }),
                ),
            ];
            if let Some(username) = &opts.username {
                fields.push(("账号", SharedString::from(username.clone())));
            }
            if opts.password.is_some() {
                fields.push(("密码", SharedString::from("已设置")));
            }
            fields
        }
        StoredOptions::MongoDB(opts) => {
            let mut fields = Vec::new();
            if let Some(uri) = &opts.connection_string {
                fields.push(("连接串", SharedString::from(uri.clone())));
            } else if !opts.hosts.is_empty() {
                let hosts = opts
                    .hosts
                    .iter()
                    .map(|host| format!("{}:{}", host.host, host.port))
                    .collect::<Vec<_>>()
                    .join(", ");
                fields.push(("主机", SharedString::from(hosts)));
            }
            if let Some(rs) = &opts.replica_set {
                fields.push(("副本集", SharedString::from(rs.clone())));
            }
            if let Some(auth) = &opts.auth_source {
                fields.push(("认证库", SharedString::from(auth.clone())));
            }
            if let Some(username) = &opts.username {
                fields.push(("账号", SharedString::from(username.clone())));
            }
            if opts.password.is_some() {
                fields.push(("密码", SharedString::from("已设置")));
            }
            fields.push((
                "TLS",
                SharedString::from(if opts.tls { "开启" } else { "关闭" }),
            ));
            fields
        }
    }
}

fn connection_summary(meta: &DataSourceMeta) -> String {
    match &meta.options {
        StoredOptions::MySQL(opts) => format!(
            "{}@{}:{} / {}",
            opts.username, opts.host, opts.port, opts.database
        ),
        StoredOptions::PostgreSQL(opts) => format!(
            "{}@{}:{} / {}",
            opts.username, opts.host, opts.port, opts.database
        ),
        StoredOptions::SQLite(opts) => format!("file: {}", opts.file_path),
        StoredOptions::SQLServer(opts) => {
            let user = opts
                .username
                .clone()
                .unwrap_or_else(|| String::from("IntegratedAuth"));
            let mut summary = format!("{}@{}:{}", user, opts.host, opts.port);
            if let Some(instance) = &opts.instance {
                summary.push_str(&format!(" ({})", instance));
            }
            summary.push_str(&format!(" / {}", opts.database));
            summary
        }
        StoredOptions::Oracle(opts) => {
            let address = match &opts.address {
                OracleAddress::ServiceName(name) => format!("service:{}", name),
                OracleAddress::Sid(sid) => format!("sid:{}", sid),
            };
            format!("{}@{}:{} [{}]", opts.username, opts.host, opts.port, address)
        }
        StoredOptions::Redis(opts) => {
            let tls = if opts.use_tls { " (TLS)" } else { "" };
            let user = opts.username.clone().unwrap_or_else(|| "default".into());
            format!(
                "{}@{}:{} db={}{}",
                user, opts.host, opts.port, opts.db, tls
            )
        }
        StoredOptions::MongoDB(opts) => {
            if let Some(uri) = &opts.connection_string {
                uri.clone()
            } else if !opts.hosts.is_empty() {
                let hosts = opts
                    .hosts
                    .iter()
                    .map(|host| format!("{}:{}", host.host, host.port))
                    .collect::<Vec<_>>()
                    .join(",");
                let mut summary = format!("mongodb://{}", hosts);
                if let Some(rs) = &opts.replica_set {
                    summary.push_str(&format!("?replicaSet={}", rs));
                }
                summary
            } else {
                "mongodb://<empty>".to_string()
            }
        }
    }
}

pub fn render_common_workspace(
    kind: DataSourceType,
    meta: &DataSourceMeta,
    notes: Vec<String>,
    cx: &mut Context<SqlerApp>,
) -> gpui::Div {
    let theme = cx.theme();
    let summary = connection_summary(meta);

    let mut section = v_flex()
        .gap(px(10.))
        .child(
            div()
                .text_base()
                .font_semibold()
                .child(format!("{} 工作区", kind.label())),
        )
        .child(
            div()
                .text_sm()
                .text_color(theme.muted_foreground)
                .child(format!("连接：{}", summary)),
        );

    for note in notes {
        section = section.child(div().text_sm().text_color(theme.muted_foreground).child(note));
    }

    section
}
