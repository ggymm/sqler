use gpui::prelude::*;
use gpui::*;
use gpui_component::button::Button;
use gpui_component::button::ButtonVariants;
use gpui_component::resizable::h_resizable;
use gpui_component::resizable::resizable_panel;
use gpui_component::resizable::ResizableState;
use gpui_component::tab::Tab;
use gpui_component::tab::TabBar;
use gpui_component::table::Column;
use gpui_component::ActiveTheme;
use gpui_component::InteractiveElementExt;
use gpui_component::Selectable;
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
use crate::option::DataSource;
use crate::option::DataSourceOptions;

pub struct MySQLWorkspace {
    meta: DataSource,
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
        let tables = meta.tables();

        let overview = TabItem::overview();
        let active_tab = overview.id.clone();
        let active_table = meta.tables().into_iter().next();
        Self {
            meta,
            tabs: vec![overview],
            active_tab,
            tables,
            active_table,
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

        let data_table = DataTable::new(
            window,
            cx,
            vec![
                Column::new("id", "ID").width(px(80.)).sortable(),
                Column::new("name", "名称").width(px(160.)).sortable(),
                Column::new("owner", "负责人").width(px(140.)),
                Column::new("updated", "更新时间").width(px(180.)).sortable(),
                Column::new("records", "记录数").width(px(120.)).text_right().sortable(),
                Column::new("status", "状态").width(px(120.)),
            ],
            (0..25)
                .map(|index| {
                    vec![
                        SharedString::from(format!("{}", index + 1)),
                        SharedString::from(format!("{} 行", table)),
                        SharedString::from("数据团队"),
                        SharedString::from(format!("2024-07-{:02} 1{:02}:32", (index % 30) + 1, index % 60)),
                        SharedString::from(format!("{} 条", (index + 1) * 128)),
                        SharedString::from(if index % 2 == 0 { "可用" } else { "维护中" }),
                    ]
                })
                .collect(),
        );

        self.tabs.push(TabItem {
            id: id.clone(),
            title: table.clone(),
            content: TabContent::DataTable(DataTableTab {
                id: id.clone(),
                title: table.clone(),
                content: data_table,
            }),
            closable: true,
        });
        cx.notify();
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
        div()
            .flex()
            .flex_col()
            .flex_1()
            .size_full()
            .min_w_0()
            .min_h_0()
            .child(tab.content.render(&container_id, cx).flex_1())
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
    title: SharedString,
    content: DataTable,
}
