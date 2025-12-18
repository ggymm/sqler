use std::path::PathBuf;

use gpui::{prelude::*, *};
use gpui_component::{
    ActiveTheme, Sizable, Size, StyledExt,
    button::Button,
    form::{Form, field},
    input::{Input, InputState},
    select::{Select, SelectState},
    switch::Switch,
};

use crate::{
    app::{
        SqlerApp,
        comps::{AppIcon, DivExt},
    },
    cache::ArcCache,
    driver::{DatabaseSession, DriverError, create_connection},
    model::DataSource,
};

use super::TransferKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ImportStep {
    Kind,
    Files,
    Table,
    Import,
}

impl ImportStep {
    fn all() -> &'static [ImportStep] {
        &[
            ImportStep::Kind,
            ImportStep::Files,
            ImportStep::Table,
            ImportStep::Import,
        ]
    }

    fn prev(self) -> Option<Self> {
        match self {
            ImportStep::Kind => None,
            ImportStep::Files => Some(ImportStep::Kind),
            ImportStep::Table => Some(ImportStep::Files),
            ImportStep::Import => Some(ImportStep::Table),
        }
    }

    fn next(self) -> Option<Self> {
        match self {
            ImportStep::Kind => Some(ImportStep::Files),
            ImportStep::Files => Some(ImportStep::Table),
            ImportStep::Table => Some(ImportStep::Import),
            ImportStep::Import => None,
        }
    }

    fn title(&self) -> &'static str {
        match self {
            ImportStep::Kind => "文件类型",
            ImportStep::Files => "选择文件",
            ImportStep::Table => "目标表",
            ImportStep::Import => "导入模式",
        }
    }

    fn description(&self) -> &'static str {
        match self {
            ImportStep::Kind => "设置文件类型与参数",
            ImportStep::Files => "选择需要导入的文件",
            ImportStep::Table => "配置源文件与目标表",
            ImportStep::Import => "选择导入模式与进度",
        }
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
    fn all() -> Vec<Self> {
        vec![
            Self::Replace,
            Self::Append,
            Self::Update,
            Self::AppendOrUpdate,
            Self::AppendNoUpdate,
        ]
    }

    fn label(&self) -> &'static str {
        match self {
            ImportMode::Replace => "替换 - 清空表后导入新数据",
            ImportMode::Append => "追加 - 在表末尾追加新数据",
            ImportMode::Update => "更新 - 更新已存在的数据",
            ImportMode::AppendOrUpdate => "追加或更新 - 存在则更新，不存在则追加",
            ImportMode::AppendNoUpdate => "追加不更新 - 仅追加不存在的数据",
        }
    }
}

#[derive(Clone, Debug)]
struct ImportFile {
    path: PathBuf,
    option: TableOption,
    table: Entity<InputState>,
}

impl ImportFile {
    fn new(
        path: PathBuf,
        window: &mut Window,
        cx: &mut App,
    ) -> Self {
        let default_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("table").to_string();

        Self {
            path,
            option: TableOption::NewTable,
            table: cx.new(|cx| InputState::new(window, cx).default_value(&default_name)),
        }
    }

