use gpui::{div, Styled};

pub fn page() -> gpui::Div {
    div()
        .flex()
        .flex_col()
        .relative()
        .size_full()
        .min_w_0()
        .min_h_0()
}
