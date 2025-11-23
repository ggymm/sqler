use std::sync::Arc;
use std::time::Duration;

use gpui::{prelude::*, *};
use gpui_component::{
    input::{CompletionProvider, InputState},
    menu::{ContextMenuExt, PopupMenuItem},
    ActiveTheme, InteractiveElementExt, Rope, StyledExt,
};
use lsp_types::{CompletionContext, CompletionItem, CompletionResponse, CompletionTextEdit, Position, Range, TextEdit};

use crate::{
    app::{SqlerApp, TabView, WindowKind},
    model::{DataSource, DataSourceKind},
};

use common::CommonWorkspace;
use mongodb::MongoDBWorkspace;
use redis::RedisWorkspace;

mod common;
mod mongodb;
mod redis;

pub fn parse_count(value: &str) -> usize {
    value.parse::<usize>().unwrap_or(0)
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
                    // 使用 text_edit 明确指定替换范围
                    let range = Range {
                        start: start_pos,
                        end: end_pos,
                    };
                    let edit = TextEdit {
                        range,
                        new_text: item.label.clone(),
                    };
                    item.text_edit = Some(CompletionTextEdit::Edit(edit));
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

pub enum WorkspaceState {
    Common { view: Entity<CommonWorkspace> },
    Redis { view: Entity<RedisWorkspace> },
    MongoDB { view: Entity<MongoDBWorkspace> },
}

impl WorkspaceState {
    pub fn new(
        source: DataSource,
        _window: &mut Window,
        cx: &mut Context<SqlerApp>,
    ) -> Self {
        let parent = cx.weak_entity();
        match source.kind {
            DataSourceKind::MySQL
            | DataSourceKind::SQLite
            | DataSourceKind::Postgres
            | DataSourceKind::Oracle
            | DataSourceKind::SQLServer => {
                let view = cx.new(|cx| CommonWorkspace::new(source, parent.clone(), cx));
                WorkspaceState::Common { view }
            }
            DataSourceKind::Redis => {
                let view = cx.new(|cx| RedisWorkspace::new(source, parent.clone(), cx));
                WorkspaceState::Redis { view }
            }
            DataSourceKind::MongoDB => {
                let view = cx.new(|cx| MongoDBWorkspace::new(source, parent.clone(), cx));
                WorkspaceState::MongoDB { view }
            }
        }
    }

    pub fn render(&self) -> AnyElement {
        match self {
            WorkspaceState::Common { view } => view.clone().into_any_element(),
            WorkspaceState::Redis { view } => view.clone().into_any_element(),
            WorkspaceState::MongoDB { view } => view.clone().into_any_element(),
        }
    }
}

pub fn render(
    app: &mut SqlerApp,
    _window: &mut Window,
    cx: &mut Context<SqlerApp>,
) -> AnyElement {
    if let Some(tab) = app.tabs.iter_mut().find(|tab| tab.id == app.active_tab) {
        match &mut tab.view {
            TabView::Home => render_home(app, cx),
            TabView::Workspace(state) => state.render(),
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

    div()
        .id("sources")
        .grid()
        .grid_cols(4)
        .size_full()
        .p_4()
        .gap_4()
        .min_w_0()
        .min_h_0()
        .scrollable(Axis::Vertical)
        .children(app.cache.sources().iter().cloned().map(|source| {
            let display = source.display_endpoint();

            let source = source.clone();
            let source_id = source.id.clone();

            div()
                .flex()
                .flex_1()
                .flex_col()
                .p_5()
                .gap_2()
                .min_w_64()
                .rounded_lg()
                .bg(theme.secondary)
                .border_1()
                .border_color(theme.border)
                .cursor_pointer()
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
                                .child(img(source.kind.image()).size_full().rounded_lg()),
                        ),
                )
                .child(
                    div()
                        .text_sm()
                        .overflow_hidden()
                        .whitespace_nowrap()
                        .text_color(theme.muted_foreground)
                        .child(display),
                )
        }))
        .into_any_element()
}