    fn name(&self) -> SharedString {
        self.path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string()
            .into()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TableOption {
    NewTable,
    ExistTable,
}

pub struct ImportWindow {
    cache: ArcCache,
    parent: WeakEntity<SqlerApp>,

    source: DataSource,
    session: Option<Box<dyn DatabaseSession>>,

    step: ImportStep,
    files: Vec<ImportFile>,

    col_index: Entity<InputState>,
    data_index: Entity<InputState>,
    row_delimiter: Entity<InputState>,
    col_delimiter: Entity<InputState>,

    file_kinds: Entity<SelectState<Vec<SharedString>>>,
    import_modes: Entity<SelectState<Vec<SharedString>>>,
    current_tables: Entity<SelectState<Vec<SharedString>>>,
}

pub struct ImportWindowBuilder {
    cache: Option<ArcCache>,
    source: Option<DataSource>,
    parent: Option<WeakEntity<SqlerApp>>,
}

impl ImportWindowBuilder {
    pub fn new() -> Self {
        Self {
            cache: None,
            source: None,
            parent: None,
        }
    }

    pub fn cache(
        mut self,
        cache: ArcCache,
    ) -> Self {
        self.cache = Some(cache);
        self
    }

    pub fn source(
        mut self,
        source: DataSource,
    ) -> Self {
        self.source = Some(source);
        self
    }

    pub fn parent(
        mut self,
        parent: WeakEntity<SqlerApp>,
    ) -> Self {
        self.parent = Some(parent);
        self
    }

    pub fn build(
        self,
        window: &mut Window,
        cx: &mut Context<ImportWindow>,
    ) -> ImportWindow {
        let cache = self.cache.unwrap();
        let source = self.source.unwrap();
        let parent = self.parent.unwrap();

        let parent_for_release = parent.clone();
        let _ = cx.on_release(move |_, app| {
            if let Some(parent) = parent_for_release.upgrade() {
                let _ = parent.update(app, |app, _| {
                    app.close_window("import");
                });
            }
        });

        let file_kinds: Vec<SharedString> = TransferKind::all().iter().map(|f| f.label().into()).collect();
        let import_modes: Vec<SharedString> = ImportMode::all().iter().map(|m| m.label().into()).collect();

        ImportWindow {
            cache,
            parent,

            source,
            session: None,

            step: ImportStep::Kind,
            files: vec![],

            col_index: cx.new(|cx| InputState::new(window, cx).default_value("1")),
            data_index: cx.new(|cx| InputState::new(window, cx).default_value("2")),
            row_delimiter: cx.new(|cx| InputState::new(window, cx).default_value("\\n")),
            col_delimiter: cx.new(|cx| InputState::new(window, cx).default_value(",")),

            file_kinds: cx.new(|cx| SelectState::new(file_kinds, None, window, cx)),
            import_modes: cx.new(|cx| SelectState::new(import_modes, None, window, cx)),
            current_tables: cx.new(|cx| SelectState::new(vec![], None, window, cx)),
        }
    }
}

impl ImportWindow {
    fn active_session(&mut self) -> Result<&mut (dyn DatabaseSession + '_), DriverError> {
        if self.session.is_none() {
            self.session = Some(create_connection(&self.source.options)?);
        }

        match self.session.as_deref_mut() {
            Some(session) => Ok(session),
            None => Err(DriverError::Other("数据库连接不可用".into())),
        }
    }

    fn reload_tables(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // TODO
        cx.notify();
    }

    fn choose_files(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
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
                            this.files.push(ImportFile::new(path, window, cx));
                        }
                    });
                });
            }
        })
        .detach();
    }

    fn render_kind_step(
        &self,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let format = self
            .file_kinds
            .read(cx)
            .selected_value()
            .and_then(|value| TransferKind::from_label(value.as_ref()));

        div()
            .p_6()
            .col_full()
            .child(
                Form::vertical()
                    .layout(Axis::Horizontal)
                    .with_size(Size::Large)
                    .label_width(px(120.))
                    .child(
                        field()
                            .label("文件类型")
                            .child(Select::new(&self.file_kinds).placeholder("选择文件类型")),
                    )
                    .when(matches!(format, Some(TransferKind::Csv)), |this| {
                        this.child(
                            field()
                                .label("字段行")
                                .child(Input::new(&self.col_index).cleanable(true)),
                        )
                        .child(
                            field()
                                .label("数据起始行")
                                .child(Input::new(&self.data_index).cleanable(true)),
                        )
                        .child(
                            field()
                                .label("行分隔符")
                                .child(Input::new(&self.row_delimiter).cleanable(true)),
                        )
                        .child(
                            field()
                                .label("列分隔符")
                                .child(Input::new(&self.col_delimiter).cleanable(true)),
                        )
                    }),
            )
            .into_any_element()
    }

    fn render_files_step(
        &self,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let theme = cx.theme();

        div()
            .p_6()
            .gap_4()
            .col_full()
            .child(
                Button::new("choose-files")
                    .label("选择文件")
                    .outline()
                    .on_click(cx.listener({
                        // rustfmt::skip
                        |this: &mut ImportWindow, _, window, cx| {
                            this.choose_files(window, cx);
                        }
                    })),
            )
            .child(
                div()
                    .col_full()
                    .border_1()
                    .border_color(theme.border)
                    .rounded_md()
                    .child(
                        div()
                            .col_full()
                            .scrollbar_y()
                            .children(self.files.iter().enumerate().map(|(i, file)| {
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .justify_between()
                                    .px_4()
                                    .py_2()
                                    .bg(theme.list)
                                    .hover(|this| this.bg(theme.list_hover))
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(theme.foreground)
                                            .child(file.path.display().to_string()),
                                    )
                                    .child(Button::new(("remove-file", i)).label("删除").outline().on_click(
                                        cx.listener({
                                            move |this: &mut ImportWindow, _, _, cx| {
                                                if i < this.files.len() {
                                                    this.files.remove(i);
                                                    cx.notify();
                                                }
                                            }
                                        }),
                                    ))
                            })),
                    ),
            )
            .into_any_element()
    }

    fn render_table_step(
        &self,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        if self.files.is_empty() {
            return div().into_any_element();
        }
        let theme = cx.theme();
        let source_width = Length::Definite(DefiniteLength::Fraction(0.25));
        let target_width = Length::Definite(DefiniteLength::Fraction(0.75));

        div()
            .p_6()
            .col_full()
            .child(
                div().flex().flex_row().pb_4().child(
                    Button::new("reload_tables")
                        .icon(AppIcon::Relead)
                        .label("刷新表")
                        .outline()
                        .on_click(cx.listener({
                            // rustfmt::skip
                            |view: &mut Self, _, window, cx| {
                                view.reload_tables(window, cx);
                            }
                        })),
                ),
            )
            .child(
                div()
                    .col_full()
                    .border_1()
                    .border_color(theme.border)
                    .rounded_md()
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .p_2()
                            .bg(theme.table_head)
                            .child({
                                let mut column = div().text_sm().child("源文件名");
                                let style = column.style();
                                style.size.width = Some(source_width);
                                style.min_size.width = Some(source_width);
                                style.max_size.width = Some(source_width);
                                column
                            })
                            .child({
                                let mut column = div().text_sm().child("目标表设置");
                                let style = column.style();
                                style.size.width = Some(target_width);
                                style.min_size.width = Some(target_width);
                                style.max_size.width = Some(target_width);
                                column
                            }),
                    )
                    .child(
                        div().col_full().child(div().col_full().scrollbar_y().children(
                            self.files.iter().enumerate().map(|(i, file)| {
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .p_2()
                                    .border_t_1()
                                    .border_color(theme.border)
                                    .child({
                                        let mut column = div().text_sm().child(file.name());
                                        let style = column.style();
                                        style.size.width = Some(source_width);
                                        style.min_size.width = Some(source_width);
                                        style.max_size.width = Some(source_width);
                                        column
                                    })
                                    .child({
                                        let mut column = div()
                                            .flex()
                                            .flex_row()
                                            .items_center()
                                            .gap_4()
                                            .child(match file.option {
                                                TableOption::NewTable => {
                                                    Input::new(&file.table).cleanable(true).into_any_element()
                                                }
                                                TableOption::ExistTable => Select::new(&self.current_tables)
                                                    .placeholder("选择表")
                                                    .into_any_element(),
                                            })
                                            .child(
                                                Switch::new(("target-switch", i as u32))
                                                    .label("新建表")
                                                    .checked(file.option == TableOption::NewTable)
                                                    .on_click(cx.listener({
                                                        move |this: &mut ImportWindow, checked, _, cx| {
                                                            let option = if *checked {
                                                                TableOption::NewTable
                                                            } else {
                                                                TableOption::ExistTable
                                                            };
                                                            if i < this.files.len() {
                                                                this.files[i].option = option;
                                                                cx.notify();
                                                            }
                                                        }
                                                    })),
                                            );
                                        let style = column.style();
                                        style.size.width = Some(target_width);
                                        style.min_size.width = Some(target_width);
                                        style.max_size.width = Some(target_width);
                                        column
                                    })
                            }),
                        )),
                    ),
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
            .child(
                Form::vertical()
                    .layout(Axis::Horizontal)
                    .with_size(Size::Large)
                    .label_width(px(120.))
                    .child(
                        field()
                            .label("导入模式")
                            .child(Select::new(&self.import_modes).placeholder("选择导入模式")),
                    ),
            )
            .child(
                div()
                    .col_full()
                    .border_1()
                    .border_color(theme.border)
                    .rounded_md()
                    .child(div().col_full().scrollbar_y().children(self.files.iter().map(|file| {
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .justify_between()
                            .px_4()
                            .py_2()
                            .child(div().text_sm().child(file.name()))
                            .child(div().text_sm().text_color(theme.muted_foreground).child("未开始"))
                    }))),
            )
            .into_any_element()
    }
}

