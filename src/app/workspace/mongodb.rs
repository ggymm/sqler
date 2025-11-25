use gpui::{prelude::*, *};
use gpui_component::{
    button::{Button, ButtonVariants},
    input::{Input, InputState},
    resizable::{h_resizable, resizable_panel},
    table::{Table, TableState},
    ActiveTheme, Disableable, InteractiveElementExt, Sizable, Size, StyledExt,
};

use crate::{
    app::{
        comps::{comp_id, icon_close, icon_relead, icon_search, icon_sheet, DataTable, DivExt},
        SqlerApp,
    },
    driver::{create_connection, DatabaseSession, DriverError},
    model::DataSource,
};

const PAGE_SIZE: usize = 100;

struct TabItem {
    id: SharedString,
    title: SharedString,
    content: TabContent,
    closable: bool,
}

impl TabItem {
    fn overview() -> Self {
        Self {
            id: SharedString::from("mongodb-overview-tab"),
            title: SharedString::from("概览"),
            content: TabContent::Overview,
            closable: false,
        }
    }
}

enum TabContent {
    Collection(CollectionContent),
    Overview,
}

struct CollectionContent {
    id: SharedString,
    collection: SharedString,
    filter_input: Entity<InputState>,
    content: Entity<TableState<DataTable>>,
    page_no: usize,
    page_size: usize,
    total_docs: usize,
}

pub struct MongoDBWorkspace {
    source: DataSource,
    parent: WeakEntity<SqlerApp>,
    session: Option<Box<dyn DatabaseSession>>,

    tabs: Vec<TabItem>,
    active_tab: SharedString,
    collections: Vec<SharedString>,
    active_collection: Option<SharedString>,
}

impl MongoDBWorkspace {
    pub fn new(
        source: DataSource,
        parent: WeakEntity<SqlerApp>,
        _cx: &mut Context<Self>,
    ) -> Self {
        let overview = TabItem::overview();
        let active_tab = overview.id.clone();

        Self {
            source,
            parent,
            session: None,

            tabs: vec![overview],
            active_tab,
            collections: vec![],
            active_collection: None,
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

    fn reload_collections(
        &mut self,
        cx: &mut Context<Self>,
    ) {
        let result = self.active_session().and_then(|session| session.tables());

        self.collections = match result {
            Ok(collections) => collections.into_iter().map(SharedString::from).collect(),
            Err(err) => {
                eprintln!("刷新集合列表失败: {}", err);
                if !self.collections.is_empty() {
                    return;
                }
                vec![]
            }
        };
        self.active_tab = TabItem::overview().id;
        self.active_collection = None;

        self.tabs.retain(|tab| match &tab.content {
            TabContent::Collection(tab) => self.collections.iter().any(|c| c == &tab.collection),
            _ => true,
        });

        cx.notify();
    }

    fn create_collection_tab(
        &mut self,
        collection: SharedString,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let id = SharedString::from(format!("mongodb-tab-collection-{}-{}", self.source.id, collection));

        if let Some(existing) = self.tabs.iter().find(|tab| {
            matches!(
                &tab.content,
                TabContent::Collection(current) if current.id == id
            )
        }) {
            self.active_tab = existing.id.clone();
            self.active_collection = Some(collection.clone());
            cx.notify();
            return;
        }

        let filter_input = cx.new(|cx| InputState::new(window, cx));
        let content = DataTable::new(vec![], Vec::new()).build(window, cx);

        self.tabs.push(TabItem {
            id: id.clone(),
            title: collection.clone(),
            content: TabContent::Collection(CollectionContent {
                id: id.clone(),
                collection: collection.clone(),
                filter_input,
                content,
                page_no: 0,
                page_size: PAGE_SIZE,
                total_docs: 0,
            }),
            closable: true,
        });

        self.active_tab = id.clone();
        self.active_collection = Some(collection.clone());
        cx.notify();
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

    fn render_collection_tab(
        &self,
        content: &CollectionContent,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let _theme = cx.theme().clone();
        let tab_id = content.id.clone();

        let total_pages = if content.total_docs == 0 {
            1
        } else {
            (content.total_docs + content.page_size - 1) / content.page_size
        };
        let current_page = content.page_no;
        let start_doc = current_page * content.page_size + 1;
        let end_doc = ((current_page + 1) * content.page_size).min(content.total_docs);

        let filter_btn = Button::new(comp_id(["mongodb-apply-filter", &tab_id]))
            .label("应用筛选")
            .outline();

        let page_prev_btn = Button::new(comp_id(["mongodb-page-prev", &tab_id]))
            .label("上一页")
            .outline()
            .disabled(current_page == 0);

        let page_next_btn = Button::new(comp_id(["mongodb-page-next", &tab_id]))
            .label("下一页")
            .outline()
            .disabled(current_page + 1 >= total_pages);

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
                    .child(div().flex_1().child(Input::new(&content.filter_input)))
                    .child(filter_btn),
            )
            .child(
                div()
                    .flex_1()
                    .rounded_md()
                    .overflow_hidden()
                    .child(
                        Table::new(&content.content)
                            .stripe(false)
                            .bordered(false)
                            .with_size(Size::Small)
                            .scrollbar_visible(true, true),
                    )
                    .child(div()),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .child(div().flex_1())
                    .child(div().text_sm().child(format!(
                        "显示 {} - {} / 共 {} 条",
                        if content.total_docs == 0 { 0 } else { start_doc },
                        end_doc,
                        content.total_docs
                    )))
                    .child(div().flex_1())
                    .child(page_prev_btn)
                    .child(page_next_btn),
            )
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
                .scrollable(Axis::Vertical),
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
                        .on_double_click(cx.listener({
                            let collection = collection.clone();
                            move |this, _, window, cx| {
                                this.create_collection_tab(collection.clone(), window, cx);
                            }
                        }))
                        .child(icon_sheet())
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
                                    .icon(icon_close().with_size(Size::Small))
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
                            Some(TabContent::Collection(content)) => self.render_collection_tab(content, cx),
                            Some(TabContent::Overview) | None => self.render_overview_tab(cx),
                        },
                    )
                    .child(div()),
            )
            .into_any_element();

        div()
            .id(comp_id(["mongodb", id]))
            .col_full()
            .child(
                div()
                    .id(comp_id(["mongodb-header", id]))
                    .flex()
                    .flex_row()
                    .p_4()
                    .gap_2()
                    .border_b_1()
                    .border_color(theme.border)
                    .child(
                        Button::new(comp_id(["mongodb-header-refresh", id]))
                            .icon(icon_relead().with_size(Size::Small))
                            .label("刷新集合")
                            .outline()
                            .on_click(cx.listener(|view: &mut Self, _, _, cx| {
                                view.reload_collections(cx);
                            })),
                    )
                    .child(
                        Button::new(comp_id(["mongodb-header-query", id]))
                            .icon(icon_search().with_size(Size::Small))
                            .label("新建查询")
                            .outline(),
                    ),
            )
            .child(
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
