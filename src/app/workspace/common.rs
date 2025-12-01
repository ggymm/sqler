use std::{rc::Rc, time};

use gpui::{prelude::*, *};
use gpui_component::{
    button::{Button, ButtonVariants},
    input::{Input, InputState, TabSize},
    resizable::{h_resizable, resizable_panel, v_resizable},
    select::{Select, SelectState},
    sheet::Sheet,
    table::{Table, TableState},
    tooltip::Tooltip,
    ActiveTheme, Disableable, IconName, InteractiveElementExt, Sizable, Size, StyledExt,
};
use indexmap::IndexMap;
use uuid::Uuid;

use crate::{
    app::{
        comps::{
            comp_id, icon_export, icon_import, icon_relead, icon_search, icon_sheet, icon_trash, DataTable, DivExt,
        },
        SqlerApp, WindowKind,
    },
    cache::ArcCache,
    driver::{
        create_connection, DatabaseSession, DriverError, FilterCond, Operator, OrderCond, Paging, QueryReq, QueryResp,
        ValueCond,
    },
    model::{DataSource, TableInfo},
};

use super::EditorComps;

const PAGE_SIZE: usize = 500;
const ORDER_ASC: &str = "升序";
const ORDER_DESC: &str = "降序";

enum TabContent {
    Query(QueryContent),
    Table(TableContent),
    Schema(SchemaContent),
    Overview,
}

pub struct TabContext {
    title: SharedString,
    content: TabContent,
    closable: bool,
}

impl TabContext {
    pub fn overview() -> Self {
        Self {
            title: SharedString::from("概览"),
            content: TabContent::Overview,
            closable: false,
        }
    }
}

pub struct CommonWorkspace {
    pub cache: ArcCache,
    pub parent: WeakEntity<SqlerApp>,

    pub source: DataSource,
    pub session: Option<Box<dyn DatabaseSession>>,

    pub tabs: IndexMap<String, TabContext>,
    pub active_tab: String,
    pub tables: IndexMap<String, TableInfo>,
    pub active_table: Option<String>,
}

