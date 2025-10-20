use gpui::prelude::*;
use gpui::*;

use gpui_component::{
    button::{Button, ButtonVariants as _},
    form::{form_field, v_form},
    h_flex, v_flex, ActiveTheme as _, Disableable as _, Selectable as _, Sizable, StyledExt,
};

use crate::app::{DataSourceTabState, InnerTabId, SqlerApp, TabId};
use crate::option::{MySQLOptions, StoredOptions};
use crate::DataSourceType;

pub struct MySqlWorkspace<'a> {
    state: &'a DataSourceTabState,
}

impl<'a> MySqlWorkspace<'a> {
    pub fn new(state: &'a DataSourceTabState) -> Self {
        Self { state }
    }

    pub fn render(
        &self,
        tab_id: TabId,
        _window: &mut Window,
        cx: &mut Context<SqlerApp>,
    ) -> gpui::Div {
        let meta = &self.state.meta;
        debug_assert!(matches!(meta.kind, DataSourceType::MySQL));

        let options = match &meta.options {
            StoredOptions::MySQL(opts) => opts,
            other => panic!("MySqlWorkspace expects MySQL options, got {:?}", other),
        };

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
        table_list = table_list.children(self.state.tables.iter().map(|table| {
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
            .id(SharedString::from(format!("mysql-detail-scroll-{}", tab_id.raw())))
            .overflow_scroll();
        {
            let style = detail_panel.style();
            style.min_size.height = Some(Length::Definite(px(0.).into()));
            style.min_size.width = Some(Length::Definite(px(0.).into()));
        }
        let detail_panel = detail_panel
            .px(px(24.))
            .py(px(20.))
            .child(self.render_detail(options, cx));

        let right_panel = right_panel
            .child(self.inner_tab_bar(tab_id, cx))
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

        root.child(self.workspace_toolbar(tab_id, cx)).child(content_row)
    }

    fn render_detail(
        &self,
        options: &MySQLOptions,
        cx: &mut Context<SqlerApp>,
    ) -> gpui::Div {
        let meta = &self.state.meta;

        let mut config = v_form()
            .gap(px(12.))
            .child(form_field().label("名称").child(div().child(meta.name.clone())))
            .child(form_field().label("类型").child(div().child(meta.kind.label())))
            .child(form_field().label("主机").child(div().child(options.host.clone())))
            .child(form_field().label("端口").child(div().child(options.port.to_string())))
            .child(form_field().label("数据库").child(div().child(options.database.clone())))
            .child(form_field().label("账号").child(div().child(options.username.clone())));

        if let Some(charset) = &options.charset {
            config = config.child(form_field().label("字符集").child(div().child(charset.clone())));
        }
        if options.use_tls {
            config = config.child(form_field().label("TLS").child(div().child("开启")));
        }
        if options.password.is_some() {
            config = config.child(form_field().label("密码").child(div().child("已设置")));
        }

        let notes = vec![
            format!("描述：{}", meta.desc.to_string()),
            format!("表数量：{}", self.state.tables.len()),
            "MySQL 工作区规划包含连接池与慢查询分析面板。".to_string(),
        ];

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
                    .child(self.summary(options)),
            );

        for note in notes {
            notes_block = notes_block.child(
                div()
                    .text_sm()
                    .text_color(theme.muted_foreground)
                    .child(note),
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

    fn workspace_toolbar(&self, tab_id: TabId, cx: &mut Context<SqlerApp>) -> gpui::Div {
        let buttons = [
            ("tab-config", "数据源配置", false),
            ("tab-new-query", "新建查询", true),
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

    fn inner_tab_bar(&self, tab_id: TabId, cx: &mut Context<SqlerApp>) -> gpui::Div {
        h_flex()
            .gap(px(6.))
            .px(px(16.))
            .py(px(8.))
            .border_b_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().tab_bar)
            .children(self.state.inner_tabs.iter().map(move |tab| {
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
                    .when(tab_id_inner == self.state.active_inner_tab, |this| {
                        this.bg(cx.theme().tab_active)
                            .text_color(cx.theme().tab_active_foreground)
                    })
                    .when(tab_id_inner != self.state.active_inner_tab, |this| {
                        this.text_color(cx.theme().muted_foreground)
                    })
                    .child(tab.title.clone());

                pill.on_click(cx.listener(move |this, _, _, cx| {
                    this.set_active_inner_tab(tab_id, tab_id_inner, cx);
                }))
            }))
    }

    fn summary(&self, options: &MySQLOptions) -> String {
        format!(
            "连接：{}@{}:{} / {}",
            options.username, options.host, options.port, options.database
        )
    }
}
