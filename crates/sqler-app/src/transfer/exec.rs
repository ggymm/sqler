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

pub struct ExecWindow {
    source: DataSource,
    running: bool,

    file: Entity<InputState>,
    output: Entity<InputState>,
}

pub struct ExecWindowBuilder {
    source: Option<DataSource>,
}

impl ExecWindowBuilder {
    pub fn new() -> Self {
        Self { source: None }
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
        cx: &mut Context<ExecWindow>,
    ) -> ExecWindow {
        let source = self.source.unwrap();

        ExecWindow {
            source,
            running: false,

            file: cx.new(|cx| InputState::new(window, cx)),
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

impl ExecWindow {
    fn start_exec(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let file = self.file.read(cx).value().to_string();
        if file.is_empty() {
            self.output.update(cx, |state, cx| {
                state.set_value("[执行异常] 请输入 SQL 文件路径", window, cx);
            });
            cx.notify();
            return;
        }

        // 检查文件是否存在
        if !PathBuf::from(&file).exists() {
            self.output.update(cx, |state, cx| {
                state.set_value("[执行异常] SQL 文件不存在", window, cx);
            });
            cx.notify();
            return;
        }

        // 创建任务
        let task_id = format!("exec-{}", Uuid::new_v4());
        let task_dir = task_dir(&task_id);
        let task_config = serde_json::to_string_pretty(&json!({
            "task_id": task_id,
            "source_id": self.source.id,
            "operation": "exec",
            "created_at": Utc::now().to_rfc3339(),
            "exec": {
                "file": file,
                "batch": 1000,
                "timeout_seconds": 3600,
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

    fn choose_file(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let future = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            multiple: false,
            directories: false,
            prompt: Some("选择 SQL 文件".into()),
        });

        let file = self.file.clone();
        cx.spawn_in(window, async move |_, cx| {
            if let Ok(Ok(Some(paths))) = future.await {
                if let Some(path) = paths.first() {
                    let p = path.display().to_string();
                    let _ = cx.update(|window, cx| {
                        file.update(cx, |state, cx| {
                            state.set_value(&p, window, cx);
                        });
                    });
                }
            }
        })
        .detach();
    }
}

impl Render for ExecWindow {
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
                    .child(div().text_xl().font_semibold().child("执行 SQL 文件")),
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
                            .child(
                                field()
                                    .label("数据源")
                                    .child(div().text_sm().child(self.source.name.clone())),
                            )
                            .child(
                                field().label("目标文件").child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .gap_2()
                                        .child(Input::new(&self.file).cleanable(true).flex_1())
                                        .child(Button::new("browse-file").label("选择文件").outline().on_click(
                                            cx.listener(|this, _, window, cx| {
                                                this.choose_file(window, cx);
                                            }),
                                        )),
                                ),
                            ),
                    )
                    .child(
                        Input::new(&self.output)
                            .h_full()
                            .text_xs()
                            .font_family(theme.mono_font_family.clone()),
                    ),
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
                        Button::new("exec-cancel")
                            .label("取消")
                            .outline()
                            .on_click(cx.listener({
                                |_: &mut ExecWindow, _, window, _| {
                                    // rustfmt::skip
                                    window.remove_window()
                                }
                            })),
                    )
                    .child(
                        Button::new("exec-execute")
                            .when(self.running, |this| this.disabled(true))
                            .label(if self.running { "执行中" } else { "执行" })
                            .outline()
                            .on_click(cx.listener({
                                |this: &mut ExecWindow, _, window, cx| {
                                    this.start_exec(window, cx);
                                }
                            })),
                    ),
            )
    }
}
