use gpui::*;
use gpui_component::{
    Icon,
    scroll::{Scrollable, ScrollableElement},
};

mod table;

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

#[allow(unused)]
pub trait DivExt {
    fn full(self) -> Self;
    fn col_full(self) -> Self;
    fn row_full(self) -> Self;
    fn scrollbar_x(self) -> Scrollable<Self>
    where
        Self: Sized + InteractiveElement + Styled + ParentElement + Element;
    fn scrollbar_y(self) -> Scrollable<Self>
    where
        Self: Sized + InteractiveElement + Styled + ParentElement + Element;
    fn scrollbar_all(self) -> Scrollable<Self>
    where
        Self: Sized + InteractiveElement + Styled + ParentElement + Element;
}

impl DivExt for Div {
    fn full(self) -> Self {
        self.size_full().min_w_0().min_h_0()
    }

    fn col_full(self) -> Self {
        self.flex().flex_1().flex_col().h_full().min_w_0().min_h_0()
    }

    fn row_full(self) -> Self {
        self.flex().flex_1().flex_row().w_full().min_w_0().min_h_0()
    }

    fn scrollbar_x(self) -> Scrollable<Self> {
        self.overflow_x_scrollbar()
    }

    fn scrollbar_y(self) -> Scrollable<Self> {
        self.overflow_y_scrollbar()
    }

    fn scrollbar_all(self) -> Scrollable<Self> {
        self.overflow_scrollbar()
    }
}

impl DivExt for Stateful<Div> {
    fn full(self) -> Self {
        self.size_full().min_w_0().min_h_0()
    }

    fn col_full(self) -> Self {
        self.flex().flex_1().flex_col().h_full().min_w_0().min_h_0()
    }

    fn row_full(self) -> Self {
        self.flex().flex_1().flex_row().w_full().min_w_0().min_h_0()
    }

    fn scrollbar_x(self) -> Scrollable<Self> {
        self.overflow_x_scrollbar()
    }

    fn scrollbar_y(self) -> Scrollable<Self> {
        self.overflow_y_scrollbar()
    }

    fn scrollbar_all(self) -> Scrollable<Self> {
        self.overflow_scrollbar()
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

pub fn icon_trash() -> Icon {
    Icon::default().path("icons/trash.svg")
}
