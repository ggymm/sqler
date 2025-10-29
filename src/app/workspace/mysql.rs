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
use gpui_component::table::Table;
use gpui_component::ActiveTheme;
use gpui_component::Disableable;
use gpui_component::InteractiveElementExt;
use gpui_component::Selectable;
use gpui_component::Sizable;
use gpui_component::Size;
use gpui_component::StyledExt;
use serde_json::Value;
use uuid::Uuid;

use crate::app::comps::comp_id;
use crate::app::comps::icon_close;
use crate::app::comps::icon_export;
use crate::app::comps::icon_import;
use crate::app::comps::icon_relead;
use crate::app::comps::icon_search;
use crate::app::comps::icon_sheet;
use crate::app::comps::icon_trash;
use crate::app::comps::DataTable;
use crate::app::comps::DivExt;
use crate::build::create_builder;
use crate::build::DatabaseType;
use crate::build::Operator;
use crate::build::QueryConditions;
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
    session: Option<Box<dyn DatabaseSession>>,
    database: Option<String>,

    tabs: Vec<TabItem>,
    active_tab: SharedString,
    tables: Vec<SharedString>,
    active_table: Option<SharedString>,
    sidebar_resize: Entity<ResizableState>,
}

impl MySQLWorkspace {
    pub fn new(
        meta: DataSource,
        cx: &mut Context<Self>,
    ) -> Self {
        let overview = TabItem::overview();
        let active_tab = overview.id.clone();

        let tables = meta.tables();
        let database = if let DataSourceOptions::MySQL(opts) = &meta.options {
            let db = opts.database.trim();
            if db.is_empty() {
                None
            } else {
                Some(db.to_string())
            }
        } else {
            None
        };

        Self {
            meta,
            session: None,
            database,

            tabs: vec![overview],
            active_tab,
            tables,
            active_table: None,
            sidebar_resize: ResizableState::new(cx),
        }
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

    fn check_session(&mut self) -> Result<(), DriverError> {
        if self.session.is_some() {
            return Ok(());
        }

        let DataSourceOptions::MySQL(opts) = &self.meta.options else {
            return Err(DriverError::InvalidField("数据源类型不匹配".into()));
        };
        MySQLDriver.create_connection(opts).map(|s| self.session = Some(s))
    }

    fn fetch_page(
        &mut self,
        table: &str,
        conditions: &QueryConditions,
        columns: Option<Vec<String>>,
    ) -> Result<TablePage, DriverError> {
        self.check_session()?;
        let session = self.session.as_deref_mut().unwrap();

        let builder = create_builder(DatabaseType::MySQL);

        // 使用 SELECT * 查询所有列
        let (query_sql, _params) = builder.build_select_query(table, &[], conditions);

        let resp = session.query(QueryReq::Sql { statement: query_sql })?;
        let rows = match resp {
            QueryResp::Rows(rows) => rows,
            _ => Vec::new(),
        };

        // 从返回数据中提取列名，如果没有数据则使用传入的列名或查询
        let column_names = if let Some(first_row) = rows.first() {
            first_row.keys().cloned().collect()
        } else {
            match &columns {
                Some(cols) => cols.clone(),
                None => Self::fetch_column_names(session, table)?,
            }
        };

        // 转换行数据
        let mut converted_rows = Vec::with_capacity(rows.len());
        for row in rows {
            let mut record = Vec::with_capacity(column_names.len());
            for name in &column_names {
                let value = row.get(name).unwrap_or(&Value::Null);
                record.push(super::format_cell(value));
            }
            converted_rows.push(record);
        }

        // 构建并执行 COUNT 查询
        let count_conditions = QueryConditions {
            filters: conditions.filters.clone(),
            sorts: Vec::new(),
            limit: None,
            offset: None,
        };
        let (count_sql, _count_params) = builder.build_count_query(table, &count_conditions);

        // 执行 COUNT 查询获取总行数
        let count_resp = session.query(QueryReq::Sql { statement: count_sql })?;
        let total_rows = match count_resp {
            QueryResp::Rows(count_rows) => count_rows
                .first()
                .and_then(|row| row.values().next())
                .map(super::parse_count)
                .unwrap_or(0),
            _ => 0,
        };

        Ok(TablePage {
            headers: column_names.iter().map(|s| SharedString::from(s.clone())).collect(),
            rows: converted_rows,
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

    fn refresh_tables(
        &mut self,
        cx: &mut Context<Self>,
    ) {
        // 尝试从数据库查询表列表
        let new_tables = match self.database.clone() {
            Some(database) => {
                let result: Result<Vec<SharedString>, DriverError> = (|| {
                    self.check_session()?;
                    let session = self.session.as_deref_mut().unwrap();

                    // 执行 SHOW TABLES 查询
                    let statement = format!("SHOW TABLES FROM `{}`", escape_mysql_identifier(&database));
                    let resp = session.query(QueryReq::Sql { statement })?;
                    let rows = match resp {
                        QueryResp::Rows(rows) => rows,
                        _ => return Ok(vec![]),
                    };

                    // 提取表名
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
                })();

                match result {
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
                }
            }
            None => {
                // 没有配置数据库，使用缓存的表列表
                self.meta.tables()
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
            TabContent::Table(tab) => current_tables.iter().any(|name| name == &tab.table),
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

    fn create_table_tab(
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
                TabContent::Table(current) if current.id == id
            )
        }) {
            self.active_tab = existing.id.clone();
            cx.notify();
            return;
        }

        let conditions = QueryConditions {
            filters: Vec::new(),
            sorts: Vec::new(),
            limit: Some(DEFAULT_PAGE_SIZE),
            offset: Some(0),
        };
        let page = self.fetch_page(&table, &conditions, None).unwrap_or_else(|err| {
            eprintln!("加载数据表失败: {}", err);
            TablePage {
                headers: Vec::new(),
                rows: Vec::new(),
                total_rows: 0,
            }
        });

        let data_table = DataTable::new(page.headers.clone(), page.rows.clone()).build(window, cx);

        self.tabs.push(TabItem {
            id: id.clone(),
            title: table.clone(),
            content: TabContent::Table(TableContent {
                id: id.clone(),
                table: table.clone(),
                content: data_table,
                current_page: 0,
                page_size: DEFAULT_PAGE_SIZE,
                total_rows: page.total_rows,
                filter_enable: false,
                query_rules: Vec::new(),
                sort_rules: Vec::new(),
            }),
            closable: true,
        });
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

        let (table_name, page_size, total_rows, columns, mut query_conditions) = {
            let TabContent::Table(data_tab) = &self.tabs[tab_index].content else {
                return;
            };

            // 构建查询条件
            let conditions = QueryConditions::default();

            // 转换筛选规则
            for rule in &data_tab.query_rules {
                // TODO: 从 dropdown 读取选中的值
                // 暂时跳过，等 gpui-component API 确定后再实现
                let _ = rule;
            }

            // 转换排序规则
            for rule in &data_tab.sort_rules {
                // TODO: 从 dropdown 读取选中的值
                // 暂时跳过，等 gpui-component API 确定后再实现
                let _ = rule;
            }

            // 从 DataTable 中获取列名，避免重复查询
            let cols = data_tab.content.read(cx).delegate().columns().to_vec();

            (
                data_tab.table.clone(),
                data_tab.page_size,
                data_tab.total_rows,
                cols.into_iter().map(|s| s.to_string()).collect::<Vec<String>>(),
                conditions,
            )
        };

        // 设置分页
        query_conditions.limit = Some(page_size);
        query_conditions.offset = Some(page * page_size);

        let max_page = if total_rows == 0 {
            0
        } else {
            (total_rows.saturating_sub(1)) / page_size
        };
        if page > max_page {
            page = max_page;
        }

        match self.fetch_page(&table_name, &query_conditions, Some(columns)) {
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

                if let TabContent::Table(data_tab) = &mut self.tabs[tab_index].content {
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

    fn table_render(
        &self,
        tab: &TableContent,
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

        let column_btn = Button::new(comp_id(["table-choose-column", &tab_id]))
            .small()
            .outline()
            .label("列筛选");
        let filter_btn = Button::new(comp_id(["table-toggle-filter", &tab_id]))
            .small()
            .outline()
            .label(if tab.filter_enable {
                "隐藏筛选"
            } else {
                "数据筛选"
            })
            .on_click(cx.listener({
                let tab_id = tab_id.clone();
                move |view: &mut Self, _, _, cx| {
                    if let Some(content) = view.table_content(&tab_id) {
                        content.filter_enable = !content.filter_enable;
                    }
                    cx.notify();
                }
            }));

        let page_prev_btn = Button::new(comp_id(["table-page-prev", &tab_id]))
            .small()
            .outline()
            .label("上一页")
            .disabled(current_page == 0)
            .on_click(cx.listener({
                let tab_id = tab_id.clone();
                let prev_page = current_page.saturating_sub(1);
                move |view: &mut Self, _, _, cx| {
                    view.reload_table_tab(&tab_id, prev_page, cx);
                }
            }));
        let page_next_btn = Button::new(comp_id(["table-page-next", &tab_id]))
            .small()
            .outline()
            .label("下一页")
            .disabled(current_page + 1 >= total_pages)
            .on_click(cx.listener({
                let tab_id = tab_id.clone();
                let next_page = current_page.saturating_add(1);
                move |view: &mut Self, _, _, cx| {
                    view.reload_table_tab(&tab_id, next_page, cx);
                }
            }));
        let create_sort_btn = Button::new(comp_id(["filter-panel-add-sort", &tab_id]))
            .small()
            .outline()
            .label("新增排序")
            .on_click(cx.listener({
                let tab_id = tab_id.clone();
                let columns = columns.to_vec();
                move |view: &mut Self, _, window, cx| {
                    if let Some(content) = view.table_content(&tab_id) {
                        let id = SharedString::from(Uuid::new_v4().to_string());
                        let field = cx.new(|cx| DropdownState::new(columns.to_vec(), None, window, cx));

                        content.sort_rules.push(SortRule {
                            id,
                            field,
                            ascending: true,
                        });
                    }
                    cx.notify();
                }
            }));
        let create_query_btn = Button::new(comp_id(["filter-panel-add-filter", &tab_id]))
            .small()
            .outline()
            .label("新增筛选")
            .on_click(cx.listener({
                let tab_id = tab_id.clone();
                let columns = columns.to_vec();
                move |view: &mut Self, _, window, cx| {
                    if let Some(content) = view.table_content(&tab_id) {
                        let id = SharedString::from(Uuid::new_v4().to_string());

                        let field = cx.new(|cx| DropdownState::new(columns.to_vec(), None, window, cx));
                        let operator = cx.new(|cx| {
                            DropdownState::new(
                                Operator::all()
                                    .into_iter()
                                    .map(|op| SharedString::from(op.label().to_string()))
                                    .collect(),
                                None,
                                window,
                                cx,
                            )
                        });
                        let value = cx.new(|cx| InputState::new(window, cx));

                        content.query_rules.push(QueryRule {
                            id,
                            field,
                            operator,
                            value,
                        });
                    }
                    cx.notify();
                }
            }));
        let apply_cond_btn = Button::new(comp_id(["filter-panel-apply", &tab_id]))
            .small()
            .primary()
            .label("应用条件")
            .on_click(cx.listener({
                let tab_id = tab_id.clone();
                move |view: &mut Self, _, _, cx| {
                    // TODO: 应用所有筛选和排序规则
                    view.reload_table_tab(&tab_id, 0, cx);
                }
            }));
        let clear_cond_btn = Button::new(comp_id(["filter-panel-clear", &tab_id]))
            .small()
            .outline()
            .label("清除条件")
            .on_click(cx.listener({
                let tab_id = tab_id.clone();
                move |view: &mut Self, _, _, cx| {
                    if let Some(content) = view.table_content(&tab_id) {
                        content.sort_rules.clear();
                        content.query_rules.clear();
                    }
                    cx.notify();
                }
            }));

        div()
            .flex()
            .flex_1()
            .flex_col()
            .gap_2()
            .when(tab.filter_enable, |this| {
                this.child(
                    div()
                        .flex()
                        .flex_col()
                        .p_3()
                        .gap_3()
                        .rounded_lg()
                        .border_1()
                        .border_color(theme.border)
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_2()
                                .children(tab.sort_rules.iter().map(|rule| {
                                    let rule_id = rule.id.clone();
                                    let ascending = rule.ascending.clone();

                                    let field = Dropdown::new(&rule.field).small().placeholder("选择字段");

                                    div()
                                        .flex()
                                        .flex_1()
                                        .flex_row()
                                        .gap_2()
                                        .w_full()
                                        .items_center()
                                        .child(div().w_48().child(field))
                                        .child(
                                            // 排序方向选择
                                            div()
                                                .flex()
                                                .flex_row()
                                                .items_center()
                                                .gap_2()
                                                .child(div().text_sm().text_color(theme.muted_foreground).child("降序"))
                                                .child({
                                                    Switch::new(comp_id(["sort-ascending", &rule_id]))
                                                        .checked(ascending)
                                                        .on_click(cx.listener({
                                                            let tab_id = tab_id.clone();
                                                            let rule_id = rule_id.clone();
                                                            move |view: &mut Self, _, _, cx| {
                                                                if let Some(content) = view.table_content(&tab_id) {
                                                                    if let Some(rule) = content
                                                                        .sort_rules
                                                                        .iter_mut()
                                                                        .find(|r| &r.id == &rule_id)
                                                                    {
                                                                        rule.ascending = !rule.ascending;
                                                                    }
                                                                }
                                                                cx.notify();
                                                            }
                                                        }))
                                                })
                                                .child(
                                                    div().text_sm().text_color(theme.muted_foreground).child("升序"),
                                                ),
                                        )
                                        .child({
                                            Button::new(comp_id(["sort-remove", &rule_id]))
                                                .outline()
                                                .icon(icon_trash())
                                                .on_click(cx.listener({
                                                    let tab_id = tab_id.clone();
                                                    move |view: &mut Self, _, _, cx| {
                                                        if let Some(content) = view.table_content(&tab_id) {
                                                            content.sort_rules.retain(|r| &r.id != &rule_id);
                                                        }
                                                        cx.notify();
                                                    }
                                                }))
                                        })
                                })),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_2()
                                .children(tab.query_rules.iter().map(|rule| {
                                    let rule_id = rule.id.clone();
                                    let rule_field = Dropdown::new(&rule.field).small().placeholder("选择字段");
                                    let rule_operator = Dropdown::new(&rule.operator).small().placeholder("选择条件");
                                    div()
                                        .flex()
                                        .flex_1()
                                        .flex_row()
                                        .gap_2()
                                        .w_full()
                                        .items_center()
                                        .child(div().w_48().child(rule_field))
                                        .child(div().w_48().child(rule_operator))
                                        .child(div().flex().flex_1().child(TextInput::new(&rule.value).small()))
                                        .child(
                                            Button::new(comp_id(["filter-remove", &rule_id]))
                                                .outline()
                                                .icon(icon_trash())
                                                .on_click(cx.listener({
                                                    let tab_id = tab_id.clone();
                                                    move |view: &mut Self, _, _, cx| {
                                                        if let Some(content) = view.table_content(&tab_id) {
                                                            content.query_rules.retain(|r| &r.id != &rule_id);
                                                        }
                                                        cx.notify();
                                                    }
                                                })),
                                        )
                                })),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap_2()
                                .child(create_sort_btn)
                                .child(create_query_btn)
                                .child(div().flex_1())
                                .child(clear_cond_btn)
                                .child(apply_cond_btn),
                        ),
                )
            })
            .child(
                div()
                    .flex_1()
                    .rounded_lg()
                    .overflow_hidden()
                    .child(tab.content.clone())
                    .child(div()),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .p_2()
                    .gap_2()
                    .child(column_btn)
                    .child(filter_btn)
                    .child(div().flex_1())
                    .child(div().text_sm().child(format!(
                        "显示 {} - {} / 共 {} 条",
                        if tab.total_rows == 0 { 0 } else { start_row },
                        end_row,
                        tab.total_rows
                    )))
                    .child(div().flex_1())
                    .child(page_prev_btn)
                    .child(page_next_btn),
            )
            .into_any_element()
    }

    fn table_content(
        &mut self,
        id: &SharedString,
    ) -> Option<&mut TableContent> {
        self.tabs.iter_mut().find(|tab| tab.id == *id).and_then(|item| {
            if let TabContent::Table(tab) = &mut item.content {
                Some(tab)
            } else {
                None
            }
        })
    }

    fn overview_render(
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
            .gap_5()
            .col_full()
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
            div()
                .id(comp_id(["mysql-sidebar", id]))
                .p_2()
                .gap_2()
                .col_full()
                .scrollable(Axis::Vertical),
            |acc, table| {
                let active = self.active_table.as_ref() == Some(&table);
                let active_table = table.clone();
                acc.child(
                    div()
                        .id(comp_id(["mysql-sidebar-item", &self.meta.id, &table]))
                        .px_4()
                        .py_2()
                        .gap_2()
                        .row_full()
                        .items_center()
                        .text_sm()
                        .rounded_lg()
                        .when_else(
                            active,
                            |this| this.bg(theme.list_active).font_semibold(),
                            |this| this.hover(|this| this.bg(theme.list_hover)),
                        )
                        .on_double_click(cx.listener(move |this, _, window, cx| {
                            this.create_table_tab(active_table.clone(), window, cx);
                        }))
                        .child(icon_sheet())
                        .child(table.clone()),
                )
            },
        );

        let container = div()
            .col_full()
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
                                    .with_size(Size::Large)
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
                div()
                    .id(comp_id(["mysql-main", id]))
                    .p_2()
                    .col_full()
                    .child(match self.active_content() {
                        Some(TabContent::Table(tab)) => self.table_render(&tab, cx),
                        Some(TabContent::Overview) | None => self.overview_render(cx),
                    }),
            )
            .into_any_element();

        div()
            .id(comp_id(["mysql", id]))
            .col_full()
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
                div().id(comp_id(["mysql-content", id])).col_full().child(
                    h_resizable(comp_id(["mysql-content", id]), self.sidebar_resize.clone())
                        .child(
                            resizable_panel()
                                .size(px(240.0))
                                .size_range(px(120.)..px(360.))
                                .child(sidebar),
                        )
                        .child(container),
                ),
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
    Table(TableContent),
    Overview,
}

struct TableContent {
    id: SharedString,
    table: SharedString,
    content: Entity<Table<DataTable>>,
    current_page: usize,
    page_size: usize,
    total_rows: usize,
    sort_rules: Vec<SortRule>,
    query_rules: Vec<QueryRule>,
    filter_enable: bool,
}

struct QueryRule {
    id: SharedString,
    value: Entity<InputState>,
    field: Entity<DropdownState<Vec<SharedString>>>,
    operator: Entity<DropdownState<Vec<SharedString>>>,
}

struct SortRule {
    id: SharedString,
    field: Entity<DropdownState<Vec<SharedString>>>,
    ascending: bool,
}

struct TablePage {
    headers: Vec<SharedString>,
    rows: Vec<Vec<SharedString>>,
    total_rows: usize,
}
