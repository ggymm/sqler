use gpui::prelude::*;
use gpui::*;
use gpui_component::button::Button;
use gpui_component::button::ButtonVariants;
use gpui_component::input::InputState;
use gpui_component::input::TextInput;
use gpui_component::resizable::h_resizable;
use gpui_component::resizable::resizable_panel;
use gpui_component::resizable::ResizableState;
use gpui_component::tab::Tab;
use gpui_component::tab::TabBar;
use gpui_component::ActiveTheme;
use gpui_component::InteractiveElementExt;
use gpui_component::Selectable;
use gpui_component::Disableable;
use gpui_component::Sizable;
use gpui_component::Size;
use gpui_component::StyledExt;

use crate::app::comps::comp_id;
use crate::app::comps::icon_close;
use crate::app::comps::icon_export;
use crate::app::comps::icon_import;
use crate::app::comps::icon_relead;
use crate::app::comps::icon_search;
use crate::app::comps::DataTable;
use crate::app::comps::TableColumn;
use crate::app::comps::TableData;
use crate::app::comps::TableRow;
use crate::driver::DatabaseDriver;
use crate::driver::DatabaseSession;
use crate::driver::DriverError;
use crate::driver::MySQLDriver;
use crate::driver::QueryReq;
use crate::driver::QueryResp;
use crate::option::DataSource;
use crate::option::DataSourceOptions;
use serde_json::Map;
use serde_json::Value;

const DEFAULT_PAGE_SIZE: usize = 25;

pub struct MySQLWorkspace {
    meta: DataSource,
    tabs: Vec<TabItem>,
    active_tab: SharedString,
    tables: Vec<SharedString>,
    active_table: Option<SharedString>,
    sidebar_resize: Entity<ResizableState>,
    session: Option<Box<dyn DatabaseSession>>,
}

impl MySQLWorkspace {
    pub fn new(
        meta: DataSource,
        cx: &mut Context<Self>,
    ) -> Self {
        let overview = TabItem::overview();
        let active_tab = overview.id.clone();
        let mut workspace = Self {
            meta,
            tabs: vec![overview],
            active_tab,
            tables: Vec::new(),
            active_table: None,
            sidebar_resize: ResizableState::new(cx),
            session: None,
        };

        if let Err(err) = workspace.ensure_session() {
            eprintln!("初始化 MySQL 连接失败: {}", err);
        }

        workspace.tables = workspace.load_tables();
        workspace.active_table = workspace.tables.first().cloned();
        workspace
    }

    fn close_tab(
        &mut self,
        id: &SharedString,
        cx: &mut Context<Self>,
    ) {
        if let Some(i) = self.tabs.iter().position(|tab| &tab.id == id && tab.closable) {
            let was_active = self.tabs[i].id == self.active_tab;
            self.tabs.remove(i);
            if was_active {
                if let Some(tab) = self.tabs.get(i.min(self.tabs.len().saturating_sub(1))) {
                    self.active_tab = tab.id.clone();
                }
            }
            cx.notify();
        }
    }

    fn active_tab(
        &mut self,
        id: SharedString,
        title: SharedString,
        cx: &mut Context<Self>,
    ) {
        self.active_tab = id;
        self.active_table = Some(title);
        cx.notify();
    }

    fn active_content(&self) -> Option<&TabContent> {
        self.tabs
            .iter()
            .find(|tab| tab.id == self.active_tab)
            .map(|tab| &tab.content)
    }

    fn load_tables(&mut self) -> Vec<SharedString> {
        match self.try_fetch_tables() {
            Ok(tables) if !tables.is_empty() => tables,
            Ok(_) => self.meta.tables(),
            Err(err) => {
                eprintln!("加载 MySQL 表列表失败: {}", err);
                self.meta.tables()
            }
        }
    }

    fn try_fetch_tables(&mut self) -> Result<Vec<SharedString>, DriverError> {
        let database = match Self::current_database(&self.meta) {
            Some(name) => name,
            None => return Ok(vec![]),
        };

        self.with_session(|session| Self::query_table_list(session, &database))
    }

