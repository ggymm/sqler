use std::path::PathBuf;

use gpui::{prelude::*, *};
use gpui_component::{
    button::{Button, ButtonVariants},
    dropdown::{Dropdown, DropdownState},
    form::{form_field, Form},
    input::{InputState, TextInput},
    ActiveTheme, Disableable, IndexPath, Sizable, Size, StyledExt,
};

use crate::{
    app::{comps::DivExt, SqlerApp},
    model::DataSource,
};

use super::TransferFormat;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TableOption {
    NewTable,
    ExistingTable,
}

impl TableOption {
    fn label(&self) -> &'static str {
        match self {
            TableOption::NewTable => "新建表",
            TableOption::ExistingTable => "已有表",
        }
    }

    fn all() -> Vec<Self> {
        vec![Self::NewTable, Self::ExistingTable]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ImportMode {
    Replace,
    Append,
    Update,
    AppendOrUpdate,
    AppendNoUpdate,
}

impl ImportMode {
    fn label(&self) -> &'static str {
        match self {
            ImportMode::Replace => "替换",
            ImportMode::Append => "追加",
            ImportMode::Update => "更新",
            ImportMode::AppendOrUpdate => "追加或更新",
            ImportMode::AppendNoUpdate => "追加不更新",
        }
    }

    fn description(&self) -> &'static str {
        match self {
            ImportMode::Replace => "清空表后导入新数据",
            ImportMode::Append => "在表末尾追加新数据",
            ImportMode::Update => "更新已存在的数据",
            ImportMode::AppendOrUpdate => "存在则更新，不存在则追加",
            ImportMode::AppendNoUpdate => "仅追加不存在的数据",
        }
    }

    fn all() -> Vec<Self> {
        vec![
            Self::Replace,
            Self::Append,
            Self::Update,
            Self::AppendOrUpdate,
            Self::AppendNoUpdate,
        ]
    }
}

#[derive(Clone, Debug)]
struct FileImportItem {
    path: PathBuf,
    table_option: TableOption,
    new_table_name: Entity<InputState>,
    selected_table: Entity<DropdownState<Vec<SharedString>>>,
    field_mappings: Vec<FieldMapping>,
}

impl FileImportItem {
    fn new(
        path: PathBuf,
        tables: Vec<SharedString>,
        window: &mut Window,
        cx: &mut App,
    ) -> Self {
        let default_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("table").to_string();

        Self {
            path,
            table_option: TableOption::NewTable,
            new_table_name: cx.new(|cx| InputState::new(window, cx).default_value(&default_name)),
            selected_table: cx.new(|cx| DropdownState::new(tables, None, window, cx)),
            field_mappings: Self::default_mappings(),
        }
    }

    fn display_name(&self) -> SharedString {
        self.path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("未命名文件")
            .to_string()
            .into()
    }

    fn option_label(
        &self,
        index: usize,
    ) -> SharedString {
        format!("{} (文件 {})", self.display_name(), index + 1).into()
    }

    fn default_mappings() -> Vec<FieldMapping> {
        vec![
            FieldMapping::new("id"),
            FieldMapping::new("name"),
            FieldMapping::new("email"),
            FieldMapping::new("created_at"),
        ]
    }
}

#[derive(Clone, Debug)]
struct FieldMapping {
    source_field: SharedString,
    target_field: SharedString,
    field_type: SharedString,
    length: Option<u32>,
    is_primary: bool,
}

impl FieldMapping {
    fn new(field: &str) -> Self {
        let name: SharedString = field.to_string().into();
        Self {
            source_field: name.clone(),
            target_field: name,
            field_type: "VARCHAR".into(),
            length: Some(255),
            is_primary: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ImportStep {
    Files,
    Format,
    Target,
    Mapping,
    Mode,
}

impl ImportStep {
    fn all() -> &'static [ImportStep] {
        &[
            ImportStep::Files,
            ImportStep::Format,
            ImportStep::Target,
            ImportStep::Mapping,
            ImportStep::Mode,
        ]
    }

    fn title(&self) -> &'static str {
        match self {
            ImportStep::Files => "选择文件",
            ImportStep::Format => "文件类型",
            ImportStep::Target => "目标表",
            ImportStep::Mapping => "字段映射",
            ImportStep::Mode => "导入模式",
        }
    }

    fn description(&self) -> &'static str {
        match self {
            ImportStep::Files => "选择需要导入的文件",
            ImportStep::Format => "设置文件类型与参数",
            ImportStep::Target => "配置源文件与目标表",
            ImportStep::Mapping => "确认字段对应关系",
            ImportStep::Mode => "选择导入模式与进度",
        }
    }

