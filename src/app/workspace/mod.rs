use std::sync::Arc;

use gpui::{prelude::*, *};
use gpui_component::{
    input::{CompletionProvider, InputState},
    ActiveTheme, InteractiveElementExt, Rope, StyledExt,
};
use lsp_types::{CompletionContext, CompletionItem, CompletionResponse};

use crate::{
    app::{SqlerApp, TabView},
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

#[derive(Clone)]
pub struct EditorComps {
    items: Arc<Vec<CompletionItem>>,
}

impl EditorComps {
    pub fn new() -> Self {
        let buf = include_bytes!("keywords.json");
        let items = serde_json::from_slice::<Vec<CompletionItem>>(buf).unwrap();

        Self { items: Arc::new(items) }
    }
}

impl CompletionProvider for EditorComps {
    fn completions(
        &self,
        _: &Rope,
        _: usize,
        context: CompletionContext,
        _: &mut Window,
        cx: &mut Context<InputState>,
    ) -> Task<Result<CompletionResponse>> {
        let chars = context.trigger_character.unwrap_or_default();
        if chars.is_empty() {
            return Task::ready(Ok(CompletionResponse::Array(vec![])));
        }

        let items = self.items.clone();
        cx.background_spawn(async move {
            let items = items
                .iter()
                .filter(|item| item.label.starts_with(&chars))
                .take(10)
                .map(|item| {
                    let mut item = item.clone();
                    item.insert_text = Some(item.label.replace(&chars, ""));
                    item
                })
                .collect::<Vec<_>>();

            Ok(CompletionResponse::Array(items))
        })
    }

    fn is_completion_trigger(
        &self,
        _: usize,
        _: &str,
        _: &mut Context<InputState>,
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
        meta: DataSource,
        _window: &mut Window,
        cx: &mut Context<SqlerApp>,
    ) -> Self {
        let parent = cx.weak_entity();
        match meta.kind {
            DataSourceKind::MySQL
            | DataSourceKind::SQLite
            | DataSourceKind::Postgres
            | DataSourceKind::Oracle
            | DataSourceKind::SQLServer => {
                let view = cx.new(|cx| CommonWorkspace::new(meta, parent.clone(), cx));
                WorkspaceState::Common { view }
            }
            DataSourceKind::Redis => {
                let view = cx.new(|cx| RedisWorkspace::new(meta, parent.clone(), cx));
                WorkspaceState::Redis { view }
            }
            DataSourceKind::MongoDB => {
                let view = cx.new(|cx| MongoDBWorkspace::new(meta, parent.clone(), cx));
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
                .on_double_click(cx.listener(move |this, _, window, cx| {
                    this.create_tab(&source.id, window, cx);
                }))
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