    fn ensure_session(&mut self) -> Result<(), DriverError> {
        if self.session.is_some() {
            return Ok(());
        }

        let DataSourceOptions::MySQL(opts) = &self.meta.options else {
            return Err(DriverError::InvalidField("数据源类型不匹配".into()));
        };

        self.session = Some(MySQLDriver.create_connection(opts)?);
        Ok(())
    }

    fn with_session<R>(
        &mut self,
        mut operation: impl FnMut(&mut dyn DatabaseSession) -> Result<R, DriverError>,
    ) -> Result<R, DriverError> {
        let mut last_err = None;
        for _ in 0..2 {
            if let Err(err) = self.ensure_session() {
                last_err = Some(err);
                break;
            }

            if let Some(session) = self.session.as_deref_mut() {
                match operation(session) {
                    Ok(result) => return Ok(result),
                    Err(err) => {
                        last_err = Some(err);
                        self.session = None;
                        continue;
                    }
                }
            } else {
                last_err = Some(DriverError::Other("MySQL 连接不可用".into()));
                break;
            }
        }

        Err(last_err.unwrap_or_else(|| DriverError::Other("MySQL 连接不可用".into())))
    }

    fn current_database(meta: &DataSource) -> Option<String> {
        let DataSourceOptions::MySQL(opts) = &meta.options else {
            return None;
        };

        let database = opts.database.trim();
        if database.is_empty() {
            None
        } else {
            Some(database.to_string())
        }
    }

    fn query_table_list(
        session: &mut dyn DatabaseSession,
        database: &str,
    ) -> Result<Vec<SharedString>, DriverError> {
        let statement = format!(
            "SHOW TABLES FROM `{}`",
            escape_mysql_identifier(database),
        );
        let resp = session.query(QueryReq::Sql { statement })?;
        let rows = match resp {
            QueryResp::Rows(rows) => rows,
            _ => return Ok(vec![]),
        };

        let mut tables = Vec::new();
        for row in rows {
            for (_, value) in row {
                if let Some(name) = value.as_str() {
                    tables.push(SharedString::from(name.to_string()));
                    break;
                }
            }
        }
        Ok(tables)
    }

    fn fetch_table_page(
        &mut self,
        table: &str,
        filter: &str,
        page_index: usize,
        page_size: usize,
    ) -> Result<TablePage, DriverError> {
        let table_name = table.to_string();
        let filter_text = filter.to_string();
        self.with_session(|session| {
            Self::query_table_page(session, &table_name, &filter_text, page_index, page_size)
        })
    }

    fn query_table_page(
        session: &mut dyn DatabaseSession,
        table: &str,
        filter: &str,
        page_index: usize,
        page_size: usize,
    ) -> Result<TablePage, DriverError> {
        let offset = page_index.saturating_mul(page_size);
        let mut column_names = Self::fetch_column_names(session, table)?;
        let filter_clause = Self::build_filter_clause(&column_names, filter);
        let filter_sql = filter_clause.as_deref().unwrap_or("");

        let statement = format!(
            "SELECT * FROM `{}`{} LIMIT {} OFFSET {}",
            escape_mysql_identifier(table),
            filter_sql,
            page_size,
            offset,
        );

        let resp = session.query(QueryReq::Sql { statement })?;
        let rows = match resp {
            QueryResp::Rows(rows) => rows,
            _ => Vec::new(),
        };

        if column_names.is_empty() {
            if let Some(first) = rows.first() {
                column_names = first.keys().cloned().collect();
            }
        }

        let columns = column_names
            .iter()
            .map(|name| TableColumn::new(name.clone()))
            .collect::<Vec<_>>();
        let table_rows = Self::convert_rows(&column_names, rows);

        let total_rows = Self::query_table_total(session, table, filter_clause.as_deref())?;

        Ok(TablePage {
            columns,
            rows: table_rows,
            total_rows,
        })
    }

