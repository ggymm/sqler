use gpui::prelude::*;
use gpui::*;

use gpui_component::ActiveTheme;
use gpui_component::InteractiveElementExt;
use gpui_component::StyledExt;

use crate::app::SqlerApp;
use crate::app::TabView;
use crate::option::DataSource;
use crate::option::DataSourceKind;

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
        .size_full()
        .p_5()
        .gap_4()
        .min_w_0()
        .min_h_0()
        .scrollable(Axis::Vertical)
        .children(app.sources.iter().cloned().map(|meta| {
            let id = meta.id.clone();
            let name = meta.name.clone();

            div()
                .flex()
                .flex_col()
                .w(px(220.))
                .p_5()
                .gap_2()
                .rounded_lg()
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
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(8.))
                        .child(div().w_8().h_8().rounded_lg().child(img(meta.kind.image()).size_full()))
                        .child(
                            div()
                                .flex_1()
                                .text_base()
                                .font_semibold()
                                .text_color(theme.foreground)
                                .child(name),
                        ),
                )
                .child(div().text_sm().text_color(theme.muted_foreground).child(meta.name))
                .child(div().text_sm().text_color(theme.muted_foreground).child(meta.desc))
        }))
        .into_any_element()
}