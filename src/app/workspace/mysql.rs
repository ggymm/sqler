use gpui::{prelude::*, *};
use gpui_component::{
    button::{Button, ButtonVariants},
    dropdown::{Dropdown, DropdownState},
    input::{InputState, TextInput},
    resizable::{h_resizable, resizable_panel, ResizableState},
    table::Table,
    ActiveTheme, Disableable, Icon, InteractiveElementExt, Sizable, Size, StyledExt,
};
use uuid::Uuid;

use crate::{
    app::comps::{
        comp_id, icon_export, icon_import, icon_relead, icon_search, icon_sheet, icon_trash, DataTable, DivExt,
    },
    driver::{
        create_connection, ConditionValue, DatabaseSession, DriverError, FilterCond, Operator, OrderCond, QueryReq,
        QueryResp,
    },
    option::{DataSource, DataSourceOptions},
};

const PAGE_SIZE: usize = 100;
const ORDER_ASC: &str = "升序";
const ORDER_DESC: &str = "降序";

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

struct QueryRule {
    id: SharedString,
    value: Entity<InputState>,
    field: Entity<DropdownState<Vec<SharedString>>>,
    operator: Entity<DropdownState<Vec<SharedString>>>,
}

struct OrderRule {
    id: SharedString,
    field: Entity<DropdownState<Vec<SharedString>>>,
    order: Entity<DropdownState<Vec<SharedString>>>,
}

struct TableContent {
    id: SharedString,
    table: SharedString,
    columns: Vec<SharedString>,
    content: Entity<Table<DataTable>>,
    page_no: usize,
    page_size: usize,
    total_rows: usize,
    order_rules: Vec<OrderRule>,
    query_rules: Vec<QueryRule>,
    filter_enable: bool,
}

