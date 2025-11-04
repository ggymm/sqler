use gpui::{prelude::*, *};
use gpui_component::{ActiveTheme, InteractiveElementExt, StyledExt};

use crate::{
    app::{SqlerApp, TabView},
    driver::{DataSource, DataSourceKind},
};

mod mysql;
mod placeholder;

use mysql::MySQLWorkspace;
use placeholder::PlaceholderWorkspace;

pub fn parse_count(value: &str) -> usize {
    value.parse::<usize>().unwrap_or(0)
}

pub enum WorkspaceState {
    MySQL { view: Entity<MySQLWorkspace> },
    Placeholder { view: Entity<PlaceholderWorkspace> },
}

impl WorkspaceState {
    pub fn new(
        meta: DataSource,
        _window: &mut Window,
        cx: &mut Context<SqlerApp>,
    ) -> Self {
        let parent = cx.weak_entity();
        match meta.kind {
            DataSourceKind::MySQL => {
                let view = cx.new(|cx| MySQLWorkspace::new(meta, parent.clone(), cx));
                WorkspaceState::MySQL { view }
            }
            other => {
                let label = other.label();
                let view = cx.new(|_| {
                    let message = format!("{} 工作区暂未实现", label);
                    PlaceholderWorkspace::new(meta, message)
                });
                WorkspaceState::Placeholder { view }
            }
        }
    }

    pub fn render(&self) -> AnyElement {
        match self {
            WorkspaceState::MySQL { view } => view.clone().into_any_element(),
            WorkspaceState::Placeholder { view } => view.clone().into_any_element(),
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
        .children(app.sources.iter().cloned().map(|source| {
            let display = source.display();

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
