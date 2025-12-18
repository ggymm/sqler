use gpui::{prelude::*, *};
use gpui_component::{
    ActiveTheme, Sizable, StyledExt,
    button::{Button, ButtonVariants},
    resizable::{h_resizable, resizable_panel},
};

use crate::{
    app::{
        SqlerApp,
        comps::{AppIcon, DivExt, comp_id},
    },
    driver::{DatabaseSession, DriverError, create_connection},
    model::DataSource,
};

const PAGE_SIZE: usize = 100;

pub struct TabItem {
    id: SharedString,
    title: SharedString,
    content: TabContent,
    closable: bool,
}

impl TabItem {
    pub fn overview() -> Self {
        Self {
            id: SharedString::from("mongodb-overview-tab"),
            title: SharedString::from("概览"),
            content: TabContent::Overview,
            closable: false,
        }
    }
}

enum TabContent {
    Overview,
}

pub struct MongoDBWorkspace {
    pub source: DataSource,
    pub parent: WeakEntity<SqlerApp>,
    pub session: Option<Box<dyn DatabaseSession>>,

    pub tabs: Vec<TabItem>,
    pub active_tab: SharedString,
    pub collections: Vec<SharedString>,
    pub active_collection: Option<SharedString>,
}

impl MongoDBWorkspace {
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
                    self.active_collection = Some(tab.title.clone());
                } else {
                    self.active_collection = None;
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
        self.active_collection = Some(title);
        cx.notify();
    }

    fn active_session(&mut self) -> Result<&mut (dyn DatabaseSession + '_), DriverError> {
        if self.session.is_none() {
            self.session = Some(create_connection(&self.source.options)?);
        }

        match self.session.as_deref_mut() {
            Some(session) => Ok(session),
            None => Err(DriverError::Other("MongoDB 连接不可用".into())),
        }
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
            .scrollbar_y()
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

impl Render for MongoDBWorkspace {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let id = &self.source.id;
        let theme = cx.theme().clone();
        let active = &self.active_tab;

        let sidebar = self.collections.iter().cloned().fold(
            div()
                .id(comp_id(["mongodb-sidebar", id]))
                .p_2()
                .gap_2()
                .col_full()
                .scrollbar_y(),
            |acc, collection| {
                let active = self.active_collection.as_ref() == Some(&collection);
                acc.child(
                    div()
                        .id(comp_id(["mongodb-sidebar-item", &self.source.id, &collection]))
                        .px_4()
                        .py_2()
                        .gap_2()
                        .row_full()
                        .items_center()
                        .text_sm()
                        .rounded_md()
                        .when_else(
                            active,
                            |this| this.bg(theme.list_active).font_semibold(),
                            |this| this.hover(|this| this.bg(theme.list_hover)),
                        )
                        .child(AppIcon::Sheet)
                        .child(collection.clone()),
                )
            },
        );

        let container = div()
            .p_2()
            .gap_2()
            .col_full()
            .child(
                div()
                    .id(comp_id(["mongodb-tabs", id]))
                    .flex()
                    .flex_row()
                    .gap_2()
                    .min_w_0()
                    .children(self.tabs.iter().map(|tab| {
                        let tab_id = tab.id.clone();
                        let tab_active = &tab_id == active;

                        let mut item = div()
                            .id(comp_id(["mongodb-tabs-item", &tab_id]))
                            .flex()
                            .flex_row()
                            .items_center()
                            .justify_center()
                            .px_2()
                            .py_1()
                            .gap_2()
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
                                Button::new(comp_id(["mongodb-tabs-close", &tab_id]))
                                    .ghost()
                                    .xsmall()
                                    .compact()
                                    .tab_stop(false)
                                    .icon(AppIcon::Close)
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
                    .id(comp_id(["mongodb-main", id]))
                    .col_full()
                    .child(
                        match self
                            .tabs
                            .iter()
                            .find(|tab| tab.id == self.active_tab)
                            .map(|tab| &tab.content)
                        {
                            Some(TabContent::Overview) | None => self.render_overview_tab(cx),
                        },
                    )
                    .child(div()),
            )
            .into_any_element();

        div().id(comp_id(["mongodb", id])).col_full().child(
            div().id(comp_id(["mongodb-content", id])).col_full().child(
                h_resizable(comp_id(["mongodb-content", id]))
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
