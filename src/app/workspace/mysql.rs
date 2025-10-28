use gpui::prelude::*;
use gpui::*;
use gpui_component::button::Button;
use gpui_component::button::ButtonVariants;
use gpui_component::dropdown::Dropdown;
use gpui_component::dropdown::DropdownState;
use gpui_component::input::InputState;
use gpui_component::input::TextInput;
use gpui_component::resizable::h_resizable;
use gpui_component::resizable::resizable_panel;
use gpui_component::resizable::ResizableState;
use gpui_component::switch::Switch;
use gpui_component::tab::Tab;
use gpui_component::tab::TabBar;
use gpui_component::ActiveTheme;
use gpui_component::InteractiveElementExt;
use gpui_component::Selectable;
use gpui_component::Sizable;
use gpui_component::Size;
use gpui_component::StyledExt;
use serde_json::Map;
use serde_json::Value;

use crate::app::comps::comp_id;
use crate::app::comps::full_col;
use crate::app::comps::full_row;
use crate::app::comps::icon_close;
use crate::app::comps::icon_export;
use crate::app::comps::icon_import;
use crate::app::comps::icon_relead;
use crate::app::comps::icon_search;
use crate::app::comps::icon_sheet;
use crate::app::comps::DataTable;
use crate::driver::DatabaseDriver;
use crate::driver::DatabaseSession;
use crate::driver::DriverError;
use crate::driver::MySQLDriver;
use crate::driver::QueryReq;
use crate::driver::QueryResp;
use crate::option::DataSource;
use crate::option::DataSourceOptions;

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
        // workspace.active_table = workspace.tables.first().cloned();
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
                    self.active_table = Some(tab.title.clone());
                } else {
                    self.active_table = None;
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
        let statement = format!("SHOW TABLES FROM `{}`", escape_mysql_identifier(database),);
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
        sort_column: Option<&str>,
        sort_ascending: bool,
    ) -> Result<TablePage, DriverError> {
        let table_name = table.to_string();
        let filter_text = filter.to_string();
        let sort_col = sort_column.map(|s| s.to_string());
        self.with_session(|session| {
            Self::query_table_page(
                session,
                &table_name,
                &filter_text,
                page_index,
                page_size,
                sort_col.as_deref(),
                sort_ascending,
            )
        })
    }

    fn query_table_page(
        session: &mut dyn DatabaseSession,
        table: &str,
        filter: &str,
        page_index: usize,
        page_size: usize,
        sort_column: Option<&str>,
        sort_ascending: bool,
    ) -> Result<TablePage, DriverError> {
        let offset = page_index.saturating_mul(page_size);
        let mut column_names = Self::fetch_column_names(session, table)?;
        let filter_clause = Self::build_filter_clause(&column_names, filter);
        let filter_sql = filter_clause.as_deref().unwrap_or("");

        let sort_clause = if let Some(col) = sort_column {
            let direction = if sort_ascending { "ASC" } else { "DESC" };
            format!(" ORDER BY `{}` {}", escape_mysql_identifier(col), direction)
        } else {
            String::new()
        };

        let statement = format!(
            "SELECT * FROM `{}`{}{} LIMIT {} OFFSET {}",
            escape_mysql_identifier(table),
            filter_sql,
            sort_clause,
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

        let headers = column_names
            .iter()
            .map(|name| SharedString::from(name.clone()))
            .collect::<Vec<_>>();
        let table_rows = Self::convert_rows(&column_names, rows);

        let total_rows = Self::query_table_total(session, table, filter_clause.as_deref())?;

        Ok(TablePage {
            headers,
            rows: table_rows,
            total_rows,
        })
    }

    fn fetch_column_names(
        session: &mut dyn DatabaseSession,
        table: &str,
    ) -> Result<Vec<String>, DriverError> {
        let statement = format!("SHOW COLUMNS FROM `{}`", escape_mysql_identifier(table),);
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
    ) -> Vec<Vec<SharedString>> {
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
            _ => format!("SELECT COUNT(*) AS `total` FROM `{}`", escape_mysql_identifier(table),),
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

        let page = match self.fetch_table_page(&table, "", 0, DEFAULT_PAGE_SIZE, None, true) {
            Ok(page) => page,
            Err(err) => {
                eprintln!("加载数据表失败: {}", err);
                TablePage {
                    headers: Vec::new(),
                    rows: Vec::new(),
                    total_rows: 0,
                }
            }
        };

        let data_table = DataTable::new(page.headers.clone(), page.rows.clone()).build(window, cx);

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
                sort_column: None,
                sort_ascending: true,
                filter_panel_open: false,
                filter_rules: Vec::new(),
                sort_rules: Vec::new(),
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

    fn apply_sort(
        &mut self,
        tab_id: &SharedString,
        column: Option<String>,
        cx: &mut Context<Self>,
    ) {
        if let Some(tab_item) = self.tabs.iter_mut().find(|tab| tab.id == *tab_id) {
            if let TabContent::DataTable(data_tab) = &mut tab_item.content {
                // 如果点击同一列，切换排序方向
                if data_tab.sort_column == column {
                    data_tab.sort_ascending = !data_tab.sort_ascending;
                } else {
                    data_tab.sort_column = column;
                    data_tab.sort_ascending = true;
                }
            }
        }
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

    fn toggle_filter_panel(
        &mut self,
        tab_id: &SharedString,
        cx: &mut Context<Self>,
    ) {
        if let Some(tab_item) = self.tabs.iter_mut().find(|tab| tab.id == *tab_id) {
            if let TabContent::DataTable(data_tab) = &mut tab_item.content {
                data_tab.filter_panel_open = !data_tab.filter_panel_open;
            }
        }
        cx.notify();
    }

    fn add_filter_rule(
        &mut self,
        tab_id: &SharedString,
        columns: &[SharedString],
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(tab_item) = self.tabs.iter_mut().find(|tab| tab.id == *tab_id) {
            if let TabContent::DataTable(data_tab) = &mut tab_item.content {
                let rule_id = SharedString::from(format!("filter-{}", uuid::Uuid::new_v4()));
                let field_dropdown = cx.new(|cx| DropdownState::new(columns.to_vec(), None, window, cx));

                // 创建操作符下拉列表
                let operators: Vec<SharedString> = FilterOperator::all()
                    .into_iter()
                    .map(|op| SharedString::from(op.label().to_string()))
                    .collect();
                let operator_dropdown = cx.new(|cx| DropdownState::new(operators, None, window, cx));

                // 创建值输入框
                let value_input = cx.new(|cx| InputState::new(window, cx).placeholder("输入筛选值"));

                data_tab.filter_rules.push(FilterRule {
                    id: rule_id,
                    field_dropdown,
                    operator_dropdown,
                    operator: FilterOperator::Equal,
                    value_input,
                });
            }
        }
        cx.notify();
    }

    fn remove_filter_rule(
        &mut self,
        tab_id: &SharedString,
        rule_id: &SharedString,
        cx: &mut Context<Self>,
    ) {
        if let Some(tab_item) = self.tabs.iter_mut().find(|tab| tab.id == *tab_id) {
            if let TabContent::DataTable(data_tab) = &mut tab_item.content {
                data_tab.filter_rules.retain(|r| &r.id != rule_id);
            }
        }
        cx.notify();
    }

    fn add_sort_rule(
        &mut self,
        tab_id: &SharedString,
        columns: &[SharedString],
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(tab_item) = self.tabs.iter_mut().find(|tab| tab.id == *tab_id) {
            if let TabContent::DataTable(data_tab) = &mut tab_item.content {
                let rule_id = SharedString::from(format!("sort-{}", uuid::Uuid::new_v4()));
                let field_dropdown = cx.new(|cx| DropdownState::new(columns.to_vec(), None, window, cx));

                data_tab.sort_rules.push(SortRule {
                    id: rule_id,
                    field_dropdown,
                    ascending: true,
                });
            }
        }
        cx.notify();
    }

    fn remove_sort_rule(
        &mut self,
        tab_id: &SharedString,
        rule_id: &SharedString,
        cx: &mut Context<Self>,
    ) {
        if let Some(tab_item) = self.tabs.iter_mut().find(|tab| tab.id == *tab_id) {
            if let TabContent::DataTable(data_tab) = &mut tab_item.content {
                data_tab.sort_rules.retain(|r| &r.id != rule_id);
            }
        }
        cx.notify();
    }

    fn toggle_sort_direction(
        &mut self,
        tab_id: &SharedString,
        rule_id: &SharedString,
        cx: &mut Context<Self>,
    ) {
        if let Some(tab_item) = self.tabs.iter_mut().find(|tab| tab.id == *tab_id) {
            if let TabContent::DataTable(data_tab) = &mut tab_item.content {
                if let Some(rule) = data_tab.sort_rules.iter_mut().find(|r| &r.id == rule_id) {
                    rule.ascending = !rule.ascending;
                }
            }
        }
        cx.notify();
    }

    fn clear_all_rules(
        &mut self,
        tab_id: &SharedString,
        cx: &mut Context<Self>,
    ) {
        if let Some(tab_item) = self.tabs.iter_mut().find(|tab| tab.id == *tab_id) {
            if let TabContent::DataTable(data_tab) = &mut tab_item.content {
                data_tab.filter_rules.clear();
                data_tab.sort_rules.clear();
            }
        }
        cx.notify();
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

        let (table_name, page_size, total_rows, filter_handle, sort_column, sort_ascending) = {
            let TabContent::DataTable(data_tab) = &self.tabs[tab_index].content else {
                return;
            };
            (
                data_tab.table.clone(),
                data_tab.page_size,
                data_tab.total_rows,
                data_tab.filter.clone(),
                data_tab.sort_column.clone(),
                data_tab.sort_ascending,
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

        match self.fetch_table_page(
            &table_name,
            &filter_text,
            page,
            page_size,
            sort_column.as_deref(),
            sort_ascending,
        ) {
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
                    data_tab.content.update(cx, |table, cx| {
                        table.delegate_mut().update_data(table_page.headers, table_page.rows);
                        cx.notify();
                    });
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
            .flex_1()
            .flex_col()
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
        let tab_id = tab.id.clone();

        // 获取列名
        let columns = tab.content.read(cx).delegate().columns().to_vec();

        // 计算分页信息
        let total_pages = if tab.total_rows == 0 {
            1
        } else {
            (tab.total_rows + tab.page_size - 1) / tab.page_size
        };
        let current_page = tab.current_page;
        let start_row = current_page * tab.page_size + 1;
        let end_row = ((current_page + 1) * tab.page_size).min(tab.total_rows);

        div()
            .flex()
            .flex_1()
            .flex_col()
            .gap_2()
            .child(
                // 顶部操作栏
                div().flex().flex_row().items_center().gap_2().child(
                    Button::new(comp_id(["datatable-toggle-filter", &tab_id]))
                        .small()
                        .when(tab.filter_panel_open, |btn| btn.primary())
                        .when(!tab.filter_panel_open, |btn| btn.outline())
                        .label(if tab.filter_panel_open {
                            "隐藏筛选"
                        } else {
                            "筛选数据"
                        })
                        .on_click(cx.listener({
                            let tab_id = tab_id.clone();
                            move |view: &mut Self, _, _, cx| {
                                view.toggle_filter_panel(&tab_id, cx);
                            }
                        })),
                ),
            )
            .when(tab.filter_panel_open, |this| {
                this.child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_3()
                        .p_3()
                        .rounded_md()
                        .border_1()
                        .border_color(theme.border)
                        .bg(theme.secondary)
                        .child(
                            // 筛选规则列表
                            div()
                                .flex()
                                .flex_col()
                                .gap_2()
                                .when(!tab.filter_rules.is_empty(), |this| {
                                    this.child(
                                        div()
                                            .text_sm()
                                            .font_semibold()
                                            .text_color(theme.foreground)
                                            .child("筛选条件"),
                                    )
                                })
                                .children(tab.filter_rules.iter().map(|rule| {
                                    let rule_id = rule.id.clone();
                                    div()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .gap_2()
                                        .p_2()
                                        .w_full()
                                        .rounded_md()
                                        .bg(theme.background)
                                        .child(
                                            // 字段选择
                                            div().flex().flex_col().gap_1().w(px(180.)).child(
                                                Dropdown::new(&rule.field_dropdown).small().placeholder("选择字段"),
                                            ),
                                        )
                                        .child(
                                            // 条件选择
                                            div().flex().flex_col().gap_1().w(px(150.)).child(
                                                Dropdown::new(&rule.operator_dropdown).small().placeholder("选择条件"),
                                            ),
                                        )
                                        .child(
                                            // 条件值输入
                                            div()
                                                .flex()
                                                .flex_1()
                                                .flex_col()
                                                .gap_1()
                                                .child(TextInput::new(&rule.value_input).small()),
                                        )
                                        .child(
                                            // 删除按钮
                                            Button::new(comp_id(["filter-remove", &rule_id]))
                                                .xsmall()
                                                .ghost()
                                                .label("删除")
                                                .on_click(cx.listener({
                                                    let tab_id = tab_id.clone();
                                                    let rule_id = rule_id.clone();
                                                    move |view: &mut Self, _, _, cx| {
                                                        view.remove_filter_rule(&tab_id, &rule_id, cx);
                                                    }
                                                })),
                                        )
                                })),
                        )
                        .child(
                            // 排序规则列表
                            div()
                                .flex()
                                .flex_col()
                                .gap_2()
                                .when(!tab.sort_rules.is_empty(), |this| {
                                    this.child(
                                        div()
                                            .text_sm()
                                            .font_semibold()
                                            .text_color(theme.foreground)
                                            .child("排序规则"),
                                    )
                                })
                                .children(tab.sort_rules.iter().map(|rule| {
                                    let rule_id = rule.id.clone();
                                    let ascending = rule.ascending.clone();

                                    div()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .gap_2()
                                        .p_2()
                                        .w_full()
                                        .rounded_md()
                                        .bg(theme.background)
                                        .child(
                                            // 字段选择
                                            div().flex().flex_col().gap_1().w(px(180.)).child(
                                                Dropdown::new(&rule.field_dropdown).small().placeholder("选择字段"),
                                            ),
                                        )
                                        .child(
                                            // 排序方向选择
                                            div()
                                                .flex()
                                                .flex_row()
                                                .items_center()
                                                .gap_2()
                                                .child(div().text_sm().text_color(theme.muted_foreground).child("降序"))
                                                .child(
                                                    Switch::new(comp_id(["sort-ascending", &rule_id]))
                                                        .checked(ascending)
                                                        .on_click(cx.listener({
                                                            let tab_id = tab_id.clone();
                                                            let rule_id = rule_id.clone();
                                                            move |view: &mut Self, _, _, cx| {
                                                                view.toggle_sort_direction(&tab_id, &rule_id, cx);
                                                            }
                                                        })),
                                                )
                                                .child(
                                                    div().text_sm().text_color(theme.muted_foreground).child("升序"),
                                                ),
                                        )
                                        .child(div().flex_1())
                                        .child(
                                            // 删除按钮
                                            Button::new(comp_id(["sort-remove", &rule_id]))
                                                .xsmall()
                                                .ghost()
                                                .label("删除")
                                                .on_click(cx.listener({
                                                    let tab_id = tab_id.clone();
                                                    let rule_id = rule_id.clone();
                                                    move |view: &mut Self, _, _, cx| {
                                                        view.remove_sort_rule(&tab_id, &rule_id, cx);
                                                    }
                                                })),
                                        )
                                })),
                        )
                        .child(
                            // 底部按钮栏
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap_2()
                                .child(
                                    Button::new(comp_id(["filter-panel-clear", &tab_id]))
                                        .small()
                                        .outline()
                                        .label("清除所有条件")
                                        .on_click(cx.listener({
                                            let tab_id = tab_id.clone();
                                            move |view: &mut Self, _, _, cx| {
                                                view.clear_all_rules(&tab_id, cx);
                                            }
                                        })),
                                )
                                .child(
                                    Button::new(comp_id(["filter-panel-add-filter", &tab_id]))
                                        .small()
                                        .outline()
                                        .label("新增筛选")
                                        .on_click(cx.listener({
                                            let tab_id = tab_id.clone();
                                            let columns = columns.to_vec();
                                            move |view: &mut Self, _, window, cx| {
                                                view.add_filter_rule(&tab_id, &columns, window, cx);
                                            }
                                        })),
                                )
                                .child(
                                    Button::new(comp_id(["filter-panel-add-sort", &tab_id]))
                                        .small()
                                        .outline()
                                        .label("新增排序")
                                        .on_click(cx.listener({
                                            let tab_id = tab_id.clone();
                                            let columns = columns.to_vec();
                                            move |view: &mut Self, _, window, cx| {
                                                view.add_sort_rule(&tab_id, &columns, window, cx);
                                            }
                                        })),
                                )
                                .child(div().flex_1())
                                .child(
                                    Button::new(comp_id(["filter-panel-apply", &tab_id]))
                                        .small()
                                        .primary()
                                        .label("全部应用")
                                        .on_click(cx.listener({
                                            let tab_id = tab_id.clone();
                                            move |view: &mut Self, _, _, cx| {
                                                // TODO: 应用所有筛选和排序规则
                                                view.apply_filter(&tab_id, cx);
                                            }
                                        })),
                                ),
                        ),
                )
            })
            .child(
                // 表格区域
                div().flex_1().rounded_md().overflow_hidden().child(tab.content.clone()),
            )
            .child(
                // 分页控件
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .px_2()
                    .py_2()
                    .border_t_1()
                    .border_color(theme.border)
                    .child(div().text_sm().text_color(theme.muted_foreground).child(format!(
                        "显示 {} - {} / 共 {} 条",
                        if tab.total_rows == 0 { 0 } else { start_row },
                        end_row,
                        tab.total_rows
                    )))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .when(current_page > 0, |this| {
                                this.child(
                                    Button::new(comp_id(["datatable-page-first", &tab_id]))
                                        .small()
                                        .outline()
                                        .label("首页")
                                        .on_click(cx.listener({
                                            let tab_id = tab_id.clone();
                                            move |view: &mut Self, _, _, cx| {
                                                view.goto_page(&tab_id, 0, cx);
                                            }
                                        })),
                                )
                                .child(
                                    Button::new(comp_id(["datatable-page-prev", &tab_id]))
                                        .small()
                                        .outline()
                                        .label("上一页")
                                        .on_click(cx.listener({
                                            let tab_id = tab_id.clone();
                                            move |view: &mut Self, _, _, cx| {
                                                view.goto_page(&tab_id, current_page.saturating_sub(1), cx);
                                            }
                                        })),
                                )
                            })
                            .child(div().text_sm().text_color(theme.foreground).child(format!(
                                "{} / {}",
                                current_page + 1,
                                total_pages
                            )))
                            .when(current_page + 1 < total_pages, |this| {
                                this.child(
                                    Button::new(comp_id(["datatable-page-next", &tab_id]))
                                        .small()
                                        .outline()
                                        .label("下一页")
                                        .on_click(cx.listener({
                                            let tab_id = tab_id.clone();
                                            move |view: &mut Self, _, _, cx| {
                                                view.goto_page(&tab_id, current_page + 1, cx);
                                            }
                                        })),
                                )
                                .child(
                                    Button::new(comp_id(["datatable-page-last", &tab_id]))
                                        .small()
                                        .outline()
                                        .label("末页")
                                        .on_click(cx.listener({
                                            let tab_id = tab_id.clone();
                                            move |view: &mut Self, _, _, cx| {
                                                view.goto_page(&tab_id, total_pages.saturating_sub(1), cx);
                                            }
                                        })),
                                )
                            }),
                    ),
            )
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

        let sidebar = self.tables.iter().cloned().fold(
            full_col()
                .id(comp_id(["mysql-sidebar", id]))
                .p_2()
                .gap_2()
                .scrollable(Axis::Vertical),
            |acc, table| {
                let active = self.active_table.as_ref() == Some(&table);
                let active_table = table.clone();
                acc.child(
                    full_row()
                        .id(comp_id(["mysql-sidebar-item", &self.meta.id, &table]))
                        .px_4()
                        .py_2()
                        .gap_2()
                        .rounded_lg()
                        .items_center()
                        .text_sm()
                        .text_color(theme.foreground)
                        .when_else(
                            active,
                            |this| this.bg(theme.list_active).font_semibold(),
                            |this| this.hover(|this| this.bg(theme.list_hover)),
                        )
                        .on_double_click(cx.listener(move |this, _, window, cx| {
                            this.create_tab_table_data(active_table.clone(), window, cx);
                        }))
                        .child(icon_sheet())
                        .child(table.clone()),
                )
            },
        );

        let container = full_col()
            .child(
                TabBar::new(comp_id(["mysql-tabs", id]))
                    .with_size(Size::Medium)
                    .children(
                        self.tabs
                            .iter()
                            .enumerate()
                            .map(|(_, tab)| {
                                let tab_id = tab.id.clone();
                                let mut tab_item = Tab::new(tab.title.clone())
                                    .id(comp_id(["mysql-tabs-item", id, &tab_id]))
                                    .px_2()
                                    .selected(tab.id == self.active_tab)
                                    .on_click(cx.listener({
                                        let tab_id = tab.id.clone();
                                        let tab_title = tab.title.clone();
                                        move |view: &mut Self, _, _, cx| {
                                            view.active_tab(tab_id.clone(), tab_title.clone(), cx);
                                        }
                                    }));

                                if tab.closable {
                                    tab_item = tab_item.suffix(
                                        Button::new(comp_id(["mysql-tabs-close", &tab_id]))
                                            .ghost()
                                            .xsmall()
                                            .compact()
                                            .tab_stop(false)
                                            .icon(icon_close().with_size(Size::XSmall))
                                            .on_click(cx.listener(move |view: &mut Self, _, _, cx| {
                                                view.close_tab(&tab_id, cx);
                                            })),
                                    )
                                }
                                tab_item
                            })
                            .collect::<Vec<_>>(),
                    ),
            )
            .child(
                full_col()
                    .id(comp_id(["mysql-main", id]))
                    .p_2()
                    .child(match self.active_content() {
                        Some(TabContent::Overview) | None => self.render_overview(cx),
                        Some(TabContent::DataTable(tab)) => self.render_datatable(&tab, cx),
                    }),
            )
            .into_any_element();

        full_col()
            .id(comp_id(["mysql", id]))
            .child(
                div()
                    .id(comp_id(["mysql-header", id]))
                    .flex()
                    .flex_row()
                    .p_4()
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
                    ),
            )
            .child(
                full_col()
                    .id(comp_id(["mysql-content", id]))
                    .child(
                        h_resizable(comp_id(["mysql-content", id]), self.sidebar_resize.clone())
                            .child(
                                resizable_panel()
                                    .size(px(240.0))
                                    .size_range(px(120.)..px(360.))
                                    .child(sidebar),
                            )
                            .child(container),
                    )
                    .child(div()),
            )
    }
}

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

enum TabContent {
    Overview,
    DataTable(DataTableTab),
}

struct DataTableTab {
    id: SharedString,
    table: SharedString,
    content: Entity<gpui_component::table::Table<DataTable>>,
    filter: Entity<InputState>,
    current_page: usize,
    page_size: usize,
    total_rows: usize,
    sort_column: Option<String>,
    sort_ascending: bool,
    // 新增：筛选排序面板状态
    filter_panel_open: bool,
    filter_rules: Vec<FilterRule>,
    sort_rules: Vec<SortRule>,
}

struct FilterRule {
    id: SharedString,
    field_dropdown: Entity<DropdownState<Vec<SharedString>>>,
    operator_dropdown: Entity<DropdownState<Vec<SharedString>>>,
    operator: FilterOperator,
    value_input: Entity<InputState>,
}

#[derive(Clone, Debug, PartialEq)]
enum FilterOperator {
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
    GreaterOrEqual,
    LessOrEqual,
    Like,
    NotLike,
    IsNull,
    IsNotNull,
}

impl FilterOperator {
    fn all() -> Vec<Self> {
        vec![
            Self::Equal,
            Self::NotEqual,
            Self::GreaterThan,
            Self::LessThan,
            Self::GreaterOrEqual,
            Self::LessOrEqual,
            Self::Like,
            Self::NotLike,
            Self::IsNull,
            Self::IsNotNull,
        ]
    }

    fn label(&self) -> &str {
        match self {
            Self::Equal => "=",
            Self::NotEqual => "!=",
            Self::GreaterThan => ">",
            Self::LessThan => "<",
            Self::GreaterOrEqual => ">=",
            Self::LessOrEqual => "<=",
            Self::Like => "LIKE",
            Self::NotLike => "NOT LIKE",
            Self::IsNull => "IS NULL",
            Self::IsNotNull => "IS NOT NULL",
        }
    }
}

struct SortRule {
    id: SharedString,
    field_dropdown: Entity<DropdownState<Vec<SharedString>>>,
    ascending: bool,
}

struct TablePage {
    headers: Vec<SharedString>,
    rows: Vec<Vec<SharedString>>,
    total_rows: usize,
}
