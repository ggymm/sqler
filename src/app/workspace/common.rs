use gpui::{prelude::*, *};
use gpui_component::{
    button::{Button, ButtonVariants},
    input::{Input, InputState},
    resizable::{h_resizable, resizable_panel, v_resizable},
    select::{Select, SelectState},
    table::{Table, TableState},
    ActiveTheme, Disableable, IconName, InteractiveElementExt, Sizable, Size, StyledExt,
};
use uuid::Uuid;

use crate::{
    app::{
        comps::{
            comp_id, icon_export, icon_import, icon_relead, icon_search, icon_sheet, icon_trash, DataTable, DivExt,
        },
        SqlerApp,
    },
    driver::{
        create_connection, DatabaseSession, DriverError, FilterCond, Operator, OrderCond, Paging, QueryReq, QueryResp,
        ValueCond,
    },
    model::DataSource,
};

const PAGE_SIZE: usize = 500;
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
            id: SharedString::from("common-overview-tab"),
            title: SharedString::from("概览"),
            content: TabContent::Overview,
            closable: false,
        }
    }
}

enum TabContent {
    Data(DataContent),
    Query(QueryContent),
    Struct(),
    Overview,
}

struct QueryRule {
    id: SharedString,
    value: Entity<InputState>,
    field: Entity<SelectState<Vec<SharedString>>>,
    operator: Entity<SelectState<Vec<SharedString>>>,
}

struct OrderRule {
    id: SharedString,
    field: Entity<SelectState<Vec<SharedString>>>,
    order: Entity<SelectState<Vec<SharedString>>>,
}

struct DataContent {
    id: SharedString,
    table: SharedString,
    page_no: usize,
    page_size: usize,
    total_rows: usize,
    order_rules: Vec<OrderRule>,
    query_rules: Vec<QueryRule>,
    filter_enable: bool,
    columns: Vec<SharedString>,
    columns_enable: bool,
    datatable: Entity<TableState<DataTable>>,
}

struct QueryContent {
    id: SharedString,
    input: Entity<InputState>,
    datatable: Entity<TableState<DataTable>>,
}

pub struct CommonWorkspace {
    meta: DataSource,
    parent: WeakEntity<SqlerApp>,
    session: Option<Box<dyn DatabaseSession>>,

    tabs: Vec<TabItem>,
    active_tab: SharedString,
    tables: Vec<SharedString>,
    active_table: Option<SharedString>,
}

impl CommonWorkspace {
    pub fn new(
        meta: DataSource,
        parent: WeakEntity<SqlerApp>,
        _cx: &mut Context<Self>,
    ) -> Self {
        let overview = TabItem::overview();
        let active_tab = overview.id.clone();

        Self {
            meta,
            parent,
            session: None,

            tabs: vec![overview],
            active_tab,
            tables: vec![],
            active_table: None,
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
            self.session = Some(create_connection(&self.meta.options)?);
        }

        match self.session.as_deref_mut() {
            Some(session) => Ok(session),
            None => Err(DriverError::Other("数据库连接不可用".into())),
        }
    }

    fn reload_tables(
        &mut self,
        cx: &mut Context<Self>,
    ) {
        // 尝试从会话获取表列表
        let result = self.active_session().and_then(|session| session.tables());

        // 更新本地数据
        self.tables = match result {
            Ok(tables) => tables.into_iter().map(SharedString::from).collect(),
            Err(err) => {
                tracing::error!("刷新表列表失败: {}", err);
                if !self.tables.is_empty() {
                    return;
                }
                vec![]
            }
        };
        self.active_tab = TabItem::overview().id;
        self.active_table = None;

        // 清除失效的标签页
        self.tabs.retain(|tab| match &tab.content {
            TabContent::Data(tab) => self.tables.iter().any(|t| t == &tab.table),
            _ => true,
        });

        cx.notify();
    }

    fn data_content(
        &mut self,
        tab_id: &SharedString,
    ) -> Option<&mut DataContent> {
        self.tabs.iter_mut().find(|tab| tab.id == *tab_id).and_then(|item| {
            if let TabContent::Data(tab) = &mut item.content {
                Some(tab)
            } else {
                None
            }
        })
    }

