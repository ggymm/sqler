use std::{
    fs,
    io::{BufRead, BufReader},
    path::PathBuf,
    process::{Command, Stdio},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use chrono::Utc;
use dirs::{document_dir, home_dir};
use gpui::{prelude::*, *};
use gpui_component::{
    ActiveTheme, Disableable, Sizable, Size, StyledExt,
    button::Button,
    form::{Form, field},
    input::{Input, InputState, Position, TabSize},
};
use serde_json::json;
use uuid::Uuid;

use sqler_core::{DataSource, task_dir};

use crate::comps::DivExt;

pub struct DumpWindow {
    source: DataSource,

    data: bool,
    table: String,
    running: bool,

    file: Entity<InputState>,
    output: Entity<InputState>,
}

pub struct DumpWindowBuilder {
    data: Option<bool>,
    table: Option<String>,
    source: Option<DataSource>,
}

impl DumpWindowBuilder {
    pub fn new() -> Self {
        Self {
            data: None,
            table: None,
            source: None,
        }
    }

    pub fn data(
        mut self,
        data: bool,
    ) -> Self {
        self.data = Some(data);
        self
    }

    pub fn table(
        mut self,
        table: String,
    ) -> Self {
        self.table = Some(table);
        self
    }

    pub fn source(
        mut self,
        source: DataSource,
    ) -> Self {
        self.source = Some(source);
        self
    }

    pub fn build(
        self,
        window: &mut Window,
        cx: &mut Context<DumpWindow>,
    ) -> DumpWindow {
        let source = self.source.unwrap();

        let data = self.data.unwrap();
        let table = self.table.unwrap();

        // 生成默认文件路径
        let path = document_dir()
            .or_else(|| home_dir())
            .map(|dir| dir.join(format!("{}.sql", table)))
            .map(|path| path.display().to_string())
            .unwrap_or_default();

        DumpWindow {
            source,

            data,
            table,
            running: false,

            file: cx.new(|cx| InputState::new(window, cx).default_value(&path)),
            output: cx.new(|cx| {
                InputState::new(window, cx)
                    .code_editor("text")
                    .line_number(true)
                    .indent_guides(true)
                    .tab_size(TabSize {
                        tab_size: 4,
                        hard_tabs: false,
                    })
                    .soft_wrap(false)
            }),
        }
    }
}

impl DumpWindow {
    fn choose_file(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let dir = document_dir()
            .or_else(|| home_dir())
            .unwrap_or_else(|| PathBuf::from("."));
        let name = format!("{}.sql", self.table);
        let future = cx.prompt_for_new_path(&dir, Some(name.as_str()));

        let file = self.file.clone();
        cx.spawn_in(window, async move |_, cx| {
            if let Ok(Ok(Some(path))) = future.await {
                let p = path.display().to_string();
                let _ = cx.update(|window, cx| {
                    file.update(cx, |state, cx| {
                        state.set_value(&p, window, cx);
                    });
                });
            }
        })
        .detach();
    }

    fn start_dump(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let file = self.file.read(cx).value().to_string();
        if file.is_empty() {
            self.output.update(cx, |state, cx| {
                state.set_value("[执行异常] 请输入输出文件路径", window, cx);
            });
            cx.notify();
            return;
        }

        // 创建任务
        let task_id = format!("dump-{}", Uuid::new_v4());
        let task_dir = task_dir(&task_id);
        let task_config = serde_json::to_string_pretty(&json!({
            "task_id": task_id,
            "source_id": self.source.id,
            "operation": "dump",
            "created_at": Utc::now().to_rfc3339(),
            "dump": {
                "file": file,
                "table": self.table,
                "batch": 1000,
                "insert_batch": 1000,
                "timeout_seconds": 3600,
                "only_schema": !self.data,
            }
        }))
        .unwrap();
        if let Err(e) = fs::write(&task_dir.join("config.json"), &task_config) {
            self.running = false;
            self.output.update(cx, |state, cx| {
                let msg = format!("[执行异常] 创建配置文件失败: {}", e);
                state.set_value(&msg, window, cx);
            });
            cx.notify();
            return;
        }

        self.running = true;
        self.output.update(cx, |state, cx| {
            state.set_value("[执行开始]", window, cx);
            state.set_value(task_config, window, cx);
        });
        cx.notify();

        // 启动子进程
        let buffer = Arc::new(Mutex::new(vec![]));
        let writer = buffer.clone();
        thread::spawn(move || {
            let (program, args) = if cfg!(debug_assertions) {
                ("cargo", vec!["run", "-p", "sqler-task", "--", "--task-dir"])
            } else {
                ("sqler-task", vec!["--task-dir"])
            };
            let child = Command::new(program)
                .args(args)
                .arg(&task_dir)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn();

            match child {
                Ok(mut child) => {
                    if let Some(stdout) = child.stdout.take() {
                        let reader = BufReader::new(stdout);
                        for line in reader.lines() {
                            let Ok(l) = line else {
                                continue;
                            };
                            if let Ok(mut w) = writer.lock() {
                                w.push(format!("[执行日志] {}", l));
                            }
                        }
                    }

                    let result = child
                        .wait()
                        .map_err(|e| format!("[执行异常] 运行子进程异常: {}", e))
                        .and_then(|status| {
                            if status.success() {
                                Ok("[执行成功]".to_string())
                            } else {
                                Err(format!("[执行异常] 运行子进程异常: {}", status.code().unwrap_or(-1)))
                            }
                        });

                    if let Ok(mut w) = writer.lock() {
                        match result {
                            Ok(msg) => w.push(msg),
                            Err(err) => w.push(format!("[执行异常] {}", err)),
                        }
                    }
                }
                Err(e) => {
                    if let Ok(mut w) = writer.lock() {
                        w.push(format!("[执行异常] 启动子进程失败: {}", e));
                    }
                }
            }
        });

        // 在后台线程刷新日志
        let output = self.output.clone();
        cx.spawn_in(window, async move |this, cx| {
            loop {
                cx.background_executor().timer(Duration::from_secs(1)).await;

                let lines = {
                    let mut buffer = buffer.lock().unwrap();
                    let lines = buffer.drain(..).collect::<Vec<_>>();
                    lines
                };
                if lines.is_empty() {
                    continue;
                }

                let mut stop = false;
                let _ = cx.update(|window, cx| {
                    let mut logs = String::new();
                    for line in lines {
                        if line.starts_with("[执行成功]") || line.starts_with("[执行异常]") {
                            stop = true;
                        }

                        if !logs.is_empty() {
                            logs.push('\n');
                        }
                        logs.push_str(&line);
                    }

                    // 更新组件内容
                    output.update(cx, |state, cx| {
                        let value = state.value();
                        let current = if value.is_empty() {
                            logs
                        } else {
                            format!("{}\n{}", value, logs)
                        };
                        let list: Vec<&str> = current.lines().collect();
                        let line = list.len().saturating_sub(1) as u32;

                        state.set_value(&current, window, cx);
                        state.set_cursor_position(Position::new(line, 0), window, cx);

                        cx.notify()
                    });
                });

                if stop {
                    // 如果检测到结束消息，更新状态并退出
                    let _ = cx.update(|_, cx| {
                        let _ = this.update(cx, |this, cx| {
                            this.running = false;
                            cx.notify();
                        });
                    });
                    break;
                }
            }
        })
        .detach();
    }
}

impl Render for DumpWindow {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let theme = cx.theme().clone();

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
                    .child(div().text_xl().font_semibold().child("表数据转储")),
            )
            .child(
                div()
                    .flex()
                    .flex_1()
                    .flex_col()
                    .p_6()
                    .gap_4()
                    .child(
                        Form::vertical()
                            .layout(Axis::Horizontal)
                            .with_size(Size::Large)
                            .label_width(px(80.))
                            .child(field().label("表名").child(div().text_sm().child(self.table.clone())))
                            .child(
                                field()
                                    .label("数据源")
                                    .child(div().text_sm().child(self.source.name.clone())),
                            )
                            .child(field().label("包含数据").child(div().text_sm().child(if self.data {
                                "是"
                            } else {
                                "否"
                            })))
                            .child(
                                field().label("输出路径").child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .gap_2()
                                        .child(Input::new(&self.file).cleanable(true).flex_1())
                                        .child(Button::new("browse-path").label("保存位置").outline().on_click(
                                            cx.listener(|this, _, window, cx| {
                                                this.choose_file(window, cx);
                                            }),
                                        )),
                                ),
                            ),
                    )
                    .child(Input::new(&self.output).h_full()),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_end()
                    .px_8()
                    .py_5()
                    .gap_4()
                    .bg(theme.secondary)
                    .border_t_1()
                    .border_color(theme.border)
                    .child(
                        Button::new("dump-cancel")
                            .label("取消")
                            .outline()
                            .on_click(cx.listener({
                                |_: &mut DumpWindow, _, window, _| {
                                    // rustfmt::skip
                                    window.remove_window()
                                }
                            })),
                    )
                    .child(
                        Button::new("dump-execute")
                            .when(self.running, |this| this.disabled(true))
                            .label(if self.running { "执行中" } else { "执行" })
                            .outline()
                            .on_click(cx.listener({
                                |this: &mut DumpWindow, _, window, cx| {
                                    this.start_dump(window, cx);
                                }
                            })),
                    ),
            )
    }
}
