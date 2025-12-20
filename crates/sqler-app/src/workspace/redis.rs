use std::collections::HashMap;

use gpui::{prelude::*, *};
use gpui_component::{
    ActiveTheme, Icon, IconName, Selectable, StyledExt,
    button::{Button, ButtonGroup},
    input::{Input, InputState, TabSize},
    list::ListItem,
    resizable::{resizable_panel, v_resizable},
    tree::{TreeItem, TreeState, tree},
};
use serde_json::Value;

use sqler_core::{DataSource, DatabaseSession, DriverError, QueryReq, QueryResp, create_connection};

use crate::{
    SqlerApp,
    comps::{AppIcon, DivExt, comp_id},
};

const PAGE_SIZE: usize = 500;

#[derive(Clone)]
pub struct KeyInfo {
    pub key: SharedString,
    pub ttl: SharedString,
    pub kind: SharedString,
    pub size: SharedString,
}

fn build_tree(keys: &HashMap<String, KeyInfo>) -> Vec<TreeItem> {
    let mut parent_map: HashMap<String, Vec<String>> = HashMap::new();

    for key in keys.keys() {
        let parts: Vec<&str> = key.split(':').collect();
        if parts.is_empty() {
            continue;
        }

        for i in 0..parts.len() {
            let parent = if i == 0 { "" } else { &parts[0..i].join(":") };
            let current_path = parts[0..=i].join(":");

            parent_map
                .entry(parent.to_string())
                .or_insert_with(Vec::new)
                .push(current_path);
        }
    }

    fn build_node(
        path: &str,
        tree: &HashMap<String, Vec<String>>,
        keys: &HashMap<String, KeyInfo>,
    ) -> TreeItem {
        let name = path.split(':').last().unwrap_or("").to_string();

        let children = tree
            .get(path)
            .map(|child_paths| {
                let mut items: Vec<_> = child_paths.iter().collect();
                items.sort();
                items.dedup();
                items
                    .into_iter()
                    .map(|child| build_node(&child, tree, keys))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        if children.is_empty() {
            TreeItem::new(path.to_string(), name)
        } else {
            TreeItem::new(format!("folder:{}", path), format!("{} ({})", name, children.len())).children(children)
        }
    }

    let mut roots: Vec<String> = parent_map.get("").cloned().unwrap_or_default();
    roots.sort();
    roots.dedup();
    roots
        .into_iter()
        .map(|path| build_node(&path, &parent_map, keys))
        .collect()
}

pub struct RedisWorkspace {
    pub parent: WeakEntity<SqlerApp>,

    pub source: DataSource,
    pub session: Option<Box<dyn DatabaseSession>>,

    pub active: ViewType,
    pub browse: Option<BrowseContent>,
    pub command: Option<CommandContent>,
    pub overview: Option<OverviewContent>,
}

impl RedisWorkspace {
    fn active_session(&mut self) -> Result<&mut (dyn DatabaseSession + '_), DriverError> {
        if self.session.is_none() {
            self.session = Some(create_connection(&self.source.options)?);
        }

        match self.session.as_deref_mut() {
            Some(session) => Ok(session),
            None => Err(DriverError::Other("数据库连接不可用".into())),
        }
    }

    fn exec_cmds(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(command) = self.command.as_ref() else {
            return;
        };

        // 读取编辑器内容并解析命令
        let cmd = command.editor.read(cx).text().to_string();
        let cmd = cmd.trim();

        if cmd.is_empty() {
            if let Some(command) = self.command.as_mut() {
                command.message = Some(SharedString::from("错误: 命令不能为空"));
            }
            cx.notify();
            return;
        }

        let part: Vec<&str> = cmd.split_whitespace().collect();
        let name = part[0].to_uppercase();
        let args: Vec<Value> = part[1..].iter().map(|s| Value::String(s.to_string())).collect();

        // 获取数据库连接
        let session = match self.active_session() {
            Ok(_) => self.session.take(),
            Err(err) => {
                tracing::error!("获取数据库连接失败: {}", err);
                if let Some(command) = self.command.as_mut() {
                    command.message = Some(SharedString::from(format!("连接错误: {}", err)));
                }
                cx.notify();
                return;
            }
        };
        let Some(mut session) = session else {
            return;
        };

        // 在后台线程执行命令
        cx.spawn_in(window, async move |this, cx| {
            let ret = cx
                .background_executor()
                .spawn(async move {
                    let resp = session.query(QueryReq::Command { name, args })?;
                    Ok::<_, DriverError>((resp, session))
                })
                .await;

            // 更新 UI
            let _ = cx.update(|window, cx| {
                let _ = this.update(cx, |this, cx| match ret {
                    Ok((resp, session)) => {
                        this.session = Some(session);

                        // 提取 Value 并格式化
                        let result = match resp {
                            QueryResp::Value(value) => match value {
                                Value::Null => "null".to_string(),
                                Value::Bool(b) => b.to_string(),
                                Value::Number(n) => n.to_string(),
                                Value::String(s) => s,
                                Value::Array(arr) => {
                                    serde_json::to_string_pretty(&arr).unwrap_or_else(|_| format!("{:?}", arr))
                                }
                                Value::Object(obj) => {
                                    serde_json::to_string_pretty(&obj).unwrap_or_else(|_| format!("{:?}", obj))
                                }
                            },
                            _ => {
                                tracing::error!("Redis 驱动返回了非预期类型: {:?}", resp);
                                "内部错误: 返回类型不匹配".to_string()
                            }
                        };

                        if let Some(command) = this.command.as_mut() {
                            command.result.update(cx, |state, cx| {
                                state.set_value(&result, window, cx);
                            });
                            command.message = Some("success".into());
                            cx.notify();
                        };
                    }
                    Err(err) => {
                        tracing::error!("执行 Redis 命令失败: {}", err);
                        this.session = None;
                        if let Some(command) = this.command.as_mut() {
                            command.message = Some(SharedString::from(format!("执行错误: {}", err)));
                        }
                        cx.notify();
                    }
                });
            });
        })
        .detach();
    }

    fn fetch_keys(
        &mut self,
        all: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(browse) = self.browse.as_mut() else {
            return;
        };
        if browse.loading || (!browse.keys.is_empty() && browse.cursor.as_ref() == "0") {
            return;
        }
        browse.loading = true; // 设置加载状态
        cx.notify();

        // 获取当前游标
        let cursor = browse.cursor.to_string();

        // 获取数据库连接
        let session = match self.active_session() {
            Ok(_) => self.session.take(),
            Err(err) => {
                tracing::error!("获取数据库连接失败: {}", err);
                return;
            }
        };
        let Some(mut session) = session else {
            return;
        };

        // 在后台线程执行查询
        cx.spawn_in(window, async move |this, cx| {
            let ret = cx
                .background_executor()
                .spawn(async move {
                    let mut keys: Vec<String> = vec![];
                    let mut cursor = cursor;

                    loop {
                        match session.query(QueryReq::Command {
                            name: "SCAN".to_string(),
                            args: vec![
                                Value::String(cursor.clone()),
                                Value::String("COUNT".to_string()),
                                Value::Number(PAGE_SIZE.into()),
                            ],
                        }) {
                            Ok(QueryResp::Value(Value::Array(arr))) => {
                                if arr.len() < 2 {
                                    break;
                                }

                                // 获取游标
                                let new_cursor = match &arr[0] {
                                    Value::String(s) => s.clone(),
                                    Value::Number(n) => n.to_string(),
                                    _ => "0".to_string(),
                                };
                                cursor = new_cursor;

                                // 获取列表
                                if let Value::Array(arr) = &arr[1] {
                                    arr.iter().for_each(|key| {
                                        if let Value::String(k) = key {
                                            keys.push(k.clone());
                                        }
                                    });
                                }

                                // 检查是否继续加载
                                if cursor == "0" {
                                    break;
                                } else {
                                    if !all && keys.len() >= PAGE_SIZE {
                                        break;
                                    }
                                }
                            }
                            Ok(other) => {
                                tracing::error!("获取 SCAN 返回类型错误: {:?}", other);
                                break;
                            }
                            Err(err) => {
                                tracing::error!("执行 SCAN 命令失败: {}", err);
                                break;
                            }
                        }
                    }

                    // 获取每个 key 的详细信息
                    let mut infos = HashMap::new();
                    for key in keys {
                        let kind = match session.query(QueryReq::Command {
                            name: "TYPE".to_string(),
                            args: vec![Value::String(key.clone())],
                        }) {
                            Ok(QueryResp::Value(Value::String(t))) => t,
                            _ => "unknown".to_string(),
                        };

                        let size = match session.query(QueryReq::Command {
                            name: "MEMORY".to_string(),
                            args: vec![Value::String("USAGE".to_string()), Value::String(key.clone())],
                        }) {
                            Ok(QueryResp::Value(Value::Number(n))) => n.to_string(),
                            _ => "-".to_string(),
                        };

                        let ttl = match session.query(QueryReq::Command {
                            name: "TTL".to_string(),
                            args: vec![Value::String(key.clone())],
                        }) {
                            Ok(QueryResp::Value(Value::Number(n))) => n.to_string(),
                            _ => "-".to_string(),
                        };

                        infos.insert(
                            key.clone(),
                            KeyInfo {
                                key: SharedString::from(key),
                                ttl: SharedString::from(ttl),
                                kind: SharedString::from(kind),
                                size: SharedString::from(size),
                            },
                        );
                    }
                    Ok::<_, String>((cursor, infos, session))
                })
                .await;

            // 更新 UI
            let _ = cx.update(|_, cx| {
                let _ = this.update(cx, |this, cx| match ret {
                    Ok((cursor, infos, session)) => {
                        this.session = Some(session);
                        let Some(browse) = this.browse.as_mut() else {
                            return;
                        };
                        browse.keys.extend(infos);
                        browse.cursor = SharedString::from(cursor);

                        let tree_items = build_tree(&browse.keys);
                        browse.tree_state.update(cx, |tree, cx| {
                            tree.set_items(tree_items, cx);
                            cx.notify();
                        });

                        browse.loading = false;
                        cx.notify();
                    }
                    Err(err) => {
                        tracing::error!("加载 Redis keys 失败: {}", err);
                        this.session = None;
                        let Some(browse) = this.browse.as_mut() else {
                            return;
                        };
                        browse.loading = false;
                        cx.notify();
                    }
                });
            });
        })
        .detach();
    }

    fn fetch_detail(
        &mut self,
        key: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // 获取数据库连接
        let session = match self.active_session() {
            Ok(_) => self.session.take(),
            Err(err) => {
                tracing::error!("获取数据库连接失败: {}", err);
                return;
            }
        };
        let Some(mut session) = session else {
            return;
        };

        cx.spawn_in(window, async move |this, cx| {
            let ret = cx
                .background_executor()
                .spawn(async move {
                    let kind = match session.query(QueryReq::Command {
                        name: "TYPE".to_string(),
                        args: vec![Value::String(key.clone())],
                    }) {
                        Ok(QueryResp::Value(Value::String(t))) => t,
                        _ => "unknown".to_string(),
                    };

                    // 根据类型获取完整值
                    let value = match kind.as_str() {
                        "string" => match session.query(QueryReq::Command {
                            name: "GET".to_string(),
                            args: vec![Value::String(key.clone())],
                        }) {
                            Ok(QueryResp::Value(Value::String(v))) => v,
                            _ => "".to_string(),
                        },
                        "list" => match session.query(QueryReq::Command {
                            name: "LRANGE".to_string(),
                            args: vec![
                                Value::String(key.clone()),
                                Value::Number(0.into()),
                                Value::Number((-1).into()),
                            ],
                        }) {
                            Ok(QueryResp::Value(Value::Array(arr))) => arr
                                .iter()
                                .enumerate()
                                .map(|(i, v)| {
                                    if let Value::String(s) = v {
                                        format!("[{}] {}", i, s)
                                    } else {
                                        format!("[{}] {:?}", i, v)
                                    }
                                })
                                .collect::<Vec<_>>()
                                .join("\n"),
                            _ => "".to_string(),
                        },
                        "set" => match session.query(QueryReq::Command {
                            name: "SMEMBERS".to_string(),
                            args: vec![Value::String(key.clone())],
                        }) {
                            Ok(QueryResp::Value(Value::Array(arr))) => arr
                                .iter()
                                .enumerate()
                                .map(|(i, v)| {
                                    if let Value::String(s) = v {
                                        format!("[{}] {}", i, s)
                                    } else {
                                        format!("[{}] {:?}", i, v)
                                    }
                                })
                                .collect::<Vec<_>>()
                                .join("\n"),
                            _ => "".to_string(),
                        },
                        "hash" => match session.query(QueryReq::Command {
                            name: "HGETALL".to_string(),
                            args: vec![Value::String(key.clone())],
                        }) {
                            Ok(QueryResp::Value(Value::Array(arr))) => {
                                let mut result = String::new();
                                for chunk in arr.chunks(2) {
                                    if chunk.len() != 2 {
                                        continue;
                                    }
                                    let field = match &chunk[0] {
                                        Value::String(s) => s.clone(),
                                        _ => "".to_string(),
                                    };
                                    let value = match &chunk[1] {
                                        Value::String(s) => s.clone(),
                                        _ => "".to_string(),
                                    };
                                    result.push_str(&format!("{}: {}\n", field, value));
                                }
                                result
                            }
                            _ => "".to_string(),
                        },
                        "zset" => match session.query(QueryReq::Command {
                            name: "ZRANGE".to_string(),
                            args: vec![
                                Value::String(key.clone()),
                                Value::Number(0.into()),
                                Value::Number((-1).into()),
                                Value::String("WITHSCORES".to_string()),
                            ],
                        }) {
                            Ok(QueryResp::Value(Value::Array(arr))) => {
                                let mut result = String::new();
                                for chunk in arr.chunks(2) {
                                    if chunk.len() != 2 {
                                        continue;
                                    }
                                    let member = match &chunk[0] {
                                        Value::String(s) => s.clone(),
                                        _ => "".to_string(),
                                    };
                                    let score = match &chunk[1] {
                                        Value::String(s) => s.clone(),
                                        Value::Number(n) => n.to_string(),
                                        _ => "".to_string(),
                                    };
                                    result.push_str(&format!("{} ({})\n", member, score));
                                }
                                result
                            }
                            _ => "".to_string(),
                        },
                        _ => "".to_string(),
                    };
                    Ok::<_, String>((value, session))
                })
                .await;

            // 更新 UI
            let _ = cx.update(|window, cx| {
                let _ = this.update(cx, |this, cx| match ret {
                    Ok((value, session)) => {
                        this.session = Some(session);
                        let Some(browse) = this.browse.as_mut() else {
                            return;
                        };
                        browse.selected_value.update(cx, |state, cx| {
                            state.set_value(&value, window, cx);
                        });
                        cx.notify();
                    }
                    Err(err) => {
                        tracing::error!("加载 Redis Key 详情失败: {}", err);
                        this.session = None;
                        cx.notify();
                    }
                });
            });
        })
        .detach();
    }

    fn render_browse_view(
        &self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let browse = self.browse.as_ref().unwrap();
        let theme = cx.theme().clone();
        let id = &self.source.id;

        let tree = div()
            .col_full()
            .child(
                div()
                    .flex()
                    .flex_row()
                    .px_4()
                    .py_1()
                    .text_sm()
                    .font_semibold()
                    .child(div().pl_2().flex_1().child("键"))
                    .child(div().w_20().child("类型"))
                    .child(div().w_20().child("大小"))
                    .child(div().w_20().child("TTL")),
            )
            .child(tree(&browse.tree_state, {
                let keys = browse.keys.clone();
                let entity = cx.entity().clone();
                move |_, entry, selected, _, _| {
                    let key = entry.item();

                    let item = ListItem::new(key.id.clone())
                        .pl(px(16.) * entry.depth() + px(20.))
                        .text_sm()
                        .selected(selected);

                    if entry.is_folder() {
                        let icon_name = if entry.is_expanded() {
                            IconName::FolderOpen
                        } else {
                            IconName::FolderClosed
                        };

                        item.child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap_2()
                                .child(Icon::new(icon_name))
                                .child(key.label.clone()),
                        )
                    } else {
                        let Some(info) = keys.get(key.id.as_str()) else {
                            return item;
                        };
                        item.on_click({
                            let view = entity.clone();
                            let key = info.key.clone();
                            move |_, window, cx| {
                                let _ = view.update(cx, |this, cx| {
                                    let Some(browse) = this.browse.as_mut() else {
                                        return;
                                    };
                                    browse.selected = Some(key.clone());
                                    cx.notify();

                                    // 加载详情
                                    this.fetch_detail(key.to_string(), window, cx);
                                });
                            }
                        })
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .child(
                                    div().flex_1().child(
                                        div()
                                            .flex()
                                            .flex_row()
                                            .items_center()
                                            .gap_2()
                                            .child(Icon::new(IconName::File))
                                            .child(key.label.clone()),
                                    ),
                                )
                                .child(div().w_20().child(info.kind.clone()))
                                .child(div().w_20().child(info.size.clone()))
                                .child(div().w_20().child(info.ttl.clone())),
                        )
                    }
                }
            }));

        let detail = if let Some(key) = &browse.selected {
            div()
                .p_4()
                .gap_2()
                .col_full()
                .text_sm()
                .child(div().child(key.clone()))
                .child(
                    div()
                        .col_full()
                        .border_1()
                        .border_color(theme.border)
                        .rounded_md()
                        .scrollbar_y()
                        .child(
                            Input::new(&browse.selected_value)
                                .h_full()
                                .disabled(true)
                                .appearance(false),
                        ),
                )
                .into_any_element()
        } else {
            div().into_any_element()
        };

        v_resizable(comp_id(["browse-content", &id]))
            .child(
                resizable_panel()
                    .size(px(400.0))
                    .size_range(px(200.0)..Pixels::MAX)
                    .child(tree),
            )
            .child(detail)
            .into_any_element()
    }

    fn render_command_view(
        &self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let command = self.command.as_ref().unwrap();
        let theme = cx.theme();
        let id = &self.source.id;

        v_resizable(comp_id(["command-content", &id]))
            .child(
                resizable_panel()
                    .size(px(200.0))
                    .size_range(px(200.)..Pixels::MAX)
                    .child(
                        div().flex_1().child(
                            Input::new(&command.editor)
                                .p_0()
                                .h_full()
                                .appearance(false)
                                .text_sm()
                                .font_family(theme.mono_font_family.clone())
                                .focus_bordered(false),
                        ),
                    )
                    .child(div()),
            )
            .child(
                div()
                    .col_full()
                    .child(
                        div().p_4().col_full().child(
                            div()
                                .col_full()
                                .scrollbar_y()
                                .child(Input::new(&command.result).h_full().disabled(true).appearance(false)),
                        ),
                    )
                    .into_any_element(),
            )
            .into_any_element()
    }

    fn render_overview_view(
        &self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> AnyElement {
        div().col_full().into_any_element()
    }
}

impl Render for RedisWorkspace {
    fn render(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let id = &self.source.id;
        let theme = cx.theme().clone();

        match &self.active {
            ViewType::Overview if self.overview.is_none() => {
                self.overview = Some(OverviewContent {});
            }
            ViewType::Browse if self.browse.is_none() => {
                let keys = HashMap::new();
                let items = build_tree(&keys);

                self.browse = Some(BrowseContent {
                    keys,
                    tree_state: cx.new(|cx| TreeState::new(cx).items(items)),

                    loading: false,
                    cursor: SharedString::from("0"),

                    selected: None,
                    selected_value: cx.new(|cx| InputState::new(window, cx).multi_line(true).searchable(false)),
                });
            }
            ViewType::Command if self.command.is_none() => {
                self.command = Some(CommandContent {
                    editor: cx.new(|cx| {
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
                    result: cx.new(|cx| InputState::new(window, cx).multi_line(true).searchable(false)),
                    message: None,
                });
            }
            _ => {}
        }

        div()
            .id(comp_id(["redis", id]))
            .col_full()
            .child(
                div()
                    .id(comp_id(["redis-header", id]))
                    .flex()
                    .flex_row()
                    .p_4()
                    .gap_2()
                    .border_b_1()
                    .border_color(theme.border)
                    .child(
                        Button::new(comp_id(["redis-header-reload", id]))
                            .icon(AppIcon::Relead)
                            .label("刷新")
                            .outline(),
                    )
                    .when(matches!(self.active, ViewType::Browse), |this| {
                        this.child(
                            Button::new(comp_id(["redis-header-load-more", id]))
                                .icon(AppIcon::ArrowDown)
                                .label("获取更多")
                                .outline()
                                .on_click(cx.listener({
                                    |view, _, window, cx| {
                                        view.fetch_keys(false, window, cx);
                                    }
                                })),
                        )
                        .child(
                            Button::new(comp_id(["redis-header-load-all", id]))
                                .icon(AppIcon::ArrowDownLine)
                                .label("获取全部")
                                .outline()
                                .on_click(cx.listener({
                                    |view, _, window, cx| {
                                        view.fetch_keys(true, window, cx);
                                    }
                                })),
                        )
                        .child(
                            Button::new(comp_id(["redis-header-add", id]))
                                .icon(AppIcon::Create)
                                .label("新建数据")
                                .outline(),
                        )
                    })
                    .when(matches!(self.active, ViewType::Command), |this| {
                        this.child(
                            Button::new(comp_id(["redis-header-exec", &id]))
                                .icon(AppIcon::Execute)
                                .label("执行查询")
                                .outline()
                                .on_click(cx.listener({
                                    // rustfmt::skip
                                    |view, _, window, cx| {
                                        view.exec_cmds(window, cx);
                                    }
                                })),
                        )
                    })
                    .child(div().flex_1())
                    .child(
                        ButtonGroup::new(comp_id(["redis-view-switcher", id]))
                            .outline()
                            .compact()
                            .child(
                                Button::new(comp_id(["redis-view-overview", id]))
                                    .px_4()
                                    .label("概览视图")
                                    .selected(matches!(self.active, ViewType::Overview)),
                            )
                            .child(
                                Button::new(comp_id(["redis-view-browse", id]))
                                    .px_4()
                                    .label("数据视图")
                                    .selected(matches!(self.active, ViewType::Browse)),
                            )
                            .child(
                                Button::new(comp_id(["redis-view-command", id]))
                                    .px_4()
                                    .label("命令视图")
                                    .selected(matches!(self.active, ViewType::Command)),
                            )
                            .on_click(cx.listener({
                                move |view, selected: &Vec<usize>, _, cx| {
                                    match selected[0] {
                                        1 => view.active = ViewType::Browse,
                                        2 => view.active = ViewType::Command,
                                        0 => view.active = ViewType::Overview,
                                        _ => {}
                                    }
                                    cx.notify();
                                }
                            })),
                    ),
            )
            .child(
                div()
                    .id(comp_id(["redis-content", id]))
                    .col_full()
                    .child(match &self.active {
                        ViewType::Browse => self.render_browse_view(window, cx),
                        ViewType::Command => self.render_command_view(window, cx),
                        ViewType::Overview => self.render_overview_view(window, cx),
                    }),
            )
    }
}

pub enum ViewType {
    Browse,
    Command,
    Overview,
}

pub struct BrowseContent {
    pub keys: HashMap<String, KeyInfo>,
    pub tree_state: Entity<TreeState>,

    pub loading: bool,

    pub cursor: SharedString,
    pub selected: Option<SharedString>,
    pub selected_value: Entity<InputState>,
}

pub struct CommandContent {
    pub editor: Entity<InputState>,
    pub result: Entity<InputState>,
    pub message: Option<SharedString>,
}

pub struct OverviewContent {}
