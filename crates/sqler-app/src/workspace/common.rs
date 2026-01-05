use std::{rc::Rc, time};

use gpui::{prelude::*, *};
use gpui_component::{
    ActiveTheme, Disableable, IconName, InteractiveElementExt, Selectable, Sizable, Size, StyledExt,
    button::{Button, ButtonGroup, ButtonVariants},
    form::{Form, field},
    input::{Input, InputState, TabSize},
    menu::ContextMenuExt,
    resizable::{ResizableState, h_resizable, resizable_panel, v_resizable},
    select::{Select, SelectState},
    table::{Table, TableEvent, TableState},
};
use indexmap::IndexMap;
use serde::Deserialize;
use uuid::Uuid;

use sqler_core::{
    ArcCache, ColumnInfo, DataSource, DatabaseSession, DriverError, FilterCond, Operator, OrderCond, Paging, QueryReq,
    QueryResp, TableInfo, ValueCond, create_connection,
};

use crate::{
    app::{SqlerApp, WindowKind},
    comps::{AppIcon, DataTable, DivExt, comp_id},
};

use super::{EditorComps, parse_elapsed};

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

    pub tabs: IndexMap<SharedString, TabContext>,
    pub active_tab: SharedString,
    pub tables: IndexMap<String, TableInfo>,
    pub active_table: Option<String>,
}

impl CommonWorkspace {
    fn table_action(
        &mut self,
        a: &TableAction,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match a.op {
            TableOp::Open => self.create_table_tab(a.table.clone(), window, cx),
            TableOp::Schema => self.create_schema_tab(a.table.clone(), window, cx),
            TableOp::Create | TableOp::Import | TableOp::Export => {
                // TODO: 暂不实现
            }
            TableOp::DumpData | TableOp::DumpSchema => {
                let Some(parent) = self.parent.upgrade() else {
                    return;
                };
                let source = self.source.clone();
                let _ = parent.update(cx, |app, cx| {
                    app.create_window(
                        WindowKind::Dump {
                            data: matches!(a.op, TableOp::DumpData),
                            table: a.table.clone(),
                            source,
                        },
                        cx,
                    );
                });
            }
            TableOp::Exec => {
                let Some(parent) = self.parent.upgrade() else {
                    return;
                };
                let source = self.source.clone();
                let _ = parent.update(cx, |app, cx| {
                    app.create_window(WindowKind::Exec(source), cx);
                });
            }
        }
    }

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
        self.active_table = None;

