use std::path::PathBuf;

use gpui::{prelude::*, *};
use gpui_component::{
    button::Button,
    dropdown::{Dropdown, DropdownState},
    form::{form_field, Form},
    input::{InputState, TextInput},
    ActiveTheme, Sizable, Size, StyledExt,
};

use crate::{app::comps::DivExt, model::DataSource};

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
        }
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
    fn new(source_field: SharedString) -> Self {
        Self {
            source_field: source_field.clone(),
            target_field: source_field,
            field_type: "VARCHAR".into(),
            length: Some(255),
            is_primary: false,
        }
    }
}

pub struct ImportTable {
    meta: DataSource,

    format: Entity<DropdownState<Vec<SharedString>>>,
    selected_format: TransferFormat,

    tables: Vec<SharedString>,
    selected_files: Vec<FileImportItem>,

    row_delimiter: Entity<InputState>,
    column_delimiter: Entity<InputState>,
    header_row: Entity<InputState>,
    data_start_row: Entity<InputState>,

    import_mode: Entity<DropdownState<Vec<SharedString>>>,

    field_mappings: Vec<FieldMapping>,
}

impl ImportTable {
    pub fn new(
        meta: DataSource,
        tables: Vec<SharedString>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let formats: Vec<SharedString> = TransferFormat::all()
            .iter()
            .map(|f| format!("{} - {}", f.label(), f.description()).into())
            .collect();

        let import_modes: Vec<SharedString> = ImportMode::all()
            .iter()
            .map(|m| format!("{} - {}", m.label(), m.description()).into())
            .collect();

        Self {
            meta,
            format: cx.new(|cx| DropdownState::new(formats, None, window, cx)),
            selected_format: TransferFormat::Csv,
            tables,
            selected_files: Vec::new(),
            row_delimiter: cx.new(|cx| InputState::new(window, cx).default_value("\\n")),
            column_delimiter: cx.new(|cx| InputState::new(window, cx).default_value(",")),
            header_row: cx.new(|cx| InputState::new(window, cx).default_value("1")),
            data_start_row: cx.new(|cx| InputState::new(window, cx).default_value("2")),
            import_mode: cx.new(|cx| DropdownState::new(import_modes, None, window, cx)),
            field_mappings: Vec::new(),
        }
    }

