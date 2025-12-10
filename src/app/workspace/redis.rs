use std::{collections::HashMap, time::Instant};

use gpui::{prelude::*, *};
use gpui_component::input::TabSize;
use gpui_component::{
    ActiveTheme, Icon, IconName, Selectable, StyledExt,
    button::{Button, ButtonGroup},
    input::{Input, InputState},
    list::ListItem,
    resizable::{resizable_panel, v_resizable},
    tree::{TreeItem, TreeState, tree},
};
use serde_json::Value;

use crate::driver::DriverError;
use crate::{
    app::{
        SqlerApp,
        comps::{DivExt, comp_id, icon_arrow_down, icon_arrow_down_line, icon_create, icon_relead},
    },
    driver::{DatabaseSession, QueryReq, QueryResp, create_connection},
    model::DataSource,
};

#[derive(Clone)]
pub struct KeyInfo {
    pub key: String,
    pub kind: String,
    pub size: String,
    pub ttl: String,
}

fn build_tree_items(keys: &HashMap<String, KeyInfo>) -> Vec<TreeItem> {
    let mut tree_map: HashMap<String, Vec<String>> = HashMap::new();

    for key in keys.keys() {
        let parts: Vec<String> = key.split(':').map(|s| s.to_string()).collect();
        if parts.is_empty() {
            continue;
        }

        for i in 0..parts.len() {
            let parent_path = if i == 0 { String::new() } else { parts[0..i].join(":") };
            let current_path = parts[0..=i].join(":");

            tree_map.entry(parent_path).or_insert_with(Vec::new).push(current_path);
        }
    }

    fn build_node(
        path: &str,
        tree_map: &HashMap<String, Vec<String>>,
        keys: &HashMap<String, KeyInfo>,
    ) -> TreeItem {
        let parts: Vec<&str> = if path.is_empty() {
            vec![]
        } else {
            path.split(':').collect()
        };
        let name = parts.last().unwrap_or(&"").to_string();

        let children = tree_map
            .get(path)
            .map(|child_paths| {
                let mut unique_paths: Vec<String> = child_paths.iter().cloned().collect();
                unique_paths.sort();
                unique_paths.dedup();

                unique_paths
                    .into_iter()
                    .map(|child_path| build_node(&child_path, tree_map, keys))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        if children.is_empty() {
            // 叶子节点：使用原始路径作为ID
            TreeItem::new(path.to_string(), name)
        } else {
            // 文件夹节点：使用路径加前缀作为ID，避免与叶子节点ID冲突
            TreeItem::new(format!("folder:{}", path), format!("{} ({})", name, children.len())).children(children)
        }
    }

    let root_children = tree_map.get("").cloned().unwrap_or_default();
    let mut unique_roots: Vec<String> = root_children;
    unique_roots.sort();
    unique_roots.dedup();

    unique_roots
        .into_iter()
        .map(|path| build_node(&path, &tree_map, keys))
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

    /// 安全地显示值，处理二进制数据
    fn format_value(value: &str) -> String {
        // 检查是否包含不可打印字符或无效UTF-8
        let has_non_printable = value.bytes().any(|b| b < 32 && b != b'\n' && b != b'\r' && b != b'\t');

        if has_non_printable {
            // 如果包含不可打印字符，显示为十六进制
            format!("[二进制数据 {} 字节]", value.len())
        } else {
            value.to_string()
        }
    }

    fn load_key_detail(
        &mut self,
        key: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // 获取数据库连接
        let session = match self.active_session() {
            Ok(_) => self.session.take(),
            Err(err) => {
                tracing::error!("{}", err);
                return;
            }
        };

        let Some(session) = session else {
            tracing::error!("无法获取 Redis 连接");
            return;
        };

        cx.spawn_in(window, async move |this, cx| {
            let ret = cx
                .background_executor()
                .spawn(async move {
                    let mut session = session;

                    // 获取 TYPE
                    let key_type = match session.query(QueryReq::Command {
                        name: "TYPE".to_string(),
                        args: vec![Value::String(key.clone())],
                    }) {
                        Ok(QueryResp::Value(Value::String(t))) => t,
                        _ => "unknown".to_string(),
                    };

                    // 根据类型获取完整值
                    let full_value = match key_type.as_str() {
                        "string" => match session.query(QueryReq::Command {
                            name: "GET".to_string(),
                            args: vec![Value::String(key.clone())],
                        }) {
                            Ok(QueryResp::Value(Value::String(v))) => RedisWorkspace::format_value(&v),
                            _ => "".to_string(),
                        },
                        "hash" => match session.query(QueryReq::Command {
                            name: "HGETALL".to_string(),
                            args: vec![Value::String(key.clone())],
                        }) {
                            Ok(QueryResp::Value(Value::Array(arr))) => {
                                // HGETALL 返回 [field1, value1, field2, value2, ...]
                                let mut result = String::new();
                                for chunk in arr.chunks(2) {
                                    if chunk.len() == 2 {
                                        if let (Value::String(field), Value::String(val)) = (&chunk[0], &chunk[1]) {
                                            result.push_str(&format!("{}: {}\n", field, val));
                                        }
                                    }
                                }
                                result
                            }
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
                                .map(|v| {
                                    if let Value::String(s) = v {
                                        s.clone()
                                    } else {
                                        format!("{:?}", v)
                                    }
                                })
                                .collect::<Vec<_>>()
                                .join("\n"),
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
                                // ZRANGE WITHSCORES 返回 [member1, score1, member2, score2, ...]
                                let mut result = String::new();
                                for chunk in arr.chunks(2) {
                                    if chunk.len() == 2 {
                                        if let (Value::String(member), score) = (&chunk[0], &chunk[1]) {
                                            let score_str = match score {
                                                Value::String(s) => s.clone(),
                                                Value::Number(n) => n.to_string(),
                                                _ => format!("{:?}", score),
                                            };
                                            result.push_str(&format!("{} ({})\n", member, score_str));
                                        }
                                    }
                                }
                                result
                            }
                            _ => "".to_string(),
                        },
                        _ => "".to_string(),
                    };

                    Ok::<_, String>((full_value, session))
                })
                .await;

            // 更新 UI
            let _ = cx.update(|_, cx| {
                let _ = this.update(cx, |this, cx| match ret {
                    Ok((full_value, session)) => {
                        // 归还连接
                        this.session = Some(session);

                        // 更新选中 key 的完整值到专用字段
                        if let Some(browse) = this.browse.as_mut() {
                            browse.selected_key_value = full_value;
                        }

                        cx.notify();
                    }
                    Err(err) => {
                        tracing::error!("加载 key 详情失败: {}", err);
                        this.session = None;
                        cx.notify();
                    }
                });
            });
        })
        .detach();
    }

    fn load_more_keys(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.load_keys(500, false, window, cx);
    }

    fn load_all_keys(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.load_keys(usize::MAX, true, window, cx);
    }

    fn load_keys(
        &mut self,
        count: usize,
        load_all: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // 获取 browse content
        let Some(browse) = self.browse.as_mut() else {
            return;
        };

        // 检查是否正在加载
        if browse.loading {
            return;
        }

        // 如果游标为 0 且不是第一次加载，说明已经加载完所有 keys
        if browse.scan_cursor == "0" && !browse.keys.is_empty() {
            return;
        }

        // 设置加载状态
        browse.loading = true;
        cx.notify();

        // 获取当前游标
        let cursor = browse.scan_cursor.clone();

        // 获取数据库连接
        let session = match self.active_session() {
            Ok(_) => self.session.take(),
            Err(err) => {
                tracing::error!("{}", err);
                if let Some(browse) = self.browse.as_mut() {
                    browse.loading = false;
                }
                cx.notify();
                return;
            }
        };

        let Some(session) = session else {
            tracing::error!("无法获取 Redis 连接");
            if let Some(browse) = self.browse.as_mut() {
                browse.loading = false;
            }
            cx.notify();
            return;
        };

        // 在后台线程执行 SCAN 命令
        cx.spawn_in(window, async move |this, cx| {
            let ret = cx
                .background_executor()
                .spawn(async move {
                    let mut session = session;
                    let mut all_keys: Vec<String> = Vec::new();
                    let mut current_cursor = cursor;
                    let mut loaded_count = 0;

                    loop {
                        // 执行 SCAN 命令
                        let scan_result = session.query(QueryReq::Command {
                            name: "SCAN".to_string(),
                            args: vec![
                                Value::String(current_cursor.clone()),
                                Value::String("COUNT".to_string()),
                                Value::Number(500.into()),
                            ],
                        });

                        match scan_result {
                            Ok(QueryResp::Value(value)) => {
                                // 解析 SCAN 返回值: [cursor, [keys...]]
                                if let Value::Array(arr) = value {
                                    if arr.len() >= 2 {
                                        // 获取新游标
                                        let new_cursor = match &arr[0] {
                                            Value::String(s) => s.clone(),
                                            Value::Number(n) => n.to_string(),
                                            _ => "0".to_string(),
                                        };

                                        // 获取 keys
                                        if let Value::Array(keys) = &arr[1] {
                                            for key in keys {
                                                if let Value::String(k) = key {
                                                    all_keys.push(k.clone());
                                                    loaded_count += 1;
                                                }
                                            }
                                        }

                                        // 更新游标
                                        current_cursor = new_cursor;

                                        // 检查是否继续加载
                                        if current_cursor == "0" || (!load_all && loaded_count >= count) {
                                            break;
                                        }
                                    } else {
                                        break;
                                    }
                                } else {
                                    tracing::error!("SCAN 返回值格式错误: {:?}", value);
                                    break;
                                }
                            }
                            Ok(other) => {
                                tracing::error!("SCAN 返回类型错误: {:?}", other);
                                break;
                            }
                            Err(err) => {
                                tracing::error!("执行 SCAN 命令失败: {}", err);
                                break;
                            }
                        }
                    }

                    // 获取每个 key 的详细信息
                    let mut key_infos = HashMap::new();
                    for key in all_keys {
                        // 获取 TYPE
                        let kind = match session.query(QueryReq::Command {
                            name: "TYPE".to_string(),
                            args: vec![Value::String(key.clone())],
                        }) {
                            Ok(QueryResp::Value(Value::String(t))) => t,
                            _ => "unknown".to_string(),
                        };

                        // 获取 TTL
                        let ttl = match session.query(QueryReq::Command {
                            name: "TTL".to_string(),
                            args: vec![Value::String(key.clone())],
                        }) {
                            Ok(QueryResp::Value(Value::Number(n))) => {
                                if let Some(ttl_val) = n.as_i64() {
                                    if ttl_val == -1 {
                                        "No TTL".to_string()
                                    } else if ttl_val == -2 {
                                        "已过期".to_string()
                                    } else {
                                        format!("{}秒", ttl_val)
                                    }
                                } else {
                                    "-".to_string()
                                }
                            }
                            _ => "-".to_string(),
                        };

                        // 获取大小估算
                        let size = match session.query(QueryReq::Command {
                            name: "MEMORY".to_string(),
                            args: vec![Value::String("USAGE".to_string()), Value::String(key.clone())],
                        }) {
                            Ok(QueryResp::Value(Value::Number(n))) => {
                                if let Some(bytes) = n.as_u64() {
                                    if bytes < 1024 {
                                        format!("{}B", bytes)
                                    } else if bytes < 1024 * 1024 {
                                        format!("{:.1}KB", bytes as f64 / 1024.0)
                                    } else {
                                        format!("{:.1}MB", bytes as f64 / (1024.0 * 1024.0))
                                    }
                                } else {
                                    "-".to_string()
                                }
                            }
                            _ => "-".to_string(),
                        };

                        key_infos.insert(key.clone(), KeyInfo { key, kind, size, ttl });
                    }

                    Ok::<_, String>((current_cursor, key_infos, session))
                })
                .await;

            // 更新 UI
            let _ = cx.update(|_, cx| {
                let _ = this.update(cx, |this, cx| match ret {
                    Ok((cursor, key_infos, session)) => {
                        // 归还连接
                        this.session = Some(session);

                        // 更新 browse content
                        if let Some(browse) = this.browse.as_mut() {
                            // 合并新 keys 前记录数量
                            let old_count = browse.keys.len();
                            browse.keys.extend(key_infos);
                            let new_count = browse.keys.len();

                            // 更新游标
                            browse.scan_cursor = cursor;

                            // 只有当有新keys加入时才重建树（避免不必要的重建）
                            if new_count > old_count {
                                let tree_items = build_tree_items(&browse.keys);
                                browse.tree_state.update(cx, |tree, cx| {
                                    tree.set_items(tree_items, cx);
                                    cx.notify();
                                });
                            }

                            // 更新加载状态
                            browse.loading = false;
                            browse.last_refresh_time = Instant::now();
                        }

                        cx.notify();
                    }
                    Err(err) => {
                        tracing::error!("加载 Redis keys 失败: {}", err);
                        this.session = None;

                        if let Some(browse) = this.browse.as_mut() {
                            browse.loading = false;
                        }

                        cx.notify();
                    }
                });
            });
        })
        .detach();
    }

    fn render_browse_view(
        &self,
        browse: &BrowseContent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let theme = cx.theme().clone();
        let id = &self.source.id;

        let tree = div()
            .px_2()
            .col_full()
            .child(
                div()
                    .flex()
                    .flex_row()
                    .px_4()
                    .py_1()
                    .text_sm()
                    .font_semibold()
                    .child(div().flex_1().child("键"))
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
                        .pl(px(16.) * entry.depth() + px(16.))
                        .text_sm()
                        .rounded_md()
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
                                    browse.selected_key = Some(key.clone());
                                    cx.notify();

                                    // 加载详情
                                    this.load_key_detail(key.clone(), window, cx);
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

        let detail = if let Some(key) = &browse.selected_key {
            div()
                .p_4()
                .gap_4()
                .col_full()
                .text_sm()
                .child(div().child(key.clone()))
                .child(
                    div()
                        .col_full()
                        .bg(theme.secondary)
                        .border_1()
                        .border_color(theme.border)
                        .rounded_md()
                        .scrollbar_y()
                        .child(div().p_2().flex_1().child(browse.selected_key_value.clone())),
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
        command: &CommandContent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let theme = cx.theme();
        let id = &self.source.id;

        let editor = div().flex_1().child(
            Input::new(&command.editor)
                .p_0()
                .h_full()
                .appearance(false)
                .text_sm()
                .font_family(theme.mono_font_family.clone())
                .focus_bordered(false),
        );

        let result = div().into_any_element();

        v_resizable(comp_id(["command-content", &id]))
            .child(
                resizable_panel()
                    .size(px(200.0))
                    .size_range(px(80.)..px(800.))
                    .child(editor),
            )
            .child(result)
            .into_any_element()
    }

    fn render_overview_view(
        &self,
        _overview: &OverviewContent,
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
                let tree_items = build_tree_items(&keys);

                self.browse = Some(BrowseContent {
                    keys,
                    tree_state: cx.new(|cx| TreeState::new(cx).items(tree_items)),
                    selected_key: None,
                    selected_key_value: String::new(),
                    last_refresh_time: Instant::now(),
                    scan_cursor: "0".to_string(),
                    loading: false,
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
                    result: None,
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
                            .icon(icon_relead())
                            .label("刷新")
                            .outline()
                            .on_click({
                                cx.listener(|view, _, window, cx| {
                                    if let Some(browse) = view.browse.as_mut() {
                                        browse.keys.clear();
                                        browse.scan_cursor = "0".to_string();
                                        browse.loading = false;
                                    }
                                    view.load_more_keys(window, cx);
                                })
                            }),
                    )
                    .child(
                        Button::new(comp_id(["redis-header-load-more", id]))
                            .icon(icon_arrow_down())
                            .label("获取更多")
                            .outline()
                            .on_click({
                                cx.listener(|view, _event, window, cx| {
                                    view.load_more_keys(window, cx);
                                })
                            }),
                    )
                    .child(
                        Button::new(comp_id(["redis-header-load-all", id]))
                            .icon(icon_arrow_down_line())
                            .label("获取全部")
                            .outline()
                            .on_click({
                                cx.listener(|view, _event, window, cx| {
                                    view.load_all_keys(window, cx);
                                })
                            }),
                    )
                    .child(
                        Button::new(comp_id(["redis-header-add", id]))
                            .icon(icon_create())
                            .label("新建数据")
                            .outline(),
                    )
                    .child(div().flex_1())
                    .child(
                        ButtonGroup::new(comp_id(["redis-view-switcher", id]))
                            .outline()
                            .compact()
                            .child(
                                Button::new(comp_id(["redis-view-overview", id]))
                                    .label("数据源概览")
                                    .selected(matches!(self.active, ViewType::Overview)),
                            )
                            .child(
                                Button::new(comp_id(["redis-view-browse", id]))
                                    .label("数据视图")
                                    .selected(matches!(self.active, ViewType::Browse)),
                            )
                            .child(
                                Button::new(comp_id(["redis-view-command", id]))
                                    .label("命令视图")
                                    .selected(matches!(self.active, ViewType::Command)),
                            )
                            .on_click(
                                cx.listener(move |view, selected: &Vec<usize>, _window, cx| match selected[0] {
                                    0 => {
                                        if !matches!(view.active, ViewType::Overview) {
                                            view.active = ViewType::Overview;
                                            cx.notify();
                                        }
                                    }
                                    1 => {
                                        if !matches!(view.active, ViewType::Browse) {
                                            view.active = ViewType::Browse;
                                            cx.notify();
                                        }
                                    }
                                    2 => {
                                        if !matches!(view.active, ViewType::Command) {
                                            view.active = ViewType::Command;
                                            cx.notify();
                                        }
                                    }
                                    _ => {}
                                }),
                            ),
                    ),
            )
            .child(
                div()
                    .id(comp_id(["redis-content", id]))
                    .col_full()
                    .child(match &self.active {
                        ViewType::Browse => self.render_browse_view(self.browse.as_ref().unwrap(), window, cx),
                        ViewType::Command => self.render_command_view(self.command.as_ref().unwrap(), window, cx),
                        ViewType::Overview => self.render_overview_view(self.overview.as_ref().unwrap(), window, cx),
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
    pub scan_cursor: String,
    pub last_refresh_time: Instant,

    pub selected_key: Option<String>,
    pub selected_key_value: String,
}

pub struct CommandContent {
    pub editor: Entity<InputState>,
    pub result: Option<String>,
}

pub struct OverviewContent {}
