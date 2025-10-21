use gpui::prelude::*;
use gpui::*;

use gpui_component::{v_flex, ActiveTheme as _, StyledExt};

use crate::option::{DataSource, DataSourceKind, DataSourceOptions, MySQLOptions};

pub struct MySqlWorkspace {
    meta: DataSource,
}

impl MySqlWorkspace {
    pub fn new(meta: DataSource) -> Self {
        Self { meta }
    }

}

impl Render for MySqlWorkspace {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        debug_assert!(matches!(self.meta.kind, DataSourceKind::MySQL));

        let options = match &self.meta.options {
            DataSourceOptions::MySQL(opts) => opts,
            _ => panic!("MySQL workspace expects MySQL options"),
        };

        let theme = cx.theme();
        let tables = self.meta.tables();

        let mut body = v_flex()
            .gap(px(12.))
            .child(div().text_lg().font_semibold().child(self.meta.name.clone()))
            .child(
                div()
                    .text_sm()
                    .text_color(theme.muted_foreground)
                    .child(format!("描述：{}", self.meta.desc)),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(theme.muted_foreground)
                    .child(format!(
                        "连接：{}@{}:{} / {}",
                        options.username, options.host, options.port, options.database
                    )),
            );

        if !tables.is_empty() {
            let preview = tables
                .iter()
                .take(3)
                .map(|t| t.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            body = body.child(
                div()
                    .text_sm()
                    .text_color(theme.muted_foreground)
                    .child(format!("示例表：{}", preview)),
            );
        }

        body
            .child(
                div()
                    .text_sm()
                    .text_color(theme.muted_foreground)
                    .child("MySQL 工作区规划包含连接池与慢查询分析面板。"),
            )
    }
}