    fn next(self) -> Option<Self> {
        match self {
            ImportStep::Files => Some(ImportStep::Format),
            ImportStep::Format => Some(ImportStep::Target),
            ImportStep::Target => Some(ImportStep::Mapping),
            ImportStep::Mapping => Some(ImportStep::Mode),
            ImportStep::Mode => None,
        }
    }

    fn prev(self) -> Option<Self> {
        match self {
            ImportStep::Files => None,
            ImportStep::Format => Some(ImportStep::Files),
            ImportStep::Target => Some(ImportStep::Format),
            ImportStep::Mapping => Some(ImportStep::Target),
            ImportStep::Mode => Some(ImportStep::Mapping),
        }
    }
}

pub struct ImportWindow {
    parent: WeakEntity<SqlerApp>,
    meta: DataSource,
    format: Entity<DropdownState<Vec<SharedString>>>,
    tables: Vec<SharedString>,
    selected_files: Vec<FileImportItem>,
    row_delimiter: Entity<InputState>,
    column_delimiter: Entity<InputState>,
    header_row: Entity<InputState>,
    data_start_row: Entity<InputState>,
    import_mode: Entity<DropdownState<Vec<SharedString>>>,
    mapping_selector: Entity<DropdownState<Vec<SharedString>>>,
    current_step: ImportStep,
}

impl ImportWindow {
    pub fn new(
        meta: DataSource,
        tables: Vec<SharedString>,
        parent: WeakEntity<SqlerApp>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let parent_for_release = parent.clone();
        let _ = cx.on_release(move |_, app| {
            if let Some(parent) = parent_for_release.upgrade() {
                let _ = parent.update(app, |app, cx| {
                    app.close_import_window();
                    cx.notify();
                });
            }
        });

        let format_options: Vec<SharedString> = TransferFormat::all().iter().map(|f| f.label().into()).collect();
        let import_modes: Vec<SharedString> = ImportMode::all()
            .iter()
            .map(|m| format!("{} - {}", m.label(), m.description()).into())
            .collect();

        Self {
            parent,
            meta,
            format: cx.new(|cx| DropdownState::new(format_options, Some(IndexPath::new(0)), window, cx)),
            tables: tables.clone(),
            selected_files: Vec::new(),
            row_delimiter: cx.new(|cx| InputState::new(window, cx).default_value("\\n")),
            column_delimiter: cx.new(|cx| InputState::new(window, cx).default_value(",")),
            header_row: cx.new(|cx| InputState::new(window, cx).default_value("1")),
            data_start_row: cx.new(|cx| InputState::new(window, cx).default_value("2")),
            import_mode: cx.new(|cx| DropdownState::new(import_modes, Some(IndexPath::new(0)), window, cx)),
            mapping_selector: cx.new(|cx| DropdownState::new(Vec::<SharedString>::new(), None, window, cx)),
            current_step: ImportStep::Files,
        }
    }

    fn cancel(
        &self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(parent) = self.parent.upgrade() {
            let _ = parent.update(cx, |app, cx| {
                app.close_import_window();
                cx.notify();
            });
        }
    }

    fn current_format(
        &self,
        cx: &Context<Self>,
    ) -> Option<TransferFormat> {
        self.format
            .read(cx)
            .selected_value()
            .and_then(|value| TransferFormat::from_label(value.as_ref()))
    }