    pub fn choose_files(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let path_future = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            multiple: true,
            directories: false,
            prompt: Some("选择导入文件".into()),
        });

        cx.spawn_in(window, async move |this, cx| {
            if let Ok(Ok(Some(paths))) = path_future.await {
                let _ = cx.update(|window, cx| {
                    this.update(cx, |this, cx| {
                        let tables = this.tables.clone();
                        for path in paths {
                            let item = FileImportItem::new(path, tables.clone(), window, cx);
                            this.selected_files.push(item);
                        }
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

    fn load_csv_fields(
        &mut self,
        cx: &mut Context<Self>,
    ) {
        // TODO: 实际应该解析CSV文件获取字段
        // 这里先用模拟数据展示UI
        self.field_mappings = vec![
            FieldMapping::new("id".into()),
            FieldMapping::new("name".into()),
            FieldMapping::new("email".into()),
            FieldMapping::new("created_at".into()),
        ];
        cx.notify();
    }
}

impl Render for ImportTable {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let theme = cx.theme().clone();

        div()
            .flex()
            .flex_col()
            .size_full()
            .child(div().flex_1().overflow_hidden().child(self.render_form(&theme, cx)))
            .child(self.render_footer(&theme, cx))
    }
}

impl ImportTable {
    fn render_form(
        &self,
        theme: &gpui_component::theme::Theme,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div()
            .p_6()
            .gap_5()
            .col_full()
            .scrollable(Axis::Vertical)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(div().text_base().font_semibold().child("文件格式"))
                    .child(
                        Form::vertical()
                            .layout(Axis::Horizontal)
                            .with_size(Size::Large)
                            .label_width(px(100.))
                            .child(
                                form_field()
                                    .label("文件格式")
                                    .child(Dropdown::new(&self.format).with_size(Size::Large)),
                            ),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .mt_4()
                    .child(div().text_base().font_semibold().child("选择文件"))
                    .child(
                        Button::new("choose-files")
                            .outline()
                            .label("选择文件")
                            .on_click(cx.listener(|this: &mut ImportTable, _ev, window, cx| {
                                this.choose_files(window, cx);
                            })),
                    ),
            )
            .when(!self.selected_files.is_empty(), |this| {
                this.child(self.render_file_list(theme, cx))
            })
            .when(true, |this| {
                this.child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .mt_4()
                        .child(div().text_base().font_semibold().child("CSV 格式配置"))
                        .child(
                            Form::vertical()
                                .layout(Axis::Horizontal)
                                .with_size(Size::Large)
                                .label_width(px(100.))
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
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .mt_4()
                    .child(div().text_base().font_semibold().child("导入模式"))
                    .child(
                        Form::vertical()
                            .layout(Axis::Horizontal)
                            .with_size(Size::Large)
                            .label_width(px(100.))
                            .child(
                                form_field()
                                    .label("导入模式")
                                    .child(Dropdown::new(&self.import_mode).with_size(Size::Large)),
                            ),
                    ),
            )
            .when(!self.field_mappings.is_empty(), |this| {
                this.child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .mt_4()
                        .child(div().text_base().font_semibold().child("字段映射关系"))
                        .child(self.render_field_mappings(theme)),
                )
            })
    }

    fn render_file_list(
        &self,
        theme: &gpui_component::theme::Theme,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_2()
            .mt_4()
            .child(div().text_base().font_semibold().child("已选择的文件"))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .border_1()
                    .border_color(theme.border)
                    .rounded_lg()
                    .p_3()
                    .children(
                        self.selected_files
                            .iter()
                            .enumerate()
                            .map(|(idx, file)| {
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_2()
                                    .p_3()
                                    .bg(theme.list)
                                    .rounded_md()
                                    .border_1()
                                    .border_color(theme.border)
                                    .child(
                                        div()
                                            .flex()
                                            .flex_row()
                                            .items_center()
                                            .justify_between()
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .text_color(theme.muted_foreground)
                                                    .child(file.path.display().to_string()),
                                            )
                                            .child(Button::new(("remove-file", idx)).outline().label("删除").on_click(
                                                cx.listener({
                                                    let idx = idx;
                                                    move |this: &mut ImportTable, _ev, _window, cx| {
                                                        this.remove_file(idx, cx);
                                                    }
                                                }),
                                            )),
                                    )
                                    .child(
                                        div().flex().flex_row().gap_2().children(
                                            TableOption::all()
                                                .iter()
                                                .map(|opt| {
                                                    let is_selected = file.table_option == *opt;
                                                    div()
                                                        .flex_1()
                                                        .p_2()
                                                        .text_sm()
                                                        .text_center()
                                                        .bg(if is_selected { theme.primary } else { theme.background })
                                                        .text_color(if is_selected {
                                                            theme.primary_foreground
                                                        } else {
                                                            theme.foreground
                                                        })
                                                        .border_1()
                                                        .border_color(if is_selected {
                                                            theme.primary
                                                        } else {
                                                            theme.border
                                                        })
                                                        .rounded_md()
                                                        .cursor_pointer()
                                                        .id(SharedString::from(format!(
                                                            "table-option-{}-{}",
                                                            idx, *opt as u64
                                                        )))
                                                        .hover(|this| {
                                                            if !is_selected {
                                                                this.bg(theme.list_hover)
                                                            } else {
                                                                this
                                                            }
                                                        })
                                                        .child(opt.label())
                                                        .on_click(cx.listener({
                                                            let idx = idx;
                                                            let opt = *opt;
                                                            move |this: &mut ImportTable, _ev, _window, cx| {
                                                                this.toggle_file_table_option(idx, opt, cx);
                                                            }
                                                        }))
                                                        .into_any_element()
                                                })
                                                .collect::<Vec<_>>(),
                                        ),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .gap_2()
                                            .when(file.table_option == TableOption::NewTable, |this| {
                                                this.child(
                                                    Form::vertical()
                                                        .layout(Axis::Horizontal)
                                                        .with_size(Size::Large)
                                                        .label_width(px(80.))
                                                        .child(
                                                            form_field().label("新表名").child(
                                                                TextInput::new(&file.new_table_name).cleanable(),
                                                            ),
                                                        ),
                                                )
                                            })
                                            .when(file.table_option == TableOption::ExistingTable, |this| {
                                                this.child(
                                                    Form::vertical()
                                                        .layout(Axis::Horizontal)
                                                        .with_size(Size::Large)
                                                        .label_width(px(80.))
                                                        .child(form_field().label("选择表").child(
                                                            Dropdown::new(&file.selected_table).with_size(Size::Large),
                                                        )),
                                                )
                                            }),
                                    )
                                    .into_any_element()
                            })
                            .collect::<Vec<_>>(),
                    ),
            )
    }

    fn render_field_mappings(
        &self,
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
            .children(
                self.field_mappings
                    .iter()
                    .map(|mapping| {
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
                    })
                    .collect::<Vec<_>>(),
            )
    }

    fn render_footer(
        &self,
        theme: &gpui_component::theme::Theme,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div()
            .flex()
            .flex_row()
            .items_center()
            .justify_end()
            .px_6()
            .py_4()
            .gap_3()
            .border_t_1()
            .border_color(theme.border)
            .child(
                Button::new("import-execute")
                    .outline()
                    .label("开始导入")
                    .on_click(cx.listener(|_this: &mut ImportTable, _ev, _window, _cx| {
                        // 导入逻辑将在后续实现
                    })),
            )
    }
}