    fn query_content(
        &mut self,
        tab_id: &SharedString,
    ) -> Option<&mut QueryContent> {
        self.tabs.iter_mut().find(|tab| tab.id == *tab_id).and_then(|item| {
            if let TabContent::Query(tab) = &mut item.content {
                Some(tab)
            } else {
                None
            }
        })
    }

    fn create_data_tab(
        &mut self,
        table: SharedString,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let tab_id = SharedString::from(format!("common-table-tab-{}-{}", self.meta.id, table));

        // 检查标签页是否已存在
        if let Some(existing) = self.tabs.iter().find(|tab| {
            matches!(
                &tab.content,
                TabContent::Data(current) if current.id == tab_id
            )
        }) {
            self.active_tab = existing.id.clone();
            self.active_table = Some(table.clone());
            cx.notify();
            return;
        }

        // 新建标签页
        self.tabs.push(TabItem {
            id: tab_id.clone(),
            title: table.clone(),
            content: TabContent::Data(DataContent {
                id: tab_id.clone(),
                table: table.clone(),
                columns: vec![],
                page_no: 0,
                page_size: PAGE_SIZE,
                total_rows: 0,
                query_rules: Vec::new(),
                order_rules: Vec::new(),
                filter_enable: false,
                columns_enable: false,
                datatable: DataTable::new(vec![], Vec::new()).build(window, cx),
            }),
            closable: true,
        });
        self.active_tab = tab_id.clone();
        self.active_table = Some(table.clone());
        cx.notify();

        // 异步加载数据
        self.reload_data_tab(&tab_id, window, cx);
    }