    fn fetch_column_names(
        session: &mut dyn DatabaseSession,
        table: &str,
    ) -> Result<Vec<String>, DriverError> {
        let statement = format!(
            "SHOW COLUMNS FROM `{}`",
            escape_mysql_identifier(table),
        );
        let resp = session.query(QueryReq::Sql { statement })?;
        let rows = match resp {
            QueryResp::Rows(rows) => rows,
            _ => return Ok(vec![]),
        };

        let mut columns = Vec::new();
        for row in rows {
            if let Some(field) = row.get("Field") {
                if let Some(name) = field.as_str() {
                    columns.push(name.to_string());
                    continue;
                }
            }

            if let Some((_, value)) = row.into_iter().next() {
                if let Some(name) = value.as_str() {
                    columns.push(name.to_string());
                }
            }
        }
        Ok(columns)
    }

    fn build_filter_clause(
        column_names: &[String],
        raw_filter: &str,
    ) -> Option<String> {
        let trimmed = raw_filter.trim();
        if trimmed.is_empty() {
            return None;
        }

        let lower = trimmed.to_ascii_lowercase();
        if lower.starts_with("where ") {
            return Some(format!(" {}", trimmed));
        }

        if trimmed.contains('=')
            || trimmed.contains('<')
            || trimmed.contains('>')
            || lower.contains(" like ")
            || lower.starts_with("order ")
            || lower.starts_with("group ")
            || lower.starts_with("limit ")
        {
            return Some(format!(" WHERE {}", trimmed));
        }

        if column_names.is_empty() {
            return None;
        }

        let like_pattern = trimmed
            .replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_")
            .replace('\'', "''");
        let mut conditions = Vec::new();
        for name in column_names {
            conditions.push(format!(
                "CAST(`{}` AS CHAR) LIKE '%{}%' ESCAPE '\\\\'",
                escape_mysql_identifier(name),
                like_pattern
            ));
        }
        if conditions.is_empty() {
            None
        } else {
            Some(format!(" WHERE {}", conditions.join(" OR ")))
        }
    }

    fn convert_rows(
        column_names: &[String],
        rows: Vec<Map<String, Value>>,
    ) -> Vec<TableRow> {
        let mut records = Vec::with_capacity(rows.len());
        for row in rows {
            let mut record = Vec::with_capacity(column_names.len());
            for name in column_names {
                let value = row.get(name).unwrap_or(&Value::Null);
                record.push(Self::format_cell(value));
            }
            records.push(record);
        }
        records
    }

    fn format_cell(value: &Value) -> SharedString {
        match value {
            Value::Null => SharedString::from(String::new()),
            Value::String(text) => SharedString::from(text.clone()),
            Value::Number(num) => SharedString::from(num.to_string()),
            Value::Bool(flag) => SharedString::from(if *flag { "true" } else { "false" }),
            other => SharedString::from(other.to_string()),
        }
    }

    fn query_table_total(
        session: &mut dyn DatabaseSession,
        table: &str,
        where_clause: Option<&str>,
    ) -> Result<usize, DriverError> {
        let statement = match where_clause {
            Some(clause) if !clause.is_empty() => format!(
                "SELECT COUNT(*) AS `total` FROM `{}`{}",
                escape_mysql_identifier(table),
                clause,
            ),
            _ => format!(
                "SELECT COUNT(*) AS `total` FROM `{}`",
                escape_mysql_identifier(table),
            ),
        };
        let resp = session.query(QueryReq::Sql { statement })?;
        let rows = match resp {
            QueryResp::Rows(rows) => rows,
            _ => return Ok(0),
        };

        if let Some(row) = rows.first() {
            for value in row.values() {
                if let Some(count) = value.as_u64() {
                    return Ok(count as usize);
                }
                if let Some(text) = value.as_str() {
                    if let Ok(count) = text.parse::<u64>() {
                        return Ok(count as usize);
                    }
                }
            }
        }
        Ok(0)
    }

