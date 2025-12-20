use std::{sync::Arc, time::Duration};

use gpui::{prelude::*, *};
use gpui_component::{
    ActiveTheme, InteractiveElementExt, Rope, StyledExt,
    input::{CompletionProvider, InputState},
    menu::{ContextMenuExt, PopupMenuItem},
};
use indexmap::IndexMap;
use lsp_types::{CompletionContext, CompletionItem, CompletionResponse, CompletionTextEdit, Position, Range, TextEdit};

use sqler_core::{ArcCache, DataSource, DataSourceKind};

use crate::{
    app::{SqlerApp, TabContent, WindowKind},
    comps::DivExt,
};

mod common;
mod mongodb;
mod redis;

pub fn parse_elapsed(elapsed: f64) -> String {
    let ms = elapsed * 1000.0;

    if ms < 1.0 {
        "< 1ms".to_string()
    } else if ms < 1000.0 {
        format!("{}ms", ms.round() as u32)
    } else {
        let s = elapsed;
        if s < 60.0 {
            format!("{:.2}s", s)
        } else {
            let m = (s / 60.0).floor() as u32;
            let rem = s % 60.0;
            format!("{}m {}s", m, rem.round() as u32)
        }
    }
}

pub fn parse_position(
    text: &str,
    offset: usize,
) -> Position {
    let before = &text[..offset.min(text.len())];
    Position {
        line: before.chars().filter(|&c| c == '\n').count() as u32,
        character: before.rfind('\n').map(|i| offset - i - 1).unwrap_or(offset) as u32,
    }
}

#[derive(Clone)]
pub struct EditorComps {
    items: Arc<Vec<CompletionItem>>,
}

impl EditorComps {
    pub fn new() -> Self {
        let buf = include_bytes!("keywords.json");
        let mut items = serde_json::from_slice::<Vec<CompletionItem>>(buf).unwrap();

        // 按照 label 长度排序
        items.sort_by_key(|item| item.label.len());

        Self { items: Arc::new(items) }
    }
}

impl CompletionProvider for EditorComps {
    fn completions(
        &self,
        rope: &Rope,
        offset: usize,
        _trigger: CompletionContext,
        _window: &mut Window,
        cx: &mut Context<InputState>,
    ) -> Task<Result<CompletionResponse>> {
        let full_text = {
            let text = rope.to_string();
            text[..offset.min(text.len())].to_string()
        };

        let word_start = full_text
            .rfind(|c: char| c.is_whitespace() || "(),;".contains(c))
            .map(|i| i + 1)
            .unwrap_or(0);

        if full_text[word_start..].is_empty() {
            return Task::ready(Ok(CompletionResponse::Array(vec![])));
        }
        let word = full_text[word_start..].to_string();
        let word_upper = word.to_ascii_uppercase();

        let start_pos = parse_position(&full_text, word_start);
        let end_pos = parse_position(&full_text, full_text.len());

        let items = self.items.clone();
        cx.background_spawn(async move {
            Timer::after(Duration::from_millis(20)).await;

            let items = items
                .iter()
                .filter(|item| {
                    // rustfmt::skip
                    item.label.starts_with(&word) || item.label.starts_with(&word_upper)
                })
                .take(10)
                .map(|item| {
                    let mut item = item.clone();
                    item.text_edit = Some(CompletionTextEdit::Edit(TextEdit {
                        range: Range {
                            start: start_pos,
                            end: end_pos,
                        },
                        new_text: item.label.clone(),
                    }));
                    item.filter_text = Some(word.clone());
                    item
                })
                .collect::<Vec<_>>();

            Ok(CompletionResponse::Array(items))
        })
    }

    fn is_completion_trigger(
        &self,
        _offset: usize,
        _new_text: &str,
        _cx: &mut Context<InputState>,
    ) -> bool {
        true
    }
}

pub enum Workspace {
    Common { view: Entity<common::CommonWorkspace> },
    Redis { view: Entity<redis::RedisWorkspace> },
    MongoDB { view: Entity<mongodb::MongoDBWorkspace> },
}

