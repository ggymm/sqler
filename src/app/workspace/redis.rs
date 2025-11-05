use gpui::{prelude::*, *};
use gpui_component::{
    button::{Button, ButtonVariants},
    input::{InputState, TextInput},
    resizable::{h_resizable, resizable_panel, ResizableState},
    table::Table,
    ActiveTheme, Sizable, Size, StyledExt,
};

use crate::{
    app::{
        comps::{comp_id, icon_relead, icon_search, DataTable, DivExt},
        SqlerApp,
    },
    driver::{create_connection, DataSource, DatabaseSession, DriverError},
};

struct TabItem {
    id: SharedString,
    title: SharedString,
    content: TabContent,
    closable: bool,
}

impl TabItem {
    fn overview() -> Self {
        Self {
            id: SharedString::from("redis-overview-tab"),
            title: SharedString::from("概览"),
            content: TabContent::Overview,
            closable: false,
        }
    }
}

enum TabContent {
    Command(CommandContent),
    Overview,
}

struct CommandContent {
    id: SharedString,
    command_input: Entity<InputState>,
    result_table: Entity<Table<DataTable>>,
}

pub struct RedisWorkspace {
    meta: DataSource,
    parent: WeakEntity<SqlerApp>,
    session: Option<Box<dyn DatabaseSession>>,

    tabs: Vec<TabItem>,
    active_tab: SharedString,
    sidebar_resize: Entity<ResizableState>,
}

impl RedisWorkspace {
    pub fn new(
        meta: DataSource,
        parent: WeakEntity<SqlerApp>,
        cx: &mut Context<Self>,
    ) -> Self {
        let overview = TabItem::overview();
        let active_tab = overview.id.clone();

        Self {
            meta,
            parent,
            session: None,

            tabs: vec![overview],
            active_tab,
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
                }
            }
            cx.notify();
        }
    }

    fn active_tab(
        &mut self,
        id: SharedString,
        cx: &mut Context<Self>,
    ) {
        self.active_tab = id;
        cx.notify();
    }

    fn active_session(&mut self) -> Result<&mut (dyn DatabaseSession + '_), DriverError> {
        if self.session.is_none() {
            self.session = Some(create_connection(&self.meta.options)?);
        }

        match self.session.as_deref_mut() {
            Some(session) => Ok(session),
            None => Err(DriverError::Other("Redis 连接不可用".into())),
        }
    }

    fn create_command_tab(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let tab_id = SharedString::from(format!("redis-tab-command-{}", uuid::Uuid::new_v4()));
        let command_input = cx.new(|cx| InputState::new(window, cx));
        let result_table = DataTable::new(vec![], Vec::new()).build(window, cx);

        self.tabs.push(TabItem {
            id: tab_id.clone(),
            title: SharedString::from("命令执行"),
            content: TabContent::Command(CommandContent {
                id: tab_id.clone(),
                command_input,
                result_table,
            }),
            closable: true,
        });

        self.active_tab = tab_id;
        cx.notify();
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
            .child(
                div()
                    .text_base()
                    .text_color(theme.muted_foreground)
                    .child(format!("描述：{}", self.meta.desc)),
            )
            .child(detail_card)
            .into_any_element()
    }

    fn render_command_tab(
        &self,
        content: &CommandContent,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let theme = cx.theme().clone();
        let tab_id = content.id.clone();

        div()
            .flex()
            .flex_1()
            .flex_col()
            .gap_2()
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .child(div().flex_1().child(TextInput::new(&content.command_input)))
                    .child(
                        Button::new(comp_id(["redis-execute-command", &tab_id]))
                            .outline()
                            .label("执行")
                            .on_click(cx.listener({
                                let tab_id = tab_id.clone();
                                move |_view, _, _window, cx| {
                                    // TODO: 实现命令执行逻辑
                                    cx.notify();
                                }
                            })),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .rounded_lg()
                    .overflow_hidden()
                    .child(content.result_table.clone()),
            )
            .into_any_element()
    }
}

impl Render for RedisWorkspace {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let id = &self.meta.id;
        let theme = cx.theme().clone();
        let active = &self.active_tab;

        let sidebar = div()
            .id(comp_id(["redis-sidebar", id]))
            .p_2()
            .gap_2()
            .col_full()
            .scrollable(Axis::Vertical)
            .child(
                div()
                    .px_4()
                    .py_2()
                    .text_sm()
                    .text_color(theme.muted_foreground)
                    .child("Redis 键值存储"),
            );

        let container = div()
            .p_2()
            .gap_2()
            .col_full()
            .child(
                div()
                    .id(comp_id(["redis-tabs", id]))
                    .flex()
                    .flex_row()
                    .gap_2()
                    .min_w_0()
                    .children(self.tabs.iter().map(|tab| {
                        let tab_id = tab.id.clone();
                        let tab_active = &tab_id == active;

                        let mut item = div()
                            .id(comp_id(["redis-tabs-item", &tab_id]))
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
                                move |this, _, _, cx| {
                                    this.active_tab(tab_id.clone(), cx);
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
                                Button::new(comp_id(["redis-tabs-close", &tab_id]))
                                    .ghost()
                                    .xsmall()
                                    .compact()
                                    .tab_stop(false)
                                    .icon(crate::app::comps::icon_close().with_size(Size::Small))
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
                    .id(comp_id(["redis-main", id]))
                    .col_full()
                    .child(
                        match self
                            .tabs
                            .iter()
                            .find(|tab| tab.id == self.active_tab)
                            .map(|tab| &tab.content)
                        {
                            Some(TabContent::Command(content)) => self.render_command_tab(content, cx),
                            Some(TabContent::Overview) | None => self.render_overview_tab(cx),
                        },
                    )
                    .child(div()),
            )
            .into_any_element();

        div()
            .id(comp_id(["redis", id]))
            .col_full()
            .child(
                div()
                    .id(comp_id(["redis-header", id]))
                    .flex()
                    .flex_row()
                    .p_4()
                    .gap_2()
                    .border_b_1()
                    .border_color(theme.border)
                    .child(
                        Button::new(comp_id(["redis-header-refresh", id]))
                            .outline()
                            .icon(icon_relead().with_size(Size::Small))
                            .label("刷新连接"),
                    )
                    .child(
                        Button::new(comp_id(["redis-header-command", id]))
                            .outline()
                            .icon(icon_search().with_size(Size::Small))
                            .label("新建命令")
                            .on_click(cx.listener(|view: &mut Self, _, window, cx| {
                                view.create_command_tab(window, cx);
                            })),
                    ),
            )
            .child(
                div().id(comp_id(["redis-content", id])).col_full().child(
                    h_resizable(comp_id(["redis-content", id]), self.sidebar_resize.clone())
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
