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
    file_path: Entity<InputState>,

    row_delimiter: Entity<InputState>,
    column_delimiter: Entity<InputState>,
    header_row: Entity<InputState>,
    data_start_row: Entity<InputState>,

    table_option: Option<TableOption>,
    selected_table: Entity<DropdownState<Vec<SharedString>>>,
    new_table_name: Entity<InputState>,

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
            meta: meta,
            format: cx.new(|cx| DropdownState::new(formats, None, window, cx)),
            selected_format: TransferFormat::Csv,
            file_path: cx.new(|cx| InputState::new(window, cx)),
            row_delimiter: cx.new(|cx| InputState::new(window, cx).default_value("\\n")),
            column_delimiter: cx.new(|cx| InputState::new(window, cx).default_value(",")),
            header_row: cx.new(|cx| InputState::new(window, cx).default_value("1")),
            data_start_row: cx.new(|cx| InputState::new(window, cx).default_value("2")),
            table_option: None,
            selected_table: cx.new(|cx| DropdownState::new(tables, None, window, cx)),
            new_table_name: cx.new(|cx| InputState::new(window, cx)),
            import_mode: cx.new(|cx| DropdownState::new(import_modes, None, window, cx)),
            field_mappings: Vec::new(),
        }
    }

    pub fn choose_files(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let path = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            multiple: true,
            directories: false,
            prompt: Some("选择数据库文件".into()),
        });

        // let filepath = self.filepath.clone();
        cx.spawn_in(window, async move |_, cx| {
            if let Ok(Ok(Some(mut paths))) = path.await {
                if let Some(path) = paths.pop() {
                    let p = path.display().to_string();
                    let _ = cx.update(|window, cx| {
                        // filepath.update(cx, |this, cx| {
                        //     this.set_value(&p, window, cx);
                        // });
                    });
                }
            }
        })
            .detach();
    }

    fn select_table_option(
        &mut self,
        option: TableOption,
        cx: &mut Context<Self>,
    ) {
        self.table_option = Some(option);
        // 选择表选项后，加载字段映射（模拟CSV解析）
        self.load_csv_fields(cx);
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
                Form::vertical()
                    .layout(Axis::Horizontal)
                    .with_size(Size::Large)
                    .label_width(px(100.))
                    .child(
                        form_field()
                            .label("文件路径")
                            .child(TextInput::new(&self.file_path).cleanable()),
                    )
                    .child(
                        form_field()
                            .label("文件格式")
                            .child(Dropdown::new(&self.format).with_size(Size::Large)),
                    ),
            )
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
                    .child(div().text_base().font_semibold().child("目标表")),
            )
            .children(
                TableOption::all()
                    .iter()
                    .map(|opt| {
                        let is_selected = self.table_option == Some(*opt);
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .p_4()
                            .gap_4()
                            .w_full()
                            .bg(theme.list)
                            .border_1()
                            .when(is_selected, |this| this.border_color(theme.primary))
                            .when(!is_selected, |this| this.border_color(theme.border))
                            .rounded_lg()
                            .cursor_pointer()
                            .id(("table-option-{}", *opt as u64))
                            .hover(|this| this.bg(theme.list_hover))
                            .child(div().text_base().font_semibold().child(opt.label()))
                            .on_click(cx.listener({
                                let opt = *opt;
                                move |this: &mut ImportTable, _ev, _window, cx| {
                                    this.select_table_option(opt, cx);
                                }
                            }))
                            .into_any_element()
                    })
                    .collect::<Vec<_>>(),
            )
            .when(self.table_option == Some(TableOption::ExistingTable), |this| {
                this.child(
                    Form::vertical()
                        .layout(Axis::Horizontal)
                        .with_size(Size::Large)
                        .label_width(px(100.))
                        .child(
                            form_field()
                                .label("选择表")
                                .child(Dropdown::new(&self.selected_table).with_size(Size::Large)),
                        ),
                )
            })
            .when(self.table_option == Some(TableOption::NewTable), |this| {
                this.child(
                    Form::vertical()
                        .layout(Axis::Horizontal)
                        .with_size(Size::Large)
                        .label_width(px(100.))
                        .child(
                            form_field()
                                .label("新表名称")
                                .child(TextInput::new(&self.new_table_name).cleanable()),
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
            .when(self.table_option.is_some() && !self.field_mappings.is_empty(), |this| {
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
