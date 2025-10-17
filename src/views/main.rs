use gpui::{div, AnyElement, Context, IntoElement, ParentElement, Styled, Window};

use crate::{comps, SqlerApp};

use super::{content, header};

pub fn render(
    app: &mut SqlerApp,
    window: &mut Window,
    cx: &mut Context<SqlerApp>,
) -> AnyElement {
    let header = header::render(app, window, cx);
    let content = content::render(app, window, cx);

    comps::page()
        .child(header)
        .child(div().flex_1().size_full().child(content))
        .into_any_element()
}