    fn refresh_tables(
        &mut self,
        cx: &mut Context<Self>,
    ) {
        let new_tables = match self.try_fetch_tables() {
            Ok(tables) if !tables.is_empty() => tables,
            Ok(_) => self.meta.tables(),
            Err(err) => {
                eprintln!("刷新 MySQL 表列表失败: {}", err);
                if self.tables.is_empty() {
                    self.meta.tables()
                } else {
                    return;
                }
            }
        };

        self.tables = new_tables;
        if self.tables.is_empty() {
            self.active_table = None;
        } else if let Some(active) = self.active_table.clone() {
            if !self.tables.iter().any(|name| name == &active) {
                self.active_table = self.tables.first().cloned();
            }
        } else {
            self.active_table = self.tables.first().cloned();
        }

        let current_tables = self.tables.clone();
        self.tabs.retain(|tab| match &tab.content {
            TabContent::DataTable(tab) => current_tables.iter().any(|name| name == &tab.table),
            _ => true,
        });

        if !self.tabs.iter().any(|tab| tab.id == self.active_tab) {
            if let Some(tab) = self.tabs.iter().find(|tab| !tab.closable) {
                self.active_tab = tab.id.clone();
            } else if let Some(tab) = self.tabs.first() {
                self.active_tab = tab.id.clone();
            }
        }

        cx.notify();
    }

    fn create_tab_table_data(
        &mut self,
        table: SharedString,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let id = SharedString::from(format!("mysql-tab-table-data-{}-{}", self.meta.id, table));
        self.active_tab = id.clone();
        self.active_table = Some(table.clone());
        if let Some(existing) = self.tabs.iter().find(|tab| {
            matches!(
                &tab.content,
                TabContent::DataTable(current) if current.id == id
            )
        }) {
            self.active_tab = existing.id.clone();
            cx.notify();
            return;
        }

        let filter = cx.new(|cx| InputState::new(window, cx).placeholder("输入筛选条件，如 id > 10"));

        let page = match self.fetch_table_page(&table, "", 0, DEFAULT_PAGE_SIZE) {
            Ok(page) => page,
            Err(err) => {
                eprintln!("加载数据表失败: {}", err);
                TablePage {
                    columns: Vec::new(),
                    rows: Vec::new(),
                    total_rows: 0,
                }
            }
        };

        let data_table = DataTable::new(TableData::new(page.columns.clone(), page.rows.clone()));

        self.tabs.push(TabItem {
            id: id.clone(),
            title: table.clone(),
            content: TabContent::DataTable(DataTableTab {
                id: id.clone(),
                table: table.clone(),
                content: data_table,
                filter,
                current_page: 0,
                page_size: DEFAULT_PAGE_SIZE,
                total_rows: page.total_rows,
            }),
            closable: true,
        });
        cx.notify();
    }

    fn apply_filter(
        &mut self,
        tab_id: &SharedString,
        cx: &mut Context<Self>,
    ) {
        self.reload_table_tab(tab_id, 0, cx);
    }

    fn clear_filter(
        &mut self,
        tab_id: &SharedString,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(tab_item) = self.tabs.iter_mut().find(|tab| tab.id == *tab_id) {
            if let TabContent::DataTable(data_tab) = &mut tab_item.content {
                let _ = data_tab.filter.update(cx, |state, cx| {
                    state.set_value("", window, cx);
                });
            }
        }
        self.apply_filter(tab_id, cx);
    }

    fn goto_page(
        &mut self,
        tab_id: &SharedString,
        page: usize,
        cx: &mut Context<Self>,
    ) {
        self.reload_table_tab(tab_id, page, cx);
    }

