use gpui::*;
use gpui_component::button::Button;
use gpui_component::button::ButtonCustomVariant;
use gpui_component::button::ButtonVariants;
use gpui_component::ActiveTheme;

pub fn button(
    cx: &App,
    id: impl Into<ElementId>,
) -> Button {
    Button::new(id).h_9().custom(
        ButtonCustomVariant::new(cx)
            .color(rgb(0x3f3f3f).into())
            .hover(rgb(0x444444).into())
            .active(rgb(0x393939).into())
            .border(rgb(0x474747).into())
            .foreground(rgb(0xfffffff).into()),
    )
}
