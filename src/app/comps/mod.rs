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

pub fn icon_close() -> Icon {
    Icon::default().path("icons/close.svg")
}

pub fn icon_search() -> Icon {
    Icon::default().path("icons/search.svg")
}

pub fn icon_import() -> Icon {
    Icon::default().path("icons/download.svg")
}

pub fn icon_export() -> Icon {
    Icon::default().path("icons/upload.svg")
}
