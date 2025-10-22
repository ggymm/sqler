use gpui::prelude::*;
use gpui::*;

use gpui_component::{
    button::ButtonVariants as _, h_flex, v_flex, ActiveTheme as _, InteractiveElementExt as _, Sizable, StyledExt,
};

use crate::app::{SqlerApp, TabView};
use crate::option::{DataSource, DataSourceKind};

mod mysql;
mod placeholder;

use mysql::MySQLWorkspace;
use placeholder::PlaceholderWorkspace;

pub enum WorkspaceState {
    MySQL {
        id: String,
        view: Entity<MySQLWorkspace>,
    },
    Placeholder {
        id: String,
        view: Entity<PlaceholderWorkspace>,
    },
}

impl WorkspaceState {
    pub fn new(
        meta: DataSource,
        _window: &mut Window,
        cx: &mut Context<SqlerApp>,
    ) -> Self {
        match meta.kind {
            DataSourceKind::MySQL => {
                let id = meta.id.clone();
                let view = cx.new(|_| MySQLWorkspace::new(meta));
                WorkspaceState::MySQL { id, view }
            }
            other => {
                let label = other.label();
                let id = meta.id.clone();
                let view = cx.new(|_| {
                    let message = format!("{} 工作区暂未实现", label);
                    PlaceholderWorkspace::new(meta, message)
                });
                WorkspaceState::Placeholder { id, view }
            }
        }
    }

    pub fn id(&self) -> &str {
        match self {
            WorkspaceState::MySQL { id, .. } => id,
            WorkspaceState::Placeholder { id, .. } => id,
        }
    }

    pub fn render(&self) -> AnyElement {
        match self {
            WorkspaceState::MySQL { view, .. } => view.clone().into_any_element(),
            WorkspaceState::Placeholder { view, .. } => view.clone().into_any_element(),
        }
    }
}

pub fn render(
    app: &mut SqlerApp,
    window: &mut Window,
    cx: &mut Context<SqlerApp>,
) -> AnyElement {
    if let Some(tab) = app.tabs.iter_mut().find(|tab| tab.id == app.active_tab) {
        match &mut tab.view {
            TabView::Home => render_home(&app.saved_sources, window, cx).into_any_element(),
            TabView::Workspace(state) => state.render(),
        }
    } else {
        v_flex()
            .flex_1()
            .child(div().child("未找到可渲染的标签页"))
            .into_any_element()
    }
}

fn render_home(
    sources: &[DataSource],
    _window: &mut Window,
    cx: &mut Context<SqlerApp>,
) -> AnyElement {
    let theme = cx.theme().clone();

    div()
        .flex()
        .flex_col()
        .size_full()
        .min_w_0()
        .min_h_0()
        .child(
            div()
                .id("data-source-list")
                .flex()
                .flex_col()
                .p_5()
                .gap_4()
                .flex_1()
                .min_w_0()
                .min_h_0()
                .scrollable(Axis::Vertical)
                .child(
                    h_flex()
                        .flex_wrap()
                        .gap(px(12.))
                        .children(sources.iter().cloned().map(|meta| {
                            let id = meta.id.clone();
                            let name = meta.name.clone();

                            v_flex()
                                .w(px(220.))
                                .gap(px(8.))
                                .p(px(14.))
                                .rounded(px(8.))
                                .bg(theme.secondary)
                                .border_1()
                                .border_color(theme.border)
                                .cursor_pointer()
                                .id(SharedString::from(format!("source-card-{}", id)))
                                .hover(|this| this.bg(theme.secondary_hover))
                                .on_double_click(cx.listener(move |this, _, window, cx| {
                                    this.open_data_source_tab(&meta.id, window, cx);
                                }))
                                .child(
                                    h_flex()
                                        .items_center()
                                        .gap(px(8.))
                                        .child(
                                            div()
                                                .flex_shrink_0()
                                                .w(px(32.))
                                                .h(px(32.))
                                                .child(img(meta.kind.image()).size_full()),
                                        )
                                        .child(
                                            div()
                                                .flex_1()
                                                .text_base()
                                                .font_semibold()
                                                .text_color(theme.foreground)
                                                .child(name),
                                        ),
                                )
                                .child(div().text_sm().text_color(cx.theme().muted_foreground).child(meta.name))
                                .child(div().text_sm().text_color(cx.theme().muted_foreground).child(meta.desc))
                        })),
                ),
        )
        .into_any_element()
}
