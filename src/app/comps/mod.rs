use gpui::*;
use gpui_component::Icon;

pub mod table;
pub use table::DataTable;

pub fn comp_id<I>(parts: I) -> ElementId
where
    I: IntoIterator,
    I::Item: ToString,
{
    let mut name = String::new();
    for part in parts {
        if !name.is_empty() {
            name.push('-');
        }
        name.push_str(&part.to_string());
    }

    ElementId::Name(SharedString::from(name))
}

pub trait DivExt {
    fn full_col(self) -> Self;
    fn full_row(self) -> Self;
}

impl DivExt for Div {
    fn full_col(self) -> Self {
        self.flex().flex_1().flex_col().size_full().min_w_0().min_h_0()
    }

    fn full_row(self) -> Self {
        self.flex().flex_1().flex_row().size_full().min_w_0().min_h_0()
    }
}

pub fn icon_close() -> Icon {
    Icon::default().path("icons/close.svg")
}

pub fn icon_export() -> Icon {
    Icon::default().path("icons/upload.svg")
}

pub fn icon_import() -> Icon {
    Icon::default().path("icons/download.svg")
}

pub fn icon_relead() -> Icon {
    Icon::default().path("icons/relead.svg")
}

pub fn icon_search() -> Icon {
    Icon::default().path("icons/search.svg")
}

pub fn icon_sheet() -> Icon {
    Icon::default().path("icons/sheet.svg")
}
