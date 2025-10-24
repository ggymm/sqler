use gpui::prelude::*;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::resizable::{h_resizable, resizable_panel, ResizableState};
use gpui_component::scroll::Scrollable;
use gpui_component::tab::{Tab, TabBar};
use gpui_component::table::Column;
use gpui_component::{ActiveTheme as _, InteractiveElementExt, Selectable, Sizable, Size, StyledExt};

use crate::app::comps::comp_id;
use crate::app::comps::icon_close;
use crate::app::comps::icon_export;
use crate::app::comps::icon_import;
use crate::app::comps::icon_search;
use crate::app::comps::DataTable;
use crate::option::DataSource;
use crate::option::DataSourceOptions;

pub struct MySQLWorkspace {
    sidebar_resize: Entity<ResizableState>,

    meta: DataSource,
    selected_table: Option<SharedString>,
    tabs: Vec<TabItem>,
    active_tab: SharedString,
    next_tab_seq: u64,
}

impl MySQLWorkspace {
    pub fn new(
        meta: DataSource,
        cx: &mut Context<Self>,
    ) -> Self {
        let selected_table = meta.tables().into_iter().next();

        let overview = TabItem::overview();
        let active = overview.id.clone();
        Self {
            sidebar_resize: ResizableState::new(cx),
            meta,
            selected_table,
            tabs: vec![overview],
            active_tab: active,
            next_tab_seq: 1,
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
                }
            }
            cx.notify();
        }
    }
    
    fn active_tab( &mut self, id: SharedString, cx: &mut Context<Self>) {
        self.active_tab = id;
        cx.notify();
    }

    fn create_tab_table_data(
        &mut self,
        table: SharedString,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(existing) = self.tabs.iter().find(|tab| {
            matches!(
                &tab.content,
                TabContent::TableBrowser(current) if current.table_name == table
            )
        }) {
            self.active_tab = existing.id.clone();
            cx.notify();
            return;
        }

        let columns = self.build_table_columns();
        let rows = self.build_sample_rows(&table);
        let data_table = DataTable::new(window, cx, columns, rows);

        let tab_id = SharedString::from(format!("mysql-table-tab-{}-{}", self.meta.id, self.next_tab_seq));
        let tab_title = table.clone();

        let tab_item = TabItem {
            id: tab_id.clone(),
            title: tab_title,
            closable: true,
            content: TabContent::TableBrowser(TableBrowserTab {
                view_id: tab_id.clone(),
                table_name: table,
                data_table,
            }),
        };

        self.tabs.push(tab_item);
        self.active_tab = tab_id;
        self.next_tab_seq += 1;
        cx.notify();
    }

    fn open_table_tab(
        &mut self,
        table_name: SharedString,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(existing) = self.tabs.iter().find(|tab| {
            matches!(
                &tab.content,
                TabContent::TableBrowser(current) if current.table_name == table_name
            )
        }) {
            self.active_tab = existing.id.clone();
            cx.notify();
            return;
        }

        let columns = self.build_table_columns();
        let rows = self.build_sample_rows(&table_name);
        let data_table = DataTable::new(window, cx, columns, rows);

        let tab_id = SharedString::from(format!("mysql-table-tab-{}-{}", self.meta.id, self.next_tab_seq));
        let tab_title = table_name.clone();

        let tab_item = TabItem {
            id: tab_id.clone(),
            title: tab_title,
            closable: true,
            content: TabContent::TableBrowser(TableBrowserTab {
                view_id: tab_id.clone(),
                table_name,
                data_table,
            }),
        };

        self.tabs.push(tab_item);
        self.active_tab = tab_id;
        self.next_tab_seq += 1;
        cx.notify();
    }

    fn build_table_columns(&self) -> Vec<Column> {
        vec![
            Column::new("id", "ID").width(px(80.)).sortable(),
            Column::new("name", "名称").width(px(160.)).sortable(),
            Column::new("owner", "负责人").width(px(140.)),
            Column::new("updated", "更新时间").width(px(180.)).sortable(),
            Column::new("records", "记录数").width(px(120.)).text_right().sortable(),
            Column::new("status", "状态").width(px(120.)),
        ]
    }

    fn build_sample_rows(
        &self,
        table_name: &SharedString,
    ) -> Vec<Vec<SharedString>> {
        (0..25)
            .map(|index| {
                vec![
                    SharedString::from(format!("{}", index + 1)),
                    SharedString::from(format!("{} 行", table_name)),
                    SharedString::from("数据团队"),
                    SharedString::from(format!("2024-07-{:02} 1{:02}:32", (index % 30) + 1, index % 60)),
                    SharedString::from(format!("{} 条", (index + 1) * 128)),
                    SharedString::from(if index % 2 == 0 { "可用" } else { "维护中" }),
                ]
            })
            .collect()
    }

    fn active_content(&self) -> Option<&TabContent> {
        self.tabs
            .iter()
            .find(|tab| tab.id == self.active_tab)
            .map(|tab| &tab.content)
    }

    fn render_table_browser(
        &self,
        tab: &TableBrowserTab,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let theme = cx.theme().clone();
        let container_id = tab.view_id.to_string();
        div()
            .id("table")
            .flex()
            .flex_col()
            .gap(px(12.))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .justify_between()
                    .items_center()
                    .text_sm()
                    .text_color(theme.muted_foreground)
                    .child(format!("数据表：{}", tab.table_name)),
            )
            .child(tab.data_table.render(&container_id, cx))
            .into_any_element()
    }

    fn render_overview(
        &self,
        cx: &mut Context<Self>,
    ) -> Scrollable<Div> {
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
    }
}

