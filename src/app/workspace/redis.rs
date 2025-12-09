use std::{collections::HashMap, time::Instant};

use gpui::{prelude::*, *};
use gpui_component::{
    ActiveTheme, Disableable, Icon, IconName, Selectable, Sizable, Size, StyledExt,
    button::{Button, ButtonGroup},
    input::{Input, InputState},
    list::ListItem,
    resizable::{resizable_panel, v_resizable},
    select::{Select, SelectState},
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
    pub full_key: String,
    pub key_type: String,
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

    fn browse_content(&mut self) -> Option<&mut BrowseContent> {
        self.browse.as_mut()
    }

    fn command_content(&mut self) -> Option<&mut CommandContent> {
        self.command.as_mut()
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

    fn select_key(
        &mut self,
        key: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(browse) = self.browse_content() {
            browse.selected_key = Some(key.clone());
            cx.notify();

            // 加载完整的 key 值
            self.load_key_detail(key, window, cx);
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

        let key_for_closure = key.clone();

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

    fn find_key_info(
        &self,
        key: &str,
    ) -> Option<KeyInfo> {
        self.browse.as_ref().and_then(|browse| browse.keys.get(key).cloned())
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
                        let key_type = match session.query(QueryReq::Command {
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
                                        "永不过期".to_string()
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

                        key_infos.insert(
                            key.clone(),
                            KeyInfo {
                                full_key: key,
                                key_type,
                                size,
                                ttl,
                            },
                        );
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

    fn switch_to_browse(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if matches!(self.active, ViewType::Browse) {
            return;
        }

        if self.browse.is_none() {
            let detail_types = vec![
                "string".into(),
                "hash".into(),
                "list".into(),
                "set".into(),
                "zset".into(),
            ];

            let ttl_options = vec![
                "无TTL".into(),
                "10秒".into(),
                "1分钟".into(),
                "10分钟".into(),
                "1小时".into(),
            ];

            let keys = HashMap::new();
            let tree_items = build_tree_items(&keys);
            let tree_state = cx.new(|cx| TreeState::new(cx).items(tree_items));

            self.browse = Some(BrowseContent {
                keys,
                tree_state,
                selected_key: None,
                selected_key_value: String::new(),
                last_refresh_time: Instant::now(),
                detail_key_type: cx.new(|cx| SelectState::new(detail_types, None, window, cx)),
                detail_ttl: cx.new(|cx| SelectState::new(ttl_options, None, window, cx)),
                scan_cursor: "0".to_string(),
                loading: false,
            });
        }

        self.active = ViewType::Browse;
        cx.notify();
    }

    fn switch_to_command(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if matches!(self.active, ViewType::Command) {
            return;
        }

        if self.command.is_none() {
            self.command = Some(CommandContent {
                command_input: cx.new(|cx| {
                    InputState::new(window, cx).placeholder("输入 Redis 命令，例如: GET key 或 SET key value")
                }),
                command_result: None,
            });
        }

        self.active = ViewType::Command;
        cx.notify();
    }

    fn switch_to_overview(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if matches!(self.active, ViewType::Overview) {
            return;
        }

        if self.overview.is_none() {
            self.overview = Some(OverviewContent {});
        }

        self.active = ViewType::Overview;
        cx.notify();
    }

    fn render_browse_view(
        &self,
        browse: &BrowseContent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let theme = cx.theme().clone();
        let id = &self.source.id;
        let keys = browse.keys.clone();
        let view_entity = cx.entity().clone();

        // 渲染键列表（左侧面板）
        let key_list = div()
            .flex()
            .flex_col()
            .gap_2()
            .col_full()
            .child(
                div()
                    .flex()
                    .flex_row()
                    .px_2()
                    .py_1()
                    .gap_4()
                    .text_xs()
                    .font_semibold()
                    .text_color(theme.muted_foreground)
                    .border_b_1()
                    .border_color(theme.border)
                    .child(div().flex_1().child("键"))
                    .child(div().w(px(60.)).child("类型"))
                    .child(div().w(px(60.)).child("大小"))
                    .child(div().w(px(80.)).child("TTL")),
            )
            .child(
                tree(&browse.tree_state, move |_ix, entry, selected, _window, _cx| {
                    let item = entry.item();
                    let key_id = item.id.to_string();
                    let key_info = keys.get(key_id.as_str());

                    let list_item = ListItem::new(item.id.clone())
                        .pl(px(16.) * entry.depth() + px(12.))
                        .rounded_md()
                        .selected(selected);

                    if entry.is_folder() {
                        // 文件夹节点：添加展开/折叠图标
                        let icon_name = if entry.is_expanded() {
                            IconName::ChevronDown
                        } else {
                            IconName::ChevronRight
                        };

                        list_item.child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap_1()
                                .child(Icon::new(icon_name).size_3())
                                .child(item.label.clone()),
                        )
                    } else if let Some(info) = key_info {
                        list_item
                            .on_click({
                                let view = view_entity.clone();
                                let key = key_id.clone();
                                move |_event, window, cx| {
                                    let _ = view.update(cx, |this, cx| {
                                        this.select_key(key.clone(), window, cx);
                                    });
                                }
                            })
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .gap_1()
                                    .child(div().flex_1().text_sm().child(item.label.clone()))
                                    .child(
                                        div()
                                            .flex()
                                            .flex_row()
                                            .gap_4()
                                            .text_xs()
                                            .child(div().w(px(60.)).child(info.key_type.clone()))
                                            .child(div().w(px(60.)).child(info.size.clone()))
                                            .child(div().w(px(80.)).child(info.ttl.clone())),
                                    ),
                            )
                    } else {
                        list_item
                    }
                })
                .flex_1(),
            );

        // 渲染键详情（右侧面板）
        let key_detail = if let Some(key) = &browse.selected_key {
            if let Some(info) = self.find_key_info(key) {
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .p_4()
                    .col_full()
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .w(px(80.))
                                    .text_sm()
                                    .text_color(theme.muted_foreground)
                                    .child("键名称:"),
                            )
                            .child(div().flex_1().text_sm().child(info.full_key)),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .w(px(80.))
                                    .text_sm()
                                    .text_color(theme.muted_foreground)
                                    .child("键类型:"),
                            )
                            .child(
                                div()
                                    .w(px(200.))
                                    .child(Select::new(&browse.detail_key_type).with_size(Size::Small)),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(div().text_sm().text_color(theme.muted_foreground).child("值:"))
                            .child(
                                div()
                                    .flex_1()
                                    .min_h(px(200.))
                                    .p_2()
                                    .text_sm()
                                    .border_1()
                                    .border_color(theme.border)
                                    .rounded_md()
                                    .bg(theme.secondary)
                                    .child(browse.selected_key_value.clone()),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .w(px(80.))
                                    .text_sm()
                                    .text_color(theme.muted_foreground)
                                    .child("TTL:"),
                            )
                            .child(
                                div()
                                    .w(px(200.))
                                    .child(Select::new(&browse.detail_ttl).with_size(Size::Small)),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap_2()
                            .justify_end()
                            .child(Button::new(comp_id(["redis-detail-apply"])).label("应用").outline())
                            .child(Button::new(comp_id(["redis-detail-cancel"])).label("放弃").outline()),
                    )
                    .into_any_element()
            } else {
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .col_full()
                    .text_sm()
                    .text_color(theme.muted_foreground)
                    .child("请选择一个键查看详情")
                    .into_any_element()
            }
        } else {
            div()
                .flex()
                .items_center()
                .justify_center()
                .col_full()
                .text_sm()
                .text_color(theme.muted_foreground)
                .child("请选择一个键查看详情")
                .into_any_element()
        };

        v_resizable(comp_id(["redis-content", id]))
            .child(
                resizable_panel()
                    .size(px(300.0))
                    .size_range(px(200.0)..Pixels::MAX)
                    .child(key_list),
            )
            .child(key_detail)
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

        div()
            .flex()
            .flex_col()
            .gap_3()
            .p_4()
            .col_full()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .text_sm()
                            .font_semibold()
                            .text_color(theme.foreground)
                            .child("Redis 命令执行"),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap_2()
                            .child(
                                div()
                                    .flex_1()
                                    .child(Input::new(&command.command_input).with_size(Size::Small)),
                            )
                            .child(
                                Button::new(comp_id(["redis-command-execute", id]))
                                    .label("执行")
                                    .outline(),
                            )
                            .child(
                                Button::new(comp_id(["redis-command-clear", id]))
                                    .label("清空")
                                    .outline(),
                            ),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .flex_1()
                    .child(
                        div()
                            .text_sm()
                            .font_semibold()
                            .text_color(theme.foreground)
                            .child("执行结果"),
                    )
                    .child(
                        div()
                            .id(comp_id(["redis-command-result", id]))
                            .flex_1()
                            .p_3()
                            .text_sm()
                            .border_1()
                            .border_color(theme.border)
                            .rounded_md()
                            .bg(theme.secondary)
                            .scrollbar_y()
                            .when_some(command.command_result.as_ref(), |this, result| {
                                this.child(result.clone())
                            })
                            .when(command.command_result.is_none(), |this| {
                                this.child(
                                    div()
                                        .text_color(theme.muted_foreground)
                                        .child("命令执行结果将显示在此处"),
                                )
                            }),
                    ),
            )
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

        let is_overview = matches!(self.active, ViewType::Overview);
        let is_browse = matches!(self.active, ViewType::Browse);
        let is_command = matches!(self.active, ViewType::Command);
        let is_loading = self.browse.as_ref().map(|b| b.loading).unwrap_or(false);

        div()
            .id(comp_id(["redis", id]))
            .col_full()
            .child(
                div()
                    .id(comp_id(["redis-header", id]))
                    .flex()
                    .flex_row()
                    .items_center()
                    .p_4()
                    .gap_2()
                    .border_b_1()
                    .border_color(theme.border)
                    .child(
                        Button::new(comp_id(["redis-header-reload", id]))
                            .icon(icon_relead().with_size(Size::Small))
                            .label("刷新")
                            .outline()
                            .disabled(is_loading)
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
                            .loading(is_loading)
                            .on_click({
                                cx.listener(|view, _event, window, cx| {
                                    view.load_more_keys(window, cx);
                                })
                            }),
                    )
                    .child(
                        Button::new(comp_id(["redis-header-load-all", id]))
                            .icon(icon_arrow_down_line().with_size(Size::Small))
                            .label("获取全部")
                            .outline()
                            .loading(is_loading)
                            .on_click({
                                cx.listener(|view, _event, window, cx| {
                                    view.load_all_keys(window, cx);
                                })
                            }),
                    )
                    .child(
                        Button::new(comp_id(["redis-header-add", id]))
                            .icon(icon_create().with_size(Size::Small))
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
                                    .selected(is_overview),
                            )
                            .child(
                                Button::new(comp_id(["redis-view-browse", id]))
                                    .label("浏览数据")
                                    .selected(is_browse),
                            )
                            .child(
                                Button::new(comp_id(["redis-view-command", id]))
                                    .label("执行命令")
                                    .selected(is_command),
                            )
                            .on_click(
                                cx.listener(move |view, selected: &Vec<usize>, window, cx| match selected[0] {
                                    0 => view.switch_to_overview(window, cx),
                                    1 => view.switch_to_browse(window, cx),
                                    2 => view.switch_to_command(window, cx),
                                    _ => {}
                                }),
                            ),
                    ),
            )
            .child(match &self.active {
                ViewType::Overview => {
                    if let Some(overview) = &self.overview {
                        self.render_overview_view(overview, window, cx)
                    } else {
                        self.switch_to_overview(window, cx);
                        div().col_full().into_any_element()
                    }
                }
                ViewType::Browse => {
                    if let Some(browse) = self.browse.as_ref() {
                        self.render_browse_view(browse, window, cx)
                    } else {
                        self.switch_to_browse(window, cx);
                        div().col_full().into_any_element()
                    }
                }
                ViewType::Command => {
                    if let Some(command) = &self.command {
                        self.render_command_view(command, window, cx)
                    } else {
                        self.switch_to_command(window, cx);
                        div().col_full().into_any_element()
                    }
                }
            })
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
    pub selected_key: Option<String>,
    pub selected_key_value: String,
    pub last_refresh_time: Instant,
    pub detail_key_type: Entity<SelectState<Vec<SharedString>>>,
    pub detail_ttl: Entity<SelectState<Vec<SharedString>>>,
    pub scan_cursor: String,
    pub loading: bool,
}

pub struct CommandContent {
    pub command_input: Entity<InputState>,
    pub command_result: Option<String>,
}

pub struct OverviewContent {}