    fn reload_table_tab(
        &mut self,
        tab_id: &SharedString,
        mut page: usize,
        cx: &mut Context<Self>,
    ) {
        let Some(tab_index) = self.tabs.iter().position(|tab| tab.id == *tab_id) else {
            return;
        };

        let (table_name, page_size, total_rows, filter_handle) = {
            let TabContent::DataTable(data_tab) = &self.tabs[tab_index].content else {
                return;
            };
            (
                data_tab.table.clone(),
                data_tab.page_size,
                data_tab.total_rows,
                data_tab.filter.clone(),
            )
        };

        let filter_value = filter_handle.update(cx, |state, _cx| state.value());
        let filter_text = filter_value.trim().to_string();

        let max_page = if total_rows == 0 {
            0
        } else {
            (total_rows.saturating_sub(1)) / page_size
        };
        if page > max_page {
            page = max_page;
        }

        match self.fetch_table_page(&table_name, &filter_text, page, page_size) {
            Ok(table_page) => {
                let new_total = table_page.total_rows;
                if new_total > 0 {
                    let new_max_page = (new_total.saturating_sub(1)) / page_size;
                    if page > new_max_page {
                        self.reload_table_tab(tab_id, new_max_page, cx);
                        return;
                    }
                } else {
                    page = 0;
                }

                if let TabContent::DataTable(data_tab) = &mut self.tabs[tab_index].content {
                    data_tab.current_page = page;
                    data_tab.page_size = page_size;
                    data_tab.total_rows = new_total;
                data_tab.content.set_data(TableData::new(table_page.columns, table_page.rows));
                }
                cx.notify();
            }
            Err(err) => {
                eprintln!("加载数据表失败: {}", err);
                self.session = None;
            }
        }
    }

    fn render_overview(
        &self,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let theme = cx.theme();
        let options = match &self.meta.options {
            DataSourceOptions::MySQL(opts) => opts,
            _ => panic!("MySQL workspace expects MySQL options"),
        };

        let host = if options.host.trim().is_empty() {
            "未配置"
        } else {
            options.host.as_str()
        };
        let database = if options.database.trim().is_empty() {
            "未配置"
        } else {
            options.database.as_str()
        };
        let charset = options
            .charset
            .as_deref()
            .filter(|v| !v.trim().is_empty())
            .unwrap_or("默认字符集");
        let tls = if options.use_tls {
            "TLS 已启用"
        } else {
            "未启用 TLS"
        };

        let connection_rows = [
            ("连接地址", format!("{}:{}", host, options.port)),
            ("数据库", database.to_string()),
            ("字符集", charset.to_string()),
            ("安全性", tls.to_string()),
        ];

        let detail_card = div()
            .flex()
            .flex_col()
            .gap(px(6.))
            .rounded_lg()
            .border_1()
            .border_color(theme.border)
            .bg(theme.secondary)
            .px(px(14.))
            .py(px(12.))
            .children(connection_rows.into_iter().map(|(label, value)| {
                div()
                    .flex()
                    .flex_row()
                    .justify_between()
                    .text_sm()
                    .text_color(theme.muted_foreground)
                    .child(div().text_color(theme.muted_foreground).child(label))
                    .child(div().text_color(theme.foreground).child(value))
                    .into_any_element()
            }));

        div()
            .flex()
            .flex_col()
            .flex_1()
            .size_full()
            .min_w_0()
            .min_h_0()
            .gap_5()
            .scrollable(Axis::Vertical)
            .child(
                div()
                    .text_base()
                    .font_semibold()
                    .child(format!("名称：{}", self.meta.name)),
            )
            .child(
                div()
                    .text_base()
                    .text_color(theme.muted_foreground)
                    .child(format!("描述：{}", self.meta.desc)),
            )
            .child(detail_card)
            .into_any_element()
    }

    fn render_datatable(
        &self,
        tab: &DataTableTab,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let theme = cx.theme().clone();
        let container_id = tab.id.to_string();
        let tab_id = tab.id.clone();
        let current_page = tab.current_page;

        let has_prev = tab.current_page > 0;
        let has_next = {
            let next_offset = (current_page + 1) * tab.page_size;
            next_offset < tab.total_rows
        };

        let filter_bar = div()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(8.))
            .child(
                div()
                    .flex()
                    .flex_1()
                    .child(
                        TextInput::new(&tab.filter)
                            .cleanable()
                            .with_size(Size::Small),
                   ),
           )
            .child(
                Button::new(comp_id(["mysql-tab-filter-apply", &tab_id]))
                    .outline()
                    .with_size(Size::Small)
                    .label("应用筛选")
                    .on_click(cx.listener({
                        let tab_id = tab_id.clone();
                        move |view: &mut Self, _, _, cx| {
                            view.apply_filter(&tab_id, cx);
                        }
                    })),
            )
            .child(
                Button::new(comp_id(["mysql-tab-filter-reset", &tab_id]))
                    .ghost()
                    .with_size(Size::Small)
                    .label("清空条件")
                    .on_click(cx.listener({
                        let tab_id = tab_id.clone();
                        move |view: &mut Self, _, window, cx| {
                            view.clear_filter(&tab_id, window, cx);
                        }
                    })),
            );

