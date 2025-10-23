use gpui::prelude::*;
use gpui::*;
use gpui_component::button::Button;
use gpui_component::button::ButtonVariants;
use gpui_component::resizable::h_resizable;
use gpui_component::resizable::resizable_panel;
use gpui_component::resizable::ResizableState;
use gpui_component::scroll::Scrollable;
use gpui_component::tab::Tab;
use gpui_component::tab::TabBar;
use gpui_component::ActiveTheme;
use gpui_component::Icon;
use gpui_component::InteractiveElementExt;
use gpui_component::Sizable;
use gpui_component::Size;
use gpui_component::StyledExt;

use crate::app::comps::comp_id;
use crate::app::comps::icon_export;
use crate::app::comps::icon_import;
use crate::app::comps::icon_search;
use crate::option::DataSource;
use crate::option::DataSourceKind;
use crate::option::DataSourceOptions;

pub struct MySQLWorkspace {
    sidebar_resize: Entity<ResizableState>,

    meta: DataSource,
    selected_table: Option<SharedString>,
    tabs: Vec<TabItem>,
    active_tab: SharedString,
}

impl MySQLWorkspace {
    pub fn new(
        meta: DataSource,
        cx: &mut Context<Self>,
    ) -> Self {
        let selected_table = meta.tables().into_iter().next();

        let active = SharedString::from("mysql-tab-overview");
        Self {
            sidebar_resize: ResizableState::new(cx),
            meta,
            selected_table,
            active_tab: active.clone(),
            tabs: vec![
                TabItem {
                    id: active,
                    title: SharedString::from("概览"),
                    closable: false,
                    content: TabContent::Overview,
                },
                TabItem {
                    id: SharedString::from("mysql-tab-overview1"),
                    title: SharedString::from("概览1"),
                    closable: true,
                    content: TabContent::Overview,
                },
                TabItem {
                    id: SharedString::from("mysql-tab-overview2"),
                    title: SharedString::from("概览2"),
                    closable: true,
                    content: TabContent::Overview,
                },
                TabItem {
                    id: SharedString::from("mysql-tab-overview3"),
                    title: SharedString::from("概览3"),
                    closable: true,
                    content: TabContent::Overview,
                },
            ],
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

    fn render_overview(
        &self,
        cx: &mut Context<Self>,
    ) -> Scrollable<Div> {
        let theme = cx.theme();
        let options = match &self.meta.options {
            DataSourceOptions::MySQL(opts) => opts.clone(),
            _ => panic!("MySQL workspace expects MySQL options"),
        };
        div()
            .flex()
            .flex_col()
            .flex_1()
            .size_full()
            .min_w_0()
            .min_h_0()
            .gap(px(12.))
            .scrollable(Axis::Vertical)
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
            )))
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

        let theme = cx.theme().clone();
        let id = &self.meta.id;
        let tables = self.meta.tables();
        let has_selection = self
            .selected_table
            .as_ref()
            .map(|current| tables.iter().any(|name| name == current))
            .unwrap_or(false);
        if !has_selection {
            self.selected_table = tables.first().cloned();
        }

        let selected_index = self.tabs.iter().position(|tab| tab.id == self.active_tab).unwrap_or(0);

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
                        .on_double_click(cx.listener(move |this, _, _, cx| {
                            this.select_table(click_table.clone(), cx);
                        }))
                        .child(table.clone()),
                )
            },
        );

        let tabs = TabBar::new(comp_id(["mysql-main-tabs", id]))
            .with_size(Size::Large)
            .children(
                self.tabs
                    .iter()
                    .enumerate()
                    .map(|(_, tab)| {
                        let tab_id = tab.id.clone();
                        Tab::new(tab.title.clone())
                            .id(comp_id(["mysql-main-tab-item", id, &tab_id]))
                            .with_size(Size::Small)
                            .when(tab.closable, |this| {
                                this.suffix(
                                    Button::new(comp_id(["mysql-main-tab-close", &tab_id]))
                                        .ghost()
                                        .xsmall()
                                        .tab_stop(false)
                                        .icon(Icon::default().path("icons/close.svg").with_size(Size::XSmall))
                                        .on_click(cx.listener(move |view: &mut Self, _, _, cx| {
                                            view.close_tab(&tab_id, cx);
                                        }))
                                        .into_any_element(),
                                )
                            })
                            .on_click(cx.listener({
                                let tab_id = tab.id.clone();
                                move |view: &mut Self, _, _, cx| {
                                    view.set_active_tab(tab_id.clone(), cx);
                                }
                            }))
                    })
                    .collect::<Vec<_>>(),
            )
            .selected_index(selected_index);

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
                                            .child(match self.active_content() {
                                                TabContent::Overview => self.render_overview(cx),
                                                TabContent::Placeholder => div()
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
                                                    ),
                                            }),
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
    Placeholder,
}
