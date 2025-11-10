use std::path::PathBuf;

use gpui::{prelude::*, *};
use gpui_component::{
    button::Button,
    dropdown::{Dropdown, DropdownState},
    form::{form_field, Form},
    input::{InputState, TextInput},
    switch::Switch,
    ActiveTheme, IndexPath, Sizable, Size, StyledExt,
};

use crate::{
    app::{comps::DivExt, SqlerApp},
    model::DataSource,
};

use super::TransferFormat;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TableOption {
    NewTable,
    ExistTable,
}

impl TableOption {}

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
struct ImportFile {
    path: PathBuf,
    option: TableOption,
    new_table: Entity<InputState>,
    exist_table: Entity<DropdownState<Vec<SharedString>>>,
    field_mappings: Vec<FieldMapping>,
}

impl ImportFile {
    fn new(
        path: PathBuf,
        tables: Vec<SharedString>,
        window: &mut Window,
        cx: &mut App,
    ) -> Self {
        let default_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("table").to_string();

        Self {
            path,
            option: TableOption::NewTable,
            new_table: cx.new(|cx| InputState::new(window, cx).default_value(&default_name)),
            exist_table: cx.new(|cx| DropdownState::new(tables, None, window, cx)),
            field_mappings: Self::default_mappings(),
        }
    }

    fn display_name(&self) -> SharedString {
        self.path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
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
    meta: DataSource,
    step: ImportStep,
    parent: WeakEntity<SqlerApp>,

    files: Vec<ImportFile>,
    tables: Vec<SharedString>,

    format: Entity<DropdownState<Vec<SharedString>>>,
    col_index: Entity<InputState>,
    data_index: Entity<InputState>,
    row_delimiter: Entity<InputState>,
    col_delimiter: Entity<InputState>,

    mapping_selector: Entity<DropdownState<Vec<SharedString>>>,

    import_mode: Entity<DropdownState<Vec<SharedString>>>,
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

        let formats: Vec<SharedString> = TransferFormat::all().iter().map(|f| f.label().into()).collect();
        let import_modes: Vec<SharedString> = ImportMode::all()
            .iter()
            .map(|m| format!("{} - {}", m.label(), m.description()).into())
            .collect();

        Self {
            meta,
            step: ImportStep::Files,
            parent,

            files: Vec::new(),
            tables: tables.clone(),

            format: cx.new(|cx| DropdownState::new(formats, Some(IndexPath::new(0)), window, cx)),
            col_index: cx.new(|cx| InputState::new(window, cx).default_value("1")),
            data_index: cx.new(|cx| InputState::new(window, cx).default_value("2")),
            row_delimiter: cx.new(|cx| InputState::new(window, cx).default_value("\\n")),
            col_delimiter: cx.new(|cx| InputState::new(window, cx).default_value(",")),

            mapping_selector: cx.new(|cx| DropdownState::new(Vec::<SharedString>::new(), None, window, cx)),

            import_mode: cx.new(|cx| DropdownState::new(import_modes, Some(IndexPath::new(0)), window, cx)),
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
        let future = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            multiple: true,
            directories: false,
            prompt: Some("选择导入文件".into()),
        });