        let pagination = div()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .pt(px(8.))
            .child(
                div()
                    .text_sm()
                    .text_color(theme.muted_foreground)
                    .child(format!(
                        "第 {} 页 · 每页 {} 行 · 共 {} 行",
                        tab.current_page + 1,
                        tab.page_size,
                        tab.total_rows
                    )),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap(px(8.))
                    .child(
                        Button::new(comp_id(["mysql-tab-page-prev", &tab_id]))
                            .outline()
                            .with_size(Size::Small)
                            .label("上一页")
                            .disabled(!has_prev)
                            .on_click(cx.listener({
                                let tab_id = tab_id.clone();
                        move |view: &mut Self, _, _, cx| {
                            if has_prev {
                                let prev = current_page.saturating_sub(1);
                                view.goto_page(&tab_id, prev, cx);
                            }
                        }
                            })),
                    )
                    .child(
                        Button::new(comp_id(["mysql-tab-page-next", &tab_id]))
                            .outline()
                            .with_size(Size::Small)
                            .label("下一页")
                            .disabled(!has_next)
                            .on_click(cx.listener({
                                let tab_id = tab_id.clone();
                                let next_page = current_page + 1;
                                move |view: &mut Self, _, _, cx| {
                                    if has_next {
                                        view.goto_page(&tab_id, next_page, cx);
                                    }
                                }
                            })),
                    ),
            );

        div()
            .flex()
            .flex_col()
            .flex_1()
            .size_full()
            .min_w_0()
            .min_h_0()
            .gap(px(8.))
            .child(filter_bar)
            .child(tab.content.render(&container_id, cx))
            .child(pagination)
            .into_any_element()
    }

    fn render_placeholder(
        &self,
        _cx: &mut Context<Self>,
    ) -> AnyElement {
        div()
            .flex()
            .flex_col()
            .scrollable(Axis::Vertical)
            .gap(px(8.))
            .child(div().text_base().font_semibold().child("自定义视图"))
            .child(div().text_sm().child("在这里扩展你的分析组件。"))
            .into_any_element()
    }
}

fn escape_mysql_identifier(value: &str) -> String {
    value.replace('`', "``")
}

impl Render for MySQLWorkspace {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let id = &self.meta.id;
        let theme = cx.theme().clone();

        let menu = self.tables.iter().cloned().fold(
            div()
                .id(comp_id(["mysql-menu", id]))
                .flex()
                .flex_col()
                .flex_1()
                .p_2()
                .gap_2()
                .min_w_0()
                .min_h_0()
                .scrollable(Axis::Vertical),
            |acc, table| {
                let selected = self.active_table.as_ref() == Some(&table);
                let click_table = table.clone();
                acc.child(
                    div()
                        .flex()
                        .id(comp_id(["mysql-menu-table", &self.meta.id, &table]))
                        .px_4()
                        .py_2()
                        .rounded_lg()
                        .text_sm()
                        .text_color(theme.muted_foreground)
                        .hover(|this| this.bg(theme.secondary_hover))
                        .when(selected, |this| {
                            this.bg(theme.secondary_hover)
                                .text_color(theme.foreground)
                                .font_semibold()
                        })
                        .on_double_click(cx.listener(move |this, _, window, cx| {
                            this.create_tab_table_data(click_table.clone(), window, cx);
                        }))
                        .child(table.clone()),
                )
            },
        );