pub struct MySQLWorkspace {
    meta: DataSource,
    parent: WeakEntity<crate::app::SqlerApp>,
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
        parent: WeakEntity<crate::app::SqlerApp>,
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
            parent,
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
            self.session = Some(create_connection(self.meta.kind, &self.meta.options)?);
        }

        match self.session.as_deref_mut() {
            Some(session) => Ok(session),
            None => Err(DriverError::Other("MySQL 连接不可用".into())),
        }
    }

    fn reload_tables(
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
        let result: Result<Vec<SharedString>, DriverError> = (|| {
            let session = self.active_session()?;
            let sql = format!("SHOW TABLES FROM {}", &database);
            let rows = match session.query(QueryReq::Sql { sql, args: Vec::new() })? {
                QueryResp::Rows(rows) => rows,
                _ => return Ok(vec![]),
            };

            let mut tables = Vec::new();
            for row in rows {
                if let Some(value) = row.values().next() {
                    tables.push(SharedString::from(value.clone()));
                }
            }
            Ok(tables)
        })();

        // 更新本地数据
        self.tables = match result {
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
                columns: vec![],
                content,
                page_no: 0,
                page_size: PAGE_SIZE,
                total_rows: 0,
                filter_enable: false,
                query_rules: Vec::new(),
                order_rules: Vec::new(),
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
        let (table, mut page, size, total, orders, filters) = {
            let Some(content) = self.table_content(tab_id) else {
                return;
            };

            // 设置表格加载状态
            content.content.update(cx, |t, cx| {
                t.delegate_mut().update_loading(true);
                cx.notify();
            });

            // 转换排序规则
            let mut orders = Vec::new();
            for rule in &content.order_rules {
                // 读取字段名
                let Some(field) = rule.field.read(cx).selected_value() else {
                    continue;
                };

                // 读取排序方式
                let Some(order) = rule.order.read(cx).selected_value() else {
                    continue;
                };
                let ascending = order.as_ref() == ORDER_ASC;

                orders.push(OrderCond {
                    field: field.to_string(),
                    ascending,
                });
            }

            // 转换筛选规则
            let mut filters = Vec::new();
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
                let input = rule.value.read(cx).text().to_string();
                if input.trim().is_empty() && !matches!(operator, Operator::IsNull | Operator::IsNotNull) {
                    continue;
                }

                // 构建条件值
                let value = match operator {
                    Operator::IsNull | Operator::IsNotNull => ConditionValue::Null,
                    _ => ConditionValue::String(input),
                };

                filters.push(FilterCond {
                    field: field.to_string(),
                    operator,
                    value,
                });
            }

            (
                content.table.clone(),
                content.page_no,
                content.page_size,
                content.total_rows,
                orders,
                filters,
            )
        };

        // 确定页码
        let max_page = if total == 0 {
            0
        } else {
            (total.saturating_sub(1)) / size
        };
        if page > max_page {
            page = max_page;
        }

        // 设置分页
        let limit = Some(size);
        let offset = Some(page * size);

        let tab_id = tab_id.clone();
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

        // 在后台线程执行数据库查询
        cx.spawn_in(window, async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move {
                    // 使用传递过来的连接
                    let mut session = session;

                    // 查询列名
                    let sql = format!("SHOW COLUMNS FROM {}", &table);
                    let column_resp = session.query(QueryReq::Sql { sql, args: Vec::new() })?;
                    let column_rows = match column_resp {
                        QueryResp::Rows(rows) => rows,
                        _ => vec![],
                    };

                    let mut columns = Vec::new();
                    for row in column_rows {
                        if let Some(field) = row.get("Field") {
                            columns.push(field.to_string());
                            continue;
                        }

                        if let Some((_, value)) = row.into_iter().next() {
                            columns.push(value);
                        }
                    }

                    // 查询总数
                    let count_resp = session.query(QueryReq::Builder {
                        table: table.to_string(),
                        columns: vec!["COUNT(*)".to_string()],
                        limit: None,
                        offset: None,
                        orders: Vec::new(),
                        filters: filters.clone(),
                    })?;
                    let total_count = match count_resp {
                        QueryResp::Rows(count_rows) => count_rows
                            .first()
                            .and_then(|row| row.values().next())
                            .map(|s| super::parse_count(s.as_str()))
                            .unwrap_or(0),
                        _ => 0,
                    };

                    // 查询数据
                    let query_resp = session.query(QueryReq::Builder {
                        table: table.to_string(),
                        columns: Vec::new(),
                        limit,
                        offset,
                        orders,
                        filters,
                    })?;
                    let rows = match query_resp {
                        QueryResp::Rows(rows) => rows,
                        _ => Vec::new(),
                    };

                    // 构建渲染数据
                    let table_cols: Vec<SharedString> = columns.iter().map(|s| SharedString::from(s.clone())).collect();
                    let mut table_rows = Vec::with_capacity(rows.len());
                    for row in rows {
                        let mut record = Vec::with_capacity(columns.len());
                        for name in &columns {
                            let value = row.get(name).cloned().unwrap_or_default();
                            record.push(SharedString::from(value));
                        }
                        table_rows.push(record);
                    }

                    Ok::<_, DriverError>(((table_cols, table_rows, total_count), session))
                })
                .await;

            // 更新 UI
            let _ = cx.update(|_window, cx| {
                let _ = this.update(cx, |this, cx| match result {
                    Ok((data, session)) => {
                        this.session = Some(session);

                        let (columns, rows, total_rows) = data;
                        let Some(content) = this.table_content(&tab_id) else {
                            return;
                        };
                        content.page_no = page;
                        content.page_size = size;
                        content.total_rows = total_rows;
                        content.columns = columns.clone();

                        content.content.update(cx, |t, cx| {
                            t.delegate_mut().update_data(columns, rows);
                            t.delegate_mut().update_loading(false);
                            t.refresh(cx);
                            cx.notify();
                        });
                        cx.notify();
                    }
                    Err(err) => {
                        eprintln!("加载数据表失败: {}", err);
                        this.session = None;

                        if let Some(content) = this.table_content(&tab_id) {
                            content.content.update(cx, |t, cx| {
                                t.delegate_mut().update_loading(false);
                                cx.notify();
                            });
                        }
                        cx.notify();
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

        let order_ops = vec![SharedString::from(ORDER_ASC), SharedString::from(ORDER_DESC)];
        let filter_ops: Vec<SharedString> = Operator::all()
            .into_iter()
            .map(|op| SharedString::from(op.label().to_string()))
            .collect();

        // 获取列名
        let headers = &tab.columns;

        // 计算分页信息
        let total_pages = if tab.total_rows == 0 {
            1
        } else {
            (tab.total_rows + tab.page_size - 1) / tab.page_size
        };
        let current_page = tab.page_no;
        let start_row = current_page * tab.page_size + 1;
        let end_row = ((current_page + 1) * tab.page_size).min(tab.total_rows);

        let column_btn = Button::new(comp_id(["table-choose-column", &tab_id]))
            .outline()
            .label("字段筛选");
        let filter_btn = Button::new(comp_id(["table-toggle-filter", &tab_id]))
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
            .outline()
            .label("上一页")
            .disabled(current_page == 0)
            .on_click(cx.listener({
                let tab_id = tab_id.clone();
                let prev_page = current_page.saturating_sub(1);
                move |view: &mut Self, _, window, cx| {
                    if let Some(content) = view.table_content(&tab_id) {
                        content.page_no = prev_page;
                    }
                    view.reload_table_tab(&tab_id, window, cx);
                }
            }));
        let page_next_btn = Button::new(comp_id(["table-page-next", &tab_id]))
            .outline()
            .label("下一页")
            .disabled(current_page + 1 >= total_pages)
            .on_click(cx.listener({
                let tab_id = tab_id.clone();
                let next_page = current_page.saturating_add(1);
                move |view: &mut Self, _, window, cx| {
                    if let Some(content) = view.table_content(&tab_id) {
                        content.page_no = next_page;
                    }
                    view.reload_table_tab(&tab_id, window, cx);
                }
            }));
        let create_order_btn = Button::new(comp_id(["create-order", &tab_id]))
            .small()
            .outline()
            .label("新增排序")
            .on_click(cx.listener({
                let tab_id = tab_id.clone();
                let headers = headers.clone();
                move |view: &mut Self, _, window, cx| {
                    if let Some(content) = view.table_content(&tab_id) {
                        content.order_rules.push(OrderRule {
                            id: SharedString::from(Uuid::new_v4().to_string()),
                            field: cx.new(|cx| {
                                // rustfmt::skip
                                DropdownState::new(headers.clone(), None, window, cx)
                            }),
                            order: cx.new(|cx| {
                                // rustfmt::skip
                                DropdownState::new(order_ops.clone(), None, window, cx)
                            }),
                        });
                    }
                    cx.notify();
                }
            }));
        let create_query_btn = Button::new(comp_id(["create-filter", &tab_id]))
            .small()
            .outline()
            .label("新增筛选")
            .on_click(cx.listener({
                let tab_id = tab_id.clone();
                let headers = headers.clone();
                move |view: &mut Self, _, window, cx| {
                    if let Some(content) = view.table_content(&tab_id) {
                        content.query_rules.push(QueryRule {
                            id: SharedString::from(Uuid::new_v4().to_string()),
                            field: cx.new(|cx| {
                                // rustfmt::skip
                                DropdownState::new(headers.clone(), None, window, cx)
                            }),
                            operator: cx.new(|cx| {
                                // rustfmt::skip
                                DropdownState::new(filter_ops.clone(), None, window, cx)
                            }),
                            value: cx.new(|cx| InputState::new(window, cx)),
                        });
                    }
                    cx.notify();
                }
            }));
        let apply_cond_btn = Button::new(comp_id(["filter-apply", &tab_id]))
            .small()
            .outline()
            .label("应用条件")
            .on_click(cx.listener({
                let tab_id = tab_id.clone();
                move |view: &mut Self, _, window, cx| {
                    if let Some(content) = view.table_content(&tab_id) {
                        content.page_no = 0;
                    }
                    view.reload_table_tab(&tab_id, window, cx);
                }
            }));
        let clear_cond_btn = Button::new(comp_id(["filter-clear", &tab_id]))
            .small()
            .outline()
            .label("清除条件")
            .on_click(cx.listener({
                let tab_id = tab_id.clone();
                move |view: &mut Self, _, _, cx| {
                    if let Some(content) = view.table_content(&tab_id) {
                        content.order_rules.clear();
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
                        .p_2()
                        .border_1()
                        .rounded_lg()
                        .border_color(theme.border)
                        .child(div().flex().flex_col().children(tab.order_rules.iter().map(|rule| {
                            let rule_id = rule.id.clone();
                            let rule_field = Dropdown::new(&rule.field).small().placeholder("");
                            let rule_order = Dropdown::new(&rule.order).small().placeholder("");

                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .mb_2()
                                .gap_2()
                                .w_full()
                                .child(div().w_48().child(rule_field))
                                .child(div().w_48().child(rule_order))
                                .child(
                                    Button::new(comp_id(["order-remove", &rule_id]))
                                        .ghost()
                                        .icon(icon_trash())
                                        .on_click(cx.listener({
                                            let tab_id = tab_id.clone();
                                            move |view: &mut Self, _, _, cx| {
                                                if let Some(content) = view.table_content(&tab_id) {
                                                    content.order_rules.retain(|r| &r.id != &rule_id);
                                                }
                                                cx.notify();
                                            }
                                        })),
                                )
                        })))
                        .child(div().flex().flex_col().children(tab.query_rules.iter().map(|rule| {
                            let rule_id = rule.id.clone();
                            let rule_field = Dropdown::new(&rule.field).small().placeholder("");
                            let rule_operator = Dropdown::new(&rule.operator).small().placeholder("");

                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .mb_2()
                                .gap_2()
                                .w_full()
                                .child(div().w_48().child(rule_field))
                                .child(div().w_48().child(rule_operator))
                                .child(div().flex().flex_1().child(TextInput::new(&rule.value).small()))
                                .child(
                                    Button::new(comp_id(["filter-remove", &rule_id]))
                                        .ghost()
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
                        })))
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap_2()
                                .child(create_order_btn)
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
                    .gap_2()
                    .child(filter_btn)
                    .child(column_btn)
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

impl Render for MySQLWorkspace {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let id = &self.meta.id;
        let theme = cx.theme().clone();
        let active = &self.active_tab;

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
            .p_2()
            .gap_2()
            .col_full()
            .child(
                div()
                    .id(comp_id(["mysql-tabs", id]))
                    .flex()
                    .flex_row()
                    .gap_2()
                    .min_w_0()
                    .children(self.tabs.iter().map(|tab| {
                        let tab_id = tab.id.clone();
                        let tab_active = &tab_id == active;

                        let mut item = div()
                            .id(comp_id(["mysql-tabs-item", &tab_id]))
                            .flex()
                            .flex_row()
                            .items_center()
                            .justify_center()
                            .px_2()
                            .py_1()
                            .gap_2()
                            .border_1()
                            .border_color(theme.border)
                            .rounded_lg()
                            .text_sm()
                            .cursor_pointer()
                            .when(tab_active, |this| {
                                this.bg(theme.tab_active).text_color(theme.tab_active_foreground)
                            })
                            .when(!tab_active, |this| {
                                this.bg(theme.tab_bar).text_color(theme.muted_foreground)
                            })
                            .on_click(cx.listener({
                                let tab_id = tab.id.clone();
                                let tab_title = tab.title.clone();
                                move |this, _, _, cx| {
                                    this.active_tab(tab_id.clone(), tab_title.clone(), cx);
                                }
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
                                Button::new(comp_id(["mysql-tabs-close", &tab_id]))
                                    .ghost()
                                    .xsmall()
                                    .compact()
                                    .tab_stop(false)
                                    .icon(Icon::default().path("icons/close.svg").with_size(Size::Small))
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.close_tab(&tab_id, cx);
                                    })),
                            );
                        }

                        {
                            let style = item.style();
                            style.flex_grow = Some(0.);
                            style.flex_shrink = Some(1.);
                            style.flex_basis = Some(Length::Definite(px(120.).into()));
                            style.min_size.width = Some(Length::Definite(px(0.).into()));
                        }

                        item.into_any_element()
                    })),
            )
            .child(
                div()
                    .id(comp_id(["mysql-main", id]))
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
                                view.reload_tables(cx);
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
                            .label("数据导入")
                            .on_click(cx.listener(|view: &mut Self, _, window, cx| {
                                if let Some(parent) = view.parent.upgrade() {
                                    let _ = parent.update(cx, |app, cx| {
                                        app.display_transfer_window(window, cx);
                                    });
                                }
                            })),
                    )
                    .child(
                        Button::new(comp_id(["mysql-header-export", id]))
                            .outline()
                            .icon(icon_export().with_size(Size::Small))
                            .label("数据导出")
                            .on_click(cx.listener(|view: &mut Self, _, window, cx| {
                                if let Some(parent) = view.parent.upgrade() {
                                    let _ = parent.update(cx, |app, cx| {
                                        app.display_transfer_window(window, cx);
                                    });
                                }
                            })),
                    ),
            )
            .child(
                div().id(comp_id(["mysql-content", id])).col_full().child(
                    h_resizable(comp_id(["mysql-content", id]), self.sidebar_resize.clone())
                        .child(
                            resizable_panel()
                                .size(px(200.0))
                                .size_range(px(100.)..px(400.))
                                .child(sidebar),
                        )
                        .child(container),
                ),
            )
    }
}