        cx.spawn_in(window, async move |this, cx| {
            if let Ok(Ok(Some(paths))) = future.await {
                let _ = cx.update(|window, cx| {
                    let _ = this.update(cx, |this, cx| {
                        for path in paths {
                            let item = ImportFile::new(path, tables.clone(), window, cx);
                            this.files.push(item);
                        }
                    });
                });
            }
        })
        .detach();
    }

    fn toggle_file_table_option(
        &mut self,
        index: usize,
        option: TableOption,
        cx: &mut Context<Self>,
    ) {
        if index < self.files.len() {
            self.files[index].option = option;
            cx.notify();
        }
    }

    fn sync_mapping_selector(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let options: Vec<SharedString> = self
            .files
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
        self.files
            .iter()
            .enumerate()
            .find(|(idx, file)| file.option_label(*idx) == selected)
            .map(|(idx, _)| idx)
    }

    fn render_files_step(
        &self,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let theme = cx.theme();

        div()
            .gap_4()
            .col_full()
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
                    .children(self.files.iter().enumerate().map(|(i, file)| {
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .justify_between()
                            .p_4()
                            .bg(theme.list)
                            .hover(|this| this.bg(theme.list_hover))
                            .rounded_lg()
                            .child(
                                div()
                                    .text_color(theme.foreground)
                                    .child(file.path.display().to_string()),
                            )
                            .child(
                                Button::new(("remove-file", i))
                                    .outline()
                                    .label("删除")
                                    .on_click(cx.listener({
                                        move |this: &mut ImportWindow, _ev, _window, cx| {
                                            if i < this.files.len() {
                                                this.files.remove(i);
                                                cx.notify();
                                            }
                                        }
                                    })),
                            )
                    })),
            )
            .into_any_element()
    }

    fn render_format_step(
        &self,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let format = self.current_format(cx);

        div()
            .col_full()
            .child(
                Form::vertical()
                    .layout(Axis::Horizontal)
                    .with_size(Size::Large)
                    .label_width(px(120.))
                    .child(form_field().label("文件类型").child(Dropdown::new(&self.format)))
                    .when(matches!(format, Some(TransferFormat::Csv)), |this| {
                        this.child(
                            form_field()
                                .label("字段行")
                                .child(TextInput::new(&self.col_index).cleanable()),
                        )
                        .child(
                            form_field()
                                .label("数据起始行")
                                .child(TextInput::new(&self.data_index).cleanable()),
                        )
                        .child(
                            form_field()
                                .label("行分隔符")
                                .child(TextInput::new(&self.row_delimiter).cleanable()),
                        )
                        .child(
                            form_field()
                                .label("列分隔符")
                                .child(TextInput::new(&self.col_delimiter).cleanable()),
                        )
                    }),
            )
            .into_any_element()
    }

    fn render_target_step(
        &self,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let theme = cx.theme();

        if self.files.is_empty() {
            return div()
                .text_base()
                .text_color(theme.muted_foreground)
                .child("请先在第一步选择文件")
                .into_any_element();
        }

        div()
            .col_full()
            .child(
                div()
                    .border_1()
                    .border_color(theme.border)
                    .rounded_lg()
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .p_4()
                            .bg(theme.table_head)
                            .child(div().flex_1().child("源文件名"))
                            .child(div().flex_1().child("目标表设置")),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .children(self.files.iter().enumerate().map(|(i, file)| {
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .p_4()
                                    .border_t_1()
                                    .border_color(theme.border)
                                    .child(div().flex_1().child(file.display_name()))
                                    .child(
                                        div()
                                            .flex()
                                            .flex_1()
                                            .flex_row()
                                            .items_center()
                                            .gap_2()
                                            .child(match file.option {
                                                TableOption::NewTable => {
                                                    TextInput::new(&file.new_table).cleanable().into_any_element()
                                                }
                                                TableOption::ExistTable => {
                                                    Dropdown::new(&file.exist_table).into_any_element()
                                                }
                                            })
                                            .child(
                                                Switch::new(("target-switch", i as u32))
                                                    .label("新建表")
                                                    .checked(file.option == TableOption::NewTable)
                                                    .on_click(cx.listener({
                                                        move |this: &mut ImportWindow, checked, _window, cx| {
                                                            let option = if *checked {
                                                                TableOption::NewTable
                                                            } else {
                                                                TableOption::ExistTable
                                                            };
                                                            this.toggle_file_table_option(i, option, cx);
                                                        }
                                                    })),
                                            ),
                                    )
                            })),
                    ),
            )
            .into_any_element()
    }

    fn render_mapping_step(
        &self,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let theme = cx.theme().clone();
        if self.files.is_empty() {
            return div()
                .p_6()
                .text_sm()
                .text_color(theme.muted_foreground)
                .child("暂无文件可配置字段")
                .into_any_element();
        }

        let selected_index = self.mapping_selected_index(cx).unwrap_or(0).min(self.files.len() - 1);
        let file = &self.files[selected_index];

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
            .child(
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
                                Button::new(("refresh-mapping", selected_index as u32))
                                    .outline()
                                    .label("刷新字段")
                                    .on_click(cx.listener({
                                        let idx = selected_index;
                                        move |this: &mut ImportWindow, _ev, _window, cx| {
                                            if let Some(file) = this.files.get_mut(idx) {
                                                file.field_mappings = ImportFile::default_mappings();
                                            }
                                            cx.notify();
                                        }
                                    })),
                            ),
                    )
                    .child(
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
                            .children(file.field_mappings.iter().map(|mapping| {
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
                                    .child(div().w(px(100.)).px_4().py_3().text_sm().child(
                                        mapping.length.map(|l| l.to_string()).unwrap_or_else(|| "-".to_string()),
                                    ))
                                    .child(div().w(px(80.)).px_4().py_3().text_sm().child(if mapping.is_primary {
                                        "是"
                                    } else {
                                        "否"
                                    }))
                                    .into_any_element()
                            })),
                    )
                    .into_any_element(),
            )
            .into_any_element()
    }

    fn render_mode_step(
        &self,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let theme = cx.theme();
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
                    .child({
                        if self.files.is_empty() {
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
                            .children(self.files.iter().map(|file| {
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

        let content: AnyElement = match self.step {
            ImportStep::Files => self.render_files_step(cx),   // 修改完成
            ImportStep::Format => self.render_format_step(cx), // 修改完成
            ImportStep::Target => self.render_target_step(cx), // 修改完成
            ImportStep::Mapping => self.render_mapping_step(cx),
            ImportStep::Mode => self.render_mode_step(cx),
        };

        div()
            .col_full()
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .px_8()
                    .py_5()
                    .gap_4()
                    .bg(theme.secondary)
                    .border_b_1()
                    .border_color(theme.border)
                    .children(ImportStep::all().iter().enumerate().map(|(i, step)| {
                        let active = *step == self.step;
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .justify_center()
                            .gap_2()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .w_8()
                                    .h_8()
                                    .text_sm()
                                    .font_semibold()
                                    .when(active, |this| this.bg(theme.primary))
                                    .when(!active, |this| this.bg(theme.secondary))
                                    .text_color(if active {
                                        theme.primary_foreground
                                    } else {
                                        theme.foreground
                                    })
                                    .rounded_full()
                                    .child((i + 1).to_string()),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .child(div().text_sm().font_semibold().child(step.title()))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(theme.muted_foreground)
                                            .child(step.description()),
                                    ),
                            )
                    })),
            )
            .child(
                div().flex_1().min_h_0().child(
                    div()
                        .p_6()
                        .gap_5()
                        .col_full()
                        .scrollable(Axis::Vertical)
                        .child(content)
                        .child(div()),
                ),
            )
            .child(
                div()
                    .relative()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .px_8()
                    .py_5()
                    .gap_4()
                    .bg(theme.secondary)
                    .border_t_1()
                    .border_color(theme.border)
                    .child(
                        Button::new("import-cancel")
                            .outline()
                            .label("取消")
                            .on_click(cx.listener(|this: &mut ImportWindow, _, _, cx| {
                                if let Some(parent) = this.parent.upgrade() {
                                    let _ = parent.update(cx, |app, cx| {
                                        app.close_import_window();
                                        cx.notify();
                                    });
                                }
                            })),
                    )
                    .child(div().flex_1())
                    .child(
                        Button::new("import-prev-step")
                            .outline()
                            .label("上一步")
                            .on_click(cx.listener(|this: &mut ImportWindow, _, _, cx| {
                                if let Some(prev) = this.step.prev() {
                                    this.step = prev;
                                    cx.notify();
                                }
                            })),
                    )
                    .child(
                        Button::new("import-next-step")
                            .outline()
                            .label("下一步")
                            .on_click(cx.listener(|this: &mut ImportWindow, _, _, cx| {
                                if let Some(next) = this.step.next() {
                                    this.step = next;
                                    cx.notify();
                                }
                            })),
                    )
                    .child(
                        Button::new("import-start")
                            .outline()
                            .label("开始导入")
                            .on_click(cx.listener(|_this: &mut ImportWindow, _, _, _cx| {
                                // 导入逻辑将在后续实现
                            })),
                    ),
            )
    }
}