        let tabs = TabBar::new(comp_id(["mysql-tabs", id]))
            .with_size(Size::Small)
            .children(
                self.tabs
                    .iter()
                    .enumerate()
                    .map(|(_, tab)| {
                        let tab_id = tab.id.clone();
                        Tab::new(tab.title.clone())
                            .id(comp_id(["mysql-tabs-item", id, &tab_id]))
                            .px_2()
                            .selected(tab.id == self.active_tab)
                            .when(tab.closable, |this| {
                                this.suffix(
                                    Button::new(comp_id(["mysql-tabs-close", &tab_id]))
                                        .ghost()
                                        .xsmall()
                                        .tab_stop(false)
                                        .icon(icon_close().with_size(Size::XSmall))
                                        .on_click(cx.listener(move |view: &mut Self, _, _, cx| {
                                            view.close_tab(&tab_id, cx);
                                        }))
                                        .into_any_element(),
                                )
                            })
                            .on_click(cx.listener({
                                let tab_id = tab.id.clone();
                                let tab_title = tab.title.clone();
                                move |view: &mut Self, _, _, cx| {
                                    view.active_tab(tab_id.clone(), tab_title.clone(), cx);
                                }
                            }))
                    })
                    .collect::<Vec<_>>(),
            );

        let main = div()
            .flex()
            .flex_col()
            .flex_1()
            .size_full()
            .min_w_0()
            .min_h_0()
            .child(tabs)
            .child(
                div()
                    .id(comp_id(["mysql-main", id]))
                    .flex()
                    .flex_col()
                    .flex_1()
                    .size_full()
                    .min_w_0()
                    .min_h_0()
                    .p_2()
                    .child(match self.active_content() {
                        Some(TabContent::Overview) | None => self.render_overview(cx),
                        Some(TabContent::DataTable(tab)) => self.render_datatable(&tab, cx),
                        Some(TabContent::Placeholder) => self.render_placeholder(cx),
                    }),
            )
            .into_any_element();

        let header = div()
            .id(comp_id(["mysql-header", id]))
            .flex()
            .flex_row()
            .px_4()
            .py_4()
            .gap_2()
            .border_b_1()
            .border_color(theme.border)
            .child(
                Button::new(comp_id(["mysql-header-refresh", id]))
                    .outline()
                    .icon(icon_relead().with_size(Size::Small))
                    .on_click(cx.listener(|view: &mut Self, _, _, cx| {
                        view.refresh_tables(cx);
                    }))
                    .label("刷新表"),
            )
            .child(
                Button::new(comp_id(["mysql-header-query", id]))
                    .outline()
                    .icon(icon_search().with_size(Size::Small))
                    .label("新建查询"),
            )
            .child(
                Button::new(comp_id(["mysql-header-import", id]))
                    .outline()
                    .icon(icon_import().with_size(Size::Small))
                    .label("数据导入"),
            )
            .child(
                Button::new(comp_id(["mysql-header-export", id]))
                    .outline()
                    .icon(icon_export().with_size(Size::Small))
                    .label("数据导出"),
            );

        let content = div()
            .flex()
            .flex_col()
            .flex_1()
            .size_full()
            .min_w_0()
            .min_h_0()
            .child(
                h_resizable(comp_id(["mysql-content", id]), self.sidebar_resize.clone())
                    .child(
                        resizable_panel()
                            .size(px(240.0))
                            .size_range(px(120.)..px(360.))
                            .child(menu),
                    )
                    .child(main),
            )
            .child(div());

        div()
            .id(comp_id(["mysql", id]))
            .flex()
            .flex_col()
            .flex_1()
            .size_full()
            .min_w_0()
            .min_h_0()
            .child(header)
            .child(content)
    }
}

#[derive(Clone)]
struct TabItem {
    id: SharedString,
    title: SharedString,
    content: TabContent,
    closable: bool,
}

impl TabItem {
    fn overview() -> Self {
        Self {
            id: SharedString::from("mysql-tab-overview"),
            title: SharedString::from("概览"),
            content: TabContent::Overview,
            closable: false,
        }
    }
}

#[derive(Clone)]
enum TabContent {
    Overview,
    DataTable(DataTableTab),
    Placeholder,
}

#[derive(Clone)]
struct DataTableTab {
    id: SharedString,
    table: SharedString,
    content: DataTable,
    filter: Entity<InputState>,
    current_page: usize,
    page_size: usize,
    total_rows: usize,
}

struct TablePage {
    columns: Vec<TableColumn>,
    rows: Vec<TableRow>,
    total_rows: usize,
}
