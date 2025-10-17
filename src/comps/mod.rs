use gpui::{div, Styled};

/// 基础页面容器，提供满屏纵向布局。
pub fn page() -> gpui::Div {
    div()
        .flex()
        .flex_col()
        .size_full()
        .min_w_0()
        .min_h_0()
}
