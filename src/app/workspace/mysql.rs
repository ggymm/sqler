use gpui::prelude::*;
use gpui::*;

use crate::app::comps::{icon_export, icon_import, icon_search};
use crate::option::DataSource;
use crate::option::DataSourceKind;
use crate::option::DataSourceOptions;
use crate::option::MySQLOptions;
use gpui_component::button::Button;
use gpui_component::button::ButtonVariants;
use gpui_component::context_menu::ContextMenuExt;
use gpui_component::ActiveTheme;
use gpui_component::Icon;
use gpui_component::Sizable;
use gpui_component::Size;
use gpui_component::StyledExt;

pub struct MySQLWorkspace {
    meta: DataSource,
    selected_table: Option<SharedString>,
    tabs: Vec<TabItem>,
    active_tab: SharedString,
    next_tab_seq: u32,
}

impl MySQLWorkspace {
    pub fn new(meta: DataSource) -> Self {
        let selected_table = meta.tables().into_iter().next();
        let overview = TabItem::overview();

        Self {
            meta,
            selected_table,
            active_tab: overview.id.clone(),
            tabs: vec![overview],
            next_tab_seq: 1,
        }
    }

    fn set_active_tab(
        &mut self,
        tab_id: SharedString,
        cx: &mut Context<Self>,
    ) {
        if self.active_tab != tab_id {
            self.active_tab = tab_id;
            cx.notify();
        }
    }

    fn select_table(
        &mut self,
        table: SharedString,
        cx: &mut Context<Self>,
    ) {
        if self.selected_table.as_ref() != Some(&table) {
            self.selected_table = Some(table);
            cx.notify();
        }
    }

    fn ensure_default_tab(&mut self) {
        if !self.tabs.iter().any(|tab| !tab.closable) {
            let overview = TabItem::overview();
            self.tabs.insert(0, overview.clone());
            self.active_tab = overview.id;
        }
    }

    fn add_placeholder_tab(
        &mut self,
        cx: &mut Context<Self>,
    ) {
        let id = SharedString::from(format!("mysql-tab-{}-{}", self.meta.id, self.next_tab_seq));
        self.next_tab_seq += 1;
        let title = SharedString::from("自定义");
        self.tabs.push(TabItem {
            id: id.clone(),
            title,
            closable: true,
            content: TabContent::Placeholder,
        });
        self.active_tab = id;
        cx.notify();
    }

    fn close_tab(
        &mut self,
        tab_id: &SharedString,
        cx: &mut Context<Self>,
    ) {
        if let Some(index) = self.tabs.iter().position(|tab| &tab.id == tab_id && tab.closable) {
            let was_active = self.tabs[index].id == self.active_tab;
            self.tabs.remove(index);
            if was_active {
                if let Some(tab) = self.tabs.get(index.min(self.tabs.len().saturating_sub(1))) {
                    self.active_tab = tab.id.clone();
                } else {
                    self.ensure_default_tab();
                }
            }
            cx.notify();
        }
    }

    fn active_content(&self) -> TabContent {
        self.tabs
            .iter()
            .find(|tab| tab.id == self.active_tab)
            .map(|tab| tab.content.clone())
            .unwrap_or(TabContent::Overview)
    }