    fn reload_data_tab(
        &mut self,
        tab_id: &SharedString,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let (table, mut page, size, total, orders, filters) = {
            let Some(content) = self.data_content(tab_id) else {
                return;
            };

            // 设置表格加载状态
            content.datatable.update(cx, |t, cx| {
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
                    Operator::IsNull | Operator::IsNotNull => ValueCond::Null,
                    _ => ValueCond::String(input),
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
        let paging = Paging::new(size, page);

        let tab_id = tab_id.clone();
        let session = match self.active_session() {
            Ok(_) => self.session.take(),
            Err(err) => {
                tracing::error!("获取数据库连接失败: {}", err);
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
                    let cols = session.columns(&table)?;

                    // 查询总数
                    let count_resp = session.query(QueryReq::Builder {
                        table: table.to_string(),
                        columns: vec!["COUNT(*)".to_string()],
                        paging: None,
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
                        paging: Some(paging),
                        orders,
                        filters,
                    })?;
                    let rows = match query_resp {
                        QueryResp::Rows(rows) => rows,
                        _ => Vec::new(),
                    };

                    // 构建渲染数据
                    let table_cols: Vec<SharedString> = cols.iter().map(|s| SharedString::from(s.clone())).collect();
                    let mut table_rows = Vec::with_capacity(rows.len());
                    for row in rows {
                        let mut record = Vec::with_capacity(cols.len());
                        for name in &cols {
                            let value = row.get(name).cloned().unwrap_or_default();
                            record.push(SharedString::from(value));
                        }
                        table_rows.push(record);
                    }

                    Ok::<_, DriverError>(((table_cols, table_rows, total_count), session))
                })
                .await;

            // 更新 UI
            let _ = cx.update(|_, cx| {
                let _ = this.update(cx, |this, cx| match result {
                    Ok((data, session)) => {
                        this.session = Some(session);

                        let (columns, rows, total_rows) = data;
                        let Some(content) = this.data_content(&tab_id) else {
                            return;
                        };
                        content.page_no = page;
                        content.page_size = size;
                        content.total_rows = total_rows;
                        content.columns = columns.clone();

                        content.datatable.update(cx, |t, cx| {
                            t.delegate_mut().update_data(columns, rows);
                            t.delegate_mut().update_loading(false);
                            t.refresh(cx);
                            cx.notify();
                        });
                        cx.notify();
                    }
                    Err(err) => {
                        tracing::error!("加载数据表失败: {}", err);
                        this.session = None;

                        if let Some(content) = this.data_content(&tab_id) {
                            content.datatable.update(cx, |t, cx| {
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

    fn render_data_tab(
        &self,
        tab: &DataContent,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let theme = cx.theme().clone();
        let tab_id = tab.id.clone();

        let page = tab.page_no;
        let columns = &tab.columns;
        let total_pages = if tab.total_rows == 0 {
            1
        } else {
            (tab.total_rows + tab.page_size - 1) / tab.page_size
        };

        let filter_btn = Button::new(comp_id(["table-filter", &tab_id]))
            .label("数据筛选")
            .outline()
            .on_click(cx.listener({
                let tab_id = tab_id.clone();
                move |view: &mut Self, _, _, cx| {
                    if let Some(content) = view.data_content(&tab_id) {
                        content.filter_enable = !content.filter_enable;
                    }
                    cx.notify();
                }
            }));
        let column_btn = Button::new(comp_id(["table-column", &tab_id]))
            .label("字段筛选")
            .outline();

        let page_prev_btn = Button::new(comp_id(["table-page-prev", &tab_id]))
            .label("上一页")
            .outline()
            .disabled(page == 0)
            .on_click(cx.listener({
                let tab_id = tab_id.clone();
                let prev_page = page.saturating_sub(1);
                move |view: &mut Self, _, window, cx| {
                    if let Some(content) = view.data_content(&tab_id) {
                        content.page_no = prev_page;
                    }
                    view.reload_data_tab(&tab_id, window, cx);
                }
            }));
        let page_next_btn = Button::new(comp_id(["table-page-next", &tab_id]))
            .label("下一页")
            .outline()
            .disabled(page + 1 >= total_pages)
            .on_click(cx.listener({
                let tab_id = tab_id.clone();
                let next_page = page.saturating_add(1);
                move |view: &mut Self, _, window, cx| {
                    if let Some(content) = view.data_content(&tab_id) {
                        content.page_no = next_page;
                    }
                    view.reload_data_tab(&tab_id, window, cx);
                }
            }));

        let order_ops = vec![SharedString::from(ORDER_ASC), SharedString::from(ORDER_DESC)];
        let filter_ops: Vec<SharedString> = Operator::all()
            .into_iter()
            .map(|op| SharedString::from(op.label().to_string()))
            .collect();
        let create_order_btn = Button::new(comp_id(["table-order-create", &tab_id]))
            .small()
            .icon(IconName::Plus)
            .on_click(cx.listener({
                let tab_id = tab_id.clone();
                let headers = columns.clone();
                move |view: &mut Self, _, window, cx| {
                    if let Some(content) = view.data_content(&tab_id) {
                        content.order_rules.push(OrderRule {
                            id: SharedString::from(Uuid::new_v4().to_string()),
                            field: cx.new(|cx| {
                                // rustfmt::skip
                                SelectState::new(headers.clone(), None, window, cx)
                            }),
                            order: cx.new(|cx| {
                                // rustfmt::skip
                                SelectState::new(order_ops.clone(), None, window, cx)
                            }),
                        });
                    }
                    cx.notify();
                }
            }));
        let create_query_btn = Button::new(comp_id(["table-filter-create", &tab_id]))
            .small()
            .icon(IconName::Plus)
            .on_click(cx.listener({
                let tab_id = tab_id.clone();
                let headers = columns.clone();
                move |view: &mut Self, _, window, cx| {
                    if let Some(content) = view.data_content(&tab_id) {
                        content.query_rules.push(QueryRule {
                            id: SharedString::from(Uuid::new_v4().to_string()),
                            field: cx.new(|cx| {
                                // rustfmt::skip
                                SelectState::new(headers.clone(), None, window, cx)
                            }),
                            operator: cx.new(|cx| {
                                // rustfmt::skip
                                SelectState::new(filter_ops.clone(), None, window, cx)
                            }),
                            value: cx.new(|cx| InputState::new(window, cx)),
                        });
                    }
                    cx.notify();
                }
            }));
        let apply_cond_btn = Button::new(comp_id(["table-filter-apply", &tab_id]))
            .label("应用条件")
            .outline()
            .on_click(cx.listener({
                let tab_id = tab_id.clone();
                move |view: &mut Self, _, window, cx| {
                    if let Some(content) = view.data_content(&tab_id) {
                        content.page_no = 0;
                        content.filter_enable = false;
                    }
                    view.reload_data_tab(&tab_id, window, cx);
                }
            }));
        let clear_cond_btn = Button::new(comp_id(["table-filter-clear", &tab_id]))
            .label("清除条件")
            .outline()
            .on_click(cx.listener({
                let tab_id = tab_id.clone();
                move |view: &mut Self, _, _, cx| {
                    if let Some(content) = view.data_content(&tab_id) {
                        content.order_rules.clear();
                        content.query_rules.clear();
                    }
                    cx.notify();
                }
            }));
        let close_cond_btn = Button::new(comp_id(["table-filter-close", &tab_id]))
            .small()
            .ghost()
            .icon(IconName::Close)
            .on_click(cx.listener({
                let tab_id = tab_id.clone();
                move |view: &mut Self, _, _, cx| {
                    if let Some(content) = view.data_content(&tab_id) {
                        content.filter_enable = false;
                    }
                    cx.notify();
                }
            }));

        let mut orders = Vec::new();
        for order in tab.order_rules.iter() {
            let tab_id = tab_id.clone();
            let rule_id = order.id.clone();
            let rule_field = Select::new(&order.field).small().placeholder("");
            let rule_order = Select::new(&order.order).small().placeholder("");
            orders.push(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .w_full()
                    .child(div().w_48().child(rule_field))
                    .child(div().w_48().child(rule_order))
                    .child(
                        Button::new(comp_id(["table-order-remove", &rule_id]))
                            .ghost()
                            .icon(icon_trash())
                            .on_click(cx.listener({
                                move |view: &mut Self, _, _, cx| {
                                    if let Some(content) = view.data_content(&tab_id) {
                                        content.order_rules.retain(|r| &r.id != &rule_id);
                                    }
                                    cx.notify();
                                }
                            })),
                    ),
            )
        }
        let mut queries = Vec::new();
        for query in tab.query_rules.iter() {
            let tab_id = tab_id.clone();
            let rule_id = query.id.clone();
            let rule_field = Select::new(&query.field).small().placeholder("");
            let rule_operator = Select::new(&query.operator).small().placeholder("");

            queries.push(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .w_full()
                    .child(div().w_48().child(rule_field))
                    .child(div().w_48().child(rule_operator))
                    .child(div().flex_1().child(Input::new(&query.value).small()))
                    .child(
                        Button::new(comp_id(["table-filter-remove", &rule_id]))
                            .ghost()
                            .icon(icon_trash())
                            .on_click(cx.listener({
                                move |view: &mut Self, _, _, cx| {
                                    if let Some(content) = view.data_content(&tab_id) {
                                        content.query_rules.retain(|r| &r.id != &rule_id);
                                    }
                                    cx.notify();
                                }
                            })),
                    ),
            )
        }

        div()
            .relative()
            .col_full()
            .child(
                div().flex_1().child(
                    Table::new(&tab.datatable)
                        .stripe(false)
                        .bordered(false)
                        .with_size(Size::Small)
                        .scrollbar_visible(true, true),
                ),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .p_2()
                    .gap_2()
                    .h_12()
                    .bg(theme.secondary)
                    .border_t_1()
                    .border_color(theme.border)
                    .child(filter_btn)
                    .child(column_btn)
                    .child(div().flex_1())
                    .child(div().text_sm().child(format!(
                        "显示 {} - {} / 共 {} 条",
                        if tab.total_rows == 0 {
                            0
                        } else {
                            page * tab.page_size + 1
                        },
                        ((page + 1) * tab.page_size).min(tab.total_rows),
                        tab.total_rows
                    )))
                    .child(div().flex_1())
                    .child(page_prev_btn)
                    .child(page_next_btn),
            )
            .when(tab.filter_enable || tab.columns_enable, |this| {
                this.child(
                    div()
                        .id("overlay")
                        .top_0()
                        .left_0()
                        .right_0()
                        .bottom_0()
                        .occlude()
                        .absolute(),
                )
            })
            .when(tab.filter_enable, |this| {
                this.child(
                    div()
                        .col_full()
                        .absolute()
                        .w_2_3()
                        .h_2_3()
                        .top_0()
                        .left_1_6()
                        .bg(theme.background)
                        .border_1()
                        .border_color(theme.border)
                        .shadow_lg()
                        .rounded_lg()
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .justify_between()
                                .px_4()
                                .py_2()
                                .border_b_1()
                                .border_color(theme.border)
                                .child(div().text_base().child("筛选数据"))
                                .child(close_cond_btn),
                        )
                        .child(
                            div().flex_1().min_h_0().child(
                                div()
                                    .px_4()
                                    .py_2()
                                    .gap_2()
                                    .col_full()
                                    .scrollable(Axis::Vertical)
                                    .child(
                                        div()
                                            .gap_4()
                                            .row_full()
                                            .child(div().text_sm().font_semibold().child("排序规则"))
                                            .child(create_order_btn),
                                    )
                                    .children(orders)
                                    .child(
                                        div()
                                            .gap_4()
                                            .row_full()
                                            .child(div().text_sm().font_semibold().child("筛选规则"))
                                            .child(create_query_btn),
                                    )
                                    .children(queries),
                            ),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .px_4()
                                .py_2()
                                .gap_2()
                                .border_t_1()
                                .border_color(theme.border)
                                .child(div().flex_1())
                                .child(clear_cond_btn)
                                .child(apply_cond_btn),
                        ),
                )
            })
            .when(tab.columns_enable, |this| this.child(div()))
            .into_any_element()
    }

    fn create_query_tab(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let tab_id = SharedString::from(format!("common-query-tab-{}", Uuid::new_v4()));

        // 新建标签页
        self.tabs.push(TabItem {
            id: tab_id.clone(),
            title: SharedString::from("SQL 查询"),
            content: TabContent::Query(QueryContent {
                id: tab_id.clone(),
                input: cx.new(|cx| {
                    InputState::new(window, cx)
                        .code_editor("sql")
                        .searchable(false)
                        .line_number(true)
                        .indent_guides(true)
                }),
                datatable: DataTable::new(vec![], Vec::new()).build(window, cx),
            }),
            closable: true,
        });
        self.active_tab = tab_id;
        self.active_table = Some(SharedString::from("SQL查询"));
        cx.notify();
    }

    fn reload_query_tab(
        &mut self,
        tab_id: &SharedString,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let sql = {
            let Some(query) = self.query_content(tab_id) else {
                return;
            };

            let sql = query.input.read(cx).text().to_string();
            if sql.trim().is_empty() {
                tracing::warn!("SQL语句为空");
                return;
            }

            query.datatable.update(cx, |t, cx| {
                t.delegate_mut().update_loading(true);
                cx.notify();
            });

            sql
        };

        // 获取数据库连接
        let tab_id = tab_id.clone();
        let session = match self.active_session() {
            Ok(_) => self.session.take(),
            Err(err) => {
                tracing::error!("获取数据库连接失败: {}", err);
                return;
            }
        };
        let Some(session) = session else {
            return;
        };

        // 在后台线程执行查询
        cx.spawn_in(window, async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move {
                    let mut session = session;

                    // 执行SQL查询
                    let query_resp = session.query(QueryReq::Sql { sql, args: Vec::new() })?;

                    // 解析结果
                    let rows = match query_resp {
                        QueryResp::Rows(rows) => rows,
                        _ => Vec::new(),
                    };

                    // 提取列名和数据
                    let table_cols: Vec<SharedString> = if let Some(first_row) = rows.first() {
                        first_row.keys().map(|k| SharedString::from(k.clone())).collect()
                    } else {
                        Vec::new()
                    };

                    let mut table_rows = Vec::with_capacity(rows.len());
                    for row in rows {
                        let mut record = Vec::with_capacity(table_cols.len());
                        for col in &table_cols {
                            let value = row.get(col.as_ref()).cloned().unwrap_or_default();
                            record.push(SharedString::from(value));
                        }
                        table_rows.push(record);
                    }

                    Ok::<_, DriverError>((table_cols, table_rows, session))
                })
                .await;

            // 更新UI
            let _ = cx.update(|_, cx| {
                let _ = this.update(cx, |this, cx| match result {
                    Ok((columns, rows, session)) => {
                        this.session = Some(session);

                        if let Some(query) = this.query_content(&tab_id) {
                            query.datatable.update(cx, |t, cx| {
                                t.delegate_mut().update_data(columns, rows);
                                t.delegate_mut().update_loading(false);
                                t.refresh(cx);
                                cx.notify();
                            });
                        }
                        cx.notify();
                    }
                    Err(err) => {
                        tracing::error!("执行SQL查询失败: {}", err);
                        this.session = None;

                        if let Some(query) = this.query_content(&tab_id) {
                            query.datatable.update(cx, |t, cx| {
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

    fn render_query_tab(
        &self,
        tab: &QueryContent,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let tab_id = tab.id.clone();

        let execute_btn = Button::new(comp_id(["query-execute", &tab_id]))
            .label("执行查询")
            .small()
            .outline()
            .on_click(cx.listener({
                let tab_id = tab_id.clone();
                move |view: &mut Self, _, window, cx| {
                    view.reload_query_tab(&tab_id, window, cx);
                }
            }));

        div()
            .col_full()
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .h_8()
                    .px_2()
                    .gap_2()
                    .child(execute_btn)
                    .child(div()),
            )
            .child(
                v_resizable(comp_id(["common-content"]))
                    .child(
                        resizable_panel()
                            .size(px(180.0))
                            .size_range(px(100.)..px(320.))
                            .child(
                                div()
                                    .p_1()
                                    .flex_1()
                                    .child(Input::new(&tab.input).h_full().appearance(false).focus_bordered(false)),
                            )
                            .child(div()),
                    )
                    .child(
                        div()
                            .flex_1()
                            .child(
                                Table::new(&tab.datatable)
                                    .stripe(false)
                                    .bordered(false)
                                    .with_size(Size::Small)
                                    .scrollbar_visible(true, true),
                            )
                            .into_any_element(),
                    ),
            )
            .into_any_element()
    }

    fn _create_struct_tab(
        &mut self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
    }

    fn render_struct_tab(
        &self,
        _cx: &mut Context<Self>,
    ) -> AnyElement {
        div().into_any_element()
    }

    fn render_overview_tab(
        &self,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let theme = cx.theme();
        let overview_fields = self.meta.display_overview();

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
            .children(overview_fields.into_iter().map(|(label, value)| {
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
            .child(detail_card)
            .into_any_element()
    }
}

impl Render for CommonWorkspace {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let id = &self.meta.id;
        let theme = cx.theme().clone();
        let active = &self.active_tab;

        let mut tabs = Vec::new();
        for tab in self.tabs.iter() {
            let tab_id = tab.id.clone();
            let tab_active = &tab_id == active;

            let mut item = div()
                .id(comp_id(["common-tabs-item", &tab_id]))
                .flex()
                .flex_row()
                .items_center()
                .justify_center()
                .px_2()
                .gap_2()
                .border_r_1()
                .border_color(theme.border)
                .text_sm()
                .when(tab_active, |this| {
                    this.pb(px(1.))
                        .bg(theme.tab_active)
                        .text_color(theme.tab_active_foreground)
                })
                .when(!tab_active, |this| {
                    this.border_b(px(1.))
                        .bg(theme.tab_bar)
                        .text_color(theme.muted_foreground)
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
                        .flex()
                        .flex_row()
                        .h_8()
                        .flex_1()
                        .min_w_0()
                        .items_center()
                        .overflow_hidden()
                        .whitespace_nowrap()
                        .child(tab.title.clone()),
                );

            if tab.closable {
                item = item.child(
                    Button::new(comp_id(["common-tabs-close", &tab_id]))
                        .ghost()
                        .xsmall()
                        .compact()
                        .tab_stop(false)
                        .icon(IconName::Close)
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

            tabs.push(item)
        }
        let mut tables = Vec::new();
        for item in self.tables.iter() {
            let active = self.active_table.as_ref() == Some(&item);
            let active_table = item.clone();

            tables.push(
                div()
                    .id(comp_id(["common-sidebar-item", &self.meta.id, &item]))
                    .px_4()
                    .py_2()
                    .gap_2()
                    .row_full()
                    .items_center()
                    .rounded_lg()
                    .when_else(
                        active,
                        |this| this.bg(theme.list_active).font_semibold(),
                        |this| this.hover(|this| this.bg(theme.list_hover)),
                    )
                    .on_double_click(cx.listener(move |this, _, window, cx| {
                        this.create_data_tab(active_table.clone(), window, cx);
                    }))
                    .child(icon_sheet())
                    .child(
                        div()
                            .text_sm()
                            .overflow_hidden()
                            .whitespace_nowrap()
                            .child(item.clone()),
                    ),
            )
        }

        div()
            .id(comp_id(["common", id]))
            .col_full()
            .child(
                div()
                    .id(comp_id(["common-header", id]))
                    .flex()
                    .flex_row()
                    .p_4()
                    .gap_2()
                    .border_b_1()
                    .border_color(theme.border)
                    .child(
                        Button::new(comp_id(["common-header-refresh", id]))
                            .icon(icon_relead().with_size(Size::Small))
                            .label("刷新表")
                            .outline()
                            .on_click(cx.listener(|view: &mut Self, _, _, cx| {
                                view.reload_tables(cx);
                            })),
                    )
                    .child(
                        Button::new(comp_id(["common-header-table", id]))
                            .icon(icon_sheet().with_size(Size::Small))
                            .label("新建表")
                            .outline()
                            .on_click(cx.listener(|view: &mut Self, _, window, cx| {
                                view.create_query_tab(window, cx);
                            })),
                    )
                    .child(
                        Button::new(comp_id(["common-header-query", id]))
                            .icon(icon_search().with_size(Size::Small))
                            .label("新建查询")
                            .outline()
                            .on_click(cx.listener(|view: &mut Self, _, window, cx| {
                                view.create_query_tab(window, cx);
                            })),
                    )
                    .child(
                        Button::new(comp_id(["common-header-import", id]))
                            .icon(icon_import().with_size(Size::Small))
                            .label("数据导入")
                            .outline()
                            .on_click(cx.listener(|view: &mut Self, _, _, cx| {
                                if let Some(parent) = view.parent.upgrade() {
                                    let meta = view.meta.clone();
                                    let tables = view.tables.clone();
                                    let _ = parent.update(cx, |app, cx| {
                                        app.display_import_window(meta, tables, cx);
                                    });
                                }
                            })),
                    )
                    .child(
                        Button::new(comp_id(["common-header-export", id]))
                            .icon(icon_export().with_size(Size::Small))
                            .label("数据导出")
                            .outline()
                            .on_click(cx.listener(|view: &mut Self, _, _, cx| {
                                if let Some(parent) = view.parent.upgrade() {
                                    let meta = view.meta.clone();
                                    let tables = view.tables.clone();
                                    let _ = parent.update(cx, |app, cx| {
                                        app.display_export_window(meta, tables, cx);
                                    });
                                }
                            })),
                    ),
            )
            .child(
                div().col_full().child(
                    h_resizable(comp_id(["common-content", id]))
                        .child(
                            resizable_panel()
                                .size(px(200.0))
                                .size_range(px(100.)..px(320.))
                                .child(
                                    div()
                                        .id(comp_id(["common-sidebar", id]))
                                        .p_2()
                                        .gap_2()
                                        .col_full()
                                        .scrollable(Axis::Vertical)
                                        .children(tables),
                                )
                                .child(div()),
                        )
                        .child(
                            div()
                                .col_full()
                                .child(
                                    div()
                                        .id(comp_id(["common-tabs", id]))
                                        .flex()
                                        .flex_row()
                                        .min_w_0()
                                        .border_color(theme.border)
                                        .children(tabs)
                                        .child(div().flex_1().border_b_1().border_color(theme.border)),
                                )
                                .child(
                                    div().id(comp_id(["common-main", id])).col_full().child(
                                        match self
                                            .tabs
                                            .iter()
                                            .find(|tab| tab.id == self.active_tab)
                                            .map(|tab| &tab.content)
                                        {
                                            Some(TabContent::Data(tab)) => self.render_data_tab(tab, cx),
                                            Some(TabContent::Query(tab)) => self.render_query_tab(tab, cx),
                                            Some(TabContent::Struct()) => self.render_struct_tab(cx),
                                            Some(TabContent::Overview) | None => self.render_overview_tab(cx),
                                        },
                                    ),
                                )
                                .into_any_element(),
                        ),
                ),
            )
    }
}