impl Render for MySQLWorkspace {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let id = &self.meta.id;
        let tables = self.meta.tables();
        let active_tab = self.tabs.iter().position(|tab| tab.id == self.active_tab).unwrap_or(0);
        let active_content = self.active_content().cloned();

        let theme = cx.theme().clone();
        let menu = tables.iter().cloned().fold(
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
                let selected = self.selected_table.as_ref() == Some(&table);
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
                        .cursor_pointer()
                        .hover(|this| this.bg(theme.secondary_hover))
                        .when(selected, |this| {
                            this.bg(theme.secondary_hover)
                                .text_color(theme.foreground)
                                .font_semibold()
                        })
                        .on_double_click(cx.listener(move |this, _, window, cx| {
                            let table_id = click_table.clone();
                            this.selected_table = Some(table_id.clone());
                            this.open_table_tab(table_id, window, cx);
                            cx.notify();
                        }))
                        .child(table.clone()),
                )
            },
        );

        let tabs = TabBar::new(comp_id(["mysql-main-tabs", id]))
            .with_size(Size::Small)
            .children(
                self.tabs
                    .iter()
                    .enumerate()
                    .map(|(index, tab)| {
                        let tab_id = tab.id.clone();
                        Tab::new(tab.title.clone())
                            .id(comp_id(["mysql-main-tab-item", id, &tab_id]))
                            .px_2()
                            .when(tab.closable, |this| {
                                this.suffix(
                                    Button::new(comp_id(["mysql-main-tab-close", &tab_id]))
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
                                move |view: &mut Self, _, _, cx| {
                                    view.active_tab(tab_id.clone(), cx);
                                }
                            }))
                    })
                    .collect::<Vec<_>>(),
            )
            .selected_index(active_tab);

        let tab_body = match active_content {
            Some(TabContent::TableBrowser(tab)) => self.render_table_browser(&tab, cx),
            Some(TabContent::Overview) | None => self.render_overview(cx).into_any_element(),
            Some(TabContent::Placeholder) => div()
                .flex()
                .flex_col()
                .scrollable(Axis::Vertical)
                .gap(px(8.))
                .child(div().text_base().font_semibold().child("自定义视图"))
                .child(
                    div()
                        .text_sm()
                        .text_color(theme.muted_foreground)
                        .child("在这里扩展你的分析组件。"),
                )
                .into_any_element(),
        };

        div()
            .id(comp_id(["mysql", id]))
            .flex()
            .flex_col()
            .flex_1()
            .size_full()
            .min_w_0()
            .min_h_0()
            .child(
                div()
                    .id(comp_id(["mysql-head", id]))
                    .flex()
                    .flex_row()
                    .px_4()
                    .py_4()
                    .gap_2()
                    .border_b_1()
                    .border_color(theme.border)
                    .child(
                        Button::new(comp_id(["mysql-head-refresh", id]))
                            .outline()
                            .icon(icon_search().with_size(Size::Small))
                            .label("刷新表"),
                    )
                    .child(
                        Button::new(comp_id(["mysql-head-query", id]))
                            .outline()
                            .icon(icon_search().with_size(Size::Small))
                            .label("新建查询"),
                    )
                    .child(
                        Button::new(comp_id(["mysql-head-import", id]))
                            .outline()
                            .icon(icon_import().with_size(Size::Small))
                            .label("数据导入"),
                    )
                    .child(
                        Button::new(comp_id(["mysql-head-export", id]))
                            .outline()
                            .icon(icon_export().with_size(Size::Small))
                            .label("数据导出"),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .size_full()
                    .min_w_0()
                    .min_h_0()
                    .child(
                        h_resizable(comp_id(["mysql-main", id]), self.sidebar_resize.clone())
                            .child(
                                resizable_panel()
                                    .size(px(240.0))
                                    .size_range(px(120.)..px(360.))
                                    .child(menu),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .flex_1()
                                    .size_full()
                                    .min_w_0()
                                    .min_h_0()
                                    .child(tabs)
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .flex_1()
                                            .size_full()
                                            .min_w_0()
                                            .min_h_0()
                                            .p_2()
                                            .rounded_lg()
                                            .child(tab_body),
                                    )
                                    .into_any_element(),
                            ),
                    )
                    .child(div()),
            )
    }
}

#[derive(Clone)]
struct TabItem {
    id: SharedString,
    title: SharedString,
    closable: bool,
    content: TabContent,
}

impl TabItem {
    fn overview() -> Self {
        Self {
            id: SharedString::from("mysql-tab-overview"),
            title: SharedString::from("概览"),
            closable: false,
            content: TabContent::Overview,
        }
    }
}

#[derive(Clone)]
enum TabContent {
    Overview,
    TableBrowser(TableBrowserTab),
    Placeholder,
}

#[derive(Clone)]
struct TableBrowserTab {
    view_id: SharedString,
    table_name: SharedString,
    data_table: DataTable,
}