    fn render_sidebar(
        &mut self,
        cx: &mut Context<Self>,
        theme: &gpui_component::Theme,
        tables: Vec<SharedString>,
    ) -> AnyElement {
        div()
            .flex()
            .flex_col()
            .size_full()
            .w_64()
            .min_w_0()
            .min_h_0()
            .border_r_1()
            .border_color(theme.border)
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .px(px(12.))
                    .py(px(10.))
                    .child(
                        div()
                            .text_sm()
                            .font_semibold()
                            .text_color(theme.muted_foreground)
                            .child("数据表"),
                    )
                    .child(Button::new("table-reload").label("刷新")),
            )
            .child(
                tables.iter().cloned().fold(
                    div()
                        .flex()
                        .flex_col()
                        .min_w_0()
                        .min_h_0()
                        .px_2()
                        .gap_2()
                        .scrollable(Axis::Vertical),
                    |acc, table| {
                        let selected = self.selected_table.as_ref() == Some(&table);
                        let click_table = table.clone();
                        let entry_id = SharedString::from(format!("mysql-table-entry-{}-{}", self.meta.id, table));

                        acc.child(
                            div()
                                .flex()
                                .id(entry_id.clone())
                                .px(px(12.))
                                .py(px(8.))
                                .rounded(px(6.))
                                .text_sm()
                                .text_color(theme.muted_foreground)
                                .cursor_pointer()
                                .when(selected, |this| {
                                    this.bg(theme.secondary_hover)
                                        .text_color(theme.foreground)
                                        .font_semibold()
                                })
                                .when(!selected, |this| this.hover(|this| this.bg(theme.secondary_hover)))
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.select_table(click_table.clone(), cx);
                                }))
                                .child(table.clone()),
                        )
                    },
                ),
            )
            .into_any_element()
    }

    fn render_tab_bar(
        &mut self,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let mut bar = div().flex().flex_row().gap(px(10.));

        for tab in &self.tabs {
            let tab_id = tab.id.clone();
            let mut button = Button::new(SharedString::from(format!("mysql-tab-btn-{}", tab_id)))
                .ghost()
                .small()
                .tab_stop(false)
                .icon(Icon::default().path("icons/tab.svg").with_size(Size::Small))
                .label(tab.title.clone())
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.set_active_tab(tab_id.clone(), cx);
                }));

            if self.active_tab == tab.id {
                button = button.primary();
            }

            bar = bar.child(button);

            if tab.closable {
                let close_id = tab.id.clone();
                bar = bar.child(
                    Button::new(SharedString::from(format!("mysql-tab-close-{}", close_id)))
                        .ghost()
                        .xsmall()
                        .tab_stop(false)
                        .icon(Icon::default().path("icons/close.svg").with_size(Size::XSmall))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.close_tab(&close_id, cx);
                        }))
                        .into_any_element(),
                );
            }
        }

        bar.child(
            Button::new(SharedString::from(format!("mysql-tab-add-{}", self.meta.id)))
                .ghost()
                .tab_stop(false)
                .icon(Icon::default().path("icons/add.svg").with_size(Size::Small))
                .on_click(cx.listener(|this, _, _, cx| {
                    this.add_placeholder_tab(cx);
                })),
        )
        .into_any_element()
    }

    fn render_overview(
        &self,
        theme: &gpui_component::Theme,
        options: &MySQLOptions,
    ) -> Div {
        let mut panel = div()
            .flex()
            .flex_col()
            .gap(px(12.))
            .child(div().text_lg().font_semibold().child(self.meta.name.clone()))
            .child(
                div()
                    .text_sm()
                    .text_color(theme.muted_foreground)
                    .child(format!("描述：{}", self.meta.desc)),
            )
            .child(div().text_sm().text_color(theme.muted_foreground).child(format!(
                "连接：{}@{}:{} / {}",
                options.username, options.host, options.port, options.database
            )));

        let tables = self.meta.tables();
        if !tables.is_empty() {
            let preview = tables
                .iter()
                .take(3)
                .map(|t| t.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            panel = panel.child(
                div()
                    .text_sm()
                    .text_color(theme.muted_foreground)
                    .child(format!("示例表：{}", preview)),
            );
        }

        panel.child(
            div()
                .text_sm()
                .text_color(theme.muted_foreground)
                .child("MySQL 工作区规划包含连接池与慢查询分析面板。"),
        )
    }
}

impl Render for MySQLWorkspace {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        debug_assert!(matches!(self.meta.kind, DataSourceKind::MySQL));

        self.ensure_default_tab();

        let options = match &self.meta.options {
            DataSourceOptions::MySQL(opts) => opts.clone(),
            _ => panic!("MySQL workspace expects MySQL options"),
        };

        let theme = cx.theme().clone();
        let tables = self.meta.tables();
        let has_selection = self
            .selected_table
            .as_ref()
            .map(|current| tables.iter().any(|name| name == current))
            .unwrap_or(false);
        if !has_selection {
            self.selected_table = tables.first().cloned();
        }

        let sidebar = self.render_sidebar(cx, &theme, tables);
        let tab_bar = self.render_tab_bar(cx);

        let content_body = match self.active_content() {
            TabContent::Overview => self.render_overview(&theme, &options),
            TabContent::Placeholder => div()
                .flex()
                .flex_col()
                .gap(px(8.))
                .child(div().text_base().font_semibold().child("自定义视图"))
                .child(
                    div()
                        .text_sm()
                        .text_color(theme.muted_foreground)
                        .child("在这里扩展你的分析组件。"),
                ),
        };

        let mut content = div()
            .flex()
            .flex_col()
            .flex_1()
            .min_w_0()
            .min_h_0()
            .border_1()
            .border_color(theme.border)
            .rounded(px(12.))
            .p(px(16.))
            .child(content_body);
        {
            let style = content.style();
            style.overflow.x = Some(Overflow::Scroll);
            style.overflow.y = Some(Overflow::Scroll);
        }

        div()
            .flex()
            .flex_col()
            .flex_1()
            .size_full()
            .min_w_0()
            .min_h_0()
            .child(
                div()
                    .flex()
                    .flex_row()
                    .px_4()
                    .py_4()
                    .gap_2()
                    .border_b_1()
                    .border_color(theme.border)
                    .child(
                        Button::new("query")
                            .outline()
                            .icon(icon_search().with_size(Size::Small))
                            .label("新建查询"),
                    )
                    .child(
                        Button::new("import")
                            .outline()
                            .icon(icon_import().with_size(Size::Small))
                            .label("数据导入"),
                    )
                    .child(
                        Button::new("export")
                            .outline()
                            .icon(icon_export().with_size(Size::Small))
                            .label("数据导出"),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_1()
                    .min_w_0()
                    .min_h_0()
                    .child(sidebar)
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .flex_1()
                            .min_w_0()
                            .min_h_0()
                            .gap(px(12.))
                            .px(px(16.))
                            .py(px(12.))
                            .child(tab_bar)
                            .child(content),
                    ),
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
    Placeholder,
}