impl Render for ImportWindow {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let content: AnyElement = match self.step {
            ImportStep::Files => self.render_files_step(cx),
            ImportStep::Kind => self.render_kind_step(cx),
            ImportStep::Table => self.render_table_step(cx),
            ImportStep::Import => self.render_mode_step(cx),
        };

        let theme = cx.theme();
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
            .child(div().flex_1().min_h_0().child(content))
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
                            .label("取消")
                            .outline()
                            .on_click(cx.listener({
                                // rustfmt::skip
                                |this: &mut ImportWindow, _, window, cx| {
                                    if let Some(parent) = this.parent.upgrade() {
                                        let _ = parent.update(cx, |app, _| {
                                            app.close_window("import");
                                        });
                                    }
                                    window.remove_window();
                                }
                            })),
                    )
                    .child(div().flex_1())
                    .child(
                        Button::new("import-prev-step")
                            .label("上一步")
                            .outline()
                            .on_click(cx.listener({
                                // rustfmt::skip
                                |this: &mut ImportWindow, _, _, cx| {
                                    if let Some(prev) = this.step.prev() {
                                        this.step = prev;
                                        cx.notify();
                                    }
                                }
                            })),
                    )
                    .child(
                        Button::new("import-next-step")
                            .label("下一步")
                            .outline()
                            .on_click(cx.listener({
                                // rustfmt::skip
                                |this: &mut ImportWindow, _, _, cx| {
                                    if let Some(next) = this.step.next() {
                                        this.step = next;
                                        cx.notify();
                                    }
                                }
                            })),
                    )
                    .child(
                        Button::new("import-start")
                            .label("开始导入")
                            .outline()
                            .on_click(cx.listener({
                                // rustfmt::skip
                                |_this: &mut ImportWindow, _, _, _cx| {
                                    // 导入逻辑将在后续实现
                                }
                            })),
                    ),
            )
    }
}