    fn choose_files(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let tables = self.tables.clone();
        let path_future = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            multiple: true,
            directories: false,
            prompt: Some("选择导入文件".into()),
        });

        cx.spawn_in(window, async move |this, cx| {
            if let Ok(Ok(Some(paths))) = path_future.await {
                let _ = cx.update(|window, cx| {
                    let _ = this.update(cx, |this, cx| {
                        for path in paths {
                            let item = FileImportItem::new(path, tables.clone(), window, cx);
                            this.selected_files.push(item);
                        }
                        cx.notify();
                    });
                });
            }
        })
        .detach();
    }

    fn remove_file(
        &mut self,
        index: usize,
        cx: &mut Context<Self>,
    ) {
        if index < self.selected_files.len() {
            self.selected_files.remove(index);
            cx.notify();
        }
    }

    fn toggle_file_table_option(
        &mut self,
        index: usize,
        option: TableOption,
        cx: &mut Context<Self>,
    ) {
        if index < self.selected_files.len() {
            self.selected_files[index].table_option = option;
            cx.notify();
        }
    }

    fn sync_mapping_selector(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let options: Vec<SharedString> = self
            .selected_files
            .iter()
            .enumerate()
            .map(|(idx, file)| file.option_label(idx))
            .collect();

        let current_value = self.mapping_selector.read(cx).selected_value().cloned();

        self.mapping_selector.update(cx, |state, cx| {
            state.set_items(options.clone(), window, cx);
            if let Some(value) = current_value
                .as_ref()
                .filter(|value| options.iter().any(|item| item == *value))
                .cloned()
            {
                state.set_selected_value(&value, window, cx);
            } else if let Some(value) = options.first() {
                state.set_selected_value(value, window, cx);
            } else {
                state.set_selected_index(None, window, cx);
            }
        });
    }

    fn mapping_selected_index(
        &self,
        cx: &App,
    ) -> Option<usize> {
        let selected = self.mapping_selector.read(cx).selected_value()?.clone();
        self.selected_files
            .iter()
            .enumerate()
            .find(|(idx, file)| file.option_label(*idx) == selected)
            .map(|(idx, _)| idx)
    }

    fn go_next(
        &mut self,
        cx: &mut Context<Self>,
    ) {
        if let Some(next) = self.current_step.next() {
            self.current_step = next;
            cx.notify();
        }
    }

    fn go_prev(
        &mut self,
        cx: &mut Context<Self>,
    ) {
        if let Some(prev) = self.current_step.prev() {
            self.current_step = prev;
            cx.notify();
        }
    }

    fn render_stepper(
        &self,
        theme: &gpui_component::theme::Theme,
    ) -> impl IntoElement {
        div()
            .flex()
            .flex_row()
            .gap_4()
            .px_8()
            .py_4()
            .children(ImportStep::all().iter().enumerate().map(|(idx, step)| {
                let active = *step == self.current_step;
                div()
                    .flex()
                    .flex_col()
                    .items_start()
                    .gap_1()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .w(px(28.))
                                    .h(px(28.))
                                    .rounded_full()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .text_sm()
                                    .font_semibold()
                                    .bg(if active { theme.primary } else { theme.secondary })
                                    .text_color(if active {
                                        theme.primary_foreground
                                    } else {
                                        theme.foreground
                                    })
                                    .child((idx + 1).to_string()),
                            )
                            .child(div().text_sm().font_semibold().child(step.title())),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.muted_foreground)
                            .child(step.description()),
                    )
                    .into_any_element()
            }))
    }
    fn render_header(
        &self,
        theme: &gpui_component::theme::Theme,
    ) -> AnyElement {
        div()
            .bg(theme.secondary)
            .border_b_1()
            .border_color(theme.border)
            .child(self.render_stepper(theme))
            .into_any_element()
    }

    fn render_files_step(
        &self,
        theme: &gpui_component::theme::Theme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        div()
            .flex()
            .flex_col()
            .gap_3()
            .p_6()
            .child(
                Button::new("choose-files")
                    .outline()
                    .label("选择文件")
                    .on_click(cx.listener(|this: &mut ImportWindow, _ev, window, cx| {
                        this.choose_files(window, cx);
                    })),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(div().text_sm().text_color(theme.muted_foreground).child("已选择的文件"))
                    .when(self.selected_files.is_empty(), |this| {
                        this.child(div().text_sm().text_color(theme.muted_foreground).child("暂未选择文件"))
                    })
                    .when(!self.selected_files.is_empty(), |this| {
                        this.child(div().flex().flex_col().gap_3().children(
                            self.selected_files.iter().enumerate().map(|(idx, file)| {
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .justify_between()
                                    .p_3()
                                    .rounded_md()
                                    .border_1()
                                    .border_color(theme.border)
                                    .bg(theme.list)
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .child(div().text_base().font_semibold().child(file.display_name()))
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .text_color(theme.muted_foreground)
                                                    .child(file.path.display().to_string()),
                                            ),
                                    )
                                    .child(Button::new(("remove-file", idx)).outline().label("删除").on_click(
                                        cx.listener({
                                            let idx = idx;
                                            move |this: &mut ImportWindow, _ev, _window, cx| {
                                                this.remove_file(idx, cx);
                                            }
                                        }),
                                    ))
                                    .into_any_element()
                            }),
                        ))
                    }),
            )
            .into_any_element()
    }

    fn render_format_step(
        &self,
        theme: &gpui_component::theme::Theme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let current_format = self.current_format(cx);

        div()
            .p_6()
            .gap_4()
            .col_full()
            .scrollable(Axis::Vertical)
            .child(
                Form::vertical()
                    .layout(Axis::Horizontal)
                    .with_size(Size::Large)
                    .label_width(px(120.))
                    .child(
                        form_field()
                            .label("文件类型")
                            .child(Dropdown::new(&self.format).with_size(Size::Large)),
                    ),
            )
            .when(matches!(current_format, Some(TransferFormat::Csv)), |this| {
                this.child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_3()
                        .child(div().text_base().font_semibold().child("CSV 额外参数"))
                        .child(
                            Form::vertical()
                                .layout(Axis::Horizontal)
                                .with_size(Size::Large)
                                .label_width(px(120.))
                                .child(
                                    form_field()
                                        .label("行分隔符")
                                        .child(TextInput::new(&self.row_delimiter).cleanable()),
                                )
                                .child(
                                    form_field()
                                        .label("列分隔符")
                                        .child(TextInput::new(&self.column_delimiter).cleanable()),
                                )
                                .child(
                                    form_field()
                                        .label("字段行")
                                        .child(TextInput::new(&self.header_row).cleanable()),
                                )
                                .child(
                                    form_field()
                                        .label("数据起始行")
                                        .child(TextInput::new(&self.data_start_row).cleanable()),
                                ),
                        ),
                )
            })
            .when(!matches!(current_format, Some(TransferFormat::Csv)), |this| {
                this.child(
                    div()
                        .text_sm()
                        .text_color(theme.muted_foreground)
                        .child("当前文件类型暂无额外配置"),
                )
            })
            .into_any_element()
    }

    fn render_target_step(
        &self,
        theme: &gpui_component::theme::Theme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        if self.selected_files.is_empty() {
            return div()
                .p_6()
                .text_sm()
                .text_color(theme.muted_foreground)
                .child("请先在第一步选择文件")
                .into_any_element();
        }

        div()
            .flex()
            .flex_col()
            .gap_3()
            .p_6()
            .children(self.selected_files.iter().enumerate().map(|(idx, file)| {
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .p_4()
                    .rounded_lg()
                    .border_1()
                    .border_color(theme.border)
                    .bg(theme.list)
                    .child(
                        div()
                            .text_base()
                            .font_semibold()
                            .child(format!("源文件：{}", file.display_name())),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap_2()
                            .children(TableOption::all().into_iter().map(|option| {
                                let active = file.table_option == option;
                                let option_id = match option {
                                    TableOption::NewTable => "table-option-new",
                                    TableOption::ExistingTable => "table-option-existing",
                                };
                                let mut button = Button::new((option_id, idx as u32)).label(option.label());
                                if active {
                                    button = button.outline();
                                } else {
                                    button = button.ghost();
                                }
                                button
                                    .on_click(cx.listener({
                                        let idx = idx;
                                        move |this: &mut ImportWindow, _ev, _window, cx| {
                                            this.toggle_file_table_option(idx, option, cx);
                                        }
                                    }))
                                    .into_any_element()
                            })),
                    )
                    .child({
                        let selector: AnyElement = match file.table_option {
                            TableOption::NewTable => div()
                                .flex()
                                .flex_col()
                                .gap_2()
                                .child(div().text_sm().text_color(theme.muted_foreground).child("新表名称"))
                                .child(TextInput::new(&file.new_table_name).cleanable())
                                .into_any_element(),
                            TableOption::ExistingTable => div()
                                .flex()
                                .flex_col()
                                .gap_2()
                                .child(div().text_sm().text_color(theme.muted_foreground).child("选择已有表"))
                                .child(Dropdown::new(&file.selected_table).with_size(Size::Large))
                                .into_any_element(),
                        };
                        selector
                    })
                    .into_any_element()
            }))
            .into_any_element()
    }

    fn render_mapping_step(
        &self,
        theme: &gpui_component::theme::Theme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        if self.selected_files.is_empty() {
            return div()
                .p_6()
                .text_sm()
                .text_color(theme.muted_foreground)
                .child("暂无文件可配置字段")
                .into_any_element();
        }

        let selected_index = self
            .mapping_selected_index(cx)
            .unwrap_or(0)
            .min(self.selected_files.len() - 1);
        let file = &self.selected_files[selected_index];

        div()
            .flex()
            .flex_col()
            .gap_4()
            .p_6()
            .child(
                Form::vertical()
                    .layout(Axis::Horizontal)
                    .with_size(Size::Large)
                    .label_width(px(120.))
                    .child(
                        form_field().label("选择文件").child(
                            Dropdown::new(&self.mapping_selector)
                                .with_size(Size::Large)
                                .placeholder("请选择文件"),
                        ),
                    ),
            )
            .child(self.render_mapping_card(selected_index, file, theme, cx))
            .into_any_element()
    }

    fn render_mapping_card(
        &self,
        index: usize,
        file: &FileImportItem,
        theme: &gpui_component::theme::Theme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        div()
            .flex()
            .flex_col()
            .gap_3()
            .p_4()
            .rounded_lg()
            .border_1()
            .border_color(theme.border)
            .bg(theme.list)
            .child(
                div()
                    .text_base()
                    .font_semibold()
                    .child(format!("文件 {}", file.display_name())),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .child(div().text_sm().text_color(theme.muted_foreground).child("字段对应关系"))
                    .child(
                        Button::new(("refresh-mapping", index as u32))
                            .outline()
                            .label("刷新字段")
                            .on_click(cx.listener({
                                let idx = index;
                                move |this: &mut ImportWindow, _ev, _window, cx| {
                                    if let Some(file) = this.selected_files.get_mut(idx) {
                                        file.field_mappings = FileImportItem::default_mappings();
                                    }
                                    cx.notify();
                                }
                            })),
                    ),
            )
            .child(self.render_field_mappings(&file.field_mappings, theme))
            .into_any_element()
    }

    fn render_field_mappings(
        &self,
        mappings: &[FieldMapping],
        theme: &gpui_component::theme::Theme,
    ) -> impl IntoElement {
        div()
            .w_full()
            .border_1()
            .border_color(theme.border)
            .rounded_lg()
            .overflow_hidden()
            .child(
                div()
                    .flex()
                    .flex_row()
                    .bg(theme.table_head)
                    .border_b_1()
                    .border_color(theme.border)
                    .child(div().flex_1().px_4().py_3().text_sm().font_semibold().child("源字段"))
                    .child(div().flex_1().px_4().py_3().text_sm().font_semibold().child("目标字段"))
                    .child(div().flex_1().px_4().py_3().text_sm().font_semibold().child("类型"))
                    .child(div().w(px(100.)).px_4().py_3().text_sm().font_semibold().child("长度"))
                    .child(div().w(px(80.)).px_4().py_3().text_sm().font_semibold().child("主键")),
            )
            .children(mappings.iter().map(|mapping| {
                div()
                    .flex()
                    .flex_row()
                    .border_b_1()
                    .border_color(theme.border)
                    .child(
                        div()
                            .flex_1()
                            .px_4()
                            .py_3()
                            .text_sm()
                            .child(mapping.source_field.clone()),
                    )
                    .child(
                        div()
                            .flex_1()
                            .px_4()
                            .py_3()
                            .text_sm()
                            .child(mapping.target_field.clone()),
                    )
                    .child(div().flex_1().px_4().py_3().text_sm().child(mapping.field_type.clone()))
                    .child(
                        div()
                            .w(px(100.))
                            .px_4()
                            .py_3()
                            .text_sm()
                            .child(mapping.length.map(|l| l.to_string()).unwrap_or_else(|| "-".to_string())),
                    )
                    .child(div().w(px(80.)).px_4().py_3().text_sm().child(if mapping.is_primary {
                        "是"
                    } else {
                        "否"
                    }))
                    .into_any_element()
            }))
    }

    fn render_mode_step(
        &self,
        theme: &gpui_component::theme::Theme,
        _cx: &mut Context<Self>,
    ) -> AnyElement {
        div()
            .p_6()
            .gap_4()
            .col_full()
            .scrollable(Axis::Vertical)
            .child(
                Form::vertical()
                    .layout(Axis::Horizontal)
                    .with_size(Size::Large)
                    .label_width(px(120.))
                    .child(
                        form_field()
                            .label("导入模式")
                            .child(Dropdown::new(&self.import_mode).with_size(Size::Large)),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(div().text_base().font_semibold().child("导入进度"))
                    .child(self.render_progress_section(theme)),
            )
            .into_any_element()
    }

    fn render_progress_section(
        &self,
        theme: &gpui_component::theme::Theme,
    ) -> AnyElement {
        if self.selected_files.is_empty() {
            return div()
                .text_sm()
                .text_color(theme.muted_foreground)
                .child("尚未选择文件，无法展示进度")
                .into_any_element();
        }

        div()
            .flex()
            .flex_col()
            .gap_2()
            .children(self.selected_files.iter().map(|file| {
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .p_3()
                    .rounded_md()
                    .border_1()
                    .border_color(theme.border)
                    .child(div().text_sm().child(file.display_name()))
                    .child(div().text_sm().text_color(theme.muted_foreground).child("未开始"))
                    .into_any_element()
            }))
            .into_any_element()
    }

    fn render_navigation(
        &self,
        theme: &gpui_component::theme::Theme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        div()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .px_8()
            .py_5()
            .bg(theme.secondary)
            .border_t_1()
            .border_color(theme.border)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        Button::new("transfer-cancel")
                            .outline()
                            .label("取消")
                            .on_click(cx.listener(|this: &mut ImportWindow, _ev, window, cx| {
                                this.cancel(window, cx);
                            })),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.muted_foreground)
                            .child(format!("当前数据源：{}", self.meta.name)),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap_3()
                    .child(
                        Button::new("import-prev-step")
                            .outline()
                            .label("上一步")
                            .disabled(self.current_step.prev().is_none())
                            .on_click(cx.listener(|this: &mut ImportWindow, _ev, _window, cx| {
                                this.go_prev(cx);
                            })),
                    )
                    .when(self.current_step != ImportStep::Mode, |buttons| {
                        buttons.child(
                            Button::new("import-next-step")
                                .outline()
                                .label("下一步")
                                .on_click(cx.listener(|this: &mut ImportWindow, _ev, _window, cx| {
                                    this.go_next(cx);
                                })),
                        )
                    })
                    .when(self.current_step == ImportStep::Mode, |buttons| {
                        buttons.child(
                            Button::new("import-start")
                                .outline()
                                .label("开始导入")
                                .on_click(cx.listener(|_this: &mut ImportWindow, _ev, _window, _cx| {
                                    // 导入逻辑将在后续实现
                                })),
                        )
                    }),
            )
            .into_any_element()
    }
}

impl Render for ImportWindow {
    fn render(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        self.sync_mapping_selector(window, cx);
        let theme = cx.theme().clone();

        let step_content: AnyElement = match self.current_step {
            ImportStep::Files => self.render_files_step(&theme, cx),
            ImportStep::Format => self.render_format_step(&theme, cx),
            ImportStep::Target => self.render_target_step(&theme, cx),
            ImportStep::Mapping => self.render_mapping_step(&theme, cx),
            ImportStep::Mode => self.render_mode_step(&theme, cx),
        };

        div()
            .col_full()
            .child(self.render_header(&theme))
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .child(div().flex_1().min_h_0().scrollable(Axis::Vertical).child(step_content)),
            )
            .child(self.render_navigation(&theme, cx))
            .into_any_element()
    }
}
