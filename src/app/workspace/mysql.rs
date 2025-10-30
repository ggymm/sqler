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
use crate::build::ConditionValue;
use crate::build::DatabaseType;
use crate::build::FilterCondition;
use crate::build::Operator;
use crate::build::QueryConditions;
use crate::build::SortOrder;
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
        tab_id: &SharedString,
        cx: &mut Context<Self>,
    ) {
        if let Some(i) = self.tabs.iter().position(|tab| &tab.id == tab_id && tab.closable) {
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

    fn active_session(&mut self) -> Result<&mut (dyn DatabaseSession + '_), DriverError> {
        if self.session.is_none() {
            let DataSourceOptions::MySQL(opts) = &self.meta.options else {
                return Err(DriverError::InvalidField("数据源类型不匹配".into()));
            };

            self.session = Some(MySQLDriver.create_connection(opts)?);
        }

        match self.session.as_deref_mut() {
            Some(session) => Ok(session),
            None => Err(DriverError::Other("MySQL 连接不可用".into())),
        }
    }

    fn refresh_tables(
        &mut self,
        cx: &mut Context<Self>,
    ) {
        let Some(database) = self.database.clone() else {
            self.tables = self.meta.tables();
            self.active_tab = TabItem::overview().id;
            self.active_table = None;
            cx.notify();
            return;
        };

        // 重新查询表
        let ret: Result<Vec<SharedString>, DriverError> = (|| {
            let session = self.active_session()?;

            let statement = format!("SHOW TABLES FROM `{}`", escape_mysql_identifier(&database));
            let rows = match session.query(QueryReq::Sql { statement })? {
                QueryResp::Rows(rows) => rows,
                _ => return Ok(vec![]),
            };

            let mut tables = Vec::new();
            for row in rows {
                if let Some(value) = row.values().next() {
                    tables.push(super::format_cell(value));
                }
            }
            Ok(tables)
        })();

        // 更新本地数据
        self.tables = match ret {
            Ok(tables) => tables,
            Err(err) => {
                eprintln!("刷新 MySQL 表列表失败: {}", err);
                if !self.tables.is_empty() {
                    return;
                }
                self.meta.tables()
            }
        };
        self.active_tab = TabItem::overview().id;
        self.active_table = None;

        // 清除失效的标签页
        self.tabs.retain(|tab| match &tab.content {
            TabContent::Table(tab) => self.tables.iter().any(|t| t == &tab.table),
            _ => true,
        });

        cx.notify();
    }

    fn table_content(
        &mut self,
        tab_id: &SharedString,
    ) -> Option<&mut TableContent> {
        self.tabs.iter_mut().find(|tab| tab.id == *tab_id).and_then(|item| {
            if let TabContent::Table(tab) = &mut item.content {
                Some(tab)
            } else {
                None
            }
        })
    }

    fn create_table_tab(
        &mut self,
        table: SharedString,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let id = SharedString::from(format!("mysql-tab-table-data-{}-{}", self.meta.id, table));

        // 检查标签页是否已存在
        if let Some(existing) = self.tabs.iter().find(|tab| {
            matches!(
                &tab.content,
                TabContent::Table(current) if current.id == id
            )
        }) {
            self.active_tab = existing.id.clone();
            self.active_table = Some(table.clone());
            cx.notify();
            return;
        }

        let content = DataTable::new(vec![], Vec::new()).build(window, cx);

        self.tabs.push(TabItem {
            id: id.clone(),
            title: table.clone(),
            content: TabContent::Table(TableContent {
                id: id.clone(),
                table: table.clone(),
                content,
                columns: vec![],
                current_page: 0,
                page_size: DEFAULT_PAGE_SIZE,
                total_rows: 0,
                filter_enable: false,
                query_rules: Vec::new(),
                sort_rules: Vec::new(),
            }),
            closable: true,
        });

        self.active_tab = id.clone();
        self.active_table = Some(table.clone());
        cx.notify();

        // 调用 reload_table_tab 加载数据
        self.reload_table_tab(&id, window, cx);
    }

    fn reload_table_tab(
        &mut self,
        tab_id: &SharedString,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let (table, mut page, size, total, mut conditions) = {
            let Some(content) = self.table_content(tab_id) else {
                return;
            };

            // 构建查询条件
            let mut conditions = QueryConditions::default();

            // 转换筛选规则
            for rule in &content.query_rules {
                // 读取字段名
                let Some(field) = rule.field.read(cx).selected_value() else {
                    continue;
                };

                // 读取操作符
                let Some(operator_label) = rule.operator.read(cx).selected_value() else {
                    continue;
                };
                let operator = Operator::from_label(operator_label.as_ref());

                // 读取值
                let value_text = rule.value.read(cx).text().to_string();
                if value_text.trim().is_empty()
                    && !matches!(operator, Operator::IsNull | Operator::IsNotNull)
                {
                    continue;
                }

                // 构建条件值
                let value = match operator {
                    Operator::IsNull | Operator::IsNotNull => ConditionValue::Null,
                    _ => ConditionValue::String(value_text),
                };

                conditions.filters.push(FilterCondition {
                    field: field.to_string(),
                    operator,
                    value,
                });
            }

            // 转换排序规则
            for rule in &content.sort_rules {
                // 读取字段名
                let Some(field) = rule.field.read(cx).selected_value() else {
                    continue;
                };

                conditions.sorts.push(SortOrder {
                    field: field.to_string(),
                    ascending: rule.ascending,
                });
            }

            (
                content.table.clone(),
                content.current_page,
                content.page_size,
                content.total_rows,
                conditions,
            )
        };

        // 设置分页
        conditions.limit = Some(size);
        conditions.offset = Some(page * size);

        let max_page = if total == 0 {
            0
        } else {
            (total.saturating_sub(1)) / size
        };
        if page > max_page {
            page = max_page;
        }

        // 获取或创建连接，然后将其移动到后台任务
        let session = match self.active_session() {
            Ok(_) => self.session.take(),
            Err(err) => {
                eprintln!("获取数据库连接失败: {}", err);
                return;
            }
        };

        let Some(session) = session else {
            return;
        };

        let tab_id = tab_id.clone();

        // 在后台线程执行数据库查询
        cx.spawn_in(window, async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move {
                    // 使用传递过来的连接
                    let mut session = session;

                    // 执行查询
                    let builder = create_builder(DatabaseType::MySQL);

                    // 查询列名
                    let statement = format!("SHOW COLUMNS FROM `{}`", escape_mysql_identifier(&table));
                    let resp = session.query(QueryReq::Sql { statement })?;
                    let column_rows = match resp {
                        QueryResp::Rows(rows) => rows,
                        _ => vec![],
                    };

                    let mut columns = Vec::new();
                    for row in column_rows {
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
                    println!("{:?}", columns);

                    // 查询数据
                    let (query_sql, _params) = builder.build_select_query(&table, &[], &conditions);
                    println!("{}", query_sql);

                    let resp = session.query(QueryReq::Sql { statement: query_sql })?;
                    let rows = match resp {
                        QueryResp::Rows(rows) => rows,
                        _ => Vec::new(),
                    };

                    // 转换行数据
                    let mut converted_rows = Vec::with_capacity(rows.len());
                    for row in rows {
                        let mut record = Vec::with_capacity(columns.len());
                        for name in &columns {
                            let value = row.get(name).unwrap_or(&Value::Null);
                            record.push(super::format_cell(value));
                        }
                        converted_rows.push(record);
                    }

                    // 查询总行数
                    let count_conditions = QueryConditions {
                        filters: conditions.filters.clone(),
                        sorts: Vec::new(),
                        limit: None,
                        offset: None,
                    };
                    let (count_sql, _) = builder.build_count_query(&table, &count_conditions);
                    println!("{}", count_sql);
                    let count_resp = session.query(QueryReq::Sql { statement: count_sql })?;
                    let total_rows = match count_resp {
                        QueryResp::Rows(count_rows) => count_rows
                            .first()
                            .and_then(|row| row.values().next())
                            .map(super::parse_count)
                            .unwrap_or(0),
                        _ => 0,
                    };

                    let headers: Vec<SharedString> = columns.iter().map(|s| SharedString::from(s.clone())).collect();

                    Ok::<_, DriverError>((
                        TablePage {
                            columns: headers,
                            rows: converted_rows,
                            total_rows,
                        },
                        session,
                    ))
                })
                .await;

            // 更新 UI
            let _ = cx.update(|_window, cx| {
                let _ = this.update(cx, |this, cx| match result {
                    Ok((table_page, session)) => {
                        // 归还连接
                        this.session = Some(session);

                        let Some(content) = this.table_content(&tab_id) else {
                            return;
                        };

                        // 提前解构 table_page，避免闭包捕获整个结构体导致所有权问题
                        let TablePage {
                            columns,
                            rows,
                            total_rows,
                        } = table_page;

                        content.current_page = page;
                        content.page_size = size;
                        content.total_rows = total_rows;
                        content.columns = columns.clone();

                        content.content.update(cx, |t, cx| {
                            t.delegate_mut().update_data(columns, rows);
                            t.refresh(cx); // 重新准备列/行结构
                            cx.notify();
                        });

                        cx.notify();
                    }
                    Err(err) => {
                        eprintln!("加载数据表失败: {}", err);
                        this.session = None;
                    }
                });
            });
        })
        .detach();
    }

    fn render_table_tab(
        &self,
        tab: &TableContent,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let theme = cx.theme().clone();
        let tab_id = tab.id.clone();

        // 获取列名
        let headers = &tab.columns;

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
                move |view: &mut Self, _, window, cx| {
                    if let Some(content) = view.table_content(&tab_id) {
                        content.current_page = prev_page;
                    }
                    view.reload_table_tab(&tab_id, window, cx);
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
                move |view: &mut Self, _, window, cx| {
                    if let Some(content) = view.table_content(&tab_id) {
                        content.current_page = next_page;
                    }
                    view.reload_table_tab(&tab_id, window, cx);
                }
            }));
        let create_sort_btn = Button::new(comp_id(["filter-panel-add-sort", &tab_id]))
            .small()
            .outline()
            .label("新增排序")
            .on_click(cx.listener({
                let tab_id = tab_id.clone();
                let headers = headers.clone();
                move |view: &mut Self, _, window, cx| {
                    if let Some(content) = view.table_content(&tab_id) {
                        let id = SharedString::from(Uuid::new_v4().to_string());
                        let field = cx.new(|cx| DropdownState::new(headers.clone(), None, window, cx));

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
                let headers = headers.clone();
                move |view: &mut Self, _, window, cx| {
                    if let Some(content) = view.table_content(&tab_id) {
                        let id = SharedString::from(Uuid::new_v4().to_string());

                        let field = cx.new(|cx| DropdownState::new(headers.clone(), None, window, cx));
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
                move |view: &mut Self, _, window, cx| {
                    // TODO: 应用所有筛选和排序规则
                    if let Some(content) = view.table_content(&tab_id) {
                        content.current_page = 0;
                    }
                    view.reload_table_tab(&tab_id, window, cx);
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

    fn create_query_tab(
        &mut self,
        _cx: &mut Context<Self>,
    ) {
    }

    fn render_overview_tab(
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
                    .child(
                        match self
                            .tabs
                            .iter()
                            .find(|tab| tab.id == self.active_tab)
                            .map(|tab| &tab.content)
                        {
                            Some(TabContent::Table(content)) => self.render_table_tab(&content, cx),
                            Some(TabContent::Overview) | None => self.render_overview_tab(cx),
                        },
                    )
                    .child(div()),
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
                            .label("新建查询")
                            .on_click(cx.listener(|view: &mut Self, _, _, cx| {
                                view.create_query_tab(cx);
                            })),
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
    columns: Vec<SharedString>,
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
    columns: Vec<SharedString>,
    rows: Vec<Vec<SharedString>>,
    total_rows: usize,
}