        // 清除失效的标签页
        self.tabs.retain(|id, _| {
            if let Some(table_name) = id.strip_suffix("_table") {
                self.tables.contains_key(table_name)
            } else if let Some(table_name) = id.strip_suffix("_schema") {
                self.tables.contains_key(table_name)
            } else {
                true
            }
        });
        cx.notify();
    }

    fn query_content(
        &mut self,
        tab_id: &SharedString,
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
        tab_id: &SharedString,
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
        tab_id: &SharedString,
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
        let tab_id = SharedString::from(Uuid::new_v4().to_string());

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
        tab_id: &SharedString,
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
                let datatable = DataTable::new(vec![], vec![], window, cx);
                content.results.push(QueryResult {
                    sql: sql.to_string(),
                    error: None,
                    count: 0,
                    running: true,
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
                                let (cols, rows) = match query_resp {
                                    QueryResp::Rows { cols, rows } => (cols, rows),
                                    _ => (vec![], vec![]),
                                };

                                // 提取列名和数据
                                let mut table_rows = Vec::with_capacity(rows.len());
                                for row in rows {
                                    let mut record = Vec::with_capacity(cols.len());
                                    for col in &cols {
                                        let value = row.get(col).cloned().unwrap_or_default();
                                        record.push(SharedString::from(value));
                                    }
                                    table_rows.push(record);
                                }
                                let table_cols: Vec<SharedString> = cols.into_iter().map(SharedString::from).collect();
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
                            ret.count = rows.len();
                            ret.running = false;
                            ret.elapsed = elapsed;
                            ret.datatable.update(cx, |t, cx| {
                                t.delegate_mut().update_data(cols, rows);
                                t.refresh(cx);
                                cx.notify();
                            });
                        }
                        content.active = 0;
                        content.summary = false;
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
        let tab_id = &tab.id;

        let mut results = vec![];
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

        let summary = Button::new(comp_id(["query-summary"]))
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

        let mut summaries = vec![];
        for ret in &tab.results {
            let item = div()
                .flex()
                .flex_col()
                .p_4()
                .gap_2()
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
                        .child(div().child(parse_elapsed(ret.elapsed))),
                )
                .when_some(ret.error.as_ref(), |this, err| {
                    this.child(div().text_color(theme.danger).child(format!("错误: {}", err)))
                })
                .into_any_element();

            summaries.push(item);
        }

        v_resizable(comp_id(["query-content", &tab_id]))
            .child(
                resizable_panel()
                    .size(px(200.0))
                    .size_range(px(200.)..Pixels::MAX)
                    .child(
                        div()
                            .flex_1()
                            .border_t_1()
                            .border_color(theme.border)
                            .child(
                                Input::new(&tab.editor)
                                    .p_0()
                                    .h_full()
                                    .appearance(false)
                                    .text_sm()
                                    .font_family(theme.mono_font_family.clone())
                                    .focus_bordered(false),
                            )
                            .child(div()),
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
                            .child(summary)
                            .children(results)
                            .child(div().flex_1())
                            .child(execute),
                    )
                    .child(
                        div()
                            .id(comp_id(["query-result-content", &tab_id]))
                            .relative()
                            .col_full()
                            .when_some(tab.results.get(tab.active), |this, ret| {
                                if tab.summary {
                                    return this.child(
                                        div()
                                            .full()
                                            .scrollbar_y()
                                            .child(div().p_2().gap_2().col_full().children(summaries)),
                                    );
                                }
                                this.child(
                                    div().flex_1().child(
                                        Table::new(&ret.datatable)
                                            .stripe(false)
                                            .bordered(false)
                                            .scrollbar_visible(true, true),
                                    ),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .h_10()
                                        .p_4()
                                        .gap_4()
                                        .bg(theme.secondary)
                                        .text_sm()
                                        .border_t_1()
                                        .border_color(theme.border)
                                        .child(div().flex_1())
                                        .child(format!("共 {} 条记录", ret.count))
                                        .child(format!("运行耗时 {}", parse_elapsed(ret.elapsed))),
                                )
                            }),
                    )
                    .into_any_element(),
            )
            .into_any_element()
    }

    fn create_table_tab(
        &mut self,
        table: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let tab_id = SharedString::from(format!("{}_table", table));
        if self.tabs.contains_key(&tab_id) {
            self.active_tab = tab_id;
            cx.notify();
            return;
        }

        let table_id = tab_id.clone();
        let datatable = DataTable::new(vec![], vec![], window, cx);
        let subscription = cx.subscribe_in(
            &datatable,
            window,
            move |view: &mut Self, _, event: &TableEvent, window, cx| {
                if let TableEvent::SelectRow(i) = event {
                    let Some(content) = view.table_content(&table_id) else {
                        return;
                    };
                    if content.form_items.is_empty() {
                        return;
                    }
                    content
                        .form_items
                        .iter()
                        .zip(content.datatable.read(cx).delegate().get_data(*i).iter())
                        .for_each(|(state, value)| {
                            let value = value.to_string();
                            state.update(cx, |state, cx| {
                                state.set_value(&value, window, cx);
                            });
                        });
                }
            },
        );

        // 新建标签页
        self.tabs.insert(
            tab_id.clone(),
            TabContext {
                title: SharedString::from(format!("{}[数据]", table)),
                content: TabContent::Table(TableContent {
                    id: tab_id.clone(),
                    page: 0,
                    count: 0,
                    table: SharedString::from(table),
                    columns: vec![],
                    form_items: vec![],
                    query_rules: vec![],
                    order_rules: vec![],
                    detail_panel: false,
                    detail_panel_idx: 0,
                    detail_panel_state: cx.new(|_| ResizableState::default()),
                    datatable,
                    _subscription: subscription,
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
        tab_id: &SharedString,
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

            (content.table.clone(), content.page, orders, filters)
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
                    // 查询数据
                    let query_resp = session.query(QueryReq::Builder {
                        table: table.to_string(),
                        columns: vec![],
                        paging: Some(Paging::new(page, PAGE_SIZE)),
                        orders,
                        filters,
                    })?;
                    let (cols, rows) = match query_resp {
                        QueryResp::Rows { cols, rows } => (cols, rows),
                        _ => (vec![], vec![]),
                    };

                    // 构建渲染数据
                    let mut table_rows = Vec::with_capacity(rows.len());
                    for row in rows {
                        let mut record = Vec::with_capacity(cols.len());
                        for col in &cols {
                            let value = row.get(col).cloned().unwrap_or_default();
                            record.push(SharedString::from(value));
                        }
                        table_rows.push(record);
                    }
                    let table_cols: Vec<SharedString> = cols.iter().map(|c| SharedString::from(c.clone())).collect();
                    Ok::<_, DriverError>(((table_cols, table_rows), session))
                })
                .await;

            // 更新 UI
            let _ = cx.update(|window, cx| {
                let _ = this.update(cx, |this, cx| match ret {
                    Ok((data, session)) => {
                        this.session = Some(session);

                        let (cols, rows) = data;
                        let Some(content) = this.table_content(&tab_id) else {
                            return;
                        };
                        content.page = page;
                        content.count = rows.len();
                        content.columns = cols.clone();
                        if content.form_items.len() != cols.len() && cols.len() > 0 {
                            content.form_items = cols
                                .iter()
                                .map(|_| {
                                    let input = cx.new(|cx| {
                                        InputState::new(window, cx)
                                            .auto_grow(1, 10)
                                            .multi_line(true)
                                            .searchable(false)
                                    });
                                    input
                                })
                                .collect();
                        }

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
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let theme = cx.theme().clone();
        let tab_id = &tab.id;

        let page = tab.page;
        let page_prev = Button::new(comp_id(["table-page-prev", &tab_id]))
            .label("上一页")
            .outline()
            .disabled(page == 0)
            .on_click(cx.listener({
                let tab_id = tab_id.clone();
                let prev_page = page.saturating_sub(1);
                move |view: &mut Self, _, window, cx| {
                    if let Some(content) = view.table_content(&tab_id) {
                        content.page = prev_page;
                    }
                    view.reload_table_tab(&tab_id, window, cx);
                }
            }));
        let page_next = Button::new(comp_id(["table-page-next", &tab_id]))
            .label("下一页")
            .outline()
            .disabled(tab.count == 0)
            .on_click(cx.listener({
                let tab_id = tab_id.clone();
                let next_page = page.saturating_add(1);
                move |view: &mut Self, _, window, cx| {
                    if let Some(content) = view.table_content(&tab_id) {
                        content.page = next_page;
                    }
                    view.reload_table_tab(&tab_id, window, cx);
                }
            }));
        let right_panel = Button::new(comp_id(["table-panel", &tab_id]))
            .outline()
            .when_else(
                tab.detail_panel,
                |this| this.icon(IconName::PanelRightClose),
                |this| this.icon(IconName::PanelRightOpen),
            )
            .on_click(cx.listener({
                let tab_id = tab_id.clone();
                move |view: &mut Self, _, _, cx| {
                    if let Some(content) = view.table_content(&tab_id) {
                        content.detail_panel = !content.detail_panel;
                    }
                    cx.notify();
                }
            }));

        let right_panel_tabs = div().row_full().min_h_12().justify_center().child(
            ButtonGroup::new("table-panel-switcher")
                .outline()
                .compact()
                .child(
                    Button::new(comp_id(["table-panel-form", &tab_id]))
                        .label("表单视图")
                        .selected(tab.detail_panel_idx == 0),
                )
                .child(
                    Button::new(comp_id(["table-panel-filter", &tab_id]))
                        .label("筛选数据")
                        .selected(tab.detail_panel_idx == 1),
                )
                .on_click(cx.listener({
                    let tab_id = tab_id.clone();
                    move |view, selected: &Vec<usize>, _, cx| {
                        if let Some(content) = view.table_content(&tab_id) {
                            content.detail_panel_idx = selected[0];
                        }
                        cx.notify();
                    }
                })),
        );

        // 表单视图
        let mut form_panel = div().px_4().py_2().col_full();
        form_panel = tab.columns.iter().enumerate().fold(form_panel, {
            // 渲染表单项
            |panel, (i, name)| {
                let Some(state) = tab.form_items.get(i) else {
                    return panel;
                };
                let input = Input::new(state).px_2().py_1().disabled(true);
                panel.child(
                    div()
                        .flex()
                        .flex_col()
                        .pb_3()
                        .gap_1()
                        .text_sm()
                        .child(div().font_semibold().child(name.clone()))
                        .child(div().child(input)),
                )
            }
        });

        // 筛选数据
        #[rustfmt::skip]
        let order_ops = vec![
            SharedString::from(ORDER_ASC), SharedString::from(ORDER_DESC)
        ];
        let query_ops: Vec<SharedString> = Operator::all()
            .into_iter()
            .map(|op| SharedString::from(op.label().to_string()))
            .collect();

        let filter_apply = Button::new(comp_id(["table-filter-apply", &tab_id]))
            .label("应用条件")
            .outline()
            .on_click(cx.listener({
                let tab_id = tab_id.clone();
                move |view: &mut Self, _, window, cx| {
                    if let Some(content) = view.table_content(&tab_id) {
                        content.page = 0;
                    }
                    view.reload_table_tab(&tab_id, window, cx);
                }
            }));
        let filter_clear = Button::new(comp_id(["table-filter-clear", &tab_id]))
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
                    let Some(content) = view.table_content(&tab_id) else {
                        return;
                    };
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
                    let Some(content) = view.table_content(&tab_id) else {
                        return;
                    };
                    content.query_rules.push(QueryRule {
                        id: Uuid::new_v4().to_string(),
                        field: cx.new(|cx| {
                            // rustfmt::skip
                            SelectState::new(columns.clone(), None, window, cx)
                        }),
                        operator: cx.new(|cx| {
                            // rustfmt::skip
                            SelectState::new(query_ops.clone(), None, window, cx)
                        }),
                        value: cx.new(|cx| {
                            // rustfmt::skip
                            InputState::new(window, cx)
                        }),
                    });
                    cx.notify();
                }
            }));

        let filter_panel = div()
            .px_4()
            .py_2()
            .gap_4()
            .col_full()
            .child(
                div()
                    .gap_4()
                    .flex()
                    .flex_row()
                    .child(div().text_sm().child("排序规则"))
                    .child(create_order),
            )
            .children(tab.order_rules.iter().map(|order| {
                let tab_id = tab_id.clone();
                let rule_id = order.id.clone();
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
                            .icon(AppIcon::Trash)
                            .on_click(cx.listener({
                                move |view: &mut Self, _, _, cx| {
                                    if let Some(content) = view.table_content(&tab_id) {
                                        content.order_rules.retain(|r| &r.id != &rule_id);
                                    }
                                    cx.notify();
                                }
                            })),
                    )
            }))
            .child(
                div()
                    .gap_4()
                    .flex()
                    .flex_row()
                    .child(div().text_sm().child("筛选规则"))
                    .child(create_query),
            )
            .children(tab.query_rules.iter().map(|query| {
                let tab_id = tab_id.clone();
                let rule_id = query.id.clone();

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
                            .icon(AppIcon::Trash)
                            .on_click(cx.listener({
                                move |view: &mut Self, _, _, cx| {
                                    if let Some(content) = view.table_content(&tab_id) {
                                        content.query_rules.retain(|r| &r.id != &rule_id);
                                    }
                                    cx.notify();
                                }
                            })),
                    )
            }));

        // 筛选字段
        h_resizable(comp_id(["table-content", &tab_id]))
            .with_state(&tab.detail_panel_state)
            .child(
                div()
                    .col_full()
                    .border_y_1()
                    .border_color(theme.border)
                    .child(
                        Table::new(&tab.datatable)
                            .stripe(false)
                            .bordered(false)
                            .scrollbar_visible(true, true),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .h_12()
                            .p_2()
                            .gap_2()
                            .bg(theme.secondary)
                            .child(div().flex_1())
                            .child(page_prev)
                            .child(div().text_sm().child(format!("第 {} 页", page + 1)))
                            .child(page_next)
                            .child(right_panel),
                    )
                    .into_any_element(),
            )
            .child(
                resizable_panel()
                    .visible(tab.detail_panel)
                    .size(px(400.))
                    .size_range(px(400.)..Pixels::MAX)
                    .child(
                        div()
                            .col_full()
                            .border_t_1()
                            .border_color(theme.border)
                            .child(right_panel_tabs)
                            .when(tab.detail_panel_idx == 0, |this| {
                                this.child(div().full().scrollbar_y().child(form_panel))
                            })
                            .when(tab.detail_panel_idx == 1, |this| {
                                this.child(div().full().scrollbar_y().child(filter_panel)).child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .h_12()
                                        .p_2()
                                        .gap_2()
                                        .w_full()
                                        .child(div().flex_1())
                                        .child(filter_clear)
                                        .child(filter_apply),
                                )
                            }),
                    ),
            )
            .into_any_element()
    }

    fn create_schema_tab(
        &mut self,
        table: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let tab_id = SharedString::from(format!("{}_schema", table));
        if self.tabs.contains_key(&tab_id) {
            self.active_tab = tab_id;
            cx.notify();
            return;
        }

        // 新建标签页
        self.tabs.insert(
            tab_id.clone(),
            TabContext {
                title: SharedString::from(format!("{}[结构]", table)),
                content: TabContent::Schema(SchemaContent {
                    id: tab_id.clone(),
                    table: SharedString::from(table.clone()),
                    columns: vec![],
                    datatable: DataTable::new(vec![], vec![], window, cx),
                }),
                closable: true,
            },
        );
        self.active_tab = tab_id.clone();
        cx.notify();

        // 异步加载数据
        self.reload_schema_tab(&tab_id, window, cx);
    }

    fn reload_schema_tab(
        &mut self,
        tab_id: &SharedString,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let table = {
            let Some(content) = self.schema_content(tab_id) else {
                return;
            };

            // 设置表格加载状态
            content.datatable.update(cx, |t, cx| {
                t.delegate_mut().update_loading(true);
                cx.notify();
            });

            content.table.clone()
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
                    // 查询列信息
                    let columns = session.columns(&table)?;
                    Ok::<_, DriverError>((columns, session))
                })
                .await;

            // 更新 UI
            let _ = cx.update(|_, cx| {
                let _ = this.update(cx, |this, cx| match ret {
                    Ok((columns, session)) => {
                        this.session = Some(session);

                        let Some(content) = this.schema_content(&tab_id) else {
                            return;
                        };

                        // 构建表格数据
                        let table_cols = vec![
                            SharedString::from("列名"),
                            SharedString::from("数据类型"),
                            SharedString::from("注释"),
                        ];
                        let mut table_rows = Vec::with_capacity(columns.len());
                        for col in &columns {
                            table_rows.push(vec![
                                SharedString::from(col.name.clone()),
                                SharedString::from(col.kind.clone()),
                                SharedString::from(col.comment.clone()),
                            ]);
                        }

                        content.columns = columns;
                        content.datatable.update(cx, |t, cx| {
                            t.delegate_mut().update_data(table_cols, table_rows);
                            t.delegate_mut().update_loading(false);
                            t.refresh(cx);
                            cx.notify();
                        });
                        cx.notify();
                    }
                    Err(err) => {
                        tracing::error!("加载表结构失败: {}", err);
                        this.session = None;

                        let Some(content) = this.schema_content(&tab_id) else {
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

    fn render_schema_tab(
        &self,
        tab: &SchemaContent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let theme = cx.theme().clone();
        let tab_id = &tab.id;

        let col_info = {
            tab.datatable
                .read(cx)
                .selected_row()
                .and_then(|idx| tab.columns.get(idx))
        };

        let detail_content = if let Some(col) = col_info {
            div().p_4().col_full().scrollbar_y().child(
                Form::vertical()
                    .layout(Axis::Horizontal)
                    .with_size(Size::Large)
                    .label_width(px(120.))
                    .child(field().label("字段").child(col.name.clone()))
                    .child(
                        field()
                            .label("主键")
                            .child(div().child(if col.primary_key { "是" } else { "否" })),
                    )
                    .child(
                        field()
                            .label("自增")
                            .child(div().child(if col.auto_increment { "是" } else { "否" })),
                    )
                    .child(
                        field()
                            .label("默认值")
                            .child(div().child(if col.default_value.is_empty() {
                                "-".to_string()
                            } else {
                                col.default_value.clone()
                            })),
                    ),
            )
        } else {
            div().col_full().scrollbar_y()
        };

        v_resizable(comp_id(["schema-content", &tab_id]))
            .child(
                div()
                    .flex_1()
                    .border_t_1()
                    .border_color(theme.border)
                    .child(
                        Table::new(&tab.datatable)
                            .stripe(false)
                            .bordered(false)
                            .scrollbar_visible(true, true),
                    )
                    .into_any_element(),
            )
            .child(
                resizable_panel()
                    .size(px(400.0))
                    .size_range(px(200.)..Pixels::MAX)
                    .child(detail_content),
            )
            .into_any_element()
    }

    fn render_overview_tab(
        &self,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let theme = cx.theme();
        let mut infos = vec![];
        infos.push(("名称", self.source.name.clone()));
        infos.extend(self.source.display_overview());

        let mut fields = vec![];
        for (label, value) in infos {
            let item = div()
                .flex()
                .flex_row()
                .items_center()
                .child(div().w_32().child(label))
                .child(
                    div()
                        .w_96()
                        .px_2()
                        .py_1()
                        .rounded_md()
                        .bg(theme.muted)
                        .border_1()
                        .border_color(theme.border)
                        .child(value),
                );
            fields.push(item);
        }
        div()
            .min_w_128()
            .p_4()
            .col_full()
            .text_sm()
            .text_color(theme.foreground)
            .child(
                div()
                    .full()
                    .scrollbar_y()
                    .child(div().gap_4().col_full().children(fields)),
            )
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
                .on_mouse_down(
                    MouseButton::Right,
                    cx.listener({
                        let name = name.clone();
                        move |this, _, _, cx| {
                            this.active_table = Some(name.clone());
                            cx.notify()
                        }
                    }),
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
                .child(AppIcon::Sheet)
                .child(
                    div()
                        .text_sm()
                        .overflow_hidden()
                        .whitespace_nowrap()
                        .child(table.name.clone()),
                );
            tables.push(item)
        }

        let id = &self.source.id;
        let sidebar = div()
            .col_full()
            .scrollbar_y()
            .child(
                div()
                    .id(comp_id(["common-tables", id]))
                    .p_2()
                    .gap_2()
                    .flex()
                    .flex_col()
                    .children(tables),
            )
            .context_menu({
                let view = cx.entity().clone();
                move |this, window, cx| {
                    let Some(table) = view.read(cx).active_table.clone() else {
                        return this;
                    };
                    this.menu("新建表", TableAction::new(TableOp::Create, table.clone()))
                        .separator()
                        .menu("打开表", TableAction::new(TableOp::Open, table.clone()))
                        .menu("设计表", TableAction::new(TableOp::Schema, table.clone()))
                        .separator()
                        .menu("导入向导", TableAction::new(TableOp::Import, table.clone()))
                        .menu("导出向导", TableAction::new(TableOp::Export, table.clone()))
                        .separator()
                        .menu("运行 SQL 文件", TableAction::new(TableOp::Exec, table.clone()))
                        .submenu("转储 SQL 文件", window, cx, {
                            let table = table.clone();
                            move |child, _, _| {
                                child
                                    .menu("数据和结构", TableAction::new(TableOp::DumpData, table.clone()))
                                    .menu("仅结构", TableAction::new(TableOp::DumpSchema, table.clone()))
                            }
                        })
                }
            });

        div()
            .id(comp_id(["common", id]))
            .col_full()
            .on_action(cx.listener(Self::table_action))
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
                            .icon(AppIcon::Relead)
                            .label("刷新表")
                            .outline()
                            .on_click(cx.listener({
                                |view: &mut Self, _, _, cx| {
                                    view.reload_tables(cx);
                                }
                            })),
                    )
                    .child(
                        Button::new(comp_id(["common-header-table", id]))
                            .icon(AppIcon::Sheet)
                            .label("新建表")
                            .outline()
                            .on_click(cx.listener({
                                |view: &mut Self, _, window, cx| {
                                    view.create_query_tab(window, cx);
                                }
                            })),
                    )
                    .child(
                        Button::new(comp_id(["common-header-query", id]))
                            .icon(AppIcon::Search)
                            .label("新建查询")
                            .outline()
                            .on_click(cx.listener({
                                |view: &mut Self, _, window, cx| {
                                    view.create_query_tab(window, cx);
                                }
                            })),
                    )
                    .child(
                        Button::new(comp_id(["common-header-import", id]))
                            .icon(AppIcon::Import)
                            .label("数据导入")
                            .outline()
                            .on_click(cx.listener({
                                |view: &mut Self, _, _, cx| {
                                    let Some(parent) = view.parent.upgrade() else {
                                        return;
                                    };
                                    let source = view.source.clone();
                                    let _ = parent.update(cx, |app, cx| {
                                        app.create_window(WindowKind::Import(source), cx);
                                    });
                                }
                            })),
                    )
                    .child(
                        Button::new(comp_id(["common-header-export", id]))
                            .icon(AppIcon::Export)
                            .label("数据导出")
                            .outline()
                            .on_click(cx.listener({
                                |view: &mut Self, _, _, cx| {
                                    let Some(parent) = view.parent.upgrade() else {
                                        return;
                                    };
                                    let source = view.source.clone();
                                    let _ = parent.update(cx, |app, cx| {
                                        app.create_window(WindowKind::Export(source), cx);
                                    });
                                }
                            })),
                    )
                    .child(div().flex_1()),
            )
            .child(
                div().col_full().child(
                    h_resizable(comp_id(["common-content", id]))
                        .child(
                            resizable_panel()
                                .size(px(240.))
                                .size_range(px(120.)..px(480.))
                                .child(sidebar),
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

#[derive(Clone, PartialEq, Eq, Deserialize)]
enum TableOp {
    Create, // 新建表

    Open,   // 打开表（查看数据）
    Schema, // 设计表（查看结构）

    Import, // 运行 SQL 文件
    Export, // 转储 SQL 文件

    Exec,       // 运行 SQL 文件
    DumpData,   // 结构 + 数据
    DumpSchema, // 仅 结构
}

#[derive(Action, Clone, PartialEq, Eq, Deserialize)]
#[action(namespace = common_workspace, no_json)]
struct TableAction {
    op: TableOp,
    table: String,
}

impl TableAction {
    fn new(
        op: TableOp,
        table: String,
    ) -> Box<dyn Action> {
        Box::new(Self { op, table })
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
    count: usize,
    running: bool,
    elapsed: f64,
    datatable: Entity<TableState<DataTable>>,
}

struct QueryContent {
    id: SharedString,
    active: usize,
    summary: bool,
    editor: Entity<InputState>,
    results: Vec<QueryResult>,
}

struct TableContent {
    id: SharedString,
    page: usize,
    count: usize,
    table: SharedString,
    columns: Vec<SharedString>,
    form_items: Vec<Entity<InputState>>,
    order_rules: Vec<OrderRule>,
    query_rules: Vec<QueryRule>,
    detail_panel: bool,
    detail_panel_idx: usize,
    detail_panel_state: Entity<ResizableState>,
    datatable: Entity<TableState<DataTable>>,
    _subscription: Subscription,
}

struct SchemaContent {
    id: SharedString,
    table: SharedString,
    columns: Vec<ColumnInfo>,
    datatable: Entity<TableState<DataTable>>,
}
