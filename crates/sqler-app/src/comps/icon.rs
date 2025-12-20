use gpui::*;
use gpui_component::{Icon, IconNamed};

#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoElement)]
pub enum AppIcon {
    /// 向下箭头 - icons/arrow-down.svg
    ArrowDown,
    /// 向下箭头 - icons/arrow-down-to-line.svg
    ArrowDownLine,
    /// 关闭 - icons/close.svg
    Close,
    /// 新建 - icons/plus.svg
    Create,
    /// 执行 - icons/play.svg
    Execute,
    /// 导出 - icons/upload.svg
    Export,
    /// 导入 - icons/download.svg
    Import,
    /// 刷新 - icons/relead.svg
    Relead,
    /// 搜索 - icons/search.svg
    Search,
    /// 表格 - icons/sheet.svg
    Sheet,
    /// 删除 - icons/trash.svg
    Trash,
}

impl IconNamed for AppIcon {
    fn path(self) -> SharedString {
        match self {
            Self::ArrowDown => "icons/arrow-down.svg",
            Self::ArrowDownLine => "icons/arrow-down-to-line.svg",
            Self::Close => "icons/close.svg",
            Self::Create => "icons/plus.svg",
            Self::Execute => "icons/play.svg",
            Self::Export => "icons/upload.svg",
            Self::Import => "icons/download.svg",
            Self::Relead => "icons/relead.svg",
            Self::Search => "icons/search.svg",
            Self::Sheet => "icons/sheet.svg",
            Self::Trash => "icons/trash.svg",
        }
        .into()
    }
}

impl From<AppIcon> for AnyElement {
    fn from(val: AppIcon) -> Self {
        Icon::new(val).into_any_element()
    }
}

impl RenderOnce for AppIcon {
    fn render(
        self,
        _: &mut Window,
        _cx: &mut App,
    ) -> impl IntoElement {
        Icon::new(self)
    }
}
