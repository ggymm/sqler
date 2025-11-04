use gpui::{prelude::*, *};
use gpui_component::{v_flex, ActiveTheme as _, StyledExt};

use crate::driver::DataSource;

pub struct PlaceholderWorkspace {
    meta: DataSource,
    message: SharedString,
}

impl PlaceholderWorkspace {
    pub fn new(
        meta: DataSource,
        message: impl Into<SharedString>,
    ) -> Self {
        Self {
            meta,
            message: message.into(),
        }
    }
}

impl Render for PlaceholderWorkspace {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let theme = cx.theme();

        v_flex()
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
                    .child(self.message.clone()),
            )
    }
}