impl CommonWorkspace {
    fn active_session(&mut self) -> Result<&mut (dyn DatabaseSession + '_), DriverError> {
        if self.session.is_none() {
            self.session = Some(create_connection(&self.source.options)?);
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
        let session = match self.active_session() {
            Ok(_) => self.session.take(),
            Err(err) => {
                tracing::error!("获取数据库连接失败: {}", err);
                return;
            }
        };
        let Some(mut session) = session else {
            return;
        };

        // 更新数据
        self.tables = match session.tables() {
            Ok(tables) => {
                {
                    let cache = self.cache.write().unwrap();
                    if let Err(err) = cache.tables_update(&self.source.id, &tables) {
                        tracing::error!("更新表缓存失败: {}", err);
                    }
                }
                tables.into_iter().map(|info| (info.name.clone(), info)).collect()
            }
            Err(err) => {
                tracing::error!("刷新表列表失败: {}", err);
                if !self.tables.is_empty() {
                    return;
                }
                IndexMap::new()
            }
        };
        // self.active_tab = 0;
        self.active_table = None;

        // 清除失效的标签页
        self.tabs.retain(|_, tab| match &tab.content {
            TabContent::Table(tab) => self.tables.contains_key(tab.table.as_ref()),
            _ => true,
        });
        cx.notify();
    }

    fn query_content(
        &mut self,
        tab_id: &String,
    ) -> Option<&mut QueryContent> {
        self.tabs.get_mut(tab_id).and_then({
            |item| {
                if let TabContent::Query(tab) = &mut item.content {
                    Some(tab)
                } else {
                    None
                }
            }
        })
    }

    fn table_content(
        &mut self,
        tab_id: &String,
    ) -> Option<&mut TableContent> {
        self.tabs.get_mut(tab_id).and_then({
            |item| {
                if let TabContent::Table(tab) = &mut item.content {
                    Some(tab)
                } else {
                    None
                }
            }
        })
    }

    fn schema_content(
        &mut self,
        tab_id: &String,
    ) -> Option<&mut SchemaContent> {
        self.tabs.get_mut(tab_id).and_then({
            |item| {
                if let TabContent::Schema(tab) = &mut item.content {
                    Some(tab)
                } else {
                    None
                }
            }
        })
    }

    fn create_query_tab(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let tab_id = Uuid::new_v4().to_string();

        let lsp = Rc::new(EditorComps::new());
        let editor = cx.new(|cx| {
            let mut editor = InputState::new(window, cx)
                .code_editor("sql")
                .line_number(true)
                .indent_guides(true)
                .tab_size(TabSize {
                    tab_size: 4,
                    hard_tabs: false,
                })
                .soft_wrap(false);

            editor.lsp.completion_provider = Some(lsp.clone());

            editor
        });

        // 新建标签页
        self.tabs.insert(
            tab_id.clone(),
            TabContext {
                title: SharedString::from("SQL 查询"),
                content: TabContent::Query(QueryContent {
                    id: tab_id.clone(),
                    active: 0,
                    summary: true,
                    editor,
                    results: vec![],
                }),
                closable: true,
            },
        );
        self.active_tab = tab_id.clone();
        cx.notify();
    }

    fn reload_query_tab(
        &mut self,
        tab_id: &String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let multi_sql = {
            let Some(content) = self.query_content(tab_id) else {
                return;
            };

            let multi_sql: Vec<String> = content
                .editor
                .read(cx)
                .text()
                .to_string()
                .split(';')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            if multi_sql.is_empty() {
                tracing::warn!("SQL语句为空");
                return;
            }

            content.results.clear();
            for (_, sql) in multi_sql.iter().enumerate() {
                let datatable = DataTable::new(vec![], vec![]).build(window, cx);
                content.results.push(QueryResult {
                    sql: sql.to_string(),
                    error: None,
                    elapsed: 0.0,
                    datatable,
                });
            }
            content.active = 0;
            content.summary = true;

            multi_sql
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
        let Some(mut session) = session else {
            return;
        };

        // 在后台线程执行查询
        cx.spawn_in(window, async move |this, cx| {
            let ret = cx
                .background_executor()
                .spawn(async move {
                    let mut results = vec![];
                    for (i, sql) in multi_sql.iter().enumerate() {
                        let start = time::Instant::now();
                        match session.query(QueryReq::Sql {
                            sql: sql.to_string(),
                            args: vec![],
                        }) {
                            Ok(query_resp) => {
                                let elapsed = start.elapsed().as_secs_f64();

                                // 解析结果
                                let rows = match query_resp {
                                    QueryResp::Rows(rows) => rows,
                                    _ => vec![],
                                };

                                // 提取列名和数据
                                let table_cols: Vec<SharedString> = if let Some(first_row) = rows.first() {
                                    first_row.keys().map(|k| SharedString::from(k.clone())).collect()
                                } else {
                                    vec![]
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
                                results.push((i, table_cols, table_rows, elapsed, None));
                            }
                            Err(err) => {
                                let elapsed = start.elapsed().as_secs_f64();
                                tracing::error!("执行SQL失败: {}", err);
                                results.push((i, vec![], vec![], elapsed, Some(err.to_string())));
                            }
                        }
                    }

                    Ok::<_, DriverError>((results, session))
                })
                .await;

            // 更新UI
            let _ = cx.update(|_, cx| {
                let _ = this.update(cx, |this, cx| match ret {
                    Ok((results, session)) => {
                        this.session = Some(session);
                        let Some(content) = this.query_content(&tab_id) else {
                            return;
                        };
                        for (i, cols, rows, elapsed, error) in results {
                            let Some(ret) = content.results.get_mut(i) else {
                                continue;
                            };
                            ret.error = error;
                            ret.elapsed = elapsed;
                            ret.datatable.update(cx, |t, cx| {
                                t.delegate_mut().update_data(cols, rows);
                                t.refresh(cx);
                                cx.notify();
                            });
                        }
                        cx.notify();
                    }
                    Err(err) => {
                        tracing::error!("执行SQL查询失败: {}", err);
                        this.session = None;
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
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let theme = cx.theme();
        let tab_id = tab.id.clone();

        let mut results = vec![{
            let mut item = Button::new(comp_id(["query-result-summary"]))
                .label("摘要")
                .small()
                .when_else(tab.summary, |this| this.outline(), |this| this.ghost())
                .on_click(cx.listener({
                    let tab_id = tab_id.clone();
                    move |view: &mut Self, _, _, cx| {
                        let Some(content) = view.query_content(&tab_id) else {
                            return;
                        };
                        content.summary = true;
                        cx.notify();
                    }
                }));
            {
                let style = item.style();
                style.flex_grow = Some(0.);
                style.flex_shrink = Some(1.);
                style.min_size.width = Some(Length::Definite(px(0.).into()));
            }
            item
        }];
        for (i, _) in tab.results.iter().enumerate() {
            let active = !tab.summary && i == tab.active;

            let mut item = Button::new(comp_id(["query-result-item", &i.to_string()]))
                .label(format!("结果 {}", i + 1))
                .small()
                .when_else(active, |this| this.outline(), |this| this.ghost())
                .on_click(cx.listener({
                    let tab_id = tab_id.clone();
                    move |view: &mut Self, _, _, cx| {
                        let Some(content) = view.query_content(&tab_id) else {
                            return;
                        };
                        content.active = i;
                        content.summary = false;
                        cx.notify();
                    }
                }));
            {
                let style = item.style();
                style.flex_grow = Some(0.);
                style.flex_shrink = Some(1.);
                style.min_size.width = Some(Length::Definite(px(0.).into()));
            }

            results.push(item);
        }
        let execute = Button::new(comp_id(["query-execute", &tab_id]))
            .label("执行查询")
            .small()
            .outline()
            .on_click(cx.listener({
                let tab_id = tab_id.clone();
                move |view: &mut Self, _, window, cx| {
                    view.reload_query_tab(&tab_id, window, cx);
                }
            }));

        let summaries: Vec<AnyElement> = tab
            .results
            .iter()
            .enumerate()
            .map(|(_, ret)| {
                let time_str = format!("{:.3}s", ret.elapsed);

                div()
                    .p_4()
                    .gap_2()
                    .col_full()
                    .rounded_md()
                    .border_1()
                    .border_color(theme.border)
                    .bg(theme.secondary)
                    .child(
                        div()
                            .text_color(theme.foreground)
                            .font_family(theme.mono_font_family.clone())
                            .child(ret.sql.clone()),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .child(div().when_else(
                                ret.error.is_none(),
                                |this| this.text_color(theme.success).child("成功"),
                                |this| this.text_color(theme.danger).child("失败"),
                            ))
                            .child(div().text_color(theme.muted_foreground).child(time_str)),
                    )
                    .when_some(ret.error.as_ref(), |this, err| {
                        this.child(div().text_color(theme.danger).child(format!("错误: {}", err)))
                    })
                    .into_any_element()
            })
            .collect();

        div()
            .col_full()
            .child(
                v_resizable(comp_id(["common-content"]))
                    .child(
                        resizable_panel()
                            .size(px(180.0))
                            .size_range(px(100.)..px(320.))
                            .child(
                                div().flex_1().child(
                                    Input::new(&tab.editor)
                                        .p_0()
                                        .h_full()
                                        .appearance(false)
                                        .text_sm()
                                        .font_family(theme.mono_font_family.clone())
                                        .focus_bordered(false),
                                ),
                            )
                            .child(div()),
                    )
                    .child(
                        div()
                            .col_full()
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .h_10()
                                    .p_2()
                                    .gap_2()
                                    .border_b_1()
                                    .border_color(theme.border)
                                    .children(results)
                                    .child(div().flex_1())
                                    .child(execute),
                            )
                            .child(
                                div()
                                    .id(comp_id(["query-result-content", &tab_id]))
                                    .relative()
                                    .col_full()
                                    .when(tab.summary, |this| {
                                        this.child(
                                            div()
                                                .p_2()
                                                .gap_2()
                                                .col_full()
                                                .scrollable(Axis::Vertical)
                                                .children(summaries),
                                        )
                                    })
                                    .when_some(
                                        (!tab.summary).then(|| tab.results.get(tab.active)).flatten(),
                                        |this, ret| {
                                            this.child(
                                                div().flex_1().child(
                                                    Table::new(&ret.datatable)
                                                        .stripe(false)
                                                        .bordered(false)
                                                        .scrollbar_visible(true, true),
                                                ),
                                            )
                                        },
                                    ),
                            )
                            .into_any_element(),
                    ),
            )
            .into_any_element()
    }

    fn create_table_tab(
        &mut self,
        table: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        for (id, tab) in &self.tabs {
            let TabContent::Table(data) = &tab.content else {
                continue;
            };
            if data.table.as_ref() == table.as_str() {
                self.active_tab = id.clone();
                cx.notify();
                return;
            }
        }
        let tab_id = Uuid::new_v4().to_string();

        // 新建标签页
        self.tabs.insert(
            tab_id.clone(),
            TabContext {
                title: SharedString::from(table.clone()),
                content: TabContent::Table(TableContent {
                    id: tab_id.clone(),
                    page_no: 0,
                    rows_count: 0,
                    query_rules: vec![],
                    order_rules: vec![],
                    filter_enable: false,
                    columns: vec![],
                    columns_enable: false,
                    table: SharedString::from(table),
                    datatable: DataTable::new(vec![], vec![]).build(window, cx),
                }),
                closable: true,
            },
        );
        self.active_tab = tab_id.clone();
        cx.notify();

        // 异步加载数据
        self.reload_table_tab(&tab_id, window, cx);
    }

    fn reload_table_tab(
        &mut self,
        tab_id: &String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let (table, page, orders, filters) = {
            let Some(content) = self.table_content(tab_id) else {
                return;
            };

            // 设置表格加载状态
            content.datatable.update(cx, |t, cx| {
                t.delegate_mut().update_loading(true);
                cx.notify();
            });

            // 转换排序规则
            let mut orders = vec![];
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
            let mut filters = vec![];
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

            (content.table.clone(), content.page_no, orders, filters)
        };

        let tab_id = tab_id.clone();
        let session = match self.active_session() {
            Ok(_) => self.session.take(),
            Err(err) => {
                tracing::error!("获取数据库连接失败: {}", err);
                return;
            }
        };
        let Some(mut session) = session else {
            return;
        };

        // 在后台线程执行数据库查询
        cx.spawn_in(window, async move |this, cx| {
            let ret = cx
                .background_executor()
                .spawn(async move {
                    // 查询列名
                    let cols = session.columns(&table)?;

                    // 查询数据
                    let query_resp = session.query(QueryReq::Builder {
                        table: table.to_string(),
                        columns: vec![],
                        paging: Some(Paging::new(page, PAGE_SIZE)),
                        orders,
                        filters,
                    })?;
                    let rows = match query_resp {
                        QueryResp::Rows(rows) => rows,
                        _ => vec![],
                    };

                    // 构建渲染数据
                    let mut table_rows = Vec::with_capacity(rows.len());
                    for row in rows {
                        let mut record = Vec::with_capacity(cols.len());
                        for col in &cols {
                            let value = row.get(&col.name).cloned().unwrap_or_default();
                            record.push(SharedString::from(value));
                        }
                        table_rows.push(record);
                    }

                    let table_cols: Vec<SharedString> = cols
                        .iter()
                        .map(|c| {
                            // rustfmt::skip
                            SharedString::from(c.name.clone())
                        })
                        .collect();
                    Ok::<_, DriverError>(((table_cols, table_rows), session))
                })
                .await;

            // 更新 UI
            let _ = cx.update(|_, cx| {
                let _ = this.update(cx, |this, cx| match ret {
                    Ok((data, session)) => {
                        this.session = Some(session);

                        let (cols, rows) = data;
                        let Some(content) = this.table_content(&tab_id) else {
                            return;
                        };
                        content.page_no = page;
                        content.rows_count = rows.len();
                        content.columns = cols.clone();
                        content.datatable.update(cx, |t, cx| {
                            t.delegate_mut().update_data(cols, rows);
                            t.delegate_mut().update_loading(false);
                            t.refresh(cx);
                            cx.notify();
                        });
                        cx.notify();
                    }
                    Err(err) => {
                        tracing::error!("加载数据表失败: {}", err);
                        this.session = None;

                        let Some(content) = this.table_content(&tab_id) else {
                            return;
                        };
                        content.datatable.update(cx, |t, cx| {
                            t.delegate_mut().update_loading(false);
                            cx.notify();
                        });
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
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let theme = cx.theme().clone();
        let tab_id = tab.id.clone();

        let page_no = tab.page_no;
        let rows_count = tab.rows_count;

        let filter = Button::new(comp_id(["table-filter", &tab_id]))
            .label("筛选数据")
            .outline()
            .on_click(cx.listener({
                let tab_id = tab_id.clone();
                move |view: &mut Self, _, _, cx| {
                    if let Some(content) = view.table_content(&tab_id) {
                        content.filter_enable = !content.filter_enable;
                    }
                    cx.notify();
                }
            }));
        let column = Button::new(comp_id(["table-column", &tab_id]))
            .label("筛选字段")
            .outline();

        let page_prev = Button::new(comp_id(["table-page-prev", &tab_id]))
            .label("上一页")
            .outline()
            .disabled(page_no == 0)
            .on_click(cx.listener({
                let tab_id = tab_id.clone();
                let prev_page = page_no.saturating_sub(1);
                move |view: &mut Self, _, window, cx| {
                    if let Some(content) = view.table_content(&tab_id) {
                        content.page_no = prev_page;
                    }
                    view.reload_table_tab(&tab_id, window, cx);
                }
            }));
        let page_next = Button::new(comp_id(["table-page-next", &tab_id]))
            .label("下一页")
            .outline()
            .disabled(rows_count == 0)
            .on_click(cx.listener({
                let tab_id = tab_id.clone();
                let next_page = page_no.saturating_add(1);
                move |view: &mut Self, _, window, cx| {
                    if let Some(content) = view.table_content(&tab_id) {
                        content.page_no = next_page;
                    }
                    view.reload_table_tab(&tab_id, window, cx);
                }
            }));

        let order_ops = vec![SharedString::from(ORDER_ASC), SharedString::from(ORDER_DESC)];
        let filter_ops: Vec<SharedString> = Operator::all()
            .into_iter()
            .map(|op| SharedString::from(op.label().to_string()))
            .collect();
        let apply_cond = Button::new(comp_id(["table-filter-apply", &tab_id]))
            .label("应用条件")
            .outline()
            .on_click(cx.listener({
                let tab_id = tab_id.clone();
                move |view: &mut Self, _, window, cx| {
                    if let Some(content) = view.table_content(&tab_id) {
                        content.page_no = 0;
                        content.filter_enable = false;
                    }
                    view.reload_table_tab(&tab_id, window, cx);
                }
            }));
        let clear_cond = Button::new(comp_id(["table-filter-clear", &tab_id]))
            .label("清除条件")
            .outline()
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
        let create_order = Button::new(comp_id(["table-order-create", &tab_id]))
            .small()
            .icon(IconName::Plus)
            .on_click(cx.listener({
                let tab_id = tab_id.clone();
                let columns = tab.columns.clone();
                move |view: &mut Self, _, window, cx| {
                    if let Some(content) = view.table_content(&tab_id) {
                        content.order_rules.push(OrderRule {
                            id: Uuid::new_v4().to_string(),
                            field: cx.new(|cx| {
                                // rustfmt::skip
                                SelectState::new(columns.clone(), None, window, cx)
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
        let create_query = Button::new(comp_id(["table-filter-create", &tab_id]))
            .small()
            .icon(IconName::Plus)
            .on_click(cx.listener({
                let tab_id = tab_id.clone();
                let columns = tab.columns.clone();
                move |view: &mut Self, _, window, cx| {
                    if let Some(content) = view.table_content(&tab_id) {
                        content.query_rules.push(QueryRule {
                            id: Uuid::new_v4().to_string(),
                            field: cx.new(|cx| {
                                // rustfmt::skip
                                SelectState::new(columns.clone(), None, window, cx)
                            }),
                            operator: cx.new(|cx| {
                                // rustfmt::skip
                                SelectState::new(filter_ops.clone(), None, window, cx)
                            }),
                            value: cx.new(|cx| {
                                // rustfmt::skip
                                InputState::new(window, cx)
                            }),
                        });
                    }
                    cx.notify();
                }
            }));

        let mut orders = vec![];
        for order in tab.order_rules.iter() {
            let tab_id = tab_id.clone();
            let rule_id = order.id.clone();
            orders.push(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .w_full()
                    .child(div().w_48().child(Select::new(&order.field).small().placeholder("")))
                    .child(div().w_48().child(Select::new(&order.order).small().placeholder("")))
                    .child(
                        Button::new(comp_id(["table-order-remove", &rule_id]))
                            .ghost()
                            .icon(icon_trash())
                            .on_click(cx.listener({
                                move |view: &mut Self, _, _, cx| {
                                    if let Some(content) = view.table_content(&tab_id) {
                                        content.order_rules.retain(|r| &r.id != &rule_id);
                                    }
                                    cx.notify();
                                }
                            })),
                    ),
            )
        }
        let mut queries = vec![];
        for query in tab.query_rules.iter() {
            let tab_id = tab_id.clone();
            let rule_id = query.id.clone();
            queries.push(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .w_full()
                    .child(div().w_48().child(Select::new(&query.field).small().placeholder("")))
                    .child(div().w_48().child(Select::new(&query.operator).small().placeholder("")))
                    .child(div().flex_1().child(Input::new(&query.value).small()))
                    .child(
                        Button::new(comp_id(["table-filter-remove", &rule_id]))
                            .ghost()
                            .icon(icon_trash())
                            .on_click(cx.listener({
                                move |view: &mut Self, _, _, cx| {
                                    if let Some(content) = view.table_content(&tab_id) {
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
                div()
                    .flex_1()
                    .border_y_1()
                    .border_color(theme.border)
                    .child(
                        Table::new(&tab.datatable)
                            .stripe(false)
                            .bordered(false)
                            .scrollbar_visible(true, true),
                    )
                    .child(div()),
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
                    .child(filter)
                    .child(column)
                    .child(div().flex_1())
                    .child(page_prev)
                    .child(div().text_sm().child(format!("第 {} 页", page_no + 1)))
                    .child(page_next),
            )
            .when(tab.filter_enable, |this| {
                let viewport_size = window.viewport_size();
                let sheet_size = viewport_size.width / 2.0;
                this.child(
                    Sheet::new(window, cx)
                        .title("筛选数据")
                        .size(sheet_size)
                        .margin_top(px(0.))
                        .on_close(cx.listener({
                            let tab_id = tab_id.clone();
                            move |view: &mut Self, _, _, cx| {
                                if let Some(content) = view.table_content(&tab_id) {
                                    content.filter_enable = false;
                                }
                                cx.notify();
                            }
                        }))
                        .gap_2()
                        .child(
                            div()
                                .gap_4()
                                .row_full()
                                .child(div().text_sm().child("排序规则"))
                                .child(create_order),
                        )
                        .children(orders)
                        .child(
                            div()
                                .gap_4()
                                .row_full()
                                .child(div().text_sm().child("筛选规则"))
                                .child(create_query),
                        )
                        .children(queries)
                        .footer(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap_2()
                                .w_full()
                                .child(div().flex_1())
                                .child(clear_cond)
                                .child(apply_cond),
                        ),
                )
            })
            .when(tab.columns_enable, |this| this.child(div()))
            .into_any_element()
    }

    fn create_schema_tab(
        &mut self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
    }

    fn render_schema_tab(
        &self,
        _tab: &SchemaContent,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> AnyElement {
        div().into_any_element()
    }

    fn render_overview_tab(
        &self,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let theme = cx.theme();
        let overview_fields = self.source.display_overview();

        let detail_card = div()
            .flex()
            .flex_col()
            .gap(px(6.))
            .rounded_md()
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
                    .child(format!("名称：{}", self.source.name)),
            )
            .child(detail_card)
            .into_any_element()
    }
}

impl Render for CommonWorkspace {
    fn render(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let theme = cx.theme().clone();
        let active_tab = self.active_tab.clone();

        let mut tabs = vec![];
        for (id, tab) in &self.tabs {
            let tab_active = id == &active_tab;

            let mut item = div()
                .id(comp_id(["common-tabs-item", &id]))
                .flex()
                .flex_row()
                .items_center()
                .justify_center()
                .h_8()
                .px_2()
                .gap_1()
                .border_1()
                .border_color(theme.border)
                .rounded_md()
                .text_sm()
                .when(tab_active, |this| {
                    this.bg(theme.tab_active).text_color(theme.tab_active_foreground)
                })
                .when(!tab_active, |this| {
                    this.bg(theme.tab_bar).text_color(theme.muted_foreground)
                })
                .on_click(cx.listener({
                    let tab_id = id.clone();
                    // rustfmt::skip
                    move |this, _, _, cx| {
                        if this.tabs.contains_key(&tab_id) {
                            this.active_tab = tab_id.clone();
                            cx.notify();
                        }
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
                    Button::new(comp_id(["common-tabs-close", &id]))
                        .ghost()
                        .xsmall()
                        .icon(IconName::Close)
                        .on_click(cx.listener({
                            let tab_id = id.clone();
                            // rustfmt::skip
                            move |this, _, _, cx| {
                                let Some(i) = this.tabs.get_index_of(&tab_id) else {
                                    return;
                                };
                                let was_active = &this.active_tab == &tab_id;
                                this.tabs.shift_remove(&tab_id);
                                if was_active && !this.tabs.is_empty() {
                                    let fallback = if i == 0 { 0 } else { i - 1 };
                                    this.active_tab = this.tabs.get_index(fallback).unwrap().0.clone();
                                }
                                cx.notify();
                            }
                        })),
                );
            }

            {
                let style = item.style();
                style.flex_grow = Some(0.);
                style.flex_shrink = Some(1.);
                style.min_size.width = Some(Length::Definite(px(0.).into()));
            }

            tabs.push(item)
        }
        let mut tables = vec![];
        for (name, table) in &self.tables {
            let active = self.active_table.as_ref() == Some(name);

            let item = div()
                .id(comp_id(["common-tables-item", &self.source.id, &table.name]))
                .pl_4()
                .pr_6()
                .py_2()
                .gap_2()
                .row_full()
                .items_center()
                .rounded_md()
                .when_else(
                    active,
                    |this| this.bg(theme.list_active),
                    |this| this.hover(|this| this.bg(theme.list_hover)),
                )
                .on_click(cx.listener({
                    let name = name.clone();
                    move |this, _, _, cx| {
                        this.active_table = Some(name.clone());
                        cx.notify();
                    }
                }))
                .on_double_click(cx.listener({
                    let name = name.clone();
                    move |this, _, window, cx| {
                        this.create_table_tab(name.clone(), window, cx);
                    }
                }))
                .child(icon_sheet())
                .child(
                    div()
                        .text_sm()
                        .overflow_hidden()
                        .whitespace_nowrap()
                        .child(table.name.clone()),
                )
                .tooltip({
                    let name = table.name.clone();
                    move |window, cx| Tooltip::new(name.clone()).build(window, cx)
                });
            tables.push(item)
        }

        let id = &self.source.id;
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
                            .on_click(cx.listener({
                                // rustfmt::skip
                                |view: &mut Self, _, _, cx| {
                                    view.reload_tables(cx);
                                }
                            })),
                    )
                    .child(
                        Button::new(comp_id(["common-header-table", id]))
                            .icon(icon_sheet().with_size(Size::Small))
                            .label("新建表")
                            .outline()
                            .on_click(cx.listener({
                                // rustfmt::skip
                                |view: &mut Self, _, window, cx| {
                                    view.create_query_tab(window, cx);
                                }
                            })),
                    )
                    .child(
                        Button::new(comp_id(["common-header-query", id]))
                            .icon(icon_search().with_size(Size::Small))
                            .label("新建查询")
                            .outline()
                            .on_click(cx.listener({
                                // rustfmt::skip
                                |view: &mut Self, _, window, cx| {
                                    view.create_query_tab(window, cx);
                                }
                            })),
                    )
                    .child(
                        Button::new(comp_id(["common-header-import", id]))
                            .icon(icon_import().with_size(Size::Small))
                            .label("数据导入")
                            .outline()
                            .on_click(cx.listener({
                                // rustfmt::skip
                                |view: &mut Self, _, _, cx| {
                                    if let Some(parent) = view.parent.upgrade() {
                                        let source = view.source.clone();
                                        let _ = parent.update(cx, |app, cx| {
                                            app.create_window(WindowKind::Import(source), cx);
                                        });
                                    }
                                }
                            })),
                    )
                    .child(
                        Button::new(comp_id(["common-header-export", id]))
                            .icon(icon_export().with_size(Size::Small))
                            .label("数据导出")
                            .outline()
                            .on_click(cx.listener({
                                // rustfmt::skip
                                |view: &mut Self, _, _, cx| {
                                    if let Some(parent) = view.parent.upgrade() {
                                        let source = view.source.clone();
                                        let _ = parent.update(cx, |app, cx| {
                                            app.create_window(WindowKind::Export(source), cx);
                                        });
                                    }
                                }
                            })),
                    )
                    .child(div().flex_1())
                    .child(
                        Button::new(comp_id(["common-header-schema", id]))
                            .icon(icon_export().with_size(Size::Small))
                            .label("编辑表结构")
                            .outline()
                            .disabled(self.active_table.is_none())
                            .on_click(cx.listener({
                                // rustfmt::skip
                                |view: &mut Self, _, _, cx| {
                                    if let Some(parent) = view.parent.upgrade() {
                                        let source = view.source.clone();
                                        let _ = parent.update(cx, |app, cx| {
                                            app.create_window(WindowKind::Export(source), cx);
                                        });
                                    }
                                }
                            })),
                    ),
            )
            .child(
                div().col_full().child(
                    h_resizable(comp_id(["common-content", id]))
                        .child(
                            resizable_panel()
                                .size(px(240.))
                                .size_range(px(100.)..px(360.))
                                .child(
                                    div()
                                        .id(comp_id(["common-tables", id]))
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
                                        .px_2()
                                        .py_1()
                                        .gap_1()
                                        .min_w_0()
                                        .children(tabs),
                                )
                                .child(div().id(comp_id(["common-main", id])).col_full().child(
                                    match self.tabs.get(&self.active_tab).map(|tab| &tab.content) {
                                        Some(TabContent::Query(tab)) => self.render_query_tab(tab, window, cx),
                                        Some(TabContent::Table(tab)) => self.render_table_tab(tab, window, cx),
                                        Some(TabContent::Schema(tab)) => self.render_schema_tab(tab, window, cx),
                                        Some(TabContent::Overview) | None => self.render_overview_tab(cx),
                                    },
                                ))
                                .into_any_element(),
                        ),
                ),
            )
    }
}

struct QueryRule {
    id: String,
    value: Entity<InputState>,
    field: Entity<SelectState<Vec<SharedString>>>,
    operator: Entity<SelectState<Vec<SharedString>>>,
}

struct OrderRule {
    id: String,
    field: Entity<SelectState<Vec<SharedString>>>,
    order: Entity<SelectState<Vec<SharedString>>>,
}

struct QueryResult {
    sql: String,
    error: Option<String>,
    elapsed: f64,
    datatable: Entity<TableState<DataTable>>,
}

struct QueryContent {
    id: String,
    active: usize,
    summary: bool,
    editor: Entity<InputState>,
    results: Vec<QueryResult>,
}

struct TableContent {
    id: String,
    page_no: usize,
    rows_count: usize,
    order_rules: Vec<OrderRule>,
    query_rules: Vec<QueryRule>,
    filter_enable: bool,
    columns: Vec<SharedString>,
    columns_enable: bool,
    table: SharedString,
    datatable: Entity<TableState<DataTable>>,
}

struct SchemaContent {}