impl Workspace {
    pub fn new(
        cache: ArcCache,
        source: DataSource,
        cx: &mut Context<SqlerApp>,
    ) -> Self {
        let parent = cx.weak_entity();

        let mut tabs = IndexMap::new();
        match source.kind {
            DataSourceKind::MySQL
            | DataSourceKind::SQLite
            | DataSourceKind::Postgres
            | DataSourceKind::Oracle
            | DataSourceKind::SQLServer => {
                let tables = {
                    cache
                        .read()
                        .unwrap()
                        .tables(&source.id)
                        .unwrap_or_default()
                        .into_iter()
                        .map(|t| (t.name.clone(), t))
                        .collect()
                };
                Workspace::Common {
                    view: cx.new(|_| {
                        let active_tab = SharedString::from("overview");
                        tabs.insert(active_tab.clone(), common::TabContext::overview());
                        common::CommonWorkspace {
                            cache,
                            parent,

                            source,
                            session: None,

                            tabs,
                            active_tab,
                            tables,
                            active_table: None,
                        }
                    }),
                }
            }
            DataSourceKind::Redis => Workspace::Redis {
                view: cx.new(|_| redis::RedisWorkspace {
                    parent,

                    source,
                    session: None,

                    active: redis::ViewType::Overview,
                    browse: None,
                    command: None,
                    overview: Some(redis::OverviewContent {}),
                }),
            },
            DataSourceKind::MongoDB => Workspace::MongoDB {
                view: cx.new(|_| mongodb::MongoDBWorkspace {
                    parent,

                    source,
                    session: None,

                    tabs: vec![mongodb::TabItem::overview()],
                    active_tab: SharedString::from(""),
                    collections: vec![],
                    active_collection: None,
                }),
            },
        }
    }

    pub fn render(&self) -> AnyElement {
        match self {
            Workspace::Common { view } => view.clone().into_any_element(),
            Workspace::Redis { view } => view.clone().into_any_element(),
            Workspace::MongoDB { view } => view.clone().into_any_element(),
        }
    }
}

pub fn render(
    app: &mut SqlerApp,
    _window: &mut Window,
    cx: &mut Context<SqlerApp>,
) -> AnyElement {
    if let Some(tab) = app.tabs.get_mut(&app.active_tab) {
        match &mut tab.content {
            TabContent::Home => render_home(app, cx),
            TabContent::Workspace(state) => state.render(),
        }
    } else {
        div().child("未找到可渲染的标签页").into_any_element()
    }
}

pub fn render_home(
    app: &SqlerApp,
    cx: &Context<SqlerApp>,
) -> AnyElement {
    let theme = cx.theme();
    let entity = cx.entity();

    let mut cards = vec![];
    for source in {
        let cache = app.cache.read().unwrap();
        cache.sources().iter().cloned().collect::<Vec<_>>()
    } {
        let display = source.display_endpoint();

        let source = source.clone();
        let source_id = source.id.clone();

        let item = div()
            .flex()
            .flex_1()
            .flex_col()
            .p_5()
            .gap_2()
            .min_w_64()
            .rounded_md()
            .bg(theme.secondary)
            .border_1()
            .border_color(theme.border)
            .id(SharedString::from(format!("source-card-{}", source.id)))
            .hover(|this| this.bg(theme.secondary_hover))
            .on_double_click(cx.listener({
                let source_id = source_id.clone();
                move |this, _, window, cx| {
                    this.create_tab(&source_id, window, cx);
                }
            }))
            .context_menu({
                let entity = entity.clone();
                let source = source.clone();
                move |this, window, _cx| {
                    this.item(PopupMenuItem::new("编辑").on_click({
                        let source = source.clone();
                        window.listener_for(&entity, move |this, _, _, cx| {
                            this.create_window(WindowKind::Create(Some(source.clone())), cx);
                        })
                    }))
                    .separator()
                    .item(PopupMenuItem::new("删除"))
                }
            })
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .flex_1()
                            .font_semibold()
                            .text_color(theme.foreground)
                            .child(source.name),
                    )
                    .child(
                        div()
                            .w_8()
                            .h_8()
                            .child(img(source.kind.image()).size_full().rounded_md()),
                    ),
            )
            .child(
                div()
                    .text_sm()
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .text_color(theme.muted_foreground)
                    .child(display),
            );

        cards.push(item);
    }

    div()
        .col_full()
        .scrollbar_y()
        .child(div().grid().grid_cols(4).p_4().gap_4().children(cards))
        .into_any_element()
}
