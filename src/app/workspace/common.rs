use gpui::prelude::*;
use gpui::*;

use gpui_component::{
    button::{Button, ButtonVariants as _},
    form::{form_field, v_form},
    h_flex, v_flex, ActiveTheme as _, Disableable as _, Selectable as _, Sizable, StyledExt,
};

use crate::app::{DataSourceTabState, InnerTab, InnerTabId, SqlerApp, TabId};

pub struct WorkspaceContext {
    pub meta_fields: Vec<(&'static str, SharedString)>,
    pub summary: String,
    pub notes: Vec<String>,
}

impl WorkspaceContext {
    pub fn new(
        meta_fields: Vec<(&'static str, SharedString)>,
        summary: String,
        notes: Vec<String>,
    ) -> Self {
        Self {
            meta_fields,
            summary,
            notes,
        }
    }
}

pub fn render_workspace(
    tab_id: TabId,
    state: &DataSourceTabState,
    _window: &mut Window,
    cx: &mut Context<SqlerApp>,
    ctx: WorkspaceContext,
) -> gpui::Div {
    let mut table_list = v_flex()
        .px(px(12.))
        .py(px(8.))
        .gap(px(6.))
        .flex_1()
        .id("workspace-table-list")
        .overflow_scroll();
    {
        let style = table_list.style();
        style.min_size.height = Some(Length::Definite(px(0.).into()));
        style.min_size.width = Some(Length::Definite(px(0.).into()));
    }
    table_list = table_list.children(state.tables.iter().map(|table| {
        div()
            .px(px(10.))
            .py(px(6.))
            .rounded(px(4.))
            .hover(|this| this.bg(cx.theme().sidebar_accent))
            .child(table.clone())
    }));

    let mut left_panel = v_flex()
        .w(px(240.))
        .flex_shrink_0()
        .bg(cx.theme().sidebar)
        .border_r_1()
        .border_color(cx.theme().border)
        .child(
            div()
                .px(px(16.))
                .py(px(18.))
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("表列表"),
        )
        .child(table_list);
    {
        let style = left_panel.style();
        style.min_size.height = Some(Length::Definite(px(0.).into()));
        style.min_size.width = Some(Length::Definite(px(0.).into()));
    }

    let mut right_panel = v_flex().flex_1().size_full();
    {
        let style = right_panel.style();
        style.min_size.height = Some(Length::Definite(px(0.).into()));
        style.min_size.width = Some(Length::Definite(px(0.).into()));
    }

    let mut detail_panel = v_flex()
        .flex_1()
        .id(SharedString::from(format!("ds-detail-scroll-{}", tab_id.raw())))
        .overflow_scroll();
    {
        let style = detail_panel.style();
        style.min_size.height = Some(Length::Definite(px(0.).into()));
        style.min_size.width = Some(Length::Definite(px(0.).into()));
    }

    let detail_panel = detail_panel
        .px(px(24.))
        .py(px(20.))
        .child(build_detail_section(state, cx, &ctx));

    let right_panel = right_panel
        .child(inner_tab_bar(tab_id, &state.inner_tabs, state.active_inner_tab, cx))
        .child(detail_panel);

    let mut content_row = h_flex().flex_1().size_full();
    {
        let style = content_row.style();
        style.align_items = Some(AlignItems::Stretch);
        style.min_size.height = Some(Length::Definite(px(0.).into()));
        style.min_size.width = Some(Length::Definite(px(0.).into()));
    }
    let content_row = content_row.child(left_panel).child(right_panel);

    let mut root = v_flex().flex_1().size_full();
    {
        let style = root.style();
        style.min_size.height = Some(Length::Definite(px(0.).into()));
        style.min_size.width = Some(Length::Definite(px(0.).into()));
    }

    root.child(workspace_toolbar(tab_id, true, cx)).child(content_row)
}

fn build_detail_section(
    state: &DataSourceTabState,
    cx: &mut Context<SqlerApp>,
    ctx: &WorkspaceContext,
) -> gpui::Div {
    let meta = &state.meta;
    let mut config = v_form()
        .gap(px(12.))
        .child(form_field().label("名称").child(div().child(meta.name.clone())))
        .child(form_field().label("类型").child(div().child(meta.kind.label())));

    for (label, value) in &ctx.meta_fields {
        config = config.child(form_field().label(*label).child(div().child(value.clone())));
    }

    config = config.child(form_field().label("描述").child(div().child(meta.desc.clone())));

    let theme = cx.theme();
    let mut notes_block = v_flex()
        .gap(px(10.))
        .child(
            div()
                .text_base()
                .font_semibold()
                .child(format!("{} 工作区", meta.kind.label())),
        )
        .child(
            div()
                .text_sm()
                .text_color(theme.muted_foreground)
                .child(format!("连接：{}", ctx.summary.clone())),
        );

    for note in &ctx.notes {
        notes_block = notes_block.child(
            div()
                .text_sm()
                .text_color(theme.muted_foreground)
                .child(note.clone()),
        );
    }

    v_flex()
        .gap(px(16.))
        .child(
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("提示：后续将补充连接测试、历史操作等信息。"),
        )
        .child(config)
        .child(notes_block)
}

fn workspace_toolbar(
    tab_id: TabId,
    has_query: bool,
    cx: &mut Context<SqlerApp>,
) -> gpui::Div {
    let buttons = [
        ("tab-config", "数据源配置", false),
        ("tab-new-query", "新建查询", !has_query),
        ("tab-import", "导入", true),
        ("tab-export", "导出", true),
    ];

    h_flex()
        .gap(px(8.))
        .px(px(16.))
        .py(px(10.))
        .bg(cx.theme().tab_bar)
        .border_b_1()
        .border_color(cx.theme().border)
        .children(buttons.into_iter().map(|(id, label, disabled)| {
            let button = Button::new((id, tab_id.raw())).ghost().small().label(label);

            if id == "tab-config" {
                button.selected(true).on_click(cx.listener(move |this, _, _, cx| {
                    this.set_active_inner_tab(tab_id, InnerTabId(0), cx);
                }))
            } else {
                button.disabled(disabled)
            }
        }))
}

fn inner_tab_bar(
    tab_id: TabId,
    tabs: &[InnerTab],
    active: InnerTabId,
    cx: &mut Context<SqlerApp>,
) -> gpui::Div {
    h_flex()
        .gap(px(6.))
        .px(px(16.))
        .py(px(8.))
        .border_b_1()
        .border_color(cx.theme().border)
        .bg(cx.theme().tab_bar)
        .children(tabs.iter().map(move |tab| {
            let tab_id_inner = tab.id;
            let pill = h_flex()
                .px(px(10.))
                .py(px(6.))
                .rounded(px(6.))
                .cursor_pointer()
                .id(SharedString::from(format!(
                    "inner-tab-{}-{}",
                    tab_id.raw(),
                    tab_id_inner.raw()
                )))
                .text_sm()
                .when(tab_id_inner == active, |this| {
                    this.bg(cx.theme().tab_active)
                        .text_color(cx.theme().tab_active_foreground)
                })
                .when(tab_id_inner != active, |this| {
                    this.text_color(cx.theme().muted_foreground)
                })
                .child(tab.title.clone());

            pill.on_click(cx.listener(move |this, _, _, cx| {
                this.set_active_inner_tab(tab_id, tab_id_inner, cx);
            }))
        }))
}
