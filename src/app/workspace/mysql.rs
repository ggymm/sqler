use gpui::prelude::*;
use gpui::*;

use gpui_component::{
    button::{Button, ButtonVariants as _},
    h_flex, v_flex, ActiveTheme as _, Sizable, StyledExt,
};

use crate::option::{DataSource, DataSourceKind, DataSourceOptions};

pub struct MySQLWorkspace {
    meta: DataSource,
    active_tab: WorkspaceTab,
    selected_table: Option<SharedString>,
}

impl MySQLWorkspace {
    pub fn new(meta: DataSource) -> Self {
        let selected_table = meta.tables().into_iter().next();
        Self {
            meta,
            active_tab: WorkspaceTab::Overview,
            selected_table,
        }
    }

    fn set_active_tab(
        &mut self,
        tab: WorkspaceTab,
        cx: &mut Context<Self>,
    ) {
        if self.active_tab != tab {
            self.active_tab = tab;
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
}

impl Render for MySQLWorkspace {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        debug_assert!(matches!(self.meta.kind, DataSourceKind::MySQL));

        let options = match &self.meta.options {
            DataSourceOptions::MySQL(opts) => opts,
            _ => panic!("MySQL workspace expects MySQL options"),
        };

        let theme = cx.theme();
        let tables = self.meta.tables();
        let has_selection = self
            .selected_table
            .as_ref()
            .map(|current| tables.iter().any(|name| name == current))
            .unwrap_or(false);
        if !has_selection {
            self.selected_table = tables.first().cloned();
        }

        let sidebar = {
            let mut column = v_flex()
                .w(px(240.))
                .py(px(12.))
                .gap(px(4.))
                .id("mysql-table-list")
                .overflow_scroll();
            {
                let style = column.style();
                style.min_size.width = Some(Length::Definite(px(200.).into()));
                style.max_size.width = Some(Length::Definite(px(280.).into()));
                style.min_size.height = Some(Length::Definite(px(0.).into()));
            }

            column = column.child(
                div()
                    .px(px(16.))
                    .pb(px(8.))
                    .text_sm()
                    .font_semibold()
                    .text_color(theme.muted_foreground)
                    .child("表列表"),
            );

            if tables.is_empty() {
                column.child(
                    div()
                        .px(px(16.))
                        .py(px(24.))
                        .text_sm()
                        .text_color(theme.muted_foreground)
                        .child("暂无可用表"),
                )
            } else {
                column.children(tables.into_iter().map(|table_name| {
                    let active = self.selected_table.as_ref() == Some(&table_name);
                    let table_label = table_name.clone();
                    let click_target = table_name.clone();
                    let item_id =
                        SharedString::from(format!("mysql-table-{}-{}", self.meta.id, table_name.to_string()));
                    let mut row = div()
                        .id(item_id)
                        .px(px(16.))
                        .py(px(8.))
                        .rounded(px(8.))
                        .cursor_pointer()
                        .text_sm()
                        .text_color(theme.muted_foreground)
                        .child(table_label)
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.select_table(click_target.clone(), cx);
                        }));

                    if active {
                        row = row.bg(theme.secondary).text_color(theme.foreground).font_semibold();
                    } else {
                        row = row.hover(|this| this.bg(theme.secondary_hover));
                    }

                    row.into_any_element()
                }))
            }
        };

        let tab_bar = h_flex()
            .gap(px(8.))
            .children(WorkspaceTab::ALL.iter().copied().map(|tab| {
                let id = SharedString::from(format!("mysql-tab-{}-{}", self.meta.id, tab.label()));
                let is_active = self.active_tab == tab;

                let mut button = Button::new(id)
                    .ghost()
                    .small()
                    .tab_stop(false)
                    .label(tab.label())
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.set_active_tab(tab, cx);
                    }));

                if is_active {
                    button = button.primary();
                }

                button.into_any_element()
            }));

        let content_body = match self.active_tab {
            WorkspaceTab::Overview => {
                let mut panel = v_flex()
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
            WorkspaceTab::Structure => {
                if let Some(table) = self.selected_table.clone() {
                    v_flex()
                        .gap(px(8.))
                        .child(div().text_base().font_semibold().child(format!("表结构：{}", table)))
                        .child(
                            div()
                                .text_sm()
                                .text_color(theme.muted_foreground)
                                .child("字段信息与索引占位，后续对接信息架构查询。"),
                        )
                } else {
                    v_flex().gap(px(8.)).child(
                        div()
                            .text_sm()
                            .text_color(theme.muted_foreground)
                            .child("左侧未找到可选表，无法加载结构信息。"),
                    )
                }
            }
            WorkspaceTab::Query => v_flex()
                .gap(px(8.))
                .child(div().text_base().font_semibold().child("查询工作台"))
                .child(
                    div()
                        .text_sm()
                        .text_color(theme.muted_foreground)
                        .child("SQL 编辑器与执行结果将在此渲染，当前为占位视图。"),
                ),
        };

        let mut content = v_flex()
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

        v_flex()
            .flex_1()
            .size_full()
            .min_w_0()
            .min_h_0()
            .child(
                h_flex()
                    .justify_between()
                    .items_center()
                    .px(px(16.))
                    .py(px(12.))
                    .border_b_1()
                    .border_color(theme.border)
                    .child(
                        v_flex()
                            .gap(px(4.))
                            .child(
                                div()
                                    .text_base()
                                    .font_semibold()
                                    .child(format!("MySQL · {}", self.meta.name)),
                            )
                            .child(div().text_sm().text_color(theme.muted_foreground).child(format!(
                                "主机 {}:{} · 数据库 {}",
                                options.host, options.port, options.database
                            ))),
                    )
                    .child(
                        h_flex()
                            .gap(px(8.))
                            .child(
                                Button::new(SharedString::from(format!("mysql-refresh-metadata-{}", self.meta.id)))
                                    .outline()
                                    .label("刷新元数据"),
                            )
                            .child(
                                Button::new(SharedString::from(format!("mysql-open-session-{}", self.meta.id)))
                                    .outline()
                                    .label("新建查询"),
                            )
                            .child(
                                Button::new(SharedString::from(format!("mysql-test-connection-{}", self.meta.id)))
                                    .ghost()
                                    .label("测试连接"),
                            ),
                    ),
            )
            .child(
                h_flex().flex_1().min_h_0().child(sidebar).child(
                    v_flex()
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum WorkspaceTab {
    Overview,
    Structure,
    Query,
}

impl WorkspaceTab {
    const ALL: [WorkspaceTab; 3] = [WorkspaceTab::Overview, WorkspaceTab::Structure, WorkspaceTab::Query];

    fn label(&self) -> &'static str {
        match self {
            WorkspaceTab::Overview => "概览",
            WorkspaceTab::Structure => "结构",
            WorkspaceTab::Query => "查询",
        }
    }
}
